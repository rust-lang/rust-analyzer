//! base_db defines basic database traits. The concrete DB is defined by ide.
// FIXME: Rename this crate, base db is non descriptive

#![cfg_attr(feature = "in-rust-tree", feature(rustc_private))]

#[cfg(feature = "in-rust-tree")]
extern crate rustc_driver as _;

pub use salsa;
pub use span::File;
use span::TextSize;

mod change;
mod editioned_file_id;
mod input;
pub mod target;

use std::{
    cell::RefCell,
    collections::hash_map::Entry,
    mem, panic,
    sync::{Arc as StdArc, Once, RwLock, atomic::AtomicUsize},
};

pub use crate::{
    change::FileChange,
    editioned_file_id::EditionedFileId,
    input::{
        BuiltCrateData, BuiltDependency, Crate, CrateBuilder, CrateBuilderId, CrateDataBuilder,
        CrateDisplayName, CrateGraphBuilder, CrateName, CrateOrigin, CratesIdMap, CratesMap,
        DependencyBuilder, Env, ExtraCrateData, FileRoot, FileRootId, FileRootKind,
        LangCrateOrigin, ProcMacroLoadingError, ProcMacroPaths, ReleaseChannel, UniqueCrateData,
    },
};
use rustc_hash::{FxHashMap, FxHashSet};
use salsa::{Durability, Setter};
pub use semver::{BuildMetadata, Prerelease, Version, VersionReq};
use triomphe::Arc;
pub use vfs::{AbsPathBuf, AnchoredPath, AnchoredPathBuf, VfsPath};

pub type FxIndexSet<T> = indexmap::IndexSet<T, rustc_hash::FxBuildHasher>;
pub type FxIndexMap<K, V> =
    indexmap::IndexMap<K, V, std::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

#[macro_export]
macro_rules! impl_intern_key {
    ($id:ident, $loc:ident) => {
        #[salsa::interned(unsafe(no_lifetime), revisions = usize::MAX)]
        #[derive(PartialOrd, Ord)]
        pub struct $id {
            #[returns(ref)]
            pub loc: $loc,
        }

        // If we derive this salsa prints the values recursively, and this causes us to blow.
        impl ::std::fmt::Debug for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_tuple(stringify!($id))
                    .field(&format_args!("{:04x}", self.0.index()))
                    .finish()
            }
        }
    };
}

pub const DEFAULT_FILE_TEXT_LRU_CAP: u16 = 16;
pub const DEFAULT_PARSE_LRU_CAP: u16 = 128;
pub const DEFAULT_BORROWCK_LRU_CAP: u16 = 2024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LineEndings {
    Unix,
    Dos,
}

impl LineEndings {
    pub fn normalize(src: String) -> (String, LineEndings) {
        let mut buf = src.into_bytes();
        let mut gap_len = 0;
        let mut tail = buf.as_mut_slice();
        let mut crlf_seen = false;

        let finder = memchr::memmem::Finder::new(b"\r\n");

        loop {
            let idx = match finder.find(&tail[gap_len..]) {
                None if crlf_seen => tail.len(),
                None => {
                    return (
                        String::from_utf8(buf).expect("input was valid UTF-8"),
                        LineEndings::Unix,
                    );
                }
                Some(idx) => {
                    crlf_seen = true;
                    idx + gap_len
                }
            };
            tail.copy_within(gap_len..idx, 0);
            tail = &mut tail[idx - gap_len..];
            if tail.len() == gap_len {
                break;
            }
            gap_len += 1;
        }

        let new_len = buf.len() - gap_len;
        // SAFETY: removing `\r` from UTF-8 `\r\n` pairs preserves UTF-8 validity.
        let src = unsafe {
            buf.set_len(new_len);
            String::from_utf8_unchecked(buf)
        };
        (src, LineEndings::Dos)
    }
}

/// Keeps loader events explicit about whether their contents enter indexed files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileVisibility {
    Indexed,
    Excluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Exists,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileLookup {
    pub file: File,
    pub status: FileStatus,
}

/// Carries contents only for pending states that still have file contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingFileChangeKind {
    Create(Vec<u8>),
    Modify(Vec<u8>),
    Delete,
}

