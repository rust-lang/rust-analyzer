//! A checkout and worktrees of it, loaded into one database.
//!
//! This is for tools that run rust-analyzer as a library and serve several copies of the same
//! repository at once, such as one checkout per agent. Loading each copy into a database of its
//! own analyzes everything once per copy. Here the base checkout is loaded once, and adding a
//! worktree only analyzes the packages that differ from the base checkout and what depends on
//! them, see [`crate::worktree`].
//!
//! A worktree need not be a git worktree: any directory with a copy of the base checkout will do.

use std::{cell::RefCell, sync::Arc};

use crossbeam_channel::{Receiver, unbounded};
use hir_expand::proc_macro::{ProcMacroLoadResult, ProcMacrosBuilder};
use ide_db::{
    ChangeWithProcMacros, FxHashMap, FxHashSet, RootDatabase,
    base_db::{ProcMacroLoadingError, ProcMacroPaths},
    prime_caches,
};
use proc_macro_api::ProcMacroClient;
use project_model::ProjectWorkspace;
use vfs::{
    AbsPath, AbsPathBuf, FileId, Vfs, VfsPath,
    loader::{Handle, LoadingProgress},
};

use crate::{
    LoadCargoConfig, ProjectFolders, SourceRootConfig, load_proc_macro, spawn_proc_macro_server,
    worktree::{
        self, DiskCache, Overlay, OverlayCrates, PulledInFile, SourceRoots, Sources,
        pulled_in_files, same_sources,
    },
};

/// The workspaces loaded into a database: a base checkout, and worktrees of it that share with
/// it the crates they have in common.
pub struct Worktrees {
    extra_env: FxHashMap<String, Option<String>>,
    /// The base checkout comes first.
    workspaces: Vec<ProjectWorkspace>,
    /// For each of `workspaces`, what it is an overlay of.
    overlays: Vec<Option<Overlay>>,
    source_root_config: SourceRootConfig,
    source_roots: Arc<SourceRoots>,
    overlay_crates: Arc<OverlayCrates>,
    pulled_in_files: FxHashMap<FileId, Vec<PulledInFile>>,
    disk_cache: RefCell<DiskCache>,
    /// The files whose text was set with [`Worktrees::set_file_text`]: what is on disk does not
    /// matter for them.
    files_in_memory: FxHashSet<VfsPath>,
    proc_macro_server: Option<Result<ProcMacroClient, ProcMacroLoadingError>>,
    /// The proc macros loaded so far, by the path of their dylib.
    proc_macros: FxHashMap<AbsPathBuf, ProcMacroLoadResult>,
    loader: Box<vfs_notify::NotifyHandle>,
    receiver: Receiver<vfs::loader::Message>,
    loader_config_version: u32,
}

impl Worktrees {
    /// Loads the base checkout.
    pub fn load(
        base: ProjectWorkspace,
        extra_env: &FxHashMap<String, Option<String>>,
        load_config: &LoadCargoConfig,
    ) -> anyhow::Result<(Worktrees, RootDatabase, Vfs)> {
        let lru_cap = std::env::var("RA_LRU_CAP").ok().and_then(|it| it.parse::<u16>().ok());
        let mut db = RootDatabase::new(lru_cap);
        db.enable_proc_attr_macros();
        let mut vfs = Vfs::default();

        let (sender, receiver) = unbounded();
        let loader = Box::new(vfs_notify::NotifyHandle::spawn(sender));
        let proc_macro_server = spawn_proc_macro_server(&base, extra_env, load_config);
        let mut this = Worktrees {
            extra_env: extra_env.clone(),
            workspaces: vec![base],
            overlays: vec![None],
            source_root_config: SourceRootConfig::default(),
            source_roots: Arc::new(SourceRoots::default()),
            overlay_crates: Arc::default(),
            pulled_in_files: FxHashMap::default(),
            disk_cache: RefCell::default(),
            files_in_memory: FxHashSet::default(),
            proc_macro_server,
            proc_macros: FxHashMap::default(),
            loader,
            receiver,
            loader_config_version: 0,
        };
        this.reload(&mut db, &mut vfs);

        if load_config.prefill_caches {
            let all = ide_db::base_db::all_crates(&db);
            prime_caches::parallel_prime_caches(&db, &all, load_config.num_worker_threads, &|_| ());
        }
        Ok((this, db, vfs))
    }

