//! Loads a Cargo project into a static instance of analysis, without support
//! for incorporating changes.
// Note, don't remove any public api from this. This API is consumed by external tools
// to run rust-analyzer as a library.

#![cfg_attr(feature = "in-rust-tree", feature(rustc_private))]

#[cfg(feature = "in-rust-tree")]
extern crate rustc_driver as _;

use std::{any::Any, collections::hash_map::Entry, mem, path::Path, sync};

use crossbeam_channel::{Receiver, unbounded};
use hir_expand::proc_macro::{
    ProcMacro, ProcMacroExpander, ProcMacroExpansionError, ProcMacroKind, ProcMacroLoadResult,
    ProcMacrosBuilder,
};
use ide_db::{
    ChangeWithProcMacros, FxHashMap, FxHashSet, RootDatabase,
    base_db::{
        CrateGraphBuilder, Env, FileRegistration, FileRoot, FileRootId, FileRootKind,
        ProcMacroLoadingError, SourceDatabase,
    },
    prime_caches,
};
use itertools::Itertools;
use proc_macro_api::{
    MacroDylib, ProcMacroClient,
    bidirectional_protocol::msg::{ParentSpan, SubRequest, SubResponse},
};
use project_model::{CargoConfig, PackageRoot, ProjectManifest, ProjectWorkspace};
use span::{File, Span, SpanAnchor, SyntaxContext};
use tt::{TextRange, TextSize};
use vfs::{
    AbsPath, AbsPathBuf, VfsPath,
    loader::{Handle, LoadingProgress},
    path_classifier::PathClassifier,
};