#[salsa::input(debug)]
pub struct FileData {
    #[returns(copy)]
    pub file: File,
    #[returns(copy)]
    pub root: Option<FileRoot>,
    #[returns(copy)]
    pub status: FileStatus,
    #[returns(ref)]
    pub text: Arc<str>,
    #[returns(copy)]
    pub line_endings: LineEndings,
}

#[salsa::input]
/// The files that participate in workspace analysis, partitioned by durability.
///
/// [`Files`] deliberately does not own this membership: it is only a cache from paths to Salsa
/// inputs. Keeping membership in a Salsa input ensures additions and removals invalidate queries
/// over the workspace.
pub struct WorkspaceFiles {
    #[returns(ref)]
    pub local_files: Arc<[File]>,
    #[returns(ref)]
    pub library_files: Arc<[File]>,
}

#[derive(Clone, Default)]
pub struct Files {
    inner: StdArc<FilesInner>,
}

#[derive(Default)]
struct FilesInner {
    state: RwLock<FilesState>,
}

#[derive(Default)]
struct FilesState {
    data: FxHashMap<File, FileData>,
    pending_changes: FxHashMap<File, PendingFileChangeKind>,
}

#[doc(hidden)]
#[salsa::interned]
pub struct InternedAnchoredPath {
    #[returns(copy)]
    pub anchor: File,
    #[returns(ref)]
    pub path: String,
}

#[salsa::db]
pub trait SourceDatabase: salsa::Database + std::fmt::Debug {
    /// Returns the database-owned workspace membership input.
    fn workspace_files(&self) -> WorkspaceFiles;

    fn file_data(&self, file: File) -> FileData;

    fn set_file_text(&mut self, file: File, text: &str) {
        self.file_data(file).set_text(self).to(Arc::from(text));
    }

    fn set_file_text_with_line_endings_and_durability(
        &mut self,
        file: File,
        text: &str,
        line_endings: LineEndings,
        durability: Durability,
    ) {
        let data = self.file_data(file);
        data.set_text(self).with_durability(durability).to(Arc::from(text));
        data.set_line_endings(self).with_durability(durability).to(line_endings);
    }

    fn file_root(&self, file: File) -> Option<FileRoot> {
        let db = self.as_dyn_database();
        let data = self.file_data(file);
        match data.status(db) {
            FileStatus::Exists => data.root(db),
            FileStatus::Deleted => None,
        }
    }

    fn resolve_path(&self, anchor: File, paths: &[&str]) -> Option<(File, usize)>;

    fn file_path(&self, file: File) -> Option<VfsPath>;

    fn file_for_path(&self, path: &VfsPath) -> Option<FileLookup>;

    fn file_for_indexed_path(&self, path: &VfsPath) -> Option<File> {
        match self.file_for_path(path) {
            Some(FileLookup { file, status: FileStatus::Exists })
                if self.file_root(file).is_some() =>
            {
                Some(file)
            }
            Some(_) | None => None,
        }
    }

    fn intern_file_path(&self, path: VfsPath) -> File;

    fn record_file_contents(
        &mut self,
        path: VfsPath,
        contents: Option<Vec<u8>>,
        visibility: FileVisibility,
    );

    fn take_file_changes(&mut self) -> FxHashMap<File, PendingFileChangeKind>;

    fn files(&self) -> Vec<File> {
        let db = self.as_dyn_database();
        let files = self.workspace_files();
        files.local_files(db).iter().chain(files.library_files(db).iter()).copied().collect()
    }

    fn files_by_kind(&self, kind: FileRootKind) -> Arc<[File]> {
        let db = self.as_dyn_database();
        let files = self.workspace_files();
        match kind {
            FileRootKind::Local => files.local_files(db).clone(),
            FileRootKind::Library => files.library_files(db).clone(),
        }
    }

    #[doc(hidden)]
    fn set_indexed_files(&mut self, files: Vec<FileRegistration>);

    #[doc(hidden)]
    fn crates_map(&self) -> Arc<CratesMap>;

    fn nonce_and_revision(&self) -> (Nonce, salsa::Revision);

