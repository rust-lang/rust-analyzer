//! The context or environment in which the language server functions. In our
//! server implementation this is know as the `WorldState`.
//!
//! Each tick provides an immutable snapshot of the state as `WorldSnapshot`.

use std::{
    cell::RefCell,
    ops::Not as _,
    panic::AssertUnwindSafe,
    sync::OnceLock,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender, unbounded};
use hir::ChangeWithProcMacros;
use ide::{Analysis, AnalysisHost, Cancellable, FileId, SourceRootId};
use ide_db::{
    MiniCore,
    base_db::{Crate, ProcMacroPaths, SourceDatabase, all_crates, salsa::Revision},
};
use itertools::Itertools;
use load_cargo::SourceRootConfig;
use lsp_types::{Notification, SemanticTokens, Uri};
use parking_lot::{
    MappedRwLockReadGuard, Mutex, RwLock, RwLockReadGuard, RwLockUpgradableReadGuard,
    RwLockWriteGuard,
};
use proc_macro_api::ProcMacroClient;
use project_model::{
    ManifestPath, ProjectWorkspace, ProjectWorkspaceKind, TargetKind, WorkspaceBuildScripts,
};
use rustc_hash::{FxHashMap, FxHashSet};
use stdx::thread;
use tracing::{Level, span, trace};
use triomphe::Arc;
use vfs::{AbsPathBuf, AnchoredPathBuf, ChangeKind, Vfs, VfsPath};

use crate::{
    client::{Client, ClientId},
    config::{Config, ConfigChange, ConfigErrors, FilesWatcher, RatomlFileKind},
    diagnostics::{CheckFixes, DiagnosticCollection},
    discover,
    flycheck::{FlycheckHandle, FlycheckMessage, PackageSpecifier},
    line_index::{LineEndings, LineIndex, PositionEncoding},
    lsp::{capabilities::ClientCapabilities, from_proto, to_proto, to_proto::url_from_abs_path},
    lsp_ext,
    main_loop::Task,
    mem_docs::MemDocs,
    op_queue::{Cause, OpQueue},
    overlay::{self, DiskCache, Overlay, OverlayCrates, PulledInFile, SourceRoots},
    priming_scope, reload,
    target_spec::{CargoTargetSpec, ProjectJsonTargetSpec, TargetSpec},
    task_pool::{DeferredTaskQueue, TaskPool},
    test_runner::{CargoTestHandle, CargoTestMessage},
};

#[derive(Debug)]
pub(crate) struct FetchWorkspaceRequest {
    pub(crate) path: Option<AbsPathBuf>,
    pub(crate) force_crate_graph_reload: bool,
}

pub(crate) struct FetchWorkspaceResponse {
    pub(crate) workspaces: Vec<anyhow::Result<ProjectWorkspace>>,
    pub(crate) force_crate_graph_reload: bool,
}

pub(crate) struct FetchBuildDataResponse {
    pub(crate) workspaces: Arc<Vec<ProjectWorkspace>>,
    pub(crate) build_scripts: Vec<anyhow::Result<WorkspaceBuildScripts>>,
}

// Enforces drop order
pub(crate) struct Handle<H, C> {
    pub(crate) handle: H,
    pub(crate) receiver: C,
}

#[derive(Clone, Debug)]
pub(crate) struct ActiveProgress {
    pub(crate) title: String,
    pub(crate) message: Option<String>,
    pub(crate) percentage: Option<u32>,
    pub(crate) cancellable: bool,
}

pub(crate) type ReqHandler = fn(&mut GlobalState, ClientId, lsp_server::Response);
pub(crate) type ReqQueue = lsp_server::ReqQueue<(String, Instant), ReqHandler>;

/// `GlobalState` is the primary mutable state of the language server
///
/// The most interesting components are `vfs`, which stores a consistent
/// snapshot of the file systems, and `analysis_host`, which stores our
/// incremental salsa database.
///
/// Note that this struct has more than one impl in various modules!
#[doc(alias = "GlobalMess")]
pub(crate) struct GlobalState {
    pub(crate) clients: FxHashMap<ClientId, Client>,
    pub(crate) active_progress: FxHashMap<lsp_types::ProgressToken, ActiveProgress>,

    pub(crate) task_pool: Handle<TaskPool<Task>, Receiver<Task>>,
    pub(crate) fmt_pool: Handle<TaskPool<Task>, Receiver<Task>>,
    pub(crate) cancellation_pool: thread::Pool,

    pub(crate) config: Arc<Config>,
    pub(crate) config_errors: Option<ConfigErrors>,
    pub(crate) analysis_host: AnalysisHost,
    pub(crate) diagnostics: DiagnosticCollection,
    pub(crate) diagnostics_forced_files: FxHashSet<FileId>,
    pub(crate) mem_docs: MemDocs,
    pub(crate) source_root_config: SourceRootConfig,
    /// A mapping that maps a local source root's `SourceRootId` to it parent's `SourceRootId`, if it has one.
    pub(crate) local_roots_parent_map: Arc<FxHashMap<SourceRootId, SourceRootId>>,
    pub(crate) semantic_tokens_cache: Arc<Mutex<FxHashMap<Uri, SemanticTokens>>>,

    // status
    pub(crate) shutdown_requested: bool,
    pub(crate) last_reported_status: lsp_ext::ServerStatusParams,

    /// Clients of the proc-macro server.
    ///
    /// One client per workspace, in the same order as `self.workspaces`. We don't store
    /// the client in `self.workspaces` so that we can reload workspaces without
    /// restarting the proc-macro server.
    pub(crate) proc_macro_clients: Arc<[Option<anyhow::Result<ProcMacroClient>>]>,

    pub(crate) build_deps_changed: bool,

    // Flycheck
    pub(crate) flycheck: Arc<[FlycheckHandle]>,
    pub(crate) flycheck_sender: Sender<FlycheckMessage>,
    pub(crate) flycheck_receiver: Receiver<FlycheckMessage>,
    pub(crate) last_flycheck_error: Option<String>,
    pub(crate) flycheck_formatted_commands: Vec<String>,

    // Test explorer
    pub(crate) test_run_session_id: usize,
    pub(crate) test_run_session: Option<Vec<CargoTestHandle>>,
    pub(crate) test_run_client: Option<ClientId>,
    pub(crate) test_run_sender: Sender<CargoTestMessage>,
    pub(crate) test_run_receiver: Receiver<CargoTestMessage>,
    pub(crate) test_run_remaining_jobs: usize,

    // Project loading
    pub(crate) discover_handles: Vec<discover::DiscoverHandle>,
    pub(crate) discover_sender: Sender<discover::DiscoverProjectMessage>,
    pub(crate) discover_receiver: Receiver<discover::DiscoverProjectMessage>,
    pub(crate) discover_jobs_active: u32,

    // Debouncing channel for fetching the workspace
    // we want to delay it until the VFS looks stable-ish (and thus is not currently in the middle
    // of a VCS operation like `git switch`)
    pub(crate) fetch_ws_receiver: Option<(Receiver<Instant>, FetchWorkspaceRequest)>,

    // VFS
    pub(crate) loader: Handle<Box<dyn vfs::loader::Handle>, Receiver<vfs::loader::Message>>,
    pub(crate) vfs: Arc<RwLock<(vfs::Vfs, FxHashMap<FileId, LineEndings>)>>,
    pub(crate) vfs_config_version: u32,
    /// Whether the last VFS config left file watching to the clients.
    pub(crate) files_watched_by_client: bool,
    pub(crate) vfs_progress_config_version: u32,
    pub(crate) vfs_done: bool,
    // used to track how long VFS loading takes. this can't be on `vfs::loader::Handle`,
    // as that handle's lifetime is the same as `GlobalState` itself.
    pub(crate) vfs_span: Option<tracing::span::EnteredSpan>,
    pub(crate) wants_to_switch: Option<Cause>,