    /// Adds a worktree of the base checkout: `worktree` is the workspace loaded from it, and
    /// `overlay` tells where the two are.
    ///
    /// Adding the workspace of a worktree that is loaded already replaces it, which is how a
    /// change to its manifests is taken into account.
    pub fn add(
        &mut self,
        db: &mut RootDatabase,
        vfs: &mut Vfs,
        worktree: ProjectWorkspace,
        overlay: Overlay,
    ) {
        match self.overlays.iter().position(|it| it.as_ref() == Some(&overlay)) {
            Some(idx) => self.workspaces[idx] = worktree,
            None => {
                self.workspaces.push(worktree);
                self.overlays.push(Some(overlay));
            }
        }
        self.reload(db, vfs);
    }

    /// Removes the worktree at `worktree_root`. Returns whether there was one.
    pub fn remove(
        &mut self,
        db: &mut RootDatabase,
        vfs: &mut Vfs,
        worktree_root: &AbsPath,
    ) -> bool {
        let Some(idx) = self
            .overlays
            .iter()
            .position(|it| it.as_ref().is_some_and(|it| it.worktree_root == worktree_root))
        else {
            return false;
        };
        self.workspaces.remove(idx);
        self.overlays.remove(idx);
        // The files of the worktree are of no use anymore, drop their text.
        let files: Vec<VfsPath> = vfs
            .iter()
            .map(|(_, path)| path)
            .filter(|path| path.as_path().is_some_and(|path| path.starts_with(worktree_root)))
            .cloned()
            .collect();
        for path in files {
            self.files_in_memory.remove(&path);
            vfs.set_file_contents(path, None);
        }
        self.reload(db, vfs);
        true
    }

    /// Replaces the base checkout's workspace, after its manifests changed.
    pub fn set_base(&mut self, db: &mut RootDatabase, vfs: &mut Vfs, base: ProjectWorkspace) {
        self.workspaces[0] = base;
        self.reload(db, vfs);
    }

    /// Sets the text of the file at `path`, `None` if there is no such file anymore. From now on
    /// the file on disk is not looked at, until [`Worktrees::reload_file`].
    ///
    /// If this makes a package of a worktree differ from the base checkout, or the same again,
    /// the crates that are shared change accordingly.
    pub fn set_file_text(
        &mut self,
        db: &mut RootDatabase,
        vfs: &mut Vfs,
        path: &AbsPath,
        text: Option<String>,
    ) {
        let path = VfsPath::from(path.to_path_buf());
        self.files_in_memory.insert(path.clone());
        vfs.set_file_contents(path, text.map(String::into_bytes));
        self.apply_changes(db, vfs);
    }

    /// Reads the file at `path` from disk again.
    pub fn reload_file(&mut self, db: &mut RootDatabase, vfs: &mut Vfs, path: &AbsPath) {
        self.files_in_memory.remove(&VfsPath::from(path.to_path_buf()));
        let contents = self.loader.load_sync(path);
        vfs.set_file_contents(VfsPath::from(path.to_path_buf()), contents);
        self.apply_changes(db, vfs);
    }

    /// The worktrees that are loaded.
    pub fn overlays(&self) -> impl Iterator<Item = &Overlay> {
        self.overlays.iter().flatten()
    }

    /// What is needed to tell how a file of a worktree relates to the base checkout. This is a
    /// snapshot: it is valid for the state of the database it was taken at.
    pub fn views(&self) -> Views {
        Views {
            overlays: self.overlays().cloned().collect(),
            overlay_crates: Arc::clone(&self.overlay_crates),
            source_roots: Arc::clone(&self.source_roots),
        }
    }