    fn line_column(&self, file: File, offset: TextSize) -> Result<(u32, u32), ()>;
}

#[salsa::tracked(returns(copy))]
fn lookup_resolve_path(db: &dyn SourceDatabase, path: InternedAnchoredPath<'_>) -> Option<File> {
    let mut base = db.file_path(path.anchor(db))?;
    base.pop();
    let path = base.join(path.path(db))?;
    let file = db.intern_file_path(path);
    match db.file_data(file).status(db) {
        FileStatus::Exists if db.file_root(file).is_some() => Some(file),
        FileStatus::Exists | FileStatus::Deleted => None,
    }
}

pub fn resolve_path(
    db: &dyn SourceDatabase,
    anchor: File,
    paths: &[&str],
) -> Option<(File, usize)> {
    for (index, path) in paths.iter().enumerate() {
        let target_id =
            lookup_resolve_path(db, InternedAnchoredPath::new(db, anchor, path.to_string()));
        match target_id {
            Some(target_id) => return Some((target_id, index)),
            None => (),
        }
    }

    None
}

impl WorkspaceFiles {
    /// Replaces workspace membership while updating each file's root classification.
    pub fn replace<DB: SourceDatabase + ?Sized>(
        self,
        db: &mut DB,
        files: &Files,
        registrations: Vec<FileRegistration>,
    ) {
        let mut active = FxHashSet::default();
        let mut local_files = Vec::new();
        let mut library_files = Vec::new();
        let mut indexed = Vec::new();
        let files = {
            let mut state = files.inner.state.write().unwrap();
            for FileRegistration { file, root } in registrations {
                active.insert(file);
                match root.kind {
                    FileRootKind::Local => local_files.push(file),
                    FileRootKind::Library => library_files.push(file),
                }
                let durability = root.kind.root_durability();
                let data = files.get_or_create_file_data(db, &mut state, file, durability);
                indexed.push((data, root, durability));
            }
            state.data.values().copied().collect::<Vec<_>>()
        };

        for (data, root, durability) in indexed {
            data.set_root(db).with_durability(durability).to(Some(root));
            data.set_status(db).with_durability(durability).to(FileStatus::Exists);
        }
        for data in files {
            if active.contains(&data.file(db)) {
                continue;
            }
            match data.status(db) {
                FileStatus::Exists => {
                    if data.root(db).is_some() {
                        data.set_root(db).with_durability(Durability::LOW).to(None);
                        data.set_status(db)
                            .with_durability(Durability::LOW)
                            .to(FileStatus::Deleted);
                    }
                }
                FileStatus::Deleted => (),
            }
        }
        local_files.sort_unstable();
        local_files.dedup();
        library_files.sort_unstable();
        library_files.dedup();
        self.set_local_files(db)
            .with_durability(FileRootKind::Local.root_durability())
            .to(local_files.into());
        self.set_library_files(db)
            .with_durability(FileRootKind::Library.root_durability())
            .to(library_files.into());
    }
}

impl Files {
    pub fn file_data(&self, file: File) -> FileData {
        let state = self.inner.state.read().unwrap();
        match state.data.get(&file).copied() {
            Some(data) => data,
            None => panic!("Unable to fetch file data for {file:?}; this is a bug"),
        }
    }

    pub fn file_path(&self, db: &dyn salsa::Database, file: File) -> Option<VfsPath> {
        let data = {
            let state = self.inner.state.read().unwrap();
            state.data.get(&file).copied()
        }?;
        match data.status(db) {
            FileStatus::Exists => Some(file.path(db).clone()),
            FileStatus::Deleted => None,
        }
    }

    pub fn file_for_path(&self, db: &dyn salsa::Database, path: &VfsPath) -> Option<FileLookup> {
        let file = File::new(db, path.clone());
        let data = self.inner.state.read().unwrap().data.get(&file).copied()?;
        Some(FileLookup { file, status: data.status(db) })
    }