    /// `workspaces` field stores the data we actually use, while the `OpQueue`
    /// stores the result of the last fetch.
    ///
    /// If the fetch (partially) fails, we do not update the current value.
    ///
    /// The handling of build data is subtle. We fetch workspace in two phases:
    ///
    /// *First*, we run `cargo metadata`, which gives us fast results for
    /// initial analysis.
    ///
    /// *Second*, we run `cargo check` which runs build scripts and compiles
    /// proc macros.
    ///
    /// We need both for the precise analysis, but we want rust-analyzer to be
    /// at least partially available just after the first phase. That's because
    /// first phase is much faster, and is much less likely to fail.
    ///
    /// This creates a complication -- by the time the second phase completes,
    /// the results of the first phase could be invalid. That is, while we run
    /// `cargo check`, the user edits `Cargo.toml`, we notice this, and the new
    /// `cargo metadata` completes before `cargo check`.
    ///
    /// An additional complication is that we want to avoid needless work. When
    /// the user just adds comments or whitespace to Cargo.toml, we do not want
    /// to invalidate any salsa caches.
    pub(crate) workspaces: Arc<Vec<ProjectWorkspace>>,
    pub(crate) crate_graph_file_dependencies: FxHashSet<vfs::VfsPath>,
    pub(crate) detached_files: FxHashSet<ManifestPath>,

    // op queues
    pub(crate) fetch_workspaces_queue: OpQueue<FetchWorkspaceRequest, FetchWorkspaceResponse>,
    pub(crate) fetch_build_data_queue: OpQueue<(), FetchBuildDataResponse>,
    pub(crate) fetch_proc_macros_queue: OpQueue<(ChangeWithProcMacros, Vec<ProcMacroPaths>), bool>,
    pub(crate) prime_caches_queue: OpQueue,

    /// A deferred task queue.
    ///
    /// This queue is used for doing database-dependent work inside of sync
    /// handlers, as accessing the database may block latency-sensitive
    /// interactions and should be moved away from the main thread.
    ///
    /// For certain features, such as [`GlobalState::handle_discover_msg`],
    /// this queue should run only *after* [`GlobalState::process_changes`] has
    /// been called.
    pub(crate) deferred_task_queue: DeferredTaskQueue,

    /// HACK: Workaround for <https://github.com/rust-lang/rust-analyzer/issues/19709>
    /// This is marked true if we failed to load a crate root file at crate graph creation,
    /// which will usually end up causing a bunch of incorrect diagnostics on startup.
    pub(crate) incomplete_crate_graph: bool,
    /// The loaded workspaces that live in a git worktree of another loaded workspace.
    pub(crate) overlays: Arc<Vec<Overlay>>,
    /// For the crates of those workspaces, whether their sources were the same as in the base
    /// checkout when the crate graph was built.
    pub(crate) overlay_crates: Arc<OverlayCrates>,
    /// The partition of the files into source roots, kept while there are overlays.
    pub(crate) overlay_source_roots: Option<Arc<SourceRoots>>,
    /// The files that source files pull in by path, which may be outside of their package.
    pub(crate) pulled_in_files: FxHashMap<FileId, Vec<PulledInFile>>,
    /// Comparisons of the files that are pulled in but not loaded, as of the last time the crate
    /// graph was built.
    pub(crate) overlay_disk_cache: RefCell<DiskCache>,

    pub(crate) minicore: MiniCoreRustAnalyzerInternalOnly,
    pub(crate) last_gc_revision: Revision,
}

// FIXME: This should move to the VFS once the rewrite is done.
#[derive(Debug, Clone, Default)]
pub(crate) struct MiniCoreRustAnalyzerInternalOnly {
    pub(crate) minicore_text: Option<Arc<str>>,
}

/// An immutable snapshot of the world's state at a point in time.
pub(crate) struct GlobalStateSnapshot {
    pub(crate) client_id: Option<ClientId>,
    pub(crate) position_encoding: PositionEncoding,
    pub(crate) caps: Option<ClientCapabilities>,
    pub(crate) config: Arc<Config>,
    pub(crate) analysis: Analysis,
    pub(crate) check_fixes: CheckFixes,
    mem_docs: MemDocs,
    pub(crate) semantic_tokens_cache: Arc<Mutex<FxHashMap<Uri, SemanticTokens>>>,
    vfs: Arc<RwLock<(vfs::Vfs, FxHashMap<FileId, LineEndings>)>>,
    pub(crate) workspaces: Arc<Vec<ProjectWorkspace>>,
    // used to signal semantic highlighting to fall back to syntax based highlighting until
    // proc-macros have been loaded
    // FIXME: Can we derive this from somewhere else?
    pub(crate) proc_macros_loaded: bool,
    pub(crate) flycheck: Arc<[FlycheckHandle]>,
    minicore: MiniCoreRustAnalyzerInternalOnly,
    overlays: Arc<Vec<Overlay>>,
    overlay_crates: Arc<OverlayCrates>,
    overlay_source_roots: Option<Arc<SourceRoots>>,
    /// The overlay that the files named by the request belong to. Paths in the response are
    /// those of its worktree.
    request_overlay: OnceLock<usize>,
    client_root: Option<AbsPathBuf>,
}

impl std::panic::UnwindSafe for GlobalStateSnapshot {}

impl GlobalState {
    pub(crate) fn new(sender: Sender<lsp_server::Message>, config: Config) -> GlobalState {
        let encoding = config.caps().negotiated_encoding();
        let mut this = Self::new_multi(config);
        this.register_client_with_id(ClientId::DEFAULT, sender, encoding);
        if let Some(client) = this.clients.get_mut(&ClientId::DEFAULT) {
            client.is_initialized = true;
            client.root = Some(this.config.default_root_path().clone());
        }
        this
    }

    pub(crate) fn new_multi(config: Config) -> GlobalState {
        let loader = {
            let (sender, receiver) = unbounded::<vfs::loader::Message>();
            let handle: vfs_notify::NotifyHandle = vfs::loader::Handle::spawn(sender);
            let handle = Box::new(handle) as Box<dyn vfs::loader::Handle>;
            Handle { handle, receiver }
        };

        let task_pool = {
            let (sender, receiver) = unbounded();
            let handle = TaskPool::new_with_threads(sender, config.main_loop_num_threads());
            Handle { handle, receiver }
        };
        let fmt_pool = {
            let (sender, receiver) = unbounded();
            let handle = TaskPool::new_with_threads(sender, 1);
            Handle { handle, receiver }
        };
        let cancellation_pool = thread::Pool::new(1);

        let deferred_task_queue = {
            let (sender, receiver) = unbounded();
            DeferredTaskQueue { sender, receiver }
        };

        let mut analysis_host = AnalysisHost::new(config.lru_parse_query_capacity());
        if let Some(capacities) = config.lru_query_capacities_config() {
            analysis_host.update_lru_capacities(capacities);
        }
        let (flycheck_sender, flycheck_receiver) = unbounded();
        let (test_run_sender, test_run_receiver) = unbounded();

        let (discover_sender, discover_receiver) = unbounded();

        let last_gc_revision = analysis_host.raw_database().nonce_and_revision().1;

        let mut this = GlobalState {
            clients: FxHashMap::default(),
            active_progress: FxHashMap::default(),
            task_pool,
            fmt_pool,
            cancellation_pool,
            loader,
            config: Arc::new(config.clone()),
            analysis_host,
            diagnostics: Default::default(),
            diagnostics_forced_files: FxHashSet::default(),
            mem_docs: MemDocs::default(),
            semantic_tokens_cache: Arc::new(Default::default()),
            shutdown_requested: false,
            last_reported_status: lsp_ext::ServerStatusParams {
                health: lsp_ext::Health::Ok,
                quiescent: true,
                message: None,
            },
            source_root_config: SourceRootConfig::default(),
            local_roots_parent_map: Arc::new(FxHashMap::default()),
            config_errors: Default::default(),

            proc_macro_clients: Arc::from_iter([]),

            build_deps_changed: false,

            flycheck: Arc::from_iter([]),
            flycheck_sender,
            flycheck_receiver,
            last_flycheck_error: None,
            flycheck_formatted_commands: vec![],

            test_run_session_id: 0,
            test_run_session: None,
            test_run_client: None,
            test_run_sender,
            test_run_receiver,
            test_run_remaining_jobs: 0,

            discover_handles: vec![],
            discover_sender,
            discover_receiver,
            discover_jobs_active: 0,

            fetch_ws_receiver: None,

            vfs: Arc::new(RwLock::new((vfs::Vfs::default(), Default::default()))),
            vfs_config_version: 0,
            files_watched_by_client: false,
            vfs_progress_config_version: 0,
            vfs_span: None,
            vfs_done: true,
            wants_to_switch: None,

            workspaces: Arc::from(Vec::new()),
            crate_graph_file_dependencies: FxHashSet::default(),
            detached_files: FxHashSet::default(),
            fetch_workspaces_queue: OpQueue::default(),
            fetch_build_data_queue: OpQueue::default(),
            fetch_proc_macros_queue: OpQueue::default(),

            prime_caches_queue: OpQueue::default(),

            deferred_task_queue,
            incomplete_crate_graph: false,
            overlays: Arc::default(),
            overlay_crates: Arc::default(),
            overlay_source_roots: None,
            pulled_in_files: FxHashMap::default(),
            overlay_disk_cache: RefCell::default(),

            minicore: MiniCoreRustAnalyzerInternalOnly::default(),
            last_gc_revision,
        };
        // Apply any required database inputs from the config.
        this.update_configuration(config);
        this
    }