    /// Loads the files of all workspaces and builds the crate graph.
    fn reload(&mut self, db: &mut RootDatabase, vfs: &mut Vfs) {
        let project_folders = ProjectFolders::new(&self.workspaces, &[], None);
        self.loader_config_version += 1;
        self.loader.set_config(vfs::loader::Config {
            load: project_folders.load,
            watch: vec![],
            version: self.loader_config_version,
        });
        self.source_root_config = project_folders.source_root_config;

        // wait until the loader has loaded all roots
        for task in &self.receiver {
            match task {
                vfs::loader::Message::Progress { n_done, config_version, .. } => {
                    if n_done == LoadingProgress::Finished
                        && config_version == self.loader_config_version
                    {
                        break;
                    }
                }
                vfs::loader::Message::Loaded { files }
                | vfs::loader::Message::Changed { files } => {
                    for (path, contents) in files {
                        let path = VfsPath::from(path);
                        if !self.files_in_memory.contains(&path) {
                            vfs.set_file_contents(path, contents);
                        }
                    }
                }
            }
        }

        let mut change = ChangeWithProcMacros::default();
        self.take_file_changes(vfs, &mut change);
        change.set_roots(self.source_root_config.partition(vfs));
        self.source_roots = Arc::new(SourceRoots::new(&self.source_root_config, vfs));
        self.set_crate_graph(vfs, &mut change);
        db.apply_change(change);
    }

    /// Brings the database up to date with the changes to the files in `vfs`.
    fn apply_changes(&mut self, db: &mut RootDatabase, vfs: &mut Vfs) {
        let mut change = ChangeWithProcMacros::default();
        let (changed_files, created_or_deleted) = self.take_file_changes(vfs, &mut change);
        if changed_files.is_empty() {
            return;
        }
        if created_or_deleted {
            change.set_roots(self.source_root_config.partition(vfs));
            self.source_roots = Arc::new(SourceRoots::new(&self.source_root_config, vfs));
        }
        if self.sharing_is_stale(vfs, &changed_files) {
            self.set_crate_graph(vfs, &mut change);
        }
        db.apply_change(change);
    }

    /// Moves the changes to the files in `vfs` into `change`. Returns the files that changed and
    /// whether any was created or deleted.
    fn take_file_changes(
        &mut self,
        vfs: &mut Vfs,
        change: &mut ChangeWithProcMacros,
    ) -> (Vec<FileId>, bool) {
        let mut changed_files = Vec::new();
        let mut created_or_deleted = false;
        for (file_id, file) in vfs.take_changes() {
            changed_files.push(file_id);
            created_or_deleted |= file.is_created_or_deleted();
            let text = match file.change {
                vfs::Change::Create(bytes, _) | vfs::Change::Modify(bytes, _) => {
                    String::from_utf8(bytes).ok()
                }
                vfs::Change::Delete => None,
            };
            let is_rust_file = vfs
                .file_path(file_id)
                .name_and_extension()
                .is_some_and(|(_, ext)| ext == Some("rs"));
            match text.as_deref().filter(|_| is_rust_file).map(pulled_in_files) {
                Some(pulled_in) if !pulled_in.is_empty() => {
                    self.pulled_in_files.insert(file_id, pulled_in);
                }
                _ => {
                    self.pulled_in_files.remove(&file_id);
                }
            }
            change.change_file(file_id, text);
        }
        (changed_files, created_or_deleted)
    }

    /// Whether a change to `changed_files` made the sources of a crate of a worktree differ
    /// from, or become the same as, those of the base checkout.
    fn sharing_is_stale(&self, vfs: &Vfs, changed_files: &[FileId]) -> bool {
        let roots = &*self.source_roots;
        let sources = Sources {
            vfs,
            roots,
            pulled_in_files: &self.pulled_in_files,
            disk_cache: &self.disk_cache,
        };
        self.overlay_crates.iter().any(|(&(worktree_file, base_file), krate)| {
            // The files that a crate pulls in by path can be anywhere.
            let touched = roots.pulls_in_files(&self.pulled_in_files, worktree_file)
                || changed_files.iter().any(|&file| {
                    roots.in_same_root(file, worktree_file) || roots.in_same_root(file, base_file)
                });
            let overlay = vfs
                .file_path(worktree_file)
                .as_path()
                .and_then(|path| self.overlays().find(|it| path.starts_with(&it.worktree_root)));
            touched
                && overlay.is_some_and(|overlay| {
                    same_sources(&sources, overlay, worktree_file, base_file) != krate.same_sources
                })
        })
    }