    pub fn intern_file_path<DB: SourceDatabase + ?Sized>(&self, db: &DB, path: VfsPath) -> File {
        let file = File::new(db, path);
        {
            let state = self.inner.state.read().unwrap();
            if state.data.contains_key(&file) {
                return file;
            }
        }

        let mut state = self.inner.state.write().unwrap();
        if !state.data.contains_key(&file) {
            self.create_file_data(db, &mut state, file, Durability::LOW);
        }
        file
    }

    pub fn record_file_contents<DB: SourceDatabase + ?Sized>(
        &self,
        db: &mut DB,
        path: VfsPath,
        contents: Option<Vec<u8>>,
        visibility: FileVisibility,
    ) {
        match visibility {
            FileVisibility::Indexed => {
                let file = File::new(db, path);
                let mut state = self.inner.state.write().unwrap();
                let data = match state.data.get(&file).copied() {
                    Some(data) => data,
                    None => match contents {
                        Some(_) => self.create_file_data(db, &mut state, file, Durability::LOW),
                        None => return,
                    },
                };
                let exists = match state.pending_changes.get(&file) {
                    Some(kind) => match kind {
                        PendingFileChangeKind::Create(_) | PendingFileChangeKind::Modify(_) => true,
                        PendingFileChangeKind::Delete => false,
                    },
                    None => match data.status(db) {
                        FileStatus::Exists => true,
                        FileStatus::Deleted => false,
                    },
                };

                let kind = match (exists, contents) {
                    (false, None) => return,
                    (false, Some(contents)) => PendingFileChangeKind::Create(contents),
                    (true, None) => PendingFileChangeKind::Delete,
                    (true, Some(contents)) => PendingFileChangeKind::Modify(contents),
                };
                merge_pending_file_change(&mut state.pending_changes, file, kind);
            }
            FileVisibility::Excluded => match contents {
                Some(contents) => {
                    let file = File::new(db, path);
                    let data = {
                        let mut state = self.inner.state.write().unwrap();
                        let data = match state.data.get(&file).copied() {
                            Some(data) => data,
                            None => self.create_file_data(db, &mut state, file, Durability::LOW),
                        };
                        state.pending_changes.remove(&file);
                        data
                    };
                    if let Ok(text) = String::from_utf8(contents) {
                        let (text, line_endings) = LineEndings::normalize(text);
                        data.set_text(db).with_durability(Durability::LOW).to(Arc::from(text));
                        data.set_line_endings(db).with_durability(Durability::LOW).to(line_endings);
                    }
                    data.set_root(db).with_durability(Durability::LOW).to(None);
                    data.set_status(db).with_durability(Durability::LOW).to(FileStatus::Exists);
                }
                None => {
                    let file = File::new(db, path);
                    let data = {
                        let mut state = self.inner.state.write().unwrap();
                        let data = state.data.get(&file).copied();
                        if data.is_some() {
                            state.pending_changes.remove(&file);
                        }
                        data
                    };
                    if let Some(data) = data {
                        match data.status(db) {
                            FileStatus::Exists if data.root(db).is_none() => {
                                data.set_status(db)
                                    .with_durability(Durability::LOW)
                                    .to(FileStatus::Deleted);
                            }
                            FileStatus::Exists | FileStatus::Deleted => (),
                        }
                    }
                }
            },
        }
    }

    pub fn take_file_changes(&self) -> FxHashMap<File, PendingFileChangeKind> {
        let mut state = self.inner.state.write().unwrap();
        mem::take(&mut state.pending_changes)
    }

    fn get_or_create_file_data<DB: SourceDatabase + ?Sized>(
        &self,
        db: &DB,
        state: &mut FilesState,
        file: File,
        durability: Durability,
    ) -> FileData {
        if let Some(data) = state.data.get(&file).copied() {
            return data;
        }
        self.create_file_data(db, state, file, durability)
    }

    fn create_file_data<DB: SourceDatabase + ?Sized>(
        &self,
        db: &DB,
        state: &mut FilesState,
        file: File,
        durability: Durability,
    ) -> FileData {
        let data =
            FileData::builder(file, None, FileStatus::Deleted, Arc::from(""), LineEndings::Unix)
                .durability(durability)
                .file_durability(Durability::NEVER_CHANGE)
                .new(db);
        state.data.insert(file, data);
        data
    }
}