    pub(crate) fn process_changes(&mut self) -> (bool, Option<Duration>) {
        let _p = span!(Level::INFO, "GlobalState::process_changes").entered();
        // We cannot directly resolve a change in a ratoml file to a format
        // that can be used by the config module because config talks
        // in `SourceRootId`s instead of `FileId`s and `FileId` -> `SourceRootId`
        // mapping is not ready until `AnalysisHost::apply_changes` has been called.
        let mut modified_ratoml_files: FxHashMap<FileId, (ChangeKind, vfs::VfsPath)> =
            FxHashMap::default();

        let mut change = ChangeWithProcMacros::default();
        let mut guard = self.vfs.write();
        let changed_files = guard.0.take_changes();
        if changed_files.is_empty() {
            return (false, None);
        }
        let changed_file_ids: Vec<FileId> = changed_files.keys().copied().collect();
        let files_created_or_deleted =
            changed_files.values().any(|file| file.is_created_or_deleted());

        let (change, modified_rust_files, workspace_structure_change) =
            self.cancellation_pool.scoped(|s| {
                // start cancellation in parallel,
                // allowing us to do meaningful work while waiting
                let analysis_host = AssertUnwindSafe(&mut self.analysis_host);
                s.spawn(thread::ThreadIntent::LatencySensitive, || {
                    { analysis_host }.0.trigger_cancellation()
                });

                // downgrade to read lock to allow more readers while we are normalizing text
                let guard = RwLockWriteGuard::downgrade_to_upgradable(guard);
                let vfs: &Vfs = &guard.0;

                let mut workspace_structure_change = None;
                // A file was added or deleted
                let mut has_structure_changes = false;
                let mut bytes = vec![];
                let mut modified_rust_files = vec![];
                for file in changed_files.into_values() {
                    let vfs_path = vfs.file_path(file.file_id);
                    if let Some(("rust-analyzer", Some("toml"))) = vfs_path.name_and_extension() {
                        // Remember ids to use them after `apply_changes`
                        modified_ratoml_files.insert(file.file_id, (file.kind(), vfs_path.clone()));
                    }

                    if let Some(path) = vfs_path.as_path() {
                        has_structure_changes |= file.is_created_or_deleted();

                        if file.is_modified() && path.extension() == Some("rs") {
                            modified_rust_files.push(file.file_id);
                        }

                        let additional_files = self
                            .config
                            .discover_workspace_config()
                            .map(|cfg| {
                                cfg.files_to_watch.iter().map(String::as_str).collect::<Vec<&str>>()
                            })
                            .unwrap_or_default();

                        let path = path.to_path_buf();
                        if file.is_created_or_deleted() {
                            workspace_structure_change.get_or_insert((path, false)).1 |=
                                self.crate_graph_file_dependencies.contains(vfs_path);
                        } else if reload::should_refresh_for_change(
                            &path,
                            file.kind(),
                            &additional_files,
                        ) {
                            trace!(?path, kind = ?file.kind(), "refreshing for a change");
                            workspace_structure_change.get_or_insert((path.clone(), false));
                        }
                    }

                    // Clear native diagnostics when their file gets deleted
                    if !file.exists() {
                        self.diagnostics.clear_native_for(file.file_id);
                    }

                    let text = if let vfs::Change::Create(v, _) | vfs::Change::Modify(v, _) =
                        file.change
                    {
                        String::from_utf8(v).ok().map(|text| {
                            // FIXME: Consider doing normalization in the `vfs` instead? That allows
                            // getting rid of some locking
                            let (text, line_endings) = LineEndings::normalize(text);
                            (text, line_endings)
                        })
                    } else {
                        None
                    };
                    let pulled_in_files = match &text {
                        Some((text, _))
                            if vfs_path
                                .name_and_extension()
                                .is_some_and(|(_, ext)| ext == Some("rs")) =>
                        {
                            overlay::pulled_in_files(text)
                        }
                        _ => Vec::new(),
                    };
                    if pulled_in_files.is_empty() {
                        self.pulled_in_files.remove(&file.file_id);
                    } else {
                        self.pulled_in_files.insert(file.file_id, pulled_in_files);
                    }
                    // delay `line_endings_map` changes until we are done normalizing the text
                    // this allows delaying the re-acquisition of the write lock
                    bytes.push((file.file_id, text));
                }
                let (vfs, line_endings_map) = &mut *RwLockUpgradableReadGuard::upgrade(guard);
                bytes.into_iter().for_each(|(file_id, text)| {
                    let text = match text {
                        None => None,
                        Some((text, line_endings)) => {
                            line_endings_map.insert(file_id, line_endings);
                            Some(text)
                        }
                    };
                    change.change_file(file_id, text);
                });
                if has_structure_changes {
                    let roots = self.source_root_config.partition(vfs);
                    change.set_roots(roots);
                }
                (change, modified_rust_files, workspace_structure_change)
            });

        let cancellation_time = self.analysis_host.apply_change(change);

        if !modified_ratoml_files.is_empty()
            || !self.config.same_source_root_parent_map(&self.local_roots_parent_map)
        {
            let config_change = {
                let _p = span!(Level::INFO, "GlobalState::process_changes/config_change").entered();
                let user_config_path = (|| {
                    let mut p = Config::user_config_dir_path()?;
                    p.push("rust-analyzer.toml");
                    Some(p)
                })();

                let user_config_abs_path = user_config_path.as_deref();

                let mut change = ConfigChange::default();
                let db = self.analysis_host.raw_database();

                // FIXME @alibektas : This is silly. There is no reason to use VfsPaths when there is SourceRoots. But how
                // do I resolve a "workspace_root" to its corresponding id without having to rely on a cargo.toml's ( or project json etc.) file id?
                let workspace_ratoml_paths = self
                    .workspaces
                    .iter()
                    .map(|ws| {
                        VfsPath::from({
                            let mut p = ws.workspace_root().to_owned();
                            p.push("rust-analyzer.toml");
                            p
                        })
                    })
                    .collect_vec();

                for (file_id, (change_kind, vfs_path)) in modified_ratoml_files {
                    tracing::info!(%vfs_path, ?change_kind, "Processing rust-analyzer.toml changes");
                    if vfs_path.as_path() == user_config_abs_path {
                        tracing::info!(%vfs_path, ?change_kind, "Use config rust-analyzer.toml changes");
                        change.change_user_config(Some(db.file_text(file_id).text(db).clone()));
                    }

                    // If change has been made to a ratoml file that
                    // belongs to a non-local source root, we will ignore it.
                    let source_root_id = db.file_source_root(file_id).source_root_id(db);
                    let source_root = db.source_root(source_root_id).source_root(db);

                    if !source_root.is_library {
                        let entry = if workspace_ratoml_paths.contains(&vfs_path) {
                            tracing::info!(%vfs_path, ?source_root_id, "workspace rust-analyzer.toml changes");
                            change.change_workspace_ratoml(
                                source_root_id,
                                vfs_path.clone(),
                                Some(db.file_text(file_id).text(db).clone()),
                            )
                        } else {
                            tracing::info!(%vfs_path, ?source_root_id, "crate rust-analyzer.toml changes");
                            change.change_ratoml(
                                source_root_id,
                                vfs_path.clone(),
                                Some(db.file_text(file_id).text(db).clone()),
                            )
                        };

                        if let Some((kind, old_path, old_text)) = entry {
                            // SourceRoot has more than 1 RATOML files. In this case lexicographically smaller wins.
                            if old_path < vfs_path {
                                tracing::error!(
                                    "Two `rust-analyzer.toml` files were found inside the same crate. {vfs_path} has no effect."
                                );
                                // Put the old one back in.
                                match kind {
                                    RatomlFileKind::Crate => {
                                        change.change_ratoml(source_root_id, old_path, old_text);
                                    }
                                    RatomlFileKind::Workspace => {
                                        change.change_workspace_ratoml(
                                            source_root_id,
                                            old_path,
                                            old_text,
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        tracing::info!(%vfs_path, "Ignoring library rust-analyzer.toml");
                    }
                }
                change.change_source_root_parent_map(self.local_roots_parent_map.clone());
                change
            };

            let (config, e, should_update) = self.config.apply_change(config_change);
            self.config_errors = e.is_empty().not().then_some(e);

            if should_update {
                self.update_configuration(config);
            } else {
                // No global or client level config was changed. So we can naively replace config.
                self.config = Arc::new(config);
            }
        }

        // FIXME: `workspace_structure_change` is computed from `should_refresh_for_change` which is
        // path syntax based. That is not sufficient for all cases so we should lift that check out
        // into a `QueuedTask`, see `handle_did_save_text_document`.
        // Or maybe instead of replacing that check, kick off a semantic one if the syntactic one
        // didn't find anything (to make up for the lack of precision).
        {
            if !matches!(&workspace_structure_change, Some((.., true))) {
                _ = self.deferred_task_queue.sender.send(
                    crate::main_loop::DeferredTask::CheckProcMacroSources(modified_rust_files),
                );
            }
            // FIXME: ideally we should only trigger a workspace fetch for non-library changes
            // but something's going wrong with the source root business when we add a new local
            // crate see https://github.com/rust-lang/rust-analyzer/issues/13029
            if let Some((path, force_crate_graph_reload)) = workspace_structure_change {
                let _p = span!(Level::INFO, "GlobalState::process_changes/ws_structure_change")
                    .entered();
                self.enqueue_workspace_fetch(path, force_crate_graph_reload);
            }
        }

        self.recheck_overlays(&changed_file_ids, files_created_or_deleted);

        (true, Some(cancellation_time))
    }

    pub(crate) fn snapshot(&self) -> GlobalStateSnapshot {
        self.snapshot_for(None)
    }

    pub(crate) fn snapshot_for(&self, client_id: Option<ClientId>) -> GlobalStateSnapshot {
        let position_encoding = client_id
            .and_then(|id| self.clients.get(&id))
            .map(|c| c.position_encoding)
            .unwrap_or_else(|| self.config.caps().negotiated_encoding());
        let caps = client_id.and_then(|id| self.clients.get(&id)).map(|c| c.caps.clone());
        let client_root =
            client_id.and_then(|id| self.clients.get(&id)).and_then(|c| c.root.clone());
        let request_overlay = OnceLock::new();
        if let Some(root) = &client_root
            && let Some(idx) =
                self.overlays.iter().position(|it| root.starts_with(&it.worktree_root))
        {
            _ = request_overlay.set(idx);
        }

        GlobalStateSnapshot {
            client_id,
            position_encoding,
            caps,
            config: Arc::clone(&self.config),
            workspaces: Arc::clone(&self.workspaces),
            analysis: self.analysis_host.analysis(),
            vfs: Arc::clone(&self.vfs),
            minicore: self.minicore.clone(),
            check_fixes: Arc::clone(&self.diagnostics.check_fixes),
            mem_docs: self.mem_docs.clone(),
            semantic_tokens_cache: Arc::clone(&self.semantic_tokens_cache),
            proc_macros_loaded: !self.config.expand_proc_macros()
                || self.fetch_proc_macros_queue.last_op_result().copied().unwrap_or(false),
            flycheck: self.flycheck.clone(),
            overlays: Arc::clone(&self.overlays),
            overlay_crates: Arc::clone(&self.overlay_crates),
            overlay_source_roots: self.overlay_source_roots.clone(),
            request_overlay,
            client_root,
        }
    }

    pub(crate) fn register_client_with_id(
        &mut self,
        client_id: ClientId,
        sender: Sender<lsp_server::Message>,
        position_encoding: PositionEncoding,
    ) {
        // Until the client sends its own `initialize` request, assume the capabilities the
        // server was configured with.
        let caps = self.config.caps().clone();
        self.clients.insert(client_id, Client::new(sender, position_encoding, caps));
    }

    pub(crate) fn cleanup_closed_document(&mut self, path: &VfsPath) {
        if let Some((file_id, _)) = self.vfs.read().0.file_id(path) {
            self.diagnostics_forced_files.remove(&file_id);
            self.diagnostics.clear_native_for(file_id);
        }

        if let Some(abs_path) = path.as_path() {
            let uri = url_from_abs_path(abs_path);
            self.semantic_tokens_cache.lock().remove(&uri);
            self.loader.handle.invalidate(abs_path.to_path_buf());
        }
    }

    /// Clients whose open buffer for `path` differs from the text that is being analyzed.
    pub(crate) fn divergent_clients(&self, path: &VfsPath) -> Vec<ClientId> {
        self.clients
            .keys()
            .copied()
            .filter(|&id| {
                self.mem_docs.get(id, path).is_some()
                    && !self.mem_docs.is_in_sync_with_vfs(id, path)
            })
            .collect()
    }

    /// Makes `contents` the analyzed text of `path`, and makes sure that those of `clients`
    /// whose buffer now matches it receive the diagnostics for this text.
    ///
    /// Returns whether the analyzed text changed.
    pub(crate) fn set_analyzed_contents(
        &mut self,
        path: &VfsPath,
        contents: Vec<u8>,
        clients: &[ClientId],
    ) -> bool {
        // Library files are immutable: the client never becomes authoritative over their
        // contents, disk is the truth.
        let vfs_changed = !self.source_root_config.path_is_library(path)
            && self.vfs.write().0.set_file_contents(path.clone(), Some(contents));
        let Some((file_id, _)) = self.vfs.read().0.file_id(path) else {
            return vfs_changed;
        };
        let mut synced =
            clients.iter().copied().filter(|&id| self.mem_docs.is_in_sync_with_vfs(id, path));
        if vfs_changed {
            // The diagnostics are going to be recomputed, make sure they are published even if
            // they turn out to be the same as before.
            if synced.next().is_some() {
                self.diagnostics_forced_files.insert(file_id);
            }
        } else {
            // Nothing is going to be recomputed, so replay what we already have.
            for id in synced {
                self.replay_diagnostics_for_file_to(id, file_id);
            }
        }
        vfs_changed
    }

    pub(crate) fn unregister_client(&mut self, client_id: ClientId) -> Option<Client> {
        let (closed_paths, to_restore) = self.mem_docs.remove_client(client_id);
        for path in &closed_paths {
            self.cleanup_closed_document(path);
        }
        let other_clients: Vec<ClientId> =
            self.clients.keys().copied().filter(|&id| id != client_id).collect();
        for (path, content) in to_restore {
            self.set_analyzed_contents(&path, content, &other_clients);
        }
        if self.test_run_client == Some(client_id) {
            self.test_run_session_id = self.test_run_session_id.wrapping_add(1);
            self.test_run_session = None;
            self.test_run_client = None;
        }
        let removed = self.clients.remove(&client_id);
        if self.any_client_watches_files() != self.files_watched_by_client {
            self.update_file_watching();
        }
        removed
    }

    /// Whether file watching is delegated to at least one of the attached clients.
    pub(crate) fn any_client_watches_files(&self) -> bool {
        matches!(self.config.files().watcher, FilesWatcher::Client)
            && self
                .clients
                .values()
                .any(|client| client.caps.did_change_watched_files_dynamic_registration())
    }

    /// The file of the base checkout that is analyzed in place of the worktree's `file_id`.
    pub(crate) fn shared_base_file(&self, vfs: &vfs::Vfs, file_id: FileId) -> Option<FileId> {
        overlay::shared_base_file(
            vfs,
            self.overlay_source_roots.as_ref()?,
            &self.overlays,
            &self.overlay_crates,
            file_id,
            true,
        )
    }

    /// The paths under which a client wants the diagnostics of the file at `path`.
    ///
    /// The diagnostics of a crate that a worktree shares with its base checkout are also those
    /// of the worktree's files. A client gets them under the worktree's paths if it works in
    /// that worktree, or has the worktree's document open. A client that works in a worktree
    /// gets none for the rest of the base checkout, and one that works elsewhere none for the
    /// worktree.
    fn diagnostics_paths_for(
        &self,
        vfs: &vfs::Vfs,
        client_id: ClientId,
        client: &Client,
        path: &VfsPath,
    ) -> Vec<VfsPath> {
        let Some(abs_path) = path.as_path().filter(|_| !self.overlays.is_empty()) else {
            return vec![path.clone()];
        };
        let overlay_of = |path: &vfs::AbsPath| {
            self.overlays.iter().position(|it| path.starts_with(&it.worktree_root))
        };
        let client_overlay = client.root.as_deref().and_then(overlay_of);
        if let Some(file_overlay) = overlay_of(abs_path) {
            let wanted = match (client_overlay, &client.root) {
                (Some(client_overlay), _) => client_overlay == file_overlay,
                (None, Some(root)) => self.overlays[file_overlay].worktree_root.starts_with(root),
                (None, None) => true,
            };
            return if wanted { vec![path.clone()] } else { Vec::new() };
        }
        let file_id = vfs.file_id(path).map(|(file_id, _)| file_id);
        let mut paths: Vec<VfsPath> = self
            .overlays
            .iter()
            .enumerate()
            .filter_map(|(idx, overlay)| {
                let worktree_path = VfsPath::from(overlay.to_worktree(abs_path)?);
                let (worktree_file, _) = vfs.file_id(&worktree_path)?;
                let wanted = client_overlay == Some(idx)
                    || (client_overlay.is_none()
                        && self.mem_docs.get(client_id, &worktree_path).is_some());
                (wanted && self.shared_base_file(vfs, worktree_file) == file_id)
                    .then_some(worktree_path)
            })
            .collect();
        let in_base_checkout_of_client =
            client_overlay.is_some_and(|idx| abs_path.starts_with(&self.overlays[idx].base_root));
        if !in_base_checkout_of_client {
            paths.push(path.clone());
        }
        paths
    }

    pub(crate) fn replay_diagnostics_for_file_to(&self, client_id: ClientId, file_id: FileId) {
        let Some(client) = self.clients.get(&client_id) else {
            return;
        };
        let vfs = self.vfs.read();
        let file_id = self.shared_base_file(&vfs.0, file_id).unwrap_or(file_id);
        let base_encoding = self.config.caps().negotiated_encoding();
        for path in self.diagnostics_paths_for(&vfs.0, client_id, client, vfs.0.file_path(file_id))
        {
            if self.mem_docs.get(client_id, &path).is_some()
                && !self.mem_docs.is_in_sync_with_vfs(client_id, &path)
            {
                // Suppress diagnostics for divergent open buffers
                continue;
            }
            let Some(uri) = path.as_path().map(url_from_abs_path) else {
                continue;
            };
            let mut diagnostics =
                self.diagnostics.diagnostics_for(file_id).cloned().collect::<Vec<_>>();
            if client.position_encoding != base_encoding {
                diagnostics = to_proto::reencode_diagnostics(
                    &self.snapshot(),
                    &uri,
                    base_encoding,
                    client.position_encoding,
                    diagnostics,
                );
            }
            let version = self.mem_docs.get(client_id, &path).map(|d| d.version);
            let not = lsp_server::Notification::new(
                lsp_types::PublishDiagnosticsNotification::METHOD.into(),
                lsp_types::PublishDiagnosticsParams { uri, diagnostics, version },
            );
            self.send(client_id, not.into());
        }
    }

    pub(crate) fn replay_diagnostics_to(&self, client_id: ClientId) {
        let files = self.diagnostics.files_with_diagnostics();
        for file_id in files {
            if self.diagnostics.diagnostics_for(file_id).next().is_none() {
                continue;
            }
            self.replay_diagnostics_for_file_to(client_id, file_id);
        }
    }

    pub(crate) fn watched_files_registration_for(
        &self,
        caps: &ClientCapabilities,
    ) -> Option<lsp_types::Registration> {
        if !caps.did_change_watched_files_dynamic_registration() {
            return None;
        }
        if let crate::config::FilesWatcher::Client = self.config.files().watcher {
            let filter = self
                .workspaces
                .iter()
                .flat_map(|ws| ws.to_roots())
                .filter(|it| it.is_local)
                .map(|it| it.include);

            let mut watchers: Vec<lsp_types::FileSystemWatcher> =
                if caps.did_change_watched_files_relative_pattern_support() {
                    // When relative patterns are supported by the client, prefer using them
                    filter
                        .flat_map(|include| {
                            include.into_iter().flat_map(|base| {
                                [
                                    (base.clone(), "**/*.rs"),
                                    (base.clone(), "**/Cargo.{lock,toml}"),
                                    (base.clone(), "**/rust-analyzer.toml"),
                                    (base, "**/*.md"),
                                ]
                            })
                        })
                        .map(|(base, pat)| lsp_types::FileSystemWatcher {
                            glob_pattern: lsp_types::GlobPattern::RelativePattern(
                                lsp_types::RelativePattern {
                                    base_uri: lsp_types::BaseUri::Uri(
                                        lsp_types::Uri::from_file_path(base).unwrap(),
                                    ),
                                    pattern: pat.to_owned(),
                                },
                            ),
                            kind: None,
                        })
                        .collect()
                } else {
                    // When they're not, integrate the base to make them into absolute patterns
                    filter
                        .flat_map(|include| {
                            include.into_iter().flat_map(|base| {
                                [
                                    format!("{base}/**/*.rs"),
                                    format!("{base}/**/Cargo.{{toml,lock}}"),
                                    format!("{base}/**/rust-analyzer.toml"),
                                    format!("{base}/**/*.md"),
                                ]
                            })
                        })
                        .map(|glob_pattern| lsp_types::FileSystemWatcher {
                            glob_pattern: lsp_types::GlobPattern::Pattern(glob_pattern),
                            kind: None,
                        })
                        .collect()
                };

            // Also explicitly watch any build files configured in JSON project files.
            for ws in self.workspaces.iter() {
                if let ProjectWorkspaceKind::Json(project_json) = &ws.kind {
                    for (_, krate) in project_json.crates() {
                        let Some(build) = &krate.build else {
                            continue;
                        };
                        watchers.push(lsp_types::FileSystemWatcher {
                            glob_pattern: lsp_types::GlobPattern::Pattern(
                                build.build_file.to_string(),
                            ),
                            kind: None,
                        });
                    }
                }
            }

            watchers.extend(
                std::iter::once(Config::user_config_dir_path().as_deref())
                    .chain(self.workspaces.iter().map(|ws| ws.manifest().map(ManifestPath::as_ref)))
                    .flatten()
                    .map(|glob_pattern| lsp_types::FileSystemWatcher {
                        glob_pattern: lsp_types::GlobPattern::Pattern(glob_pattern.to_string()),
                        kind: None,
                    }),
            );

            let registration_options =
                lsp_types::DidChangeWatchedFilesRegistrationOptions { watchers };
            Some(lsp_types::Registration {
                id: "workspace/didChangeWatchedFiles".to_owned(),
                method: "workspace/didChangeWatchedFiles".to_owned(),
                register_options: Some(serde_json::to_value(registration_options).unwrap()),
            })
        } else {
            None
        }
    }

    pub(crate) fn send_request<R: lsp_types::Request>(
        &mut self,
        params: R::Params,
        handler: ReqHandler,
    ) where
        R::Params: Clone,
    {
        self.send_request_all::<R>(params, handler);
    }

    pub(crate) fn send_request_all<R: lsp_types::Request>(
        &mut self,
        params: R::Params,
        handler: ReqHandler,
    ) where
        R::Params: Clone,
    {
        let client_ids: Vec<_> =
            self.clients.iter().filter_map(|(&id, c)| c.is_initialized.then_some(id)).collect();
        for client_id in client_ids {
            self.send_request_to::<R>(client_id, params.clone(), handler);
        }
    }

    pub(crate) fn send_request_to<R: lsp_types::Request>(
        &mut self,
        client_id: ClientId,
        params: R::Params,
        handler: ReqHandler,
    ) {
        if let Some(client) = self.clients.get_mut(&client_id) {
            let request = client.req_queue.outgoing.register(R::METHOD.into(), params, handler);
            let _ = client.sender.send(request.into());
        }
    }

    pub(crate) fn complete_request(&mut self, client_id: ClientId, response: lsp_server::Response) {
        let handler = self
            .clients
            .get_mut(&client_id)
            .and_then(|client| client.req_queue.outgoing.complete(response.id.clone()));
        if let Some(handler) = handler {
            handler(self, client_id, response);
        }
    }

    pub(crate) fn send_notification<N: lsp_types::Notification>(&self, params: N::Params) {
        let not = lsp_server::Notification::new(N::METHOD.into(), params);
        self.broadcast(not.into());
    }

    pub(crate) fn send_notification_to<N: lsp_types::Notification>(
        &self,
        client_id: ClientId,
        params: N::Params,
    ) {
        let not = lsp_server::Notification::new(N::METHOD.into(), params);
        self.send(client_id, not.into());
    }

    pub(crate) fn register_request(
        &mut self,
        client_id: ClientId,
        request: &lsp_server::Request,
        request_received: Instant,
    ) {
        if let Some(client) = self.clients.get_mut(&client_id) {
            client
                .req_queue
                .incoming
                .register(request.id.clone(), (request.method.clone(), request_received));
        }
    }

    pub(crate) fn respond(&mut self, client_id: ClientId, response: lsp_server::Response) {
        let info = self.clients.get_mut(&client_id).and_then(|client| {
            client
                .req_queue
                .incoming
                .complete(&response.id)
                .map(|(method, start)| (client.sender.clone(), method, start))
        });
        let Some((sender, method, start)) = info else {
            return;
        };
        if let Some(err) = &response.error
            && err.message.starts_with("server panicked")
        {
            self.poke_rust_analyzer_developer(format!("{}, check the log", err.message));
        }

        let duration = start.elapsed();
        tracing::debug!(name: "message response", method, %response.id, duration = format_args!("{:0.2?}", duration));
        let _ = sender.send(response.into());
    }

    pub(crate) fn cancel(&mut self, client_id: ClientId, request_id: lsp_server::RequestId) {
        if let Some(client) = self.clients.get_mut(&client_id)
            && let Some(response) = client.req_queue.incoming.cancel(request_id)
        {
            let _ = client.sender.send(response.into());
        }
    }

    pub(crate) fn is_completed(&self, client_id: ClientId, request: &lsp_server::Request) -> bool {
        let Some(client) = self.clients.get(&client_id) else {
            return true;
        };
        client.req_queue.incoming.is_completed(&request.id)
    }

    pub(crate) fn send(&self, client_id: ClientId, message: lsp_server::Message) {
        if let Some(client) = self.clients.get(&client_id) {
            let _ = client.sender.send(message);
        }
    }

    pub(crate) fn broadcast(&self, message: lsp_server::Message) {
        for client in self.clients.values() {
            if client.is_initialized {
                let _ = client.sender.send(message.clone());
            }
        }
    }

    pub(crate) fn publish_diagnostics(
        &mut self,
        uri: Uri,
        mut diagnostics: Vec<lsp_types::Diagnostic>,
    ) {
        let base_encoding = self.config.caps().negotiated_encoding();
        let path = from_proto::vfs_path(&uri).ok();
        let clients: Vec<(Sender<lsp_server::Message>, PositionEncoding, Option<i32>, Uri)> = {
            let this = &*self;
            let mem_docs = &this.mem_docs;
            let vfs = &this.vfs.read().0;
            this.clients
                .iter()
                .filter(|(_, c)| c.is_initialized)
                .flat_map(|(&id, c)| {
                    // A client may know the file under other paths than the one we analyze.
                    let targets = match &path {
                        Some(path) => this
                            .diagnostics_paths_for(vfs, id, c, path)
                            .into_iter()
                            .filter_map(|path| {
                                let uri = url_from_abs_path(path.as_path()?);
                                Some((Some(path), uri))
                            })
                            .collect(),
                        None => vec![(None, uri.clone())],
                    };
                    targets.into_iter().filter_map(move |(path, uri)| {
                        if let Some(p) = &path
                            && mem_docs.get(id, p).is_some()
                            && !mem_docs.is_in_sync_with_vfs(id, p)
                        {
                            // Suppress diagnostics for divergent open buffers
                            return None;
                        }
                        let version =
                            path.as_ref().and_then(|p| mem_docs.get(id, p).map(|d| d.version));
                        Some((c.sender.clone(), c.position_encoding, version, uri))
                    })
                })
                .collect()
        };
        if clients.is_empty() {
            return;
        }
        // Positions are computed in the server-wide encoding; a snapshot is only needed to
        // re-encode them for clients that negotiated a different one.
        let snap =
            clients.iter().any(|&(_, enc, _, _)| enc != base_encoding).then(|| self.snapshot());
        // We put this on a separate thread to avoid blocking the main thread with serialization work
        self.task_pool.handle.spawn_with_sender(stdx::thread::ThreadIntent::Worker, {
            move |_| {
                // VSCode assumes diagnostic messages to be non-empty strings, so we need to patch
                // empty diagnostics. Neither the docs of VSCode nor the LSP spec say whether
                // diagnostic messages are actually allowed to be empty or not and patching this
                // in the VSCode client does not work as the assertion happens in the protocol
                // conversion. So this hack is here to stay, and will be considered a hack
                // until the LSP decides to state that empty messages are allowed.

                // See https://github.com/rust-lang/rust-analyzer/issues/11404
                // See https://github.com/rust-lang/rust-analyzer/issues/13130
                let patch_empty = |message: &mut lsp_types::Message| match message {
                    lsp_types::Message::String(m) if m.is_empty() => {
                        " ".clone_into(m);
                    }
                    lsp_types::Message::MarkupContent(lsp_types::MarkupContent {
                        value,
                        kind: _,
                    }) if value.is_empty() => {
                        " ".clone_into(value);
                    }
                    _ => {}
                };

                for d in &mut diagnostics {
                    patch_empty(&mut d.message);
                    if let Some(dri) = &mut d.related_information {
                        for dri in dri {
                            // The LSP does not (yet?) specify that related diagnostic messages can
                            // be in Markdown format (in addition to plain text).
                            if dri.message.is_empty() {
                                " ".clone_into(&mut dri.message);
                            }
                        }
                    }
                }

                for (sender, enc, version, uri) in clients {
                    let client_diagnostics = match &snap {
                        Some(snap) if enc != base_encoding => to_proto::reencode_diagnostics(
                            snap,
                            &uri,
                            base_encoding,
                            enc,
                            diagnostics.clone(),
                        ),
                        _ => diagnostics.clone(),
                    };

                    let not = lsp_server::Notification::new(
                        lsp_types::PublishDiagnosticsNotification::METHOD.into(),
                        lsp_types::PublishDiagnosticsParams {
                            uri,
                            diagnostics: client_diagnostics,
                            version,
                        },
                    );
                    let msg: lsp_server::Message = not.into();
                    _ = sender.send(msg);
                }
            }
        });
    }

    pub(crate) fn check_workspaces_msrv(&self) -> impl Iterator<Item = String> + '_ {
        self.workspaces.iter().filter_map(|ws| {
            if let Some(toolchain) = &ws.toolchain
                && *toolchain < crate::MINIMUM_SUPPORTED_TOOLCHAIN_VERSION
            {
                return Some(format!(
                    "Workspace `{}` is using an outdated toolchain version `{}` but \
                        rust-analyzer only supports `{}` and higher.\n\
                        Consider using the rust-analyzer rustup component for your toolchain or
                        upgrade your toolchain to a supported version.\n\n",
                    ws.manifest_or_root(),
                    toolchain,
                    crate::MINIMUM_SUPPORTED_TOOLCHAIN_VERSION,
                ));
            }
            None
        })
    }

    fn enqueue_workspace_fetch(&mut self, path: AbsPathBuf, force_crate_graph_reload: bool) {
        let already_requested = self.fetch_workspaces_queue.op_requested()
            && !self.fetch_workspaces_queue.op_in_progress();
        if self.fetch_ws_receiver.is_none() && already_requested {
            // Don't queue up a new fetch request if we already have done so
            // Otherwise we will re-fetch in quick succession which is unnecessary
            // Note though, that if one is already in progress, we *want* to re-queue
            // as the in-progress fetch might not have the latest changes in it anymore
            // FIXME: We should cancel the in-progress fetch here
            return;
        }

        self.fetch_ws_receiver = Some((
            crossbeam_channel::after(Duration::from_millis(100)),
            FetchWorkspaceRequest { path: Some(path), force_crate_graph_reload },
        ));
    }

    pub(crate) fn debounce_workspace_fetch(&mut self) {
        if let Some((fetch_receiver, _)) = &mut self.fetch_ws_receiver {
            *fetch_receiver = crossbeam_channel::after(Duration::from_millis(100));
        }
    }

    /// Set of crates to prime: the transitive-dependency closure of every
    /// local Cargo workspace `lib`/`bin` target, `rust-project.json` workspace
    /// member, and detached file. Computed once when the server becomes
    /// quiescent.
    ///
    /// Test, example, and benchmark members are deliberately excluded — they're
    /// leaves, so not priming them costs no parallelism on dependency work.
    /// `bin` targets are kept because they're the crate the user is most likely
    /// editing.
    pub(crate) fn compute_priming_scope(&self) -> Arc<[Crate]> {
        let db = self.analysis_host.raw_database();
        let all = all_crates(db);

        // Map each crate-root path to its crate(s) so target roots resolve to
        // `Crate` ids. The vfs read lock is held only for this build.
        let root_to_crate: FxHashMap<AbsPathBuf, Vec<Crate>> = {
            let vfs = self.vfs.read();
            let mut root_to_crate: FxHashMap<AbsPathBuf, Vec<Crate>> = FxHashMap::default();
            for &krate in &*all {
                let root_file = krate.data(db).root_file_id;
                let path = vfs.0.file_path(root_file);
                let Some(path) = path.as_path() else {
                    continue;
                };
                root_to_crate.entry(path.to_path_buf()).or_default().push(krate);
            }
            root_to_crate
        };

        let mut seed: FxHashSet<Crate> = FxHashSet::default();
        for workspace in self.workspaces.iter() {
            match &workspace.kind {
                ProjectWorkspaceKind::Cargo { cargo, .. }
                | ProjectWorkspaceKind::DetachedFile { cargo: Some((cargo, ..)), .. } => {
                    for pkg in cargo.packages() {
                        if !cargo[pkg].is_local {
                            continue;
                        }
                        for &target in &cargo[pkg].targets {
                            if !matches!(
                                cargo[target].kind,
                                TargetKind::Lib { .. } | TargetKind::Bin
                            ) {
                                continue;
                            }
                            if let Some(krates) = root_to_crate.get(&*cargo[target].root) {
                                seed.extend(krates.iter().copied());
                            }
                        }
                    }
                }
                ProjectWorkspaceKind::Json(project_json) => seed.extend(
                    project_json
                        .crates()
                        .filter(|(_, krate)| krate.is_workspace_member)
                        .filter_map(|(_, krate)| root_to_crate.get(&krate.root_module))
                        .flat_map(|it| it.iter().copied()),
                ),
                ProjectWorkspaceKind::DetachedFile { file, cargo: None } => {
                    if let Some(krates) = root_to_crate.get(&**file) {
                        seed.extend(krates.iter().copied());
                    }
                }
            }
        }

        priming_scope::compute(db, seed)
    }
}

impl Drop for GlobalState {
    fn drop(&mut self) {
        self.analysis_host.trigger_cancellation();
    }
}

impl GlobalStateSnapshot {
    pub(crate) fn caps(&self) -> &ClientCapabilities {
        self.caps.as_ref().unwrap_or_else(|| self.config.caps())
    }

    pub(crate) fn completion_config<'a>(
        &'a self,
        source_root: Option<ide_db::base_db::SourceRootId>,
    ) -> ide_completion::CompletionConfig<'a> {
        let mut cfg = self.config.completion(source_root, self.minicore());
        let caps = self.caps();
        if !caps.completion_snippet() {
            cfg.snippet_cap = None;
        }
        let client_capability_fields = caps.completion_resolve_support_properties();
        cfg.fields_to_resolve = if self.config.client_is_neovim() {
            ide_completion::CompletionFieldsToResolve::empty()
        } else {
            ide_completion::CompletionFieldsToResolve::from_client_capabilities(
                &client_capability_fields,
            )
        };
        cfg
    }

    pub(crate) fn inlay_hints_config(&self) -> ide::InlayHintsConfig<'_> {
        let mut cfg = self.config.inlay_hints(self.minicore());
        let properties = self.caps().inlay_hint_resolve_support_properties();
        cfg.fields_to_resolve = ide::InlayFieldsToResolve::from_client_capabilities(&properties);
        cfg
    }

    fn vfs_read(&self) -> MappedRwLockReadGuard<'_, vfs::Vfs> {
        RwLockReadGuard::map(self.vfs.read(), |(it, _)| it)
    }