    /// Builds the crate graph of the workspaces into `change`.
    fn set_crate_graph(&mut self, vfs: &Vfs, change: &mut ChangeWithProcMacros) {
        let mut overlay_crates = OverlayCrates::default();
        // The files that are not loaded are compared anew each time the crate graph is built.
        self.disk_cache.borrow_mut().clear();
        let sources = Sources {
            vfs,
            roots: &self.source_roots,
            pulled_in_files: &self.pulled_in_files,
            disk_cache: &self.disk_cache,
        };
        let load = |path: &AbsPath| {
            vfs.file_id(&VfsPath::from(path.to_path_buf())).and_then(|(file_id, excluded)| {
                (excluded == vfs::FileExcluded::No).then_some(file_id)
            })
        };
        let (crate_graph, proc_macro_paths) = worktree::crate_graph(
            &self.workspaces,
            &self.extra_env,
            load,
            &self.overlays,
            Some(&sources),
            &mut overlay_crates,
        );
        self.overlay_crates = Arc::new(overlay_crates);

        let proc_macros = self.load_proc_macros(proc_macro_paths);
        change.set_crate_graph(crate_graph);
        change.set_proc_macros(proc_macros);
    }

    fn load_proc_macros(&mut self, proc_macro_paths: Vec<ProcMacroPaths>) -> ProcMacrosBuilder {
        let server = match &self.proc_macro_server {
            Some(Ok(server)) => Ok(server),
            Some(Err(e)) => Err(e.clone()),
            None => Err(ProcMacroLoadingError::ProcMacroSrvError(
                "proc-macro-srv is not running, workspace is missing a sysroot".into(),
            )),
        };
        let loaded = &mut self.proc_macros;
        proc_macro_paths
            .into_iter()
            .flatten()
            .map(|(crate_id, path)| {
                let macros = path.and_then(|(_, path)| {
                    let server = server.as_ref().map_err(Clone::clone)?;
                    loaded
                        .entry(path)
                        .or_insert_with_key(|path| load_proc_macro(server, path, &[]))
                        .clone()
                });
                (crate_id, macros)
            })
            .collect()
    }
}

/// How the files of the worktrees relate to the base checkout, as of some state of the database.
///
/// A request about a file of a worktree is answered in three steps: analyze
/// [`Views::analyzed_file`] instead of the file, leave out the results that are not
/// [`Views::in_view`], and report the rest at [`Views::path_in_view`].
#[derive(Clone)]
pub struct Views {
    overlays: Arc<[Overlay]>,
    overlay_crates: Arc<OverlayCrates>,
    source_roots: Arc<SourceRoots>,
}

impl Views {
    /// The worktree that `path` is in.
    pub fn overlay_of(&self, path: &AbsPath) -> Option<&Overlay> {
        self.overlays.iter().find(|it| path.starts_with(&it.worktree_root))
    }

    /// The file to analyze for `file`: a file of a crate that a worktree shares with its base
    /// checkout stands for the same file of the base checkout.
    ///
    /// `is_in_a_crate` tells whether a file is a module of some crate, as
    /// `ide::Analysis::crates_for` does.
    pub fn analyzed_file(
        &self,
        vfs: &Vfs,
        file: FileId,
        is_in_a_crate: impl FnOnce(FileId) -> bool,
    ) -> FileId {
        if self.overlays.is_empty() {
            return file;
        }
        match worktree::shared_base_file(
            vfs,
            &self.source_roots,
            &self.overlays,
            &self.overlay_crates,
            file,
            false,
        ) {
            Some(base_file) if !is_in_a_crate(file) => base_file,
            _ => file,
        }
    }