#[derive(Debug)]
pub struct LoadCargoConfig {
    pub load_out_dirs_from_check: bool,
    pub with_proc_macro_server: ProcMacroServerChoice,
    pub prefill_caches: bool,
    pub num_worker_threads: usize,
    pub proc_macro_processes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcMacroServerChoice {
    Sysroot,
    Explicit(AbsPathBuf),
    None,
}

pub fn load_workspace_at(
    root: &Path,
    cargo_config: &CargoConfig,
    load_config: &LoadCargoConfig,
    progress: &(dyn Fn(String) + Sync),
) -> anyhow::Result<(RootDatabase, Option<ProcMacroClient>)> {
    let root = AbsPathBuf::assert_utf8(std::env::current_dir()?.join(root));
    let root = ProjectManifest::discover_single(&root)?;
    let manifest_path = root.manifest_path().clone();
    let mut workspace = ProjectWorkspace::load(root, cargo_config, progress)?;

    if load_config.load_out_dirs_from_check {
        let build_scripts = workspace.run_build_scripts(cargo_config, progress)?;
        if let Some(error) = build_scripts.error() {
            tracing::debug!(
                "Errors occurred while running build scripts for {}: {}",
                manifest_path,
                error
            );
        }
        workspace.set_build_scripts(build_scripts)
    }

    load_workspace(workspace, &cargo_config.extra_env, load_config)
}

pub fn load_workspace(
    ws: ProjectWorkspace,
    extra_env: &FxHashMap<String, Option<String>>,
    load_config: &LoadCargoConfig,
) -> anyhow::Result<(RootDatabase, Option<ProcMacroClient>)> {
    let lru_cap = std::env::var("RA_LRU_CAP").ok().and_then(|it| it.parse::<u16>().ok());
    let mut db = RootDatabase::new(lru_cap);

    let proc_macro_server = load_workspace_into_db(ws, extra_env, load_config, &mut db)?;

    Ok((db, proc_macro_server))
}

// This variant of `load_workspace` allows deferring the loading of rust-analyzer
// into an existing database, which is useful in certain third-party scenarios,
// now that `salsa` supports extending foreign databases (e.g. `RootDatabase`).
pub fn load_workspace_into_db(
    ws: ProjectWorkspace,
    extra_env: &FxHashMap<String, Option<String>>,
    load_config: &LoadCargoConfig,
    db: &mut RootDatabase,
) -> anyhow::Result<Option<ProcMacroClient>> {
    let (sender, receiver) = unbounded();
    let mut loader = {
        let loader = vfs_notify::NotifyHandle::spawn(sender);
        Box::new(loader)
    };

    tracing::debug!(?load_config, "LoadCargoConfig");
    let proc_macro_server =
        spawn_proc_macro_server(std::slice::from_ref(&ws), extra_env, load_config);
    match &proc_macro_server {
        Some(Ok(server)) => {
            tracing::info!(manifest=%ws.manifest_or_root(), path=%server.server_path(), "Proc-macro server started")
        }
        Some(Err(e)) => {
            tracing::info!(manifest=%ws.manifest_or_root(), %e, "Failed to start proc-macro server")
        }
        None => {
            tracing::info!(manifest=%ws.manifest_or_root(), "No proc-macro server started")
        }
    }

    let mut loaded_files = FxHashMap::default();
    let (crate_graph, proc_macros) = ws.to_crate_graph(
        &mut |path: &AbsPath| {
            let contents = loader.load_sync(path);
            let path = VfsPath::from(path.to_path_buf());
            let file_id = db.intern_file_path(path.clone());
            let exists = contents.is_some();
            loaded_files.insert(file_id, (path, contents));
            exists.then_some(file_id)
        },
        extra_env,
    );
    let proc_macro_server_ref = flatten_proc_macro_server(&proc_macro_server);
    let proc_macros = proc_macros
        .into_iter()
        .map(|(crate_id, path)| {
            (
                crate_id,
                path.and_then(|(_, path)| {
                    proc_macro_server_ref.as_ref().map_err(Clone::clone).and_then(
                        |proc_macro_server| load_proc_macro(proc_macro_server, &path, &[]),
                    )
                }),
            )
        })
        .collect();

    let project_folders = ProjectFolders::new(std::slice::from_ref(&ws), &[], None);
    loader.set_config(vfs::loader::Config {
        load: project_folders.load,
        watch: vec![],
        version: 0,
    });

    load_crate_graph_into_db(
        crate_graph,
        proc_macros,
        &project_folders.file_root_config,
        loaded_files,
        &receiver,
        db,
    );

    if load_config.prefill_caches {
        let all = ide_db::base_db::all_crates(db);
        prime_caches::parallel_prime_caches(db, &all, load_config.num_worker_threads, &|_| ());
    }

    Ok(proc_macro_server.and_then(Result::ok))
}

#[derive(Default)]
pub struct ProjectFolders {
    pub load: Vec<vfs::loader::Entry>,
    pub watch: Vec<usize>,
    pub file_root_config: FileRootConfig,
}

impl ProjectFolders {
    pub fn new(
        workspaces: &[ProjectWorkspace],
        global_excludes: &[AbsPathBuf],
        user_config_dir_path: Option<&AbsPath>,
    ) -> ProjectFolders {
        let mut res = ProjectFolders::default();
        let mut classifier = PathClassifier::builder();
        let mut file_root_kinds = vec![];

        // Dedup file roots
        // Depending on the project setup, we can have duplicated file roots, or for example in
        // the case of the rustc workspace, we can end up with two file roots that are almost the
        // same but not quite, like:
        // PackageRoot { is_local: false, include: [AbsPathBuf(".../rust/src/tools/miri/cargo-miri")], exclude: [] }
        // PackageRoot {
        //     is_local: true,
        //     include: [AbsPathBuf(".../rust/src/tools/miri/cargo-miri"), AbsPathBuf(".../rust/build/x86_64-pc-windows-msvc/stage0-tools/x86_64-pc-windows-msvc/release/build/cargo-miri-85801cd3d2d1dae4/out")],
        //     exclude: [AbsPathBuf(".../rust/src/tools/miri/cargo-miri/.git"), AbsPathBuf(".../rust/src/tools/miri/cargo-miri/target")]
        // }
        //
        // The first one comes from the explicit rustc workspace which points to the rustc workspace itself
        // The second comes from the rustc workspace that we load as the actual project workspace
        // These `is_local` differing in this kind of way gives us problems, especially when trying to filter diagnostics as we don't report diagnostics for external libraries.
        // So we need to deduplicate these, usually it would be enough to deduplicate by `include`, but as the rustc example shows here that doesn't work,
        // so we need to also coalesce the includes if they overlap.

        let mut roots: Vec<_> = workspaces
            .iter()
            .flat_map(|ws| ws.to_roots())
            .update(|root| root.include.sort())
            .sorted_by(|a, b| a.include.cmp(&b.include))
            .collect();

        // map that tracks indices of overlapping roots
        let mut overlap_map = FxHashMap::<_, Vec<_>>::default();
        let mut done = false;

        while !mem::replace(&mut done, true) {
            // maps include paths to indices of the corresponding root
            let mut include_to_idx = FxHashMap::default();
            // Find and note down the indices of overlapping roots
            for (idx, root) in roots.iter().enumerate().filter(|(_, it)| !it.include.is_empty()) {
                for include in &root.include {
                    match include_to_idx.entry(include) {
                        Entry::Occupied(e) => {
                            overlap_map.entry(*e.get()).or_default().push(idx);
                        }
                        Entry::Vacant(e) => {
                            e.insert(idx);
                        }
                    }
                }
            }
            for (k, v) in overlap_map.drain() {
                done = false;
                for v in v {
                    let r = mem::replace(
                        &mut roots[v],
                        PackageRoot { is_local: false, include: vec![], exclude: vec![] },
                    );
                    roots[k].is_local |= r.is_local;
                    roots[k].include.extend(r.include);
                    roots[k].exclude.extend(r.exclude);
                }
                roots[k].include.sort();
                roots[k].exclude.sort();
                roots[k].include.dedup();
                roots[k].exclude.dedup();
            }
        }

        // Collect workspace roots not already covered by a local PackageRoot
        // (e.g. virtual workspaces where no package lives at the workspace root).
        // We need these to load workspace-root rust-analyzer.toml into a local file root.
        let uncovered_ws_roots: Vec<AbsPathBuf> = workspaces
            .iter()
            .filter_map(|ws| {
                let ws_root = ws.workspace_root().to_path_buf();
                let dominated =
                    roots.iter().any(|root| root.is_local && root.include.contains(&ws_root));
                (!dominated).then_some(ws_root)
            })
            .collect();

        for root in roots.into_iter().filter(|it| !it.include.is_empty()) {
            let root_paths: Vec<VfsPath> =
                root.include.iter().cloned().map(VfsPath::from).collect();

            let entry = {
                let mut dirs = vfs::loader::Directories::default();
                dirs.extensions.push("rs".into());
                dirs.extensions.push("toml".into());
                dirs.extensions.push("md".into());
                dirs.include.extend(root.include);
                dirs.exclude.extend(root.exclude);
                for excl in global_excludes {
                    if dirs
                        .include
                        .iter()
                        .any(|incl| incl.starts_with(excl) || excl.starts_with(incl))
                    {
                        dirs.exclude.push(excl.clone());
                    }
                }

                vfs::loader::Entry::Directories(dirs)
            };

            let file_root_kind =
                if root.is_local { FileRootKind::Local } else { FileRootKind::Library };

            if file_root_kind == FileRootKind::Local {
                res.watch.push(res.load.len());
            }
            res.load.push(entry);

            file_root_kinds.push(file_root_kind);
            classifier.add_root(root_paths)
        }

        for ws in workspaces.iter() {
            let mut root_paths: Vec<VfsPath> = vec![];
            let mut entries = vec![];

            for buildfile in ws.buildfiles() {
                root_paths.push(VfsPath::from(buildfile.to_owned()));
                entries.push(buildfile.to_owned());
            }

            if !root_paths.is_empty() {
                let entry = vfs::loader::Entry::Files(entries);
                res.watch.push(res.load.len());
                res.load.push(entry);
                file_root_kinds.push(FileRootKind::Local);
                classifier.add_root(root_paths)
            }
        }

        // For virtual workspaces, the workspace root has no local PackageRoot, so
        // rust-analyzer.toml there would fall into a library file root and be
        // ignored. Load it explicitly via Entry::Files and register the workspace
        // root as a local file-set root so the file is classified as local.
        for ws_root in &uncovered_ws_roots {
            let ratoml_path = ws_root.join("rust-analyzer.toml");
            let root_paths = vec![VfsPath::from(ws_root.clone())];
            let entry = vfs::loader::Entry::Files(vec![ratoml_path]);
            res.watch.push(res.load.len());
            res.load.push(entry);
            file_root_kinds.push(FileRootKind::Local);
            classifier.add_root(root_paths);
        }

        if let Some(user_config_path) = user_config_dir_path {
            let ratoml_path = {
                let mut p = user_config_path.to_path_buf();
                p.push("rust-analyzer.toml");
                p
            };

            let root_paths = vec![VfsPath::from(ratoml_path.to_owned())];
            let entry = vfs::loader::Entry::Files(vec![ratoml_path]);

            res.watch.push(res.load.len());
            res.load.push(entry);
            file_root_kinds.push(FileRootKind::Local);
            classifier.add_root(root_paths)
        }

        let classifier = classifier.build();
        file_root_kinds.push(FileRootKind::Library);
        res.file_root_config = FileRootConfig { classifier, file_root_kinds };

        res
    }
}

#[derive(Debug)]
pub struct FileRootConfig {
    pub classifier: PathClassifier,
    pub file_root_kinds: Vec<FileRootKind>,
}

impl Default for FileRootConfig {
    fn default() -> Self {
        Self { classifier: PathClassifier::default(), file_root_kinds: vec![FileRootKind::Library] }
    }
}

impl FileRootConfig {
    pub fn registrations(
        &self,
        files: impl IntoIterator<Item = (File, VfsPath)>,
    ) -> Vec<FileRegistration> {
        files
            .into_iter()
            .map(|(file, path)| {
                let root_id = self.classifier.classify(&path);
                FileRegistration {
                    file,
                    root: FileRoot {
                        id: FileRootId(root_id as u32),
                        kind: self.file_root_kinds[root_id],
                    },
                }
            })
            .collect()
    }