    /// Returns `None` if the file was excluded.
    pub(crate) fn url_to_file_id(&self, url: &Uri) -> anyhow::Result<Option<FileId>> {
        let Some(file_id) = url_to_file_id(&self.vfs_read(), url)? else {
            return Ok(None);
        };
        Ok(Some(self.analyzed_file(file_id)?))
    }

    /// The file that is analyzed for `file_id`: a file of a worktree that is not part of any
    /// crate stands for the same file of the base checkout, as its crate is shared with it.
    fn analyzed_file(&self, file_id: FileId) -> Cancellable<FileId> {
        if self.overlays.is_empty() {
            return Ok(file_id);
        }
        let base_file = {
            let vfs = self.vfs_read();
            let Some(path) = vfs.file_path(file_id).as_path() else {
                return Ok(file_id);
            };
            let Some(idx) = self.overlays.iter().position(|it| path.starts_with(&it.worktree_root))
            else {
                return Ok(file_id);
            };
            _ = self.request_overlay.set(idx);
            // Until the crate graph is rebuilt after a change, the file may be in no crate
            // although its package is not shared anymore: that the file is the same and that
            // its package is known to be shared has to be checked as well.
            self.overlay_source_roots.as_ref().and_then(|roots| {
                overlay::shared_base_file(
                    &vfs,
                    roots,
                    &self.overlays,
                    &self.overlay_crates,
                    file_id,
                    false,
                )
            })
        };
        match base_file {
            Some(base_file) if self.analysis.crates_for(file_id)?.is_empty() => Ok(base_file),
            _ => Ok(file_id),
        }
    }

