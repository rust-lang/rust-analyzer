//! A checkout and worktrees of it, loaded into one database.
//!
//! This is for tools that run rust-analyzer as a library and serve several copies of the same
//! repository at once, such as one checkout per agent. Loading each copy into a database of its
//! own analyzes everything once per copy. Here the base checkout is loaded once, and adding a
//! worktree only analyzes the packages that differ from the base checkout and what depends on
//! them, see [`crate::worktree`]. The files of the packages that a worktree shares with the base
//! checkout are not even loaded.
//!
//! A worktree need not be a git worktree: any directory with a copy of the base checkout will do.

use std::{cell::RefCell, fs, sync::Arc, time::SystemTime};

use crossbeam_channel::{Receiver, unbounded};
use hir_expand::proc_macro::{ProcMacroLoadResult, ProcMacrosBuilder};
use ide_db::{
    ChangeWithProcMacros, FxHashMap, FxHashSet, RootDatabase,
    base_db::{ProcMacroLoadingError, ProcMacroPaths},
    prime_caches,
};
use itertools::Itertools;
use proc_macro_api::ProcMacroClient;
use project_model::{Package, ProjectWorkspace, ProjectWorkspaceKind};
use rustc_hash::FxHasher;
use stdx::hash_once;
use vfs::{
    AbsPath, AbsPathBuf, FileId, Vfs, VfsPath,
    loader::{Directories, Entry, Handle, LoadingProgress},
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
    /// For each of `workspaces`, the directories of its packages that are not loaded, because
    /// they are the same as in the base checkout, whose files stand in for theirs.
    not_loaded: Vec<FxHashSet<AbsPathBuf>>,
    /// The directories whose files are loaded.
    loaded: FxHashSet<AbsPathBuf>,
    source_root_config: SourceRootConfig,
    source_roots: Arc<SourceRoots>,
    overlay_crates: Arc<OverlayCrates>,
    pulled_in_files: FxHashMap<FileId, Vec<PulledInFile>>,
    disk_cache: RefCell<DiskCache>,
    /// The files whose text was set with [`Worktrees::set_file_text`]: what is on disk does not
    /// matter for them.
    files_in_memory: FxHashSet<VfsPath>,
    proc_macro_server: Option<Result<ProcMacroClient, ProcMacroLoadingError>>,
    /// The proc macros of the crates in the crate graph, by the path of their dylib, with the
    /// time the dylib was modified at when they were loaded.
    proc_macros: FxHashMap<AbsPathBuf, (Option<SystemTime>, ProcMacroLoadResult)>,
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
            not_loaded: vec![FxHashSet::default()],
            loaded: FxHashSet::default(),
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
        let not_loaded = self.same_packages(vfs, &worktree, &overlay);
        match self.overlays.iter().position(|it| it.as_ref() == Some(&overlay)) {
            Some(idx) => {
                self.workspaces[idx] = worktree;
                // What is loaded stays loaded, the file comparison takes care of it.
                self.not_loaded[idx].retain(|dir| not_loaded.contains(dir));
            }
            None => {
                self.workspaces.push(worktree);
                self.overlays.push(Some(overlay));
                self.not_loaded.push(not_loaded);
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
        self.not_loaded.remove(idx);
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
        self.loaded.retain(|dir| !dir.starts_with(worktree_root));
        self.reload(db, vfs);
        true
    }

    /// Replaces the base checkout's workspace, after its manifests changed.
    pub fn set_base(&mut self, db: &mut RootDatabase, vfs: &mut Vfs, base: ProjectWorkspace) {
        self.workspaces[0] = base;
        // The crates of the worktrees are compared with other crates now. Their files that are
        // not loaded cannot be compared with anything, so load them all first.
        self.not_loaded.iter_mut().for_each(FxHashSet::clear);
        self.reload(db, vfs);
        self.unload_shared(db, vfs);
    }

    /// Drops the files of the packages of the worktrees that are loaded although they are the
    /// same as in the base checkout, as happens when an edit is undone.
    ///
    /// This compares the worktrees with the base checkout on disk, as adding them does.
    pub fn unload_shared(&mut self, db: &mut RootDatabase, vfs: &mut Vfs) {
        let mut unloads = false;
        for idx in 0..self.workspaces.len() {
            let Some(overlay) = self.overlays[idx].clone() else { continue };
            let same = self.same_packages(vfs, &self.workspaces[idx], &overlay);
            let unloaded: Vec<&AbsPathBuf> =
                same.iter().filter(|dir| !self.not_loaded[idx].contains(*dir)).collect();
            let files: Vec<VfsPath> = vfs
                .iter()
                .map(|(_, path)| path)
                .filter(|path| {
                    path.as_path()
                        .is_some_and(|path| unloaded.iter().any(|dir| path.starts_with(dir)))
                })
                .cloned()
                .collect();
            for path in files {
                vfs.set_file_contents(path, None);
            }
            self.loaded.retain(|dir| !unloaded.iter().any(|unloaded| dir.starts_with(unloaded)));
            unloads |= !unloaded.is_empty();
            self.not_loaded[idx] = same;
        }
        if unloads {
            self.reload(db, vfs);
        }
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
        self.set_file_contents(db, vfs, path, text.map(String::into_bytes), true);
    }

    /// Reads the file at `path` from disk again.
    pub fn reload_file(&mut self, db: &mut RootDatabase, vfs: &mut Vfs, path: &AbsPath) {
        let contents = self.loader.load_sync(path);
        self.set_file_contents(db, vfs, path, contents, false);
    }

    fn set_file_contents(
        &mut self,
        db: &mut RootDatabase,
        vfs: &mut Vfs,
        path: &AbsPath,
        contents: Option<Vec<u8>>,
        in_memory: bool,
    ) {
        let vfs_path = VfsPath::from(path.to_path_buf());
        let hash = contents.as_deref().map(hash_once::<FxHasher>);
        // A file that is not loaded has the contents of its counterpart in the base checkout.
        let (stands_in, not_loaded) = self.stand_in(vfs, path);
        let current = stands_in.and_then(|file| vfs.content_hash(file));
        if not_loaded && hash == current {
            // Still the same as in the base checkout, there is nothing to do or to remember.
            return;
        }
        if in_memory {
            self.files_in_memory.insert(vfs_path.clone());
        } else {
            self.files_in_memory.remove(&vfs_path);
        }
        // The packages that are not loaded and have this file, or its counterpart in the base
        // checkout, are not the same as in the base checkout anymore.
        let loads_more = self.load_packages_of(path);
        vfs.set_file_contents(vfs_path, contents);
        if loads_more {
            self.reload(db, vfs);
        } else {
            self.apply_changes(db, vfs);
        }
    }

    /// The workspace of the base checkout.
    pub fn base(&self) -> &ProjectWorkspace {
        &self.workspaces[0]
    }

    /// The workspace of a copy of the base checkout, without loading it, which is most of the
    /// time it takes to add a worktree. Returns `None` if it has to be loaded: the manifests,
    /// the lock file, the cargo configuration or the toolchain file of the copy are not the
    /// same as in the base checkout.
    pub fn workspace_of_copy(&self, overlay: &Overlay) -> Option<ProjectWorkspace> {
        let base = self.base();
        let ProjectWorkspaceKind::Cargo { cargo, .. } = &base.kind else {
            return None;
        };
        // A path that leads out of the checkout, as a path dependency or a patch can have, leads
        // somewhere else from the copy, and so can the cargo configuration above the two.
        let leads_out = cargo
            .packages()
            .any(|pkg| cargo[pkg].is_local && !cargo[pkg].manifest.starts_with(&overlay.base_root));
        // Cargo and rustup both look for their configuration in every directory above.
        let found_above =
            [".cargo/config.toml", ".cargo/config", "rust-toolchain.toml", "rust-toolchain"];
        let configs_above = |root: &AbsPath| -> Vec<AbsPathBuf> {
            std::iter::successors(root.parent(), |dir| dir.parent())
                .flat_map(|dir| found_above.map(|file| dir.join(file)))
                .filter(|config| fs::metadata(config).is_ok())
                .collect()
        };
        if leads_out || configs_above(&overlay.base_root) != configs_above(&overlay.worktree_root) {
            return None;
        }
        let workspace_root = base.workspace_root();
        // The workspace need not be at the root of the checkout.
        let configs_in_checkout = std::iter::successors(Some(workspace_root), |dir| dir.parent())
            .take_while(|dir| dir.starts_with(&overlay.base_root))
            .flat_map(|dir| found_above.map(|file| dir.join(file)));
        let manifests = cargo
            .packages()
            .map(|pkg| AbsPath::to_path_buf(&cargo[pkg].manifest))
            .chain([AbsPath::to_path_buf(cargo.manifest_path())])
            .chain([workspace_root.join("Cargo.lock")])
            .chain(configs_in_checkout)
            .filter(|path| path.starts_with(&overlay.base_root));
        let same = |path: &AbsPath| {
            overlay.to_worktree(path).is_some_and(|copy| fs::read(path).ok() == fs::read(copy).ok())
        };
        for path in manifests {
            if !same(&path) {
                return None;
            }
        }
        // Cargo also finds packages and targets by looking at what files there are.
        if discovered_by_cargo(&overlay.base_root) != discovered_by_cargo(&overlay.worktree_root) {
            return None;
        }
        base.rerooted(&overlay.base_root, &overlay.worktree_root)
    }

    /// Lets `worktree`, which is about to be [added](Worktrees::add), take over the outputs of
    /// the build scripts of the base checkout instead of running its own, which is what makes
    /// adding a worktree slow.
    ///
    /// Only the packages that are the same as in the base checkout, and depend only on such
    /// packages, get them. Returns the directories of the other packages of the worktree: their
    /// crates have no build script output and no proc macros until the build scripts of the
    /// worktree are run and its workspace is added again.
    pub fn inherit_build_scripts(
        &self,
        vfs: &Vfs,
        worktree: &mut ProjectWorkspace,
        overlay: &Overlay,
    ) -> Vec<AbsPathBuf> {
        let same = self.same_packages(vfs, worktree, overlay);
        // A library is the same package for both, if it is built with the same features.
        let same_features = self.same_features(worktree, overlay);
        let is_same = |dir: &AbsPath| {
            same_features(dir) && (!dir.starts_with(&overlay.worktree_root) || same.contains(dir))
        };
        worktree.inherit_build_scripts(self.base(), &is_same);
        match &worktree.kind {
            ProjectWorkspaceKind::Cargo { cargo, .. } => cargo
                .packages()
                .map(|pkg| cargo[pkg].manifest.parent())
                .filter(|dir| !is_same(dir))
                .map(AbsPath::to_path_buf)
                .sorted()
                .collect(),
            _ => Vec::new(),
        }
    }

    /// The worktrees that are loaded.
    pub fn overlays(&self) -> impl Iterator<Item = &Overlay> {
        self.overlays.iter().flatten()
    }

    /// What is needed to tell how a file of a worktree relates to the base checkout. This is a
    /// snapshot: it is valid for the state of the database it was taken at.
    pub fn views(&self) -> Views {
        Views {
            overlays: self
                .overlays
                .iter()
                .zip(&self.not_loaded)
                .filter_map(|(overlay, not_loaded)| {
                    Some((overlay.clone()?, not_loaded.iter().cloned().collect()))
                })
                .collect(),
            overlay_crates: Arc::clone(&self.overlay_crates),
            source_roots: Arc::clone(&self.source_roots),
        }
    }

    /// The file that holds the contents of the file at `path`, and whether that is because
    /// `path` is in a package of a worktree that is not loaded.
    fn stand_in(&self, vfs: &Vfs, path: &AbsPath) -> (Option<FileId>, bool) {
        let file =
            |path: &AbsPath| vfs.file_id(&VfsPath::from(path.to_path_buf())).map(|(file, _)| file);
        for (overlay, not_loaded) in self.overlays.iter().zip(&self.not_loaded) {
            if let Some(overlay) = overlay
                && not_loaded.iter().any(|dir| path.starts_with(dir))
            {
                return (overlay.to_base(path).and_then(|path| file(&path)), true);
            }
        }
        (file(path), false)
    }

    /// Stops standing in for the packages that have the file at `path`, or whose counterpart in
    /// the base checkout has it. Returns whether there were any.
    fn load_packages_of(&mut self, path: &AbsPath) -> bool {
        let mut loads_more = false;
        for (overlay, not_loaded) in self.overlays.iter().zip(&mut self.not_loaded) {
            let Some(overlay) = overlay else { continue };
            let worktree_path = overlay.to_worktree(path);
            let before = not_loaded.len();
            not_loaded.retain(|dir| {
                !path.starts_with(dir)
                    && !worktree_path.as_ref().is_some_and(|path| path.starts_with(dir))
            });
            loads_more |= not_loaded.len() != before;
        }
        loads_more
    }

    /// The directories of the packages of `worktree` that need not be loaded: they are the same
    /// as in the base checkout, and so is everything they depend on.
    fn same_packages(
        &self,
        vfs: &Vfs,
        worktree: &ProjectWorkspace,
        overlay: &Overlay,
    ) -> FxHashSet<AbsPathBuf> {
        let ProjectWorkspaceKind::Cargo { cargo, .. } = &worktree.kind else {
            return FxHashSet::default();
        };
        let sources = Sources {
            vfs,
            roots: &self.source_roots,
            pulled_in_files: &self.pulled_in_files,
            disk_cache: &self.disk_cache,
        };
        // What applies to every package has to be the same to begin with.
        let workspace_root = worktree.workspace_root();
        let for_all_packages = ["Cargo.toml", "Cargo.lock", ".cargo/config.toml", ".cargo/config"];
        let same_workspace = overlay.to_base(workspace_root).is_some_and(|base_root| {
            for_all_packages.iter().all(|file| {
                let read = |root: &AbsPath| fs::read(root.join(file)).ok();
                read(workspace_root) == read(&base_root)
            })
        });
        if !same_workspace {
            return FxHashSet::default();
        }

        let dirs: Vec<Directories> = ProjectFolders::new(std::slice::from_ref(worktree), &[], None)
            .load
            .into_iter()
            .filter_map(|entry| match entry {
                Entry::Directories(dirs) => Some(dirs),
                Entry::Files(_) => None,
            })
            .collect();
        // The files of the base checkout by the path of their counterpart in the worktree.
        let base_files: Vec<(AbsPathBuf, FileId)> = vfs
            .iter()
            .filter_map(|(file, path)| Some((overlay.to_worktree(path.as_path()?)?, file)))
            .collect();
        let mut same_dirs: FxHashMap<usize, bool> = FxHashMap::default();
        let mut same_dirs_of = |dir: &AbsPath| {
            let idx = dirs.iter().position(|dirs| dirs.include.iter().any(|it| it == dir))?;
            let same = *same_dirs
                .entry(idx)
                .or_insert_with(|| self.same_on_disk(&sources, overlay, &dirs[idx], &base_files));
            same.then_some(&dirs[idx])
        };

        let same_features = self.same_features(worktree, overlay);
        let in_worktree =
            |pkg: Package| cargo[pkg].manifest.parent().starts_with(&overlay.worktree_root);
        let mut same: FxHashMap<_, &Directories> = cargo
            .packages()
            .filter(|&pkg| in_worktree(pkg) && same_features(cargo[pkg].manifest.parent()))
            .filter_map(|pkg| Some((pkg, same_dirs_of(cargo[pkg].manifest.parent())?)))
            .collect();
        // A package that depends on a package that differs is analyzed anew, with its own files.
        loop {
            let before = same.len();
            let depends_on_other = |pkg: Package| {
                cargo[pkg]
                    .dependencies
                    .iter()
                    .any(|dep| in_worktree(dep.pkg) && !same.contains_key(&dep.pkg))
            };
            let differ: Vec<_> =
                same.keys().copied().filter(|&pkg| depends_on_other(pkg)).collect();
            for pkg in differ {
                same.remove(&pkg);
            }
            if same.len() == before {
                break;
            }
        }
        // Several packages can be loaded together, then all of them have to be the same.
        let packages_of = |dirs: &Directories| {
            cargo
                .packages()
                .filter(|&pkg| dirs.include.iter().any(|it| it == cargo[pkg].manifest.parent()))
                .collect::<Vec<_>>()
        };
        same.values()
            .filter(|dirs| packages_of(dirs).iter().all(|pkg| same.contains_key(pkg)))
            .flat_map(|dirs| dirs.include.iter())
            .filter(|dir| dir.starts_with(&overlay.worktree_root))
            .cloned()
            .collect()
    }

    /// Tells whether the package with its manifest in a directory is built with the same
    /// features in `worktree` as in the base checkout: a change to the manifest of one package
    /// can enable a feature of another one.
    fn same_features<'a>(
        &self,
        worktree: &ProjectWorkspace,
        overlay: &'a Overlay,
    ) -> Box<dyn Fn(&AbsPath) -> bool + 'a> {
        let features = |workspace: &ProjectWorkspace| -> FxHashMap<AbsPathBuf, Vec<String>> {
            let ProjectWorkspaceKind::Cargo { cargo, .. } = &workspace.kind else {
                return FxHashMap::default();
            };
            cargo
                .packages()
                .map(|pkg| {
                    let features = cargo[pkg].active_features.iter().cloned().sorted().collect();
                    (cargo[pkg].manifest.parent().to_path_buf(), features)
                })
                .collect()
        };
        let (base_features, features) = (features(self.base()), features(worktree));
        Box::new(move |dir: &AbsPath| {
            let base_dir = overlay.to_base(dir).unwrap_or_else(|| dir.to_path_buf());
            features.get(dir).is_some_and(|features| base_features.get(&base_dir) == Some(features))
        })
    }

    /// Whether the files of the worktree in `dirs`, as they are on disk, are the same as their
    /// counterparts in the base checkout, `base_files` by the path of the worktree's file.
    fn same_on_disk(
        &self,
        sources: &Sources<'_>,
        overlay: &Overlay,
        dirs: &Directories,
        base_files: &[(AbsPathBuf, FileId)],
    ) -> bool {
        if self
            .files_in_memory
            .iter()
            .any(|path| path.as_path().is_some_and(|path| dirs.contains_file(path)))
        {
            return false;
        }
        let base_files: FxHashMap<&AbsPath, FileId> = base_files
            .iter()
            .filter(|(path, _)| dirs.contains_file(path))
            .map(|(path, file)| (path.as_path(), *file))
            .collect();
        let mut n_files = 0;
        let mut same_file = |path: &AbsPath| {
            n_files += 1;
            let Some(&base_file) = base_files.get(path) else {
                return false;
            };
            // A change to a file that the package pulls in from elsewhere would not tell us to
            // load the package, so such a package is loaded, and compared file by file.
            let pulls_in_from_elsewhere = || {
                sources.pulled_in_files.get(&base_file).into_iter().flatten().any(
                    |pulled_in| match (pulled_in, path.parent()) {
                        (PulledInFile::Relative(relative), Some(dir)) => {
                            let pulled_in = dir.absolutize(relative);
                            !dirs.include.iter().any(|dir| pulled_in.starts_with(dir))
                        }
                        _ => true,
                    },
                )
            };
            fs::read(path).is_ok_and(|contents| {
                Some(hash_once::<FxHasher>(&*contents)) == sources.vfs.content_hash(base_file)
            }) && !pulls_in_from_elsewhere()
                && sources.pulls_in_the_same(overlay, base_file, path, &mut FxHashSet::default())
        };
        // A directory outside of the worktree, such as the output of a build script taken over
        // from the base checkout, is the very same directory for both.
        let mut in_worktree =
            dirs.include.iter().filter(|dir| dir.starts_with(&overlay.worktree_root));
        in_worktree.all(|dir| walk(dirs, dir, &mut same_file)) && n_files == base_files.len()
    }

    /// Loads the files of all workspaces and builds the crate graph.
    fn reload(&mut self, db: &mut RootDatabase, vfs: &mut Vfs) {
        let mut change = ChangeWithProcMacros::default();
        loop {
            let project_folders = ProjectFolders::new(&self.workspaces, &[], None);
            // The same directory can be loaded for several packages, for example the output of
            // a build script that a worktree took over from the base checkout, so leave out the
            // directories rather than what is loaded for a package as a whole.
            let is_not_loaded = |dir: &AbsPathBuf| {
                self.not_loaded.iter().any(|not_loaded| not_loaded.contains(dir))
            };
            // The directories that no workspace has anymore are not loaded anymore either: if
            // they come back, their files are read again.
            let current: FxHashSet<&AbsPathBuf> = project_folders
                .load
                .iter()
                .flat_map(|entry| match entry {
                    Entry::Directories(dirs) => dirs.include.as_slice(),
                    Entry::Files(_) => &[],
                })
                .collect();
            let dropped: Vec<AbsPathBuf> =
                self.loaded.iter().filter(|dir| !current.contains(dir)).cloned().collect();
            if !dropped.is_empty() {
                let files: Vec<VfsPath> = vfs
                    .iter()
                    .map(|(_, path)| path)
                    .filter(|path| {
                        path.as_path().is_some_and(|path| {
                            dropped.iter().any(|dir| path.starts_with(dir))
                                && !current.iter().any(|dir| path.starts_with(dir))
                        })
                    })
                    .cloned()
                    .collect();
                for path in files {
                    self.files_in_memory.remove(&path);
                    vfs.set_file_contents(path, None);
                }
                self.loaded.retain(|dir| !dropped.contains(dir));
            }
            // What was loaded before is not read again: changes on disk are for the embedder to
            // tell us about.
            let mut newly_loaded = Vec::new();
            let load = project_folders
                .load
                .into_iter()
                .filter_map(|entry| match entry {
                    Entry::Directories(mut dirs) => {
                        dirs.include
                            .retain(|dir| !is_not_loaded(dir) && !self.loaded.contains(dir));
                        newly_loaded.extend(dirs.include.iter().cloned());
                        (!dirs.include.is_empty()).then_some(Entry::Directories(dirs))
                    }
                    entry @ Entry::Files(_) => Some(entry),
                })
                .collect();
            self.loaded.extend(newly_loaded);
            self.loader_config_version += 1;
            self.loader.set_config(vfs::loader::Config {
                load,
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

            self.take_file_changes(vfs, &mut change);
            self.source_roots = Arc::new(SourceRoots::new(&self.source_root_config, vfs));
            // A crate of a package that is not loaded that turns out to be the worktree's own
            // needs its own files.
            if !self.set_crate_graph(vfs, &mut change) {
                break;
            }
        }
        change.set_roots(self.source_root_config.partition(vfs));
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
        if self.sharing_is_stale(vfs, &changed_files, created_or_deleted)
            && self.set_crate_graph(vfs, &mut change)
        {
            db.apply_change(change);
            return self.reload(db, vfs);
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
    fn sharing_is_stale(
        &self,
        vfs: &Vfs,
        changed_files: &[FileId],
        created_or_deleted: bool,
    ) -> bool {
        let roots = &*self.source_roots;
        let sources = Sources {
            vfs,
            roots,
            pulled_in_files: &self.pulled_in_files,
            disk_cache: &self.disk_cache,
        };
        self.overlay_crates.iter().any(|(&(worktree_file, base_file), krate)| {
            // The files that a crate pulls in by path can be anywhere, and a file that is gone
            // is in no package anymore.
            let touched = created_or_deleted
                || roots.pulls_in_files(&self.pulled_in_files, worktree_file)
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
    ///
    /// Returns whether more files have to be loaded and the crate graph built again: a crate of
    /// a package that is not loaded turned out not to be shared with the base checkout.
    fn set_crate_graph(&mut self, vfs: &Vfs, change: &mut ChangeWithProcMacros) -> bool {
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
            let (file, _) = self.stand_in(vfs, path);
            file.filter(|&file| vfs.exists(file))
        };
        let (crate_graph, proc_macro_paths, own_crates) = worktree::crate_graph(
            &self.workspaces,
            &self.extra_env,
            load,
            &self.overlays,
            Some(&sources),
            &mut overlay_crates,
        );

        // The files of the base checkout stand in for the ones of a worktree's package that is
        // not loaded. That only works for the crates the two share: a file is analyzed as part
        // of one crate.
        let mut loads_more = false;
        for ((overlay, not_loaded), own_crates) in
            self.overlays.iter().zip(&mut self.not_loaded).zip(own_crates)
        {
            let Some(overlay) = overlay else { continue };
            for root_file in own_crates {
                let worktree_path =
                    vfs.file_path(root_file).as_path().and_then(|path| overlay.to_worktree(path));
                let Some(worktree_path) = worktree_path else { continue };
                let before = not_loaded.len();
                not_loaded.retain(|dir| !worktree_path.starts_with(dir));
                loads_more |= not_loaded.len() != before;
            }
        }
        if loads_more {
            return true;
        }

        self.overlay_crates = Arc::new(overlay_crates);
        let proc_macros = self.load_proc_macros(proc_macro_paths);
        change.set_crate_graph(crate_graph);
        change.set_proc_macros(proc_macros);
        false
    }

    fn load_proc_macros(&mut self, proc_macro_paths: Vec<ProcMacroPaths>) -> ProcMacrosBuilder {
        let server = match &self.proc_macro_server {
            Some(Ok(server)) => Ok(server),
            Some(Err(e)) => Err(e.clone()),
            None => Err(ProcMacroLoadingError::ProcMacroSrvError(
                "proc-macro-srv is not running, workspace is missing a sysroot".into(),
            )),
        };
        // A dylib that was built anew is loaded again, and the ones that no crate uses anymore
        // are forgotten.
        let modified = |path: &AbsPath| fs::metadata(path).and_then(|it| it.modified()).ok();
        let mut loaded = FxHashMap::default();
        let proc_macros = proc_macro_paths
            .into_iter()
            .flatten()
            .map(|(crate_id, path)| {
                let macros = path.and_then(|(_, path)| {
                    let server = server.as_ref().map_err(Clone::clone)?;
                    let modified = modified(&path);
                    let macros = match self.proc_macros.get(&path) {
                        Some((loaded_at, macros)) if *loaded_at == modified => macros.clone(),
                        _ => load_proc_macro(server, &path, &[]),
                    };
                    loaded.insert(path, (modified, macros.clone()));
                    macros
                });
                (crate_id, macros)
            })
            .collect();
        self.proc_macros = loaded;
        proc_macros
    }
}

/// The files in the checkout at `root` whose presence tells cargo about a package or a target, by
/// their path in the checkout: manifests, and the files at the places where targets are found
/// without being declared.
fn discovered_by_cargo(root: &AbsPath) -> std::collections::BTreeSet<String> {
    fn walk(root: &AbsPath, dir: &AbsPath, res: &mut std::collections::BTreeSet<String>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        let mut record = |path: &AbsPath| {
            if let Some(in_checkout) = path.strip_prefix(root) {
                res.insert(in_checkout.as_str().to_owned());
            }
        };
        let mut subdirs = Vec::new();
        let mut is_package = false;
        for entry in entries.flatten() {
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else { continue };
            let path = dir.join(&name);
            if entry.file_type().is_ok_and(|it| it.is_dir()) {
                if name != "target" && !name.starts_with('.') {
                    subdirs.push(path);
                }
            } else if name == "Cargo.toml" {
                is_package = true;
                record(&path);
            }
        }
        if is_package {
            for target in ["src/lib.rs", "src/main.rs", "build.rs"] {
                if fs::metadata(dir.join(target)).is_ok() {
                    record(&dir.join(target));
                }
            }
            for targets in ["src/bin", "examples", "tests", "benches"] {
                let Ok(entries) = fs::read_dir(dir.join(targets)) else { continue };
                for entry in entries.flatten() {
                    let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                        continue;
                    };
                    let path = dir.join(targets).join(&name);
                    let is_target = match entry.file_type() {
                        Ok(it) if it.is_dir() => fs::metadata(path.join("main.rs")).is_ok(),
                        _ => name.ends_with(".rs"),
                    };
                    if is_target {
                        record(&path);
                    }
                }
            }
        }
        for subdir in subdirs {
            walk(root, &subdir, res);
        }
    }
    let mut res = std::collections::BTreeSet::new();
    walk(root, root, &mut res);
    res
}

/// Calls `same_file` for the files in `dir` that `dirs` has, until it returns `false`. Returns
/// whether it never did.
fn walk(dirs: &Directories, dir: &AbsPath, same_file: &mut dyn FnMut(&AbsPath) -> bool) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return true;
    };
    for entry in entries.flatten() {
        let Some(path) = entry.file_name().to_str().map(|name| dir.join(name)) else {
            return false;
        };
        let is_dir = entry.file_type().is_ok_and(|it| it.is_dir());
        let same = if is_dir {
            !dirs.contains_dir(&path) || walk(dirs, &path, same_file)
        } else {
            !dirs.contains_file(&path) || same_file(&path)
        };
        if !same {
            return false;
        }
    }
    true
}

/// How the files of the worktrees relate to the base checkout, as of some state of the database.
///
/// A request about a file of a worktree is answered in three steps: analyze
/// [`Views::analyzed_file`] of [`Views::file`] instead of the file, leave out the results that
/// are not [`Views::in_view`], and report the rest at [`Views::path_in_view`].
#[derive(Clone)]
pub struct Views {
    /// The worktrees, with the directories of their packages that are not loaded.
    overlays: Arc<[(Overlay, Vec<AbsPathBuf>)]>,
    overlay_crates: Arc<OverlayCrates>,
    source_roots: Arc<SourceRoots>,
}

impl Views {
    /// The worktree that `path` is in.
    pub fn overlay_of(&self, path: &AbsPath) -> Option<&Overlay> {
        self.overlays.iter().map(|(it, _)| it).find(|it| path.starts_with(&it.worktree_root))
    }

    fn is_not_loaded(&self, view: &Overlay, path: &AbsPath) -> bool {
        self.overlays
            .iter()
            .any(|(overlay, dirs)| overlay == view && dirs.iter().any(|dir| path.starts_with(dir)))
    }

    /// The file at `path`. The files of a package that a worktree shares with its base checkout
    /// are not loaded: the files of the base checkout stand in for them.
    pub fn file(&self, vfs: &Vfs, path: &AbsPath) -> Option<FileId> {
        let file = |path: &AbsPath| {
            let (file, excluded) = vfs.file_id(&VfsPath::from(path.to_path_buf()))?;
            (excluded == vfs::FileExcluded::No).then_some(file)
        };
        match self.overlay_of(path) {
            Some(overlay) if self.is_not_loaded(overlay, path) => file(&overlay.to_base(path)?),
            _ => file(path),
        }
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
        let overlays: Vec<Overlay> = self.overlays.iter().map(|(it, _)| it.clone()).collect();
        match worktree::shared_base_file(
            vfs,
            &self.source_roots,
            &overlays,
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
        // A library, which is the same for all.
        let Some(worktree_path) = view.to_worktree(path) else {
            return true;
        };
        let worktree_file = vfs
            .file_id(&VfsPath::from(worktree_path.clone()))
            .map(|(file, _)| file)
            .filter(|&file| vfs.exists(file));
        match worktree_file {
            Some(worktree_file) => !is_in_a_crate(worktree_file),
            // Either the base checkout's file stands in for the worktree's, or the worktree
            // does not have the file.
            None => self.is_not_loaded(view, &worktree_path),
        }
    }

    /// The path of `file` for who works in `view`: the files of the base checkout are known
    /// to a worktree at its own paths.
    pub fn path_in_view(&self, vfs: &Vfs, view: Option<&Overlay>, file: FileId) -> VfsPath {
        let path = vfs.file_path(file);
        let worktree_path = view.and_then(|view| {
            let worktree_path = view.to_worktree(path.as_path()?)?;
            let has_it = self.is_not_loaded(view, &worktree_path)
                || vfs
                    .file_id(&VfsPath::from(worktree_path.clone()))
                    .is_some_and(|(file, _)| vfs.exists(file));
            has_it.then(|| VfsPath::from(worktree_path))
        });
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
            self.worktrees.views().file(&self.vfs, &self.path(path)).unwrap()
        }

        /// Whether the file itself is loaded, rather than another one standing in for it.
        fn is_loaded(&self, path: &str) -> bool {
            self.vfs
                .file_id(&VfsPath::from(self.path(path)))
                .is_some_and(|(file, _)| self.vfs.exists(file))
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
        // Nothing of the worktree is even loaded
        assert!(!checkouts.is_loaded("wt/app/src/lib.rs"));
        assert!(!checkouts.is_loaded("wt/core_lib/src/lib.rs"));
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
        assert!(checkouts.is_loaded("leaf/app/src/lib.rs"));
        assert!(!checkouts.is_loaded("leaf/core_lib/src/lib.rs"));
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
        assert!(checkouts.is_loaded("core/core_lib/src/lib.rs"));
        assert!(checkouts.is_loaded("core/app/src/lib.rs"));
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

        // Setting the text it has already changes nothing
        checkouts.set_file_text("wt/core_lib/src/lib.rs", CORE_LIB);
        assert!(!checkouts.is_loaded("wt/core_lib/src/lib.rs"));
        assert_eq!(checkouts.crates(), 2);

        checkouts.set_file_text("wt/core_lib/src/lib.rs", "pub fn answer() -> u64 { 42 }\n");
        assert!(checkouts.is_loaded("wt/core_lib/src/lib.rs"));
        assert!(checkouts.is_loaded("wt/app/src/lib.rs"));
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

        // The files stay loaded until the worktree is saved and compared on disk again
        assert!(checkouts.is_loaded("wt/core_lib/src/lib.rs"));
        let path = checkouts.path("wt/core_lib/src/lib.rs");
        checkouts.worktrees.reload_file(&mut checkouts.db, &mut checkouts.vfs, &path);
        checkouts.worktrees.unload_shared(&mut checkouts.db, &mut checkouts.vfs);
        assert!(!checkouts.is_loaded("wt/core_lib/src/lib.rs"));
        assert!(!checkouts.is_loaded("wt/app/src/lib.rs"));
        assert_eq!(checkouts.crates(), 2);
        assert_eq!(
            checkouts.analyzed_file("wt/core_lib/src/lib.rs"),
            checkouts.file("base/core_lib/src/lib.rs")
        );
    }

    #[test]
    fn editing_the_base_checkout_stops_sharing_with_it() {
        let mut checkouts = Checkouts::new();
        checkouts.add_worktree("wt", CORE_LIB, APP);
        assert_eq!(checkouts.crates(), 2);

        checkouts.set_file_text("base/app/src/lib.rs", "pub fn run() -> u32 { 7 }\n");
        assert_eq!(checkouts.crates(), 3);
        assert!(checkouts.is_loaded("wt/app/src/lib.rs"));
        assert!(!checkouts.is_loaded("wt/core_lib/src/lib.rs"));
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

    #[test]
    fn file_created_or_deleted_in_a_shared_package_stops_sharing_it() {
        let mut checkouts = Checkouts::new();
        checkouts.add_worktree("wt", CORE_LIB, APP);
        assert_eq!(checkouts.crates(), 2);

        // A new file makes the package differ, whether a crate uses it or not
        checkouts.set_file_text("wt/core_lib/src/extra.rs", "pub fn extra() {}\n");
        assert!(checkouts.is_loaded("wt/core_lib/src/lib.rs"));
        assert_eq!(checkouts.crates(), 4);

        let path = checkouts.path("wt/core_lib/src/extra.rs");
        checkouts.worktrees.set_file_text(&mut checkouts.db, &mut checkouts.vfs, &path, None);
        assert_eq!(checkouts.crates(), 2);

        // And so does a file that is gone
        let path = checkouts.path("wt/app/src/lib.rs");
        checkouts.worktrees.set_file_text(&mut checkouts.db, &mut checkouts.vfs, &path, None);
        // The worktree has no `app` anymore, and the base checkout's is not its business
        assert_eq!(checkouts.crates(), 2);
        let view = checkouts.overlay("wt");
        assert!(!checkouts.in_view(Some(&view), "base/app/src/lib.rs"));
        assert!(checkouts.in_view(Some(&view), "base/core_lib/src/lib.rs"));
    }

    #[test]
    fn adding_a_worktree_again_takes_its_new_workspace() {
        let mut checkouts = Checkouts::new();
        checkouts.add_worktree("wt", CORE_LIB, "pub fn run() -> u32 { 1 }\n");
        assert_eq!(checkouts.crates(), 3);

        // The same worktree, loaded again: it is still one worktree
        checkouts.add_worktree("wt", CORE_LIB, "pub fn run() -> u32 { 1 }\n");
        assert_eq!(checkouts.crates(), 3);
        assert_eq!(checkouts.worktrees.overlays().count(), 1);
    }

    #[test]
    fn replacing_the_base_workspace_keeps_sharing() {
        let mut checkouts = Checkouts::new();
        checkouts.add_worktree("wt", CORE_LIB, "pub fn run() -> u32 { 1 }\n");
        assert_eq!(checkouts.crates(), 3);

        let base = load_workspace(&checkouts.dir.join("base"));
        checkouts.worktrees.set_base(&mut checkouts.db, &mut checkouts.vfs, base);
        // What the worktree shares with the new base is not loaded
        assert!(!checkouts.is_loaded("wt/core_lib/src/lib.rs"));
        assert!(checkouts.is_loaded("wt/app/src/lib.rs"));
        assert_eq!(checkouts.crates(), 3);
        assert_eq!(
            checkouts.analyzed_file("wt/core_lib/src/lib.rs"),
            checkouts.file("base/core_lib/src/lib.rs")
        );
    }

    #[test]
    fn worktree_with_another_workspace_manifest_is_loaded_and_compared() {
        let mut checkouts = Checkouts::new();
        write_checkout(&checkouts.dir.join("wt"), CORE_LIB, APP);
        // The same workspace, but we cannot tell that from the manifest
        let manifest = checkouts.dir.join("wt/Cargo.toml");
        let text = fs::read_to_string(&manifest).unwrap();
        fs::write(&manifest, format!("{text}\n# a comment\n")).unwrap();
        let overlay = checkouts.overlay("wt");
        checkouts.worktrees.add(
            &mut checkouts.db,
            &mut checkouts.vfs,
            load_workspace(&checkouts.dir.join("wt")),
            overlay,
        );

        assert!(checkouts.is_loaded("wt/core_lib/src/lib.rs"));
        assert_eq!(checkouts.crates(), 2);
    }

    #[test]
    fn crate_pulling_in_a_file_that_differs_is_loaded() {
        let pulling_in = "#[path = \"../../shared.rs\"]\nmod shared;\npub fn run() -> u32 { 1 }\n";
        let dir = std::env::temp_dir().join(format!("ra-worktrees-pull-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let dir = fs::canonicalize(dir).unwrap();
        write_checkout(&dir.join("base"), CORE_LIB, pulling_in);
        fs::write(dir.join("base/shared.rs"), "pub fn shared() {}\n").unwrap();
        let (worktrees, db, vfs) = Worktrees::load(
            load_workspace(&dir.join("base")),
            &FxHashMap::default(),
            &LoadCargoConfig {
                load_out_dirs_from_check: false,
                with_proc_macro_server: ProcMacroServerChoice::None,
                prefill_caches: true,
                num_worker_threads: 1,
                proc_macro_processes: 1,
            },
        )
        .unwrap();
        let mut checkouts = Checkouts { dir, worktrees, db, vfs };

        // `app` is the same in both, but the file it pulls in from outside of its package is not
        write_checkout(&checkouts.dir.join("differs"), CORE_LIB, pulling_in);
        fs::write(checkouts.dir.join("differs/shared.rs"), "pub fn shared() -> u8 { 1 }\n")
            .unwrap();
        let overlay = checkouts.overlay("differs");
        checkouts.worktrees.add(
            &mut checkouts.db,
            &mut checkouts.vfs,
            load_workspace(&checkouts.dir.join("differs")),
            overlay,
        );
        assert_eq!(checkouts.crates(), 3);
        assert!(checkouts.is_loaded("differs/app/src/lib.rs"));
        assert!(!checkouts.is_loaded("differs/core_lib/src/lib.rs"));

        write_checkout(&checkouts.dir.join("same"), CORE_LIB, pulling_in);
        fs::write(checkouts.dir.join("same/shared.rs"), "pub fn shared() {}\n").unwrap();
        let overlay = checkouts.overlay("same");
        checkouts.worktrees.add(
            &mut checkouts.db,
            &mut checkouts.vfs,
            load_workspace(&checkouts.dir.join("same")),
            overlay,
        );
        // The crate is shared, but its package is loaded: a change to the file it pulls in from
        // elsewhere has to be noticed
        assert_eq!(checkouts.crates(), 3);
        assert!(checkouts.is_loaded("same/app/src/lib.rs"));
        assert!(!checkouts.is_loaded("same/core_lib/src/lib.rs"));
        assert_eq!(
            checkouts.analyzed_file("same/app/src/lib.rs"),
            checkouts.file("base/app/src/lib.rs")
        );

        checkouts.set_file_text("same/shared.rs", "pub fn shared() -> u16 { 2 }\n");
        assert_eq!(checkouts.crates(), 4);
        assert_eq!(
            checkouts.analyzed_file("same/app/src/lib.rs"),
            checkouts.file("same/app/src/lib.rs")
        );
    }

    #[test]
    fn worktree_inherits_what_the_build_scripts_of_the_base_generated() {
        // Runs `cargo check`.
        if std::env::var("RUN_SLOW_TESTS").is_err() {
            return;
        }
        let dir = std::env::temp_dir().join(format!("ra-worktrees-build-{}", std::process::id()));
        _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let dir = fs::canonicalize(dir).unwrap();
        let with_build_script = |root: &std::path::Path| {
            write_checkout(root, "#[cfg(has_flag)]\npub fn answer() -> u32 { 42 }\n", APP);
            fs::write(
                root.join("core_lib/build.rs"),
                "fn main() { println!(\"cargo:rustc-cfg=has_flag\"); }\n",
            )
            .unwrap();
        };
        with_build_script(&dir.join("base"));
        with_build_script(&dir.join("wt"));

        let cargo_config = CargoConfig::default();
        let mut base = load_workspace(&dir.join("base"));
        let build_scripts = base.run_build_scripts(&cargo_config, &|_| {}).unwrap();
        assert_eq!(build_scripts.error(), None);
        base.set_build_scripts(build_scripts);
        let (worktrees, db, vfs) = Worktrees::load(
            base,
            &FxHashMap::default(),
            &LoadCargoConfig {
                load_out_dirs_from_check: true,
                with_proc_macro_server: ProcMacroServerChoice::None,
                prefill_caches: false,
                num_worker_threads: 1,
                proc_macro_processes: 1,
            },
        )
        .unwrap();
        let mut checkouts = Checkouts { dir, worktrees, db, vfs };
        let base_crates = checkouts.crates();

        // Running the build scripts wrote the lock file, which a worktree would have as well
        if let Ok(lock) = fs::read(checkouts.dir.join("base/Cargo.lock")) {
            fs::write(checkouts.dir.join("wt/Cargo.lock"), lock).unwrap();
        }

        // Without the outputs of the build scripts the crates of the worktree are not the same
        let overlay = checkouts.overlay("wt");
        let mut worktree = load_workspace(&checkouts.dir.join("wt"));
        let to_build =
            checkouts.worktrees.inherit_build_scripts(&checkouts.vfs, &mut worktree, &overlay);
        assert_eq!(to_build, Vec::<AbsPathBuf>::new());
        checkouts.worktrees.add(&mut checkouts.db, &mut checkouts.vfs, worktree, overlay);
        assert_eq!(checkouts.crates(), base_crates);
        assert!(!checkouts.is_loaded("wt/core_lib/src/lib.rs"));

        // A package that differs does not get what the base checkout's version of it generated
        with_build_script(&checkouts.dir.join("changed"));
        fs::write(
            checkouts.dir.join("changed/core_lib/build.rs"),
            "fn main() { println!(\"cargo:rustc-cfg=another_flag\"); }\n",
        )
        .unwrap();
        fs::copy(checkouts.dir.join("base/Cargo.lock"), checkouts.dir.join("changed/Cargo.lock"))
            .unwrap();
        let overlay = checkouts.overlay("changed");
        let mut worktree = load_workspace(&checkouts.dir.join("changed"));
        let to_build =
            checkouts.worktrees.inherit_build_scripts(&checkouts.vfs, &mut worktree, &overlay);
        // `app` depends on it, so what it would take over may be wrong as well
        assert_eq!(
            to_build,
            vec![checkouts.path("changed/app"), checkouts.path("changed/core_lib")]
        );
    }

    #[test]
    fn package_built_with_other_features_is_not_the_same() {
        let with_feature = |root: &std::path::Path, app_enables_it: bool| {
            write_checkout(root, CORE_LIB, APP);
            let manifest = root.join("core_lib/Cargo.toml");
            let text = fs::read_to_string(&manifest).unwrap();
            fs::write(&manifest, format!("{text}\n[features]\nspecial = []\n")).unwrap();
            if app_enables_it {
                let manifest = root.join("app/Cargo.toml");
                let text = fs::read_to_string(&manifest).unwrap();
                let text = text.replace(
                    "core_lib = { path = \"../core_lib\" }",
                    "core_lib = { path = \"../core_lib\", features = [\"special\"] }",
                );
                fs::write(&manifest, text).unwrap();
            }
        };
        let dir = std::env::temp_dir().join(format!("ra-worktrees-feat-{}", std::process::id()));
        _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let dir = fs::canonicalize(dir).unwrap();
        with_feature(&dir.join("base"), false);
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
        let mut checkouts = Checkouts { dir, worktrees, db, vfs };

        // Only the manifest of `app` differs, but it enables a feature of `core_lib`, whose
        // files are the same
        with_feature(&checkouts.dir.join("wt"), true);
        let overlay = checkouts.overlay("wt");
        let mut worktree = load_workspace(&checkouts.dir.join("wt"));
        let to_build =
            checkouts.worktrees.inherit_build_scripts(&checkouts.vfs, &mut worktree, &overlay);
        assert_eq!(to_build, vec![checkouts.path("wt/app"), checkouts.path("wt/core_lib")]);
        checkouts.worktrees.add(&mut checkouts.db, &mut checkouts.vfs, worktree, overlay);
        assert_eq!(checkouts.crates(), 4);
        assert!(checkouts.is_loaded("wt/core_lib/src/lib.rs"));
    }

    #[test]
    fn workspace_of_a_copy_is_what_loading_it_gives() {
        let mut checkouts = Checkouts::new();
        write_checkout(&checkouts.dir.join("wt"), CORE_LIB, "pub fn run() -> u32 { 1 }\n");
        let overlay = checkouts.overlay("wt");
        // A copy has the lock file of the base checkout, if that has one
        if let Ok(lock) = fs::read(checkouts.dir.join("base/Cargo.lock")) {
            fs::write(checkouts.dir.join("wt/Cargo.lock"), lock).unwrap();
        }

        let copy = checkouts.worktrees.workspace_of_copy(&overlay).unwrap();
        let loaded = load_workspace(&checkouts.dir.join("wt"));
        assert!(copy.eq_ignore_build_data(&loaded), "{copy:#?}\n{loaded:#?}");

        checkouts.worktrees.add(&mut checkouts.db, &mut checkouts.vfs, copy, overlay.clone());
        assert_eq!(checkouts.crates(), 3);
        assert!(!checkouts.is_loaded("wt/core_lib/src/lib.rs"));
        assert_eq!(
            checkouts.analyzed_file("wt/app/src/lib.rs"),
            checkouts.file("wt/app/src/lib.rs")
        );

        // With another toolchain chosen for the copy, the workspace has to be loaded
        let toolchain = checkouts.dir.join("wt/rust-toolchain.toml");
        fs::write(&toolchain, "[toolchain]\nchannel = \"stable\"\n").unwrap();
        assert!(checkouts.worktrees.workspace_of_copy(&overlay).is_none());
        fs::remove_file(&toolchain).unwrap();
        assert!(checkouts.worktrees.workspace_of_copy(&overlay).is_some());

        // With a target that cargo finds by its file, the workspace has to be loaded
        let new_target = checkouts.dir.join("wt/app/src/bin/tool.rs");
        fs::create_dir_all(new_target.parent().unwrap()).unwrap();
        fs::write(&new_target, "fn main() {}\n").unwrap();
        assert!(checkouts.worktrees.workspace_of_copy(&overlay).is_none());
        fs::remove_file(&new_target).unwrap();
        assert!(checkouts.worktrees.workspace_of_copy(&overlay).is_some());

        // With another manifest the workspace has to be loaded
        let manifest = checkouts.dir.join("wt/app/Cargo.toml");
        let text = fs::read_to_string(&manifest).unwrap();
        fs::write(&manifest, format!("{text}\n[features]\nextra = []\n")).unwrap();
        assert!(checkouts.worktrees.workspace_of_copy(&overlay).is_none());
    }

    #[test]
    fn package_that_comes_back_is_read_again() {
        let mut checkouts = Checkouts::new();
        let overlay = checkouts.add_worktree("wt", CORE_LIB, "pub fn run() -> u32 { 1 }\n");
        assert!(checkouts.is_loaded("wt/app/src/lib.rs"));

        // The worktree drops `app` from its workspace
        let manifest = checkouts.dir.join("wt/Cargo.toml");
        let with_app = fs::read_to_string(&manifest).unwrap();
        fs::write(
            &manifest,
            with_app.replace("members = [\"core_lib\", \"app\"]", "members = [\"core_lib\"]"),
        )
        .unwrap();
        let workspace = load_workspace(&checkouts.dir.join("wt"));
        checkouts.worktrees.add(&mut checkouts.db, &mut checkouts.vfs, workspace, overlay.clone());
        assert!(!checkouts.is_loaded("wt/app/src/lib.rs"));
        assert_eq!(checkouts.crates(), 2);

        // And takes it back, with other sources
        fs::write(&manifest, with_app).unwrap();
        fs::write(checkouts.dir.join("wt/app/src/lib.rs"), "pub fn run() -> u32 { 2 }\n").unwrap();
        let workspace = load_workspace(&checkouts.dir.join("wt"));
        checkouts.worktrees.add(&mut checkouts.db, &mut checkouts.vfs, workspace, overlay);
        assert_eq!(checkouts.text("wt/app/src/lib.rs"), "pub fn run() -> u32 { 2 }\n");
        assert_eq!(checkouts.crates(), 3);
    }

    #[test]
    fn copy_with_a_dependency_outside_of_the_checkout_is_loaded() {
        let dir = std::env::temp_dir().join(format!("ra-worktrees-out-{}", std::process::id()));
        _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let dir = fs::canonicalize(dir).unwrap();
        // A sibling of the checkout whose name starts like it, which `app` depends on
        let outside = dir.join("base-utils");
        fs::create_dir_all(outside.join("src")).unwrap();
        fs::write(outside.join("Cargo.toml"), "[package]\nname = \"utils\"\nversion = \"0.0.0\"\n")
            .unwrap();
        fs::write(outside.join("src/lib.rs"), "pub fn util() {}\n").unwrap();
        let with_outside_dependency = |root: &std::path::Path| {
            write_checkout(root, CORE_LIB, APP);
            let manifest = root.join("app/Cargo.toml");
            let text = fs::read_to_string(&manifest).unwrap();
            fs::write(&manifest, format!("{text}utils = {{ path = \"../../base-utils\" }}\n"))
                .unwrap();
        };
        with_outside_dependency(&dir.join("base"));
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
        let mut checkouts = Checkouts { dir, worktrees, db, vfs };
        let base_crates = checkouts.crates();

        with_outside_dependency(&checkouts.dir.join("wt"));
        let overlay = checkouts.overlay("wt");
        // The path leads to the same package from the copy here, but we do not rely on it
        assert!(checkouts.worktrees.workspace_of_copy(&overlay).is_none());

        // Moving the workspace leaves the sibling where it is
        let moved = checkouts
            .worktrees
            .base()
            .rerooted(&overlay.base_root, &overlay.worktree_root)
            .unwrap();
        let loaded = load_workspace(&checkouts.dir.join("wt"));
        assert!(moved.eq_ignore_build_data(&loaded), "{moved:#?}\n{loaded:#?}");

        checkouts.worktrees.add(&mut checkouts.db, &mut checkouts.vfs, loaded, overlay);
        assert_eq!(checkouts.crates(), base_crates);
    }
}