    /// Returns whether `path` belongs to a library (non-local) file root, such as the
    /// sysroot sources or a cargo registry dependency.
    ///
    /// Paths that belong to no configured root are *not* considered library files, as
    /// files outside of any loaded workspace (for example scratch files) fall into the
    /// catch-all root despite being client-editable.
    pub fn path_is_library(&self, path: &VfsPath) -> bool {
        match self.classifier.classify_configured(path) {
            Some(idx) => self.file_root_kinds[idx] == FileRootKind::Library,
            None => false,
        }
    }

    /// Maps local file roots to their parent file roots by bytewise comparing root paths.
    /// If a local file root does not have a parent then it is not contained in this mapping.
    pub fn file_root_parent_map(&self) -> FxHashMap<FileRootId, FileRootId> {
        let roots = self.classifier.roots();

        let mut map = FxHashMap::default();

        // See https://github.com/rust-lang/rust-analyzer/issues/17409
        //
        // We can view the connections between roots as a graph. The problem is
        // that this graph may contain cycles, so when adding edges, it is necessary
        // to check whether it will lead to a cycle.
        //
        // Since we ensure that each node has at most one outgoing edge, we can
        // use a disjoint-set to maintain connectivity. If an edge's two nodes
        // belong to the same set, they are already connected.
        let mut dsu = FxHashMap::default();
        fn find_parent(dsu: &mut FxHashMap<u64, u64>, id: u64) -> u64 {
            if let Some(&parent) = dsu.get(&id) {
                let parent = find_parent(dsu, parent);
                dsu.insert(id, parent);
                parent
            } else {
                id
            }
        }

        for (idx, (root, root_id)) in roots.iter().enumerate() {
            if self.file_root_kinds[*root_id as usize] != FileRootKind::Local
                || map.contains_key(&FileRootId(*root_id as u32))
            {
                continue;
            }

            for (root2, root2_id) in roots[..idx].iter().rev() {
                if self.file_root_kinds[*root2_id as usize] == FileRootKind::Local
                    && root_id != root2_id
                    && root.starts_with(root2)
                {
                    // check if the edge will create a cycle
                    if find_parent(&mut dsu, *root_id) != find_parent(&mut dsu, *root2_id) {
                        map.insert(FileRootId(*root_id as u32), FileRootId(*root2_id as u32));
                        dsu.insert(*root_id, *root2_id);
                    }

                    break;
                }
            }
        }

        map
    }
}

/// Spawns the proc-macro server chosen by `load_config`, searching `workspaces`
/// for a sysroot server or toolchain as needed. Does not log the outcome;
/// callers that care about a single workspace's manifest log it themselves.
fn spawn_proc_macro_server(
    workspaces: &[ProjectWorkspace],
    extra_env: &FxHashMap<String, Option<String>>,
    load_config: &LoadCargoConfig,
) -> Option<Result<ProcMacroClient, ProcMacroLoadingError>> {
    match &load_config.with_proc_macro_server {
        ProcMacroServerChoice::Sysroot => {
            let srv_path = workspaces.iter().find_map(|ws| ws.find_sysroot_proc_macro_srv());
            srv_path.map(|it| {
                let toolchain = workspaces.iter().find_map(|ws| ws.toolchain.as_ref());
                it.and_then(|it| {
                    ProcMacroClient::spawn(
                        &it,
                        extra_env,
                        toolchain,
                        load_config.proc_macro_processes,
                    )
                    .map_err(Into::into)
                })
                .map_err(|e| {
                    ProcMacroLoadingError::ProcMacroSrvError(e.to_string().into_boxed_str())
                })
            })
        }
        ProcMacroServerChoice::Explicit(path) => {
            let toolchain = workspaces.iter().find_map(|ws| ws.toolchain.as_ref());
            Some(
                ProcMacroClient::spawn(
                    path,
                    extra_env,
                    toolchain,
                    load_config.proc_macro_processes,
                )
                .map_err(|e| {
                    ProcMacroLoadingError::ProcMacroSrvError(e.to_string().into_boxed_str())
                }),
            )
        }
        ProcMacroServerChoice::None => Some(Err(ProcMacroLoadingError::Disabled)),
    }
}

/// Flattens the `Option<Result<..>>` spawn outcome into the `Result` shape
/// `load_proc_macro` callers need, synthesizing the "missing a sysroot" error
/// for the `None` (no server configured) case.
fn flatten_proc_macro_server(
    proc_macro_server: &Option<Result<ProcMacroClient, ProcMacroLoadingError>>,
) -> Result<&ProcMacroClient, ProcMacroLoadingError> {
    match proc_macro_server {
        Some(Ok(it)) => Ok(it),
        Some(Err(e)) => {
            Err(ProcMacroLoadingError::ProcMacroSrvError(e.to_string().into_boxed_str()))
        }
        None => Err(ProcMacroLoadingError::ProcMacroSrvError(
            "proc-macro-srv is not running, workspace is missing a sysroot".into(),
        )),
    }
}

/// Merges each workspace's crate graph into one via `CrateGraphBuilder::extend`,
/// loading proc macros against `proc_macro_server` as it goes. `load` interns
/// crate-root and dependency file paths while `to_crate_graph` walks each workspace.
fn load_crate_graphs(
    workspaces: &[ProjectWorkspace],
    extra_env: &FxHashMap<String, Option<String>>,
    proc_macro_server: Result<&ProcMacroClient, ProcMacroLoadingError>,
    mut load: impl FnMut(&AbsPath) -> Option<File>,
) -> (CrateGraphBuilder, ProcMacrosBuilder) {
    let mut crate_graph = CrateGraphBuilder::default();
    let mut all_proc_macros = ProcMacrosBuilder::default();

    for ws in workspaces {
        let (other, mut crate_proc_macros) = ws.to_crate_graph(&mut load, extra_env);
        crate_graph.extend(other, &mut crate_proc_macros);

        for (crate_id, path) in crate_proc_macros {
            let loaded = path.map_or_else(Err, |(_, path)| {
                proc_macro_server
                    .as_ref()
                    .map_err(Clone::clone)
                    .and_then(|server| load_proc_macro(server, &path, &[]))
            });
            all_proc_macros.insert(crate_id, loaded);
        }
    }
    crate_graph.shrink_to_fit();

    (crate_graph, all_proc_macros)
}

/// Loads multiple workspaces into a single database, merging their crate graphs.
///
/// Mirrors the LSP's multi-workspace loading pattern (`ws_to_crate_graph` in `reload.rs`):
/// each workspace produces its own crate graph, which are merged via
/// `CrateGraphBuilder::extend`. File roots are unified via `ProjectFolders::new`.
/// Returns the computed `ProjectFolders` alongside the proc-macro client so callers
/// can set up their own file watching.
pub fn load_workspaces_into_db(
    workspaces: &[ProjectWorkspace],
    extra_env: &FxHashMap<String, Option<String>>,
    load_config: &LoadCargoConfig,
    db: &mut RootDatabase,
) -> (Option<ProcMacroClient>, ProjectFolders) {
    let (sender, receiver) = unbounded();
    let mut loader = vfs_notify::NotifyHandle::spawn(sender);

    tracing::debug!(?load_config, "LoadCargoConfig (multi-workspace)");

    let proc_macro_server = spawn_proc_macro_server(workspaces, extra_env, load_config);
    let proc_macro_server_ref = flatten_proc_macro_server(&proc_macro_server);

    let mut loaded_files = FxHashMap::default();
    let load = |path: &AbsPath| {
        let contents = loader.load_sync(path);
        let path = vfs::VfsPath::from(path.to_path_buf());
        let file_id = db.intern_file_path(path.clone());
        let exists = contents.is_some();
        loaded_files.insert(file_id, (path, contents));
        exists.then_some(file_id)
    };

    let (crate_graph, all_proc_macros) =
        load_crate_graphs(workspaces, extra_env, proc_macro_server_ref, load);

    // `project_folders.load` is cloned rather than moved so the full
    // `ProjectFolders` can still be returned to the caller below.
    let project_folders = ProjectFolders::new(workspaces, &[], None);
    loader.set_config(vfs::loader::Config {
        load: project_folders.load.clone(),
        watch: vec![],
        version: 0,
    });

    load_crate_graph_into_db(
        crate_graph,
        all_proc_macros,
        &project_folders.file_root_config,
        loaded_files,
        &receiver,
        db,
    );

    if load_config.prefill_caches {
        let all = ide_db::base_db::all_crates(db);
        prime_caches::parallel_prime_caches(db, &all, load_config.num_worker_threads, &|_| ());
    }

    (proc_macro_server.and_then(Result::ok), project_folders)
}

/// Rebuild `workspaces` into an already-initialized `db`, in place.
///
/// This is the reload counterpart to [`load_workspaces_into_db`]: rather than
/// starting from an empty database, it rebuilds the crate graph, file roots,
/// and proc macros against the existing file path inputs and applies them
/// incrementally, so salsa keeps the query caches it can. Returns the set of files
/// the crate graph was derived from, so a caller can decide when a later change
/// warrants another reload, along with the `ProjectFolders` computed for the new
/// file roots.
///
/// The proc-macro server spawned here is intentionally dropped at return: its
/// expanders keep it alive via their `Arc`'d pool, so the previous server's
/// process is only garbage-collected once those expanders are replaced.
pub fn reload_workspaces_into_db(
    workspaces: &[ProjectWorkspace],
    extra_env: &FxHashMap<String, Option<String>>,
    load_config: &LoadCargoConfig,
    db: &mut RootDatabase,
) -> (FxHashSet<vfs::VfsPath>, ProjectFolders) {
    let proc_macro_server = spawn_proc_macro_server(workspaces, extra_env, load_config);
    let proc_macro_server_ref = flatten_proc_macro_server(&proc_macro_server);

    let mut crate_graph_file_dependencies = FxHashSet::default();
    let mut loaded_files = FxHashMap::default();
    let load = |path: &AbsPath| {
        let contents = std::fs::read(path).ok();
        let vfs_path = vfs::VfsPath::from(path.to_path_buf());
        crate_graph_file_dependencies.insert(vfs_path.clone());
        let file_id = db.intern_file_path(vfs_path.clone());
        let exists = contents.is_some();
        loaded_files.insert(file_id, (vfs_path, contents));
        exists.then_some(file_id)
    };

    let (crate_graph, all_proc_macros) =
        load_crate_graphs(workspaces, extra_env, proc_macro_server_ref, load);

    let project_folders = ProjectFolders::new(workspaces, &[], None);
    let existing_files: Vec<_> = db
        .files()
        .into_iter()
        .filter_map(|file| db.file_path(file).map(|path| (file, path)))
        .collect();
    apply_loaded_workspace_to_db(
        crate_graph,
        all_proc_macros,
        &project_folders.file_root_config,
        loaded_files,
        existing_files,
        db,
    );

    (crate_graph_file_dependencies, project_folders)
}

/// Load the proc-macros for the given lib path, disabling all expanders whose names are in `ignored_macros`.
pub fn load_proc_macro(
    server: &ProcMacroClient,
    path: &AbsPath,
    ignored_macros: &[Box<str>],
) -> ProcMacroLoadResult {
    let res: Result<Vec<_>, _> = (|| {
        let dylib = MacroDylib::new(path.to_path_buf());
        let vec = server.load_dylib(dylib).map_err(|e| {
            ProcMacroLoadingError::ProcMacroSrvError(format!("{e}").into_boxed_str())
        })?;
        if vec.is_empty() {
            return Err(ProcMacroLoadingError::NoProcMacros);
        }
        Ok(vec
            .into_iter()
            .map(|expander| expander_to_proc_macro(expander, ignored_macros))
            .collect())
    })();
    match res {
        Ok(proc_macros) => {
            tracing::info!(
                "Loaded proc-macros for {path}: {:?}",
                proc_macros.iter().map(|it| it.name.clone()).collect::<Vec<_>>()
            );
            Ok(proc_macros)
        }
        Err(e) => {
            tracing::warn!("proc-macro loading for {path} failed: {e}");
            Err(e)
        }
    }
}

fn load_crate_graph_into_db(
    crate_graph: CrateGraphBuilder,
    proc_macros: ProcMacrosBuilder,
    file_root_config: &FileRootConfig,
    mut loaded_files: FxHashMap<File, (VfsPath, Option<Vec<u8>>)>,
    receiver: &Receiver<vfs::loader::Message>,
    db: &mut RootDatabase,
) {
    db.enable_proc_attr_macros();

    // Wait until the loader has loaded all roots.
    for task in receiver {
        match task {
            vfs::loader::Message::Progress { n_done, .. } => {
                if n_done == LoadingProgress::Finished {
                    break;
                }
            }
            vfs::loader::Message::Loaded { files } | vfs::loader::Message::Changed { files } => {
                let _p =
                    tracing::info_span!("load_cargo::load_crate_craph/LoadedChanged").entered();
                for (path, contents) in files {
                    let path = VfsPath::from(path);
                    let file_id = db.intern_file_path(path.clone());
                    loaded_files.insert(file_id, (path, contents));
                }
            }
        }
    }

    apply_loaded_workspace_to_db(
        crate_graph,
        proc_macros,
        file_root_config,
        loaded_files,
        std::iter::empty(),
        db,
    );
}

fn apply_loaded_workspace_to_db(
    crate_graph: CrateGraphBuilder,
    proc_macros: ProcMacrosBuilder,
    file_root_config: &FileRootConfig,
    loaded_files: FxHashMap<File, (VfsPath, Option<Vec<u8>>)>,
    existing_files: impl IntoIterator<Item = (File, VfsPath)>,
    db: &mut RootDatabase,
) {
    let mut analysis_change = ChangeWithProcMacros::default();

    db.enable_proc_attr_macros();

    for (file_id, (_, contents)) in &loaded_files {
        if let Some(contents) = contents
            && let Ok(text) = String::from_utf8(contents.clone())
        {
            analysis_change.change_file(*file_id, Some(text))
        }
    }
    let mut files = existing_files.into_iter().collect::<FxHashMap<_, _>>();
    for (&file_id, (path, contents)) in &loaded_files {
        if contents.is_some() {
            files.insert(file_id, path.clone());
        } else {
            files.remove(&file_id);
        }
    }
    analysis_change.set_indexed_files(file_root_config.registrations(files));

    analysis_change.set_crate_graph(crate_graph);
    analysis_change.set_proc_macros(proc_macros);

    db.apply_change(analysis_change);
}

fn expander_to_proc_macro(
    expander: proc_macro_api::ProcMacro,
    ignored_macros: &[Box<str>],
) -> ProcMacro {
    let name = expander.name();
    let kind = match expander.kind() {
        proc_macro_api::ProcMacroKind::CustomDerive => ProcMacroKind::CustomDerive,
        proc_macro_api::ProcMacroKind::Bang => ProcMacroKind::Bang,
        proc_macro_api::ProcMacroKind::Attr => ProcMacroKind::Attr,
    };
    let disabled = ignored_macros.iter().any(|replace| **replace == *name);
    ProcMacro {
        name: intern::Symbol::intern(name),
        kind,
        expander: sync::Arc::new(Expander(expander)),
        disabled,
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Expander(proc_macro_api::ProcMacro);

impl ProcMacroExpander for Expander {
    fn expand(
        &self,
        db: &dyn SourceDatabase,
        subtree: &tt::TopSubtree,
        attrs: Option<&tt::TopSubtree>,
        env: &Env,
        def_site: Span,
        call_site: Span,
        mixed_site: Span,
        current_dir: String,
    ) -> Result<tt::TopSubtree, ProcMacroExpansionError> {
        let cb = |req| match req {
            SubRequest::LocalFilePath { file_id } => {
                // SAFETY: Proc-macro span file IDs originate in this database.
                let file_id = unsafe { File::from_raw(file_id) };
                let name = db
                    .file_path(file_id)
                    .and_then(|path| path.as_path().map(|path| path.to_string()));

                Ok(SubResponse::LocalFilePathResult { name })
            }
            // Not incremental: requires full file text.
            SubRequest::SourceText { file_id, ast_id, start, end } => {
                let range = resolve_sub_span(
                    db,
                    file_id,
                    ast_id,
                    TextRange::new(TextSize::from(start), TextSize::from(end)),
                );
                let source = db.file_data(range.file_id.file(db)).text(db);
                let text = source
                    .get(usize::from(range.range.start())..usize::from(range.range.end()))
                    .map(ToOwned::to_owned);

                Ok(SubResponse::SourceTextResult { text })
            }
            // Not incremental: requires building line index.
            SubRequest::LineColumn { file_id, ast_id, offset } => {
                let range =
                    resolve_sub_span(db, file_id, ast_id, TextRange::empty(TextSize::from(offset)));
                let (line, column) = db
                    .line_column(range.file_id.file(db), range.range.start())
                    .map(|(line, col)| (line + 1, col + 1))
                    .unwrap_or((1, 1));
                // proc_macro::Span line/column are 1-based
                Ok(SubResponse::LineColumnResult { line, column })
            }
            SubRequest::FilePath { file_id } => {
                // SAFETY: Proc-macro span file IDs originate in this database.
                let file_id = unsafe { File::from_raw(file_id) };
                let name = db
                    .file_path(file_id)
                    .and_then(|path| path.as_path().map(|path| path.to_string()))
                    .unwrap_or_default();

                Ok(SubResponse::FilePathResult { name })
            }
            // Not incremental: requires global span resolution.
            SubRequest::ByteRange { file_id, ast_id, start, end } => {
                let range = resolve_sub_span(
                    db,
                    file_id,
                    ast_id,
                    TextRange::new(TextSize::from(start), TextSize::from(end)),
                );

                Ok(SubResponse::ByteRangeResult { range: range.range.into() })
            }
            SubRequest::SpanSource { file_id, ast_id, start, end, ctx } => {
                let span = Span {
                    range: TextRange::new(TextSize::from(start), TextSize::from(end)),
                    anchor: SpanAnchor {
                        file_id: span::EditionedFileId::from_raw(file_id),
                        ast_id: span::ErasedFileAstId::from_raw(ast_id),
                    },
                    // SAFETY: We only receive spans from the server. If someone mess up the communication UB can happen,
                    // but that will be their problem.
                    ctx: unsafe { SyntaxContext::from_u32(ctx) },
                };

                let mut current_span = span;
                let mut current_ctx = span.ctx;

                while let Some(macro_call_id) = current_ctx.outer_expn(db) {
                    let macro_call_loc = hir_expand::MacroCallId::from(macro_call_id).loc(db);

                    let call_site_file = macro_call_loc.kind.file_id();

                    let resolved = hir_expand::resolve_span(db, current_span);

                    current_ctx = macro_call_loc.ctxt;
                    current_span = Span {
                        range: resolved.range,
                        anchor: SpanAnchor {
                            file_id: resolved.file_id.span_file_id(db),
                            ast_id: span::ROOT_ERASED_FILE_AST_ID,
                        },
                        ctx: current_ctx,
                    };

                    if call_site_file.file_id().is_some() {
                        break;
                    }
                }

                let resolved = hir_expand::resolve_span(db, current_span);

                Ok(SubResponse::SpanSourceResult {
                    file_id: resolved.file_id.span_file_id(db).as_u32(),
                    ast_id: span::ROOT_ERASED_FILE_AST_ID.into_raw(),
                    start: u32::from(resolved.range.start()),
                    end: u32::from(resolved.range.end()),
                    ctx: current_span.ctx.into_u32(),
                })
            }
            SubRequest::SpanParent { file_id, ast_id, start, end, ctx } => {
                let span = Span {
                    range: TextRange::new(TextSize::from(start), TextSize::from(end)),
                    anchor: SpanAnchor {
                        file_id: span::EditionedFileId::from_raw(file_id),
                        ast_id: span::ErasedFileAstId::from_raw(ast_id),
                    },
                    // SAFETY: We only receive spans from the server. If someone mess up the communication UB can happen,
                    // but that will be their problem.
                    ctx: unsafe { SyntaxContext::from_u32(ctx) },
                };

                if let Some(macro_call_id) = span.ctx.outer_expn(db) {
                    let macro_call_loc = hir_expand::MacroCallId::from(macro_call_id).loc(db);

                    let call_site_file = macro_call_loc.kind.file_id();
                    let call_site_ast_id = macro_call_loc.kind.erased_ast_id();

                    if let Some(editioned_file_id) = call_site_file.file_id() {
                        let range = hir_expand::HirFileId::from(editioned_file_id)
                            .ast_id_map(db)
                            .get_erased(call_site_ast_id)
                            .text_range();

                        let parent_span = Some(ParentSpan {
                            file_id: editioned_file_id.span_file_id(db).as_u32(),
                            ast_id: span::ROOT_ERASED_FILE_AST_ID.into_raw(),
                            start: u32::from(range.start()),
                            end: u32::from(range.end()),
                            ctx: macro_call_loc.ctxt.into_u32(),
                        });

                        return Ok(SubResponse::SpanParentResult { parent_span });
                    }
                }

                Ok(SubResponse::SpanParentResult { parent_span: None })
            }
            // FIXME: implement this
            SubRequest::SpanJoin { .. } => Ok(SubResponse::SpanJoinResult { span: None }),
        };
        match self.0.expand(
            subtree.view(),
            attrs.map(|attrs| attrs.view()),
            env.clone().into(),
            def_site,
            call_site,
            mixed_site,
            current_dir,
            Some(&cb),
        ) {
            Ok(Ok(subtree)) => Ok(subtree),
            Ok(Err(err)) => Err(ProcMacroExpansionError::Panic(err)),
            Err(err) => Err(ProcMacroExpansionError::System(err.to_string())),
        }
    }

    fn eq_dyn(&self, other: &dyn ProcMacroExpander) -> bool {
        (other as &dyn Any).downcast_ref::<Self>() == Some(self)
    }
}

fn resolve_sub_span(
    db: &dyn SourceDatabase,
    file_id: u32,
    ast_id: u32,
    range: TextRange,
) -> hir_expand::FileRange {
    let ast_id = span::ErasedFileAstId::from_raw(ast_id);
    let editioned_file_id = span::EditionedFileId::from_raw(file_id);
    let span = Span {
        range,
        anchor: SpanAnchor { file_id: editioned_file_id, ast_id },
        ctx: SyntaxContext::root(editioned_file_id.edition()),
    };
    hir_expand::resolve_span(db, span)
}

#[cfg(test)]
mod tests {
    use ide_db::base_db::{
        FileRootKind::{Library, Local},
        all_crates,
    };
    use vfs::path_classifier::PathClassifierBuilder;

    use super::*;

    #[test]
    fn test_loading_rust_analyzer() {
        let cargo_toml_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("Cargo.toml");
        let cargo_toml_path = AbsPathBuf::assert_utf8(cargo_toml_path);
        let manifest = ProjectManifest::from_manifest_file(cargo_toml_path).unwrap();

        let cargo_config = CargoConfig { set_test: true, ..CargoConfig::default() };
        let load_cargo_config = LoadCargoConfig {
            load_out_dirs_from_check: false,
            with_proc_macro_server: ProcMacroServerChoice::None,
            prefill_caches: false,
            num_worker_threads: 1,
            proc_macro_processes: 1,
        };
        let workspace = ProjectWorkspace::load(manifest, &cargo_config, &|_| {}).unwrap();
        let (db, _proc_macro) =
            load_workspace(workspace, &cargo_config.extra_env, &load_cargo_config).unwrap();

        let n_crates = all_crates(&db).len();
        // RA has quite a few crates, but the exact count doesn't matter
        assert!(n_crates > 20);
    }

    #[test]
    fn unrelated_sources() {
        let mut builder = PathClassifierBuilder::default();
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/abc".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def".to_owned())]);
        let classifier = builder.build();
        let src = FileRootConfig { classifier, file_root_kinds: vec![Local, Local, Library] };
        let vc = src.file_root_parent_map().into_iter().collect::<Vec<_>>();

        assert_eq!(vc, vec![])
    }