    /// Whether something found in `file_id` is of interest to the client that sent the request.
    ///
    /// A client that works in a worktree does not want to hear about the base checkout's copy of
    /// a crate that the worktree has its own version of, nor about other worktrees. A client
    /// that works elsewhere does not want to hear about worktrees.
    pub(crate) fn in_client_view(&self, file_id: FileId) -> Cancellable<bool> {
        if self.overlays.is_empty() {
            return Ok(true);
        }
        let worktree_file = {
            let vfs = self.vfs_read();
            let Some(path) = vfs.file_path(file_id).as_path() else {
                return Ok(true);
            };
            let worktree_of_file =
                self.overlays.iter().position(|it| path.starts_with(&it.worktree_root));
            let Some(&idx) = self.request_overlay.get() else {
                // Without knowing where the client works, show it everything.
                let outside_of_client_root = worktree_of_file.is_some_and(|idx| {
                    self.client_root
                        .as_ref()
                        .is_some_and(|root| !self.overlays[idx].worktree_root.starts_with(root))
                });
                return Ok(!outside_of_client_root);
            };
            if worktree_of_file.is_some() {
                return Ok(worktree_of_file == Some(idx));
            }
            self.overlays[idx]
                .to_worktree(path)
                .and_then(|path| vfs.file_id(&VfsPath::from(path)))
                .map(|(file_id, _)| file_id)
        };
        match worktree_file {
            Some(worktree_file) => Ok(self.analysis.crates_for(worktree_file)?.is_empty()),
            None => Ok(true),
        }
    }