fn merge_pending_file_change(
    changes: &mut FxHashMap<File, PendingFileChangeKind>,
    file: File,
    change: PendingFileChangeKind,
) {
    match changes.entry(file) {
        Entry::Vacant(entry) => {
            entry.insert(change);
        }
        Entry::Occupied(mut entry) => {
            let old = entry.get_mut();
            let merged = match (mem::replace(old, PendingFileChangeKind::Delete), change) {
                (PendingFileChangeKind::Create(_), PendingFileChangeKind::Create(text))
                | (PendingFileChangeKind::Create(_), PendingFileChangeKind::Modify(text)) => {
                    Some(PendingFileChangeKind::Create(text))
                }
                (PendingFileChangeKind::Create(_), PendingFileChangeKind::Delete) => None,
                (PendingFileChangeKind::Modify(_), PendingFileChangeKind::Create(text))
                | (PendingFileChangeKind::Modify(_), PendingFileChangeKind::Modify(text)) => {
                    Some(PendingFileChangeKind::Modify(text))
                }
                (PendingFileChangeKind::Modify(_), PendingFileChangeKind::Delete) => {
                    Some(PendingFileChangeKind::Delete)
                }
                (PendingFileChangeKind::Delete, PendingFileChangeKind::Create(text))
                | (PendingFileChangeKind::Delete, PendingFileChangeKind::Modify(text)) => {
                    Some(PendingFileChangeKind::Modify(text))
                }
                (PendingFileChangeKind::Delete, PendingFileChangeKind::Delete) => {
                    Some(PendingFileChangeKind::Delete)
                }
            };

            match merged {
                Some(kind) => *old = kind,
                None => {
                    entry.remove();
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileRegistration {
    pub file: File,
    pub root: FileRoot,
}

static NEXT_NONCE: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nonce(usize);

impl Default for Nonce {
    #[inline]
    fn default() -> Self {
        Nonce::new()
    }
}

impl Nonce {
    #[inline]
    pub const fn invalid() -> Nonce {
        Nonce(usize::MAX)
    }

    #[inline]
    pub fn new() -> Nonce {
        Nonce(NEXT_NONCE.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }
}

/// Crate related data shared by the whole workspace.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CrateWorkspaceData {
    pub target: Result<target::TargetData, target::TargetLoadError>,
    /// Toolchain version used to compile the crate.
    pub toolchain: Option<Version>,
}

impl CrateWorkspaceData {
    pub fn is_atleast_187(&self) -> bool {
        const VERSION_187: Version = Version {
            major: 1,
            minor: 87,
            patch: 0,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        };
        self.toolchain.as_ref().map_or(false, |v| *v >= VERSION_187)
    }
}

pub fn toolchain_channel(db: &dyn salsa::Database, krate: Crate) -> Option<ReleaseChannel> {
    krate.workspace_data(db).toolchain.as_ref().and_then(|v| ReleaseChannel::from_str(&v.pre))
}

#[salsa::input(singleton, debug)]
struct AllCrates {
    #[returns(clone)]
    crates: std::sync::Arc<[Crate]>,
}

pub fn set_all_crates_with_durability(
    db: &mut dyn salsa::Database,
    crates: impl IntoIterator<Item = Crate>,
    durability: Durability,
) {
    AllCrates::try_get(db)
        .unwrap_or_else(|| AllCrates::new(db, std::sync::Arc::default()))
        .set_crates(db)
        .with_durability(durability)
        .to(crates.into_iter().collect());
}

/// Returns the crates in topological order.
///
/// **Warning**: do not use this query in `hir-*` crates! It kills incrementality across crate metadata modifications.
pub fn all_crates(db: &dyn salsa::Database) -> std::sync::Arc<[Crate]> {
    AllCrates::try_get(db).map_or(std::sync::Arc::default(), |all_crates| all_crates.crates(db))
}

#[doc(hidden)]
#[salsa::interned]
pub struct InternedFileRoot {
    #[returns(copy)]
    pub root: FileRoot,
}

#[salsa::tracked(returns(deref))]
pub fn local_file_roots(db: &dyn SourceDatabase) -> Box<[FileRootId]> {
    file_roots(db, FileRootKind::Local)
}

#[salsa::tracked(returns(deref))]
pub fn library_file_roots(db: &dyn SourceDatabase) -> Box<[FileRootId]> {
    file_roots(db, FileRootKind::Library)
}

fn file_roots(db: &dyn SourceDatabase, kind: FileRootKind) -> Box<[FileRootId]> {
    let mut roots = FxHashSet::default();
    for &file in db.files_by_kind(kind).iter() {
        let Some(root) = db.file_root(file) else {
            continue;
        };
        if root.kind == kind {
            roots.insert(root.id);
        }
    }
    let mut roots = roots.into_iter().collect::<Vec<_>>();
    roots.sort();
    roots.into_boxed_slice()
}

pub fn file_root_files(db: &dyn SourceDatabase, root: FileRoot) -> &[File] {
    #[salsa::tracked(returns(deref))]
    pub fn file_root_files<'db>(
        db: &'db dyn SourceDatabase,
        root: InternedFileRoot<'db>,
    ) -> Box<[File]> {
        let root = root.root(db);
        db.files_by_kind(root.kind)
            .iter()
            .copied()
            .filter_map(|file| {
                let file_root = db.file_root(file)?;
                if file_root.id == root.id { Some(file) } else { None }
            })
            .collect()
    }

    file_root_files(db, InternedFileRoot::new(db, root))
}

#[salsa::tracked(returns(deref))]
pub fn local_files(db: &dyn SourceDatabase) -> Box<[File]> {
    db.files_by_kind(FileRootKind::Local).iter().copied().collect()
}

/// Crates whose root file belongs to `root`.
pub fn file_root_crates(db: &dyn SourceDatabase, root: FileRoot) -> &[Crate] {
    #[salsa::tracked(returns(deref))]
    pub fn file_root_crates<'db>(
        db: &'db dyn SourceDatabase,
        root: InternedFileRoot<'db>,
    ) -> Box<[Crate]> {
        let crates = AllCrates::get(db).crates(db);
        let root = root.root(db);
        crates
            .iter()
            .copied()
            .filter(|&krate| {
                let root_file = krate.data(db).root_file_id;
                let Some(file_root) = db.file_root(root_file) else {
                    return false;
                };
                file_root == root
            })
            .collect()
    }
    file_root_crates(db, InternedFileRoot::new(db, root))
}

pub fn relevant_crates(db: &dyn SourceDatabase, file: File) -> &[Crate] {
    let _p = tracing::info_span!("relevant_crates").entered();

    let Some(root) = db.file_root(file) else {
        panic!("Unable to fetch file root for {file:?}; this is a bug")
    };
    file_root_crates(db, root)
}

#[must_use]
#[non_exhaustive]
pub struct DbPanicContext;

impl Drop for DbPanicContext {
    fn drop(&mut self) {
        Self::with_ctx(|ctx| assert!(ctx.pop().is_some()));
    }
}

impl DbPanicContext {
    pub fn enter(frame: String) -> DbPanicContext {
        #[expect(clippy::print_stderr, reason = "already panicking anyway")]
        fn set_hook() {
            let default_hook = panic::take_hook();
            panic::set_hook(Box::new(move |panic_info| {
                default_hook(panic_info);
                if let Some(backtrace) = salsa::Backtrace::capture() {
                    eprintln!("{backtrace:#}");
                }
                DbPanicContext::with_ctx(|ctx| {
                    if !ctx.is_empty() {
                        eprintln!("additional context:");
                        for (idx, frame) in ctx.iter().enumerate() {
                            eprintln!("{idx:>4}: {frame}\n");
                        }
                    }
                });
            }));
        }

        static SET_HOOK: Once = Once::new();
        SET_HOOK.call_once(set_hook);

        Self::with_ctx(|ctx| ctx.push(frame));
        DbPanicContext
    }

    fn with_ctx(f: impl FnOnce(&mut Vec<String>)) {
        thread_local! {
            static CTX: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
        }
        CTX.with(|ctx| f(&mut ctx.borrow_mut()));
    }
}