    #[test]
    fn unrelated_source_sharing_dirname() {
        let mut builder = PathClassifierBuilder::default();
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/abc".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def/abc".to_owned())]);
        let classifier = builder.build();
        let src = FileRootConfig { classifier, file_root_kinds: vec![Local, Local, Library] };
        let vc = src.file_root_parent_map().into_iter().collect::<Vec<_>>();

        assert_eq!(vc, vec![])
    }

    #[test]
    fn basic_child_parent() {
        let mut builder = PathClassifierBuilder::default();
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/abc".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/abc/def".to_owned())]);
        let classifier = builder.build();
        let src = FileRootConfig { classifier, file_root_kinds: vec![Local, Local, Library] };
        let vc = src.file_root_parent_map().into_iter().collect::<Vec<_>>();

        assert_eq!(vc, vec![(FileRootId(1), FileRootId(0))])
    }

    #[test]
    fn basic_child_parent_with_unrelated_parents_sib() {
        let mut builder = PathClassifierBuilder::default();
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/abc".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def/abc".to_owned())]);
        let classifier = builder.build();
        let src =
            FileRootConfig { classifier, file_root_kinds: vec![Local, Local, Local, Library] };
        let vc = src.file_root_parent_map().into_iter().collect::<Vec<_>>();

        assert_eq!(vc, vec![(FileRootId(2), FileRootId(1))])
    }

    #[test]
    fn deep_sources_with_parent_missing() {
        let mut builder = PathClassifierBuilder::default();
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/abc".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/ghi".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def/abc".to_owned())]);
        let classifier = builder.build();
        let src =
            FileRootConfig { classifier, file_root_kinds: vec![Local, Local, Local, Library] };
        let vc = src.file_root_parent_map().into_iter().collect::<Vec<_>>();

        assert_eq!(vc, vec![])
    }

    #[test]
    fn ancestor_can_be_parent() {
        let mut builder = PathClassifierBuilder::default();
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/abc".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def/ghi/jkl".to_owned())]);
        let classifier = builder.build();
        let src =
            FileRootConfig { classifier, file_root_kinds: vec![Local, Local, Local, Library] };
        let vc = src.file_root_parent_map().into_iter().collect::<Vec<_>>();

        assert_eq!(vc, vec![(FileRootId(2), FileRootId(1))])
    }

    #[test]
    fn ancestor_can_be_parent_2() {
        let mut builder = PathClassifierBuilder::default();
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/abc".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def/ghi/jkl".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def/ghi/klm".to_owned())]);
        let classifier = builder.build();
        let src = FileRootConfig {
            classifier,
            file_root_kinds: vec![Local, Local, Local, Local, Library],
        };
        let mut vc = src.file_root_parent_map().into_iter().collect::<Vec<_>>();
        vc.sort_by_key(|x| x.0.0);

        assert_eq!(vc, vec![(FileRootId(2), FileRootId(1)), (FileRootId(3), FileRootId(1))])
    }

    #[test]
    fn non_locals_are_skipped() {
        let mut builder = PathClassifierBuilder::default();
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/abc".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def/ghi/jkl".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def/klm".to_owned())]);
        let classifier = builder.build();
        let src = FileRootConfig {
            classifier,
            file_root_kinds: vec![Local, Local, Library, Local, Library],
        };
        let mut vc = src.file_root_parent_map().into_iter().collect::<Vec<_>>();
        vc.sort_by_key(|x| x.0.0);

        assert_eq!(vc, vec![(FileRootId(3), FileRootId(1)),])
    }

    #[test]
    fn child_binds_ancestor_if_parent_nonlocal() {
        let mut builder = PathClassifierBuilder::default();
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/abc".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def/klm".to_owned())]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def/klm/jkl".to_owned())]);
        let classifier = builder.build();
        let src = FileRootConfig {
            classifier,
            file_root_kinds: vec![Local, Local, Library, Local, Library],
        };
        let mut vc = src.file_root_parent_map().into_iter().collect::<Vec<_>>();
        vc.sort_by_key(|x| x.0.0);

        assert_eq!(vc, vec![(FileRootId(3), FileRootId(1)),])
    }

    #[test]
    fn parents_with_identical_root_id() {
        let mut builder = PathClassifierBuilder::default();
        builder.add_root(vec![
            VfsPath::new_virtual_path("/ROOT/def".to_owned()),
            VfsPath::new_virtual_path("/ROOT/def/abc/def".to_owned()),
        ]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def/abc/def/ghi".to_owned())]);
        let classifier = builder.build();
        let src = FileRootConfig { classifier, file_root_kinds: vec![Local, Local, Library] };
        let mut vc = src.file_root_parent_map().into_iter().collect::<Vec<_>>();
        vc.sort_by_key(|x| x.0.0);

        assert_eq!(vc, vec![(FileRootId(1), FileRootId(0)),])
    }

    #[test]
    fn circular_reference() {
        let mut builder = PathClassifierBuilder::default();
        builder.add_root(vec![
            VfsPath::new_virtual_path("/ROOT/def".to_owned()),
            VfsPath::new_virtual_path("/ROOT/def/abc/def".to_owned()),
        ]);
        builder.add_root(vec![VfsPath::new_virtual_path("/ROOT/def/abc".to_owned())]);
        let classifier = builder.build();
        let src = FileRootConfig { classifier, file_root_kinds: vec![Local, Local, Library] };
        let mut vc = src.file_root_parent_map().into_iter().collect::<Vec<_>>();
        vc.sort_by_key(|x| x.0.0);

        assert_eq!(vc, vec![(FileRootId(1), FileRootId(0)),])
    }
}