    /// Keeps the items that are in files that are of interest to the client.
    pub(crate) fn retain_in_client_view<T>(
        &self,
        items: &mut Vec<T>,
        file_id: impl Fn(&T) -> FileId,
    ) -> Cancellable<()> {
        if self.overlays.is_empty() {
            return Ok(());
        }
        let mut res = Ok(());
        items.retain(|item| match self.in_client_view(file_id(item)) {
            Ok(in_view) => in_view,
            Err(cancelled) => {
                res = Err(cancelled);
                true
            }
        });
        res
    }

    /// The path under which the client that sent the request knows the file.
    fn client_path(&self, vfs: &vfs::Vfs, file_id: FileId) -> VfsPath {
        let path = vfs.file_path(file_id);
        let worktree_path = self
            .request_overlay
            .get()
            .and_then(|&idx| self.overlays[idx].to_worktree(path.as_path()?))
            .map(VfsPath::from)
            .filter(|path| vfs.file_id(path).is_some());
        worktree_path.unwrap_or_else(|| path.clone())
    }

    pub(crate) fn file_id_to_url(&self, id: FileId) -> Uri {
        let path = self.client_path(&self.vfs_read(), id);
        url_from_abs_path(path.as_path().unwrap())
    }

    /// Returns `None` if the file was excluded.
    pub(crate) fn vfs_path_to_file_id(&self, vfs_path: &VfsPath) -> anyhow::Result<Option<FileId>> {
        vfs_path_to_file_id(&self.vfs_read(), vfs_path)
    }