    /// Whether something found in `file` is of interest to who works in `view`, the base
    /// checkout if `None`.
    ///
    /// A worktree does not want to hear about the base checkout's copy of a crate that it has
    /// its own version of, nor about other worktrees. The base checkout does not want to hear
    /// about worktrees.
    pub fn in_view(
        &self,
        vfs: &Vfs,
        view: Option<&Overlay>,
        file: FileId,
        is_in_a_crate: impl FnOnce(FileId) -> bool,
    ) -> bool {
        let Some(path) = vfs.file_path(file).as_path() else {
            return true;
        };
        let worktree_of_file = self.overlay_of(path);
        let Some(view) = view else {
            return worktree_of_file.is_none();
        };
        if let Some(worktree_of_file) = worktree_of_file {
            return worktree_of_file == view;
        }
        let worktree_file = view
            .to_worktree(path)
            .and_then(|path| vfs.file_id(&VfsPath::from(path)))
            .map(|(file, _)| file);
        match worktree_file {
            Some(worktree_file) => !is_in_a_crate(worktree_file),
            None => true,
        }
    }

    /// The path of `file` for who works in `view`: the files of the base checkout are known
    /// to a worktree at its own paths.
    pub fn path_in_view(&self, vfs: &Vfs, view: Option<&Overlay>, file: FileId) -> VfsPath {
        let path = vfs.file_path(file);
        let worktree_path = view
            .and_then(|view| view.to_worktree(path.as_path()?))
            .map(VfsPath::from)
            .filter(|path| vfs.file_id(path).is_some());
        worktree_path.unwrap_or_else(|| path.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use ide_db::base_db::{SourceDatabase, all_crates, relevant_crates};
    use project_model::{CargoConfig, ProjectManifest};

    use super::*;
    use crate::ProcMacroServerChoice;

    /// A directory with a base checkout at `base` and copies of it next to it.
    struct Checkouts {
        dir: PathBuf,
        worktrees: Worktrees,
        db: RootDatabase,
        vfs: Vfs,
    }

    const CORE_LIB: &str = "pub fn answer() -> u32 { 42 }\n";
    const APP: &str = "pub fn run() -> u32 { core_lib::answer() }\n";

    impl Checkouts {
        fn new() -> Checkouts {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "ra-worktrees-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            let dir = fs::canonicalize({
                fs::create_dir_all(&dir).unwrap();
                dir
            })
            .unwrap();
            write_checkout(&dir.join("base"), CORE_LIB, APP);
            let (worktrees, db, vfs) = Worktrees::load(
                load_workspace(&dir.join("base")),
                &FxHashMap::default(),
                &LoadCargoConfig {
                    load_out_dirs_from_check: false,
                    with_proc_macro_server: ProcMacroServerChoice::None,
                    prefill_caches: false,
                    num_worker_threads: 1,
                    proc_macro_processes: 1,
                },
            )
            .unwrap();
            Checkouts { dir, worktrees, db, vfs }
        }

        fn path(&self, path: &str) -> AbsPathBuf {
            AbsPathBuf::assert_utf8(self.dir.join(path))
        }

        fn overlay(&self, name: &str) -> Overlay {
            Overlay { worktree_root: self.path(name), base_root: self.path("base") }
        }

        /// Adds a copy of the base checkout at `name` with these sources.
        fn add_worktree(&mut self, name: &str, core_lib: &str, app: &str) -> Overlay {
            write_checkout(&self.dir.join(name), core_lib, app);
            let overlay = self.overlay(name);
            self.worktrees.add(
                &mut self.db,
                &mut self.vfs,
                load_workspace(&self.dir.join(name)),
                overlay.clone(),
            );
            overlay
        }

        fn set_file_text(&mut self, path: &str, text: &str) {
            let path = self.path(path);
            self.worktrees.set_file_text(&mut self.db, &mut self.vfs, &path, Some(text.to_owned()));
        }

        fn crates(&self) -> usize {
            all_crates(&self.db).len()
        }

        fn file(&self, path: &str) -> FileId {
            self.vfs.file_id(&VfsPath::from(self.path(path))).unwrap().0
        }

        fn text(&self, path: &str) -> String {
            self.db.file_text(self.file(path)).text(&self.db).to_string()
        }

        fn is_in_a_crate(&self, file: FileId) -> bool {
            relevant_crates(&self.db, file).iter().any(|&krate| {
                hir::crate_def_map(&self.db, krate)
                    .modules_for_file(&self.db, file)
                    .next()
                    .is_some()
            })
        }

        fn analyzed_file(&self, path: &str) -> FileId {
            let file = self.file(path);
            self.worktrees.views().analyzed_file(&self.vfs, file, |file| self.is_in_a_crate(file))
        }

        fn in_view(&self, view: Option<&Overlay>, path: &str) -> bool {
            let file = self.file(path);
            self.worktrees.views().in_view(&self.vfs, view, file, |file| self.is_in_a_crate(file))
        }
    }