    pub(crate) fn file_line_index(&self, file_id: FileId) -> Cancellable<LineIndex> {
        let endings = match self.vfs.read().1.get(&file_id) {
            Some(&endings) => endings,
            None => return Err(ide_db::base_db::salsa::Cancelled::PendingWrite),
        };
        let index = self.analysis.file_line_index(file_id)?;
        let res = LineIndex { index, endings, encoding: self.position_encoding };
        Ok(res)
    }

    pub(crate) fn file_version(&self, file_id: FileId) -> Option<i32> {
        let path = &self.client_path(&self.vfs_read(), file_id);
        match self.client_id {
            Some(client_id) if self.mem_docs.is_in_sync_with_vfs(client_id, path) => {
                self.mem_docs.get(client_id, path).map(|d| d.version)
            }
            Some(_) => None,
            None => self.mem_docs.get_any(path).map(|d| d.version),
        }
    }

    pub(crate) fn url_file_version(&self, url: &Uri) -> Option<i32> {
        let path = from_proto::vfs_path(url).ok()?;
        match self.client_id {
            Some(client_id) if self.mem_docs.is_in_sync_with_vfs(client_id, &path) => {
                self.mem_docs.get(client_id, &path).map(|d| d.version)
            }
            Some(_) => None,
            None => self.mem_docs.get_any(&path).map(|d| d.version),
        }
    }

    pub(crate) fn file_mem_data(&self, path: &VfsPath) -> Option<Vec<u8>> {
        let client_id = self.client_id?;
        self.mem_docs.get(client_id, path).map(|d| d.data.clone())
    }

    pub(crate) fn is_file_divergent(&self, file_id: FileId) -> bool {
        let path = self.client_path(&self.vfs_read(), file_id);
        self.is_path_divergent(&path)
    }

    pub(crate) fn is_path_divergent(&self, path: &VfsPath) -> bool {
        if let Some(client_id) = self.client_id {
            if self.mem_docs.get(client_id, path).is_some() {
                // The client has the document open; it will edit its in-memory buffer.
                // It is divergent if its buffer does not match VFS.
                return !self.mem_docs.is_in_sync_with_vfs(client_id, path);
            }
            // The client does not have the document open, so it refers to the text on disk,
            // which is not what we analyze if another client has unsaved changes.
            if self.mem_docs.contains(path) {
                return !self.mem_docs.matches_disk(path);
            }
        }
        false
    }

    pub(crate) fn is_dir_divergent(&self, dir_path: &VfsPath) -> bool {
        let dir_abs = dir_path.as_path();
        let dir_str = dir_path.to_string();
        for path in self.mem_docs.iter() {
            let is_child = match (dir_abs, path.as_path()) {
                (Some(dir), Some(p)) => p.starts_with(dir),
                _ => path.to_string().starts_with(&dir_str),
            };
            if is_child && self.is_path_divergent(path) {
                return true;
            }
        }
        false
    }

    pub(crate) fn anchored_path(&self, path: &AnchoredPathBuf) -> Uri {
        let mut base = self.client_path(&self.vfs_read(), path.anchor);
        base.pop();
        let path = base.join(&path.path).unwrap();
        let path = path.as_path().unwrap();
        url_from_abs_path(path)
    }

    pub(crate) fn file_id_to_file_path(&self, file_id: FileId) -> vfs::VfsPath {
        self.vfs_read().file_path(file_id).clone()
    }

    pub(crate) fn target_spec_for_crate(&self, crate_id: Crate) -> Option<TargetSpec> {
        let file_id = self.analysis.crate_root(crate_id).ok()?;
        self.target_spec_for_file(file_id, crate_id)
    }

    pub(crate) fn target_spec_for_file(
        &self,
        file_id: FileId,
        crate_id: Crate,
    ) -> Option<TargetSpec> {
        let path = self.vfs_read().file_path(file_id).clone();
        let path = path.as_path()?;

        for workspace in self.workspaces.iter() {
            match &workspace.kind {
                ProjectWorkspaceKind::Cargo { cargo, .. }
                | ProjectWorkspaceKind::DetachedFile { cargo: Some((cargo, _, _)), .. } => {
                    let Some(target_idx) = cargo.target_by_root(path) else {
                        continue;
                    };

                    let target_data = &cargo[target_idx];
                    let package_data = &cargo[target_data.package];

                    return Some(TargetSpec::Cargo(CargoTargetSpec {
                        workspace_root: cargo.workspace_root().to_path_buf(),
                        cargo_toml: package_data.manifest.clone(),
                        crate_id,
                        package: cargo.package_flag(package_data),
                        package_id: package_data.id.clone(),
                        target: target_data.name.clone(),
                        target_kind: target_data.kind,
                        required_features: target_data.required_features.clone(),
                        features: package_data.features.keys().cloned().collect(),
                        sysroot_root: workspace.sysroot.root().map(ToOwned::to_owned),
                    }));
                }
                ProjectWorkspaceKind::Json(project) => {
                    let Some(krate) = project.crate_by_root(path) else {
                        continue;
                    };
                    let Some(build) = krate.build.clone() else {
                        continue;
                    };

                    return Some(TargetSpec::ProjectJson(ProjectJsonTargetSpec {
                        label: build.label,
                        target_kind: build.target_kind,
                        shell_runnables: project.runnables().to_owned(),
                        project_root: project.project_root().to_owned(),
                    }));
                }
                ProjectWorkspaceKind::DetachedFile { .. } => {}
            };
        }

        None
    }

    pub(crate) fn all_workspace_dependencies_for_package(
        &self,
        package: &PackageSpecifier,
    ) -> Option<FxHashSet<PackageSpecifier>> {
        match package {
            PackageSpecifier::Cargo { package_id } => {
                self.workspaces.iter().find_map(|workspace| match &workspace.kind {
                    ProjectWorkspaceKind::Cargo { cargo, .. }
                    | ProjectWorkspaceKind::DetachedFile { cargo: Some((cargo, _, _)), .. } => {
                        let package = cargo.packages().find(|p| cargo[*p].id == *package_id)?;

                        cargo[package].all_member_deps.as_ref().map(|deps| {
                            deps.iter()
                                .map(|dep| cargo[*dep].id.clone())
                                .map(|p| PackageSpecifier::Cargo { package_id: p })
                                .collect()
                        })
                    }
                    _ => None,
                })
            }
            PackageSpecifier::BuildInfo { label } => {
                self.workspaces.iter().find_map(|workspace| match &workspace.kind {
                    ProjectWorkspaceKind::Json(p) => {
                        let krate = p.crate_by_label(label)?;
                        Some(
                            krate
                                .iter_deps()
                                .filter_map(|dep| p[dep].build.as_ref())
                                .map(|build| PackageSpecifier::BuildInfo {
                                    label: build.label.clone(),
                                })
                                .collect(),
                        )
                    }
                    _ => None,
                })
            }
        }
    }

    pub(crate) fn file_exists(&self, file_id: FileId) -> bool {
        self.vfs.read().0.exists(file_id)
    }

    #[inline]
    pub(crate) fn minicore(&self) -> MiniCore<'_> {
        match &self.minicore.minicore_text {
            Some(minicore) => MiniCore::new(minicore),
            None => MiniCore::default(),
        }
    }
}

pub(crate) fn file_id_to_url(vfs: &vfs::Vfs, id: FileId) -> Uri {
    let path = vfs.file_path(id);
    let path = path.as_path().unwrap();
    url_from_abs_path(path)
}

/// Returns `None` if the file was excluded.
pub(crate) fn url_to_file_id(vfs: &vfs::Vfs, url: &Uri) -> anyhow::Result<Option<FileId>> {
    let path = from_proto::vfs_path(url)?;
    vfs_path_to_file_id(vfs, &path)
}

/// Returns `None` if the file was excluded.
pub(crate) fn vfs_path_to_file_id(
    vfs: &vfs::Vfs,
    vfs_path: &VfsPath,
) -> anyhow::Result<Option<FileId>> {
    let (file_id, excluded) =
        vfs.file_id(vfs_path).ok_or_else(|| anyhow::format_err!("file not found: {vfs_path}"))?;
    match excluded {
        vfs::FileExcluded::Yes => Ok(None),
        vfs::FileExcluded::No => Ok(Some(file_id)),
    }
}