    impl Drop for Checkouts {
        fn drop(&mut self) {
            _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn write_checkout(root: &std::path::Path, core_lib: &str, app: &str) {
        let files = [
            ("Cargo.toml", "[workspace]\nmembers = [\"core_lib\", \"app\"]\nresolver = \"2\"\n"),
            ("core_lib/Cargo.toml", "[package]\nname = \"core_lib\"\nversion = \"0.0.0\"\n"),
            ("core_lib/src/lib.rs", core_lib),
            (
                "app/Cargo.toml",
                "[package]\nname = \"app\"\nversion = \"0.0.0\"\n\n[dependencies]\ncore_lib = { path = \"../core_lib\" }\n",
            ),
            ("app/src/lib.rs", app),
        ];
        for (path, text) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
    }

    fn load_workspace(root: &std::path::Path) -> ProjectWorkspace {
        let manifest = AbsPathBuf::assert_utf8(root.join("Cargo.toml"));
        let manifest = ProjectManifest::from_manifest_file(manifest).unwrap();
        ProjectWorkspace::load(manifest, &CargoConfig::default(), &|_| {}).unwrap()
    }

    #[test]
    fn identical_worktree_adds_no_crates() {
        let mut checkouts = Checkouts::new();
        let base_crates = checkouts.crates();
        assert_eq!(base_crates, 2);

        checkouts.add_worktree("wt", CORE_LIB, APP);
        assert_eq!(checkouts.crates(), base_crates);
        assert_eq!(
            checkouts.analyzed_file("wt/app/src/lib.rs"),
            checkouts.file("base/app/src/lib.rs")
        );
    }

    #[test]
    fn worktree_adds_what_differs_and_what_depends_on_it() {
        let mut checkouts = Checkouts::new();

        // `app` differs: only it is analyzed for the worktree
        checkouts.add_worktree("leaf", CORE_LIB, "pub fn run() -> u32 { 1 }\n");
        assert_eq!(checkouts.crates(), 3);
        assert_eq!(
            checkouts.analyzed_file("leaf/core_lib/src/lib.rs"),
            checkouts.file("base/core_lib/src/lib.rs")
        );
        assert_eq!(
            checkouts.analyzed_file("leaf/app/src/lib.rs"),
            checkouts.file("leaf/app/src/lib.rs")
        );

        // `core_lib` differs: `app` depends on it, so both are analyzed for the worktree
        checkouts.add_worktree("core", "pub fn answer() -> u64 { 42 }\n", APP);
        assert_eq!(checkouts.crates(), 5);
        assert_eq!(
            checkouts.analyzed_file("core/app/src/lib.rs"),
            checkouts.file("core/app/src/lib.rs")
        );
    }

    #[test]
    fn views_keep_checkouts_apart() {
        let mut checkouts = Checkouts::new();
        let leaf = checkouts.add_worktree("leaf", CORE_LIB, "pub fn run() -> u32 { 1 }\n");
        let other = checkouts.add_worktree("other", CORE_LIB, "pub fn run() -> u32 { 2 }\n");

        // The worktree sees its own crate, the crate it shares, but not the base's copy of
        // what it has its own version of, nor another worktree
        assert!(checkouts.in_view(Some(&leaf), "leaf/app/src/lib.rs"));
        assert!(checkouts.in_view(Some(&leaf), "base/core_lib/src/lib.rs"));
        assert!(!checkouts.in_view(Some(&leaf), "base/app/src/lib.rs"));
        assert!(!checkouts.in_view(Some(&leaf), "other/app/src/lib.rs"));
        // The base checkout sees itself only
        assert!(checkouts.in_view(None, "base/app/src/lib.rs"));
        assert!(!checkouts.in_view(None, "leaf/app/src/lib.rs"));

        // A shared file is reported at the worktree's path
        let views = checkouts.worktrees.views();
        let shared = checkouts.file("base/core_lib/src/lib.rs");
        assert_eq!(
            views.path_in_view(&checkouts.vfs, Some(&other), shared),
            VfsPath::from(checkouts.path("other/core_lib/src/lib.rs"))
        );
        assert_eq!(
            views.path_in_view(&checkouts.vfs, None, shared),
            VfsPath::from(checkouts.path("base/core_lib/src/lib.rs"))
        );
    }

    #[test]
    fn editing_a_shared_package_stops_sharing_it() {
        let mut checkouts = Checkouts::new();
        checkouts.add_worktree("wt", CORE_LIB, APP);
        assert_eq!(checkouts.crates(), 2);

        checkouts.set_file_text("wt/core_lib/src/lib.rs", "pub fn answer() -> u64 { 42 }\n");
        assert_eq!(checkouts.crates(), 4);
        assert_eq!(checkouts.text("wt/core_lib/src/lib.rs"), "pub fn answer() -> u64 { 42 }\n");
        assert_eq!(checkouts.text("base/core_lib/src/lib.rs"), CORE_LIB);
        assert_eq!(
            checkouts.analyzed_file("wt/core_lib/src/lib.rs"),
            checkouts.file("wt/core_lib/src/lib.rs")
        );

        // Undoing the edit shares it again
        checkouts.set_file_text("wt/core_lib/src/lib.rs", CORE_LIB);
        assert_eq!(checkouts.crates(), 2);
    }

    #[test]
    fn editing_the_base_checkout_stops_sharing_with_it() {
        let mut checkouts = Checkouts::new();
        checkouts.add_worktree("wt", CORE_LIB, APP);
        assert_eq!(checkouts.crates(), 2);

        checkouts.set_file_text("base/app/src/lib.rs", "pub fn run() -> u32 { 7 }\n");
        assert_eq!(checkouts.crates(), 3);
        assert_eq!(checkouts.text("wt/app/src/lib.rs"), APP);
    }

    #[test]
    fn removing_a_worktree_removes_its_crates() {
        let mut checkouts = Checkouts::new();
        let overlay = checkouts.add_worktree("wt", "pub fn answer() -> u64 { 42 }\n", APP);
        assert_eq!(checkouts.crates(), 4);

        assert!(checkouts.worktrees.remove(
            &mut checkouts.db,
            &mut checkouts.vfs,
            &overlay.worktree_root
        ));
        assert_eq!(checkouts.crates(), 2);
        assert!(!checkouts.worktrees.remove(
            &mut checkouts.db,
            &mut checkouts.vfs,
            &overlay.worktree_root
        ));
    }

    #[test]
    fn text_set_in_memory_survives_adding_a_worktree() {
        let mut checkouts = Checkouts::new();
        checkouts.add_worktree("wt", CORE_LIB, APP);
        checkouts.set_file_text("wt/app/src/lib.rs", "pub fn run() -> u32 { 3 }\n");
        assert_eq!(checkouts.crates(), 3);

        // Adding another worktree loads the files from disk again
        checkouts.add_worktree("other", CORE_LIB, APP);
        assert_eq!(checkouts.text("wt/app/src/lib.rs"), "pub fn run() -> u32 { 3 }\n");
        assert_eq!(checkouts.crates(), 3);

        let path = checkouts.path("wt/app/src/lib.rs");
        checkouts.worktrees.reload_file(&mut checkouts.db, &mut checkouts.vfs, &path);
        assert_eq!(checkouts.text("wt/app/src/lib.rs"), APP);
        assert_eq!(checkouts.crates(), 2);
    }
}
