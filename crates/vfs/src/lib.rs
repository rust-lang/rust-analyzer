//! # Virtual File System
//!
//! VFS records all file changes pushed to it via [`set_file_contents`].
//! As such it only ever stores changes, not the actual content of a file at any given moment.
//! All file changes are logged, and can be retrieved via
//! [`take_changes`] method. The pack of changes is then pushed to `salsa` and
//! triggers incremental recomputation.
//!
//! Files in VFS are identified with [`FileId`]s -- interned paths. The notion of
//! the path, [`VfsPath`] is somewhat abstract: at the moment, it is represented
//! as an [`std::path::PathBuf`] internally, but this is an implementation detail.
//!
//! VFS doesn't do IO or file watching itself. For that, see the [`loader`]
//! module. [`loader::Handle`] is an object-safe trait which abstracts both file
//! loading and file watching. [`Handle`] is dynamically configured with a set of
//! directory entries which should be scanned and watched. [`Handle`] then
//! asynchronously pushes file changes. Directory entries are configured in
//! free-form via list of globs, it's up to the [`Handle`] to interpret the globs
//! in any specific way.
//!
//! VFS stores a flat list of files. [`file_set::FileSet`] can partition this list
//! of files into disjoint sets of files. Traversal-like operations (including
//! getting the neighbor file by the relative path) are handled by the [`FileSet`].
//! [`FileSet`]s are also pushed to salsa and cause it to re-check `mod foo;`
//! declarations when files are created or deleted.
//!
//! [`FileSet`] and [`loader::Entry`] play similar, but different roles.
//! Both specify the "set of paths/files", one is geared towards file watching,
//! the other towards salsa changes. In particular, single [`FileSet`]
//! may correspond to several [`loader::Entry`]. For example, a crate from
//! crates.io which uses code generation would have two [`Entries`] -- for sources
//! in `~/.cargo`, and for generated code in `./target/debug/build`. It will
//! have a single [`FileSet`] which unions the two sources.
//!
//! [`set_file_contents`]: Vfs::set_file_contents
//! [`take_changes`]: Vfs::take_changes
//! [`FileSet`]: file_set::FileSet
//! [`Handle`]: loader::Handle
//! [`Entries`]: loader::Entry

mod anchored_path;
pub mod file_set;
mod line_endings;
pub mod loader;
mod path_interner;
mod vfs_path;

use std::{fmt, hash::BuildHasherDefault, mem};

use crate::path_interner::PathInterner;

pub use crate::line_endings::LineEndings;
pub use crate::{
    anchored_path::{AnchoredPath, AnchoredPathBuf},
    vfs_path::VfsPath,
};
use indexmap::{IndexMap, map::Entry};
pub use paths::{AbsPath, AbsPathBuf};

use rustc_hash::FxHasher;
use stdx::hash_once;
use tracing::{Level, span};

/// Handle to a file in [`Vfs`]
///
/// Most functions in rust-analyzer use this when they need to refer to a file.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FileId(u32);
// pub struct FileId(NonMaxU32);

impl FileId {
    const MAX: u32 = 0x7fff_ffff;

    #[inline]
    pub const fn from_raw(raw: u32) -> FileId {
        assert!(raw <= Self::MAX);
        FileId(raw)
    }

    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// safe because `FileId` is a newtype of `u32`
impl nohash_hasher::IsEnabled for FileId {}

/// Storage for all file changes and the file id to path mapping.
///
/// For more information see the [crate-level](crate) documentation.
#[derive(Default)]
pub struct Vfs {
    interner: PathInterner,
    data: Vec<FileState>,
    changes: IndexMap<FileId, ChangedFile, BuildHasherDefault<FxHasher>>,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum FileState {
    /// The file exists with the given content hash and detected line endings.
    Exists { hash: u64, line_endings: Option<LineEndings> },
    /// The file is deleted.
    Deleted,
    /// The file was specifically excluded by the user. We still include excluded files
    /// when they're opened (without their contents).
    Excluded,
}

/// Changed file in the [`Vfs`].
#[derive(Debug)]
pub struct ChangedFile {
    /// Id of the changed file
    pub file_id: FileId,
    /// Kind of change
    pub change: Change,
}

impl ChangedFile {
    /// Returns `true` if the change is not [`Delete`](ChangeKind::Delete).
    pub fn exists(&self) -> bool {
        !matches!(self.change, Change::Delete)
    }

    /// Returns `true` if the change is [`Create`](ChangeKind::Create) or
    /// [`Delete`](Change::Delete).
    pub fn is_created_or_deleted(&self) -> bool {
        matches!(self.change, Change::Create(_, _) | Change::Delete)
    }

    /// Returns `true` if the change is [`Create`](ChangeKind::Create).
    pub fn is_created(&self) -> bool {
        matches!(self.change, Change::Create(_, _))
    }

    /// Returns `true` if the change is [`Modify`](ChangeKind::Modify).
    pub fn is_modified(&self) -> bool {
        matches!(self.change, Change::Modify(_, _))
    }

    pub fn kind(&self) -> ChangeKind {
        match self.change {
            Change::Create(_, _) => ChangeKind::Create,
            Change::Modify(_, _) => ChangeKind::Modify,
            Change::Delete => ChangeKind::Delete,
        }
    }
}

/// Kind of [file change](ChangedFile).
#[derive(Eq, PartialEq, Debug)]
pub enum Change {
    /// The file was (re-)created
    Create(FileContents, u64),
    /// The file was modified
    Modify(FileContents, u64),
    /// The file was deleted
    Delete,
}

/// File contents after UTF-8 validation and line-ending normalization.
#[derive(Eq, PartialEq, Debug, Hash)]
pub enum FileContents {
    /// UTF-8 text with CRLF normalized to LF, and its original line-ending convention.
    Text { text: String, line_endings: LineEndings },
    /// Non-UTF-8 bytes, preserved unchanged.
    Binary(Vec<u8>),
}

/// Kind of [file change](ChangedFile).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeKind {
    /// The file was (re-)created
    Create,
    /// The file was modified
    Modify,
    /// The file was deleted
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileExcluded {
    Yes,
    No,
}

impl Vfs {
    /// Id of the given path if it exists in the `Vfs` and is not deleted.
    pub fn file_id(&self, path: &VfsPath) -> Option<(FileId, FileExcluded)> {
        let file_id = self.interner.get(path)?;
        let file_state = self.get(file_id);
        match file_state {
            FileState::Exists { .. } => Some((file_id, FileExcluded::No)),
            FileState::Deleted => None,
            FileState::Excluded => Some((file_id, FileExcluded::Yes)),
        }
    }

    /// File path corresponding to the given `file_id`.
    ///
    /// # Panics
    ///
    /// Panics if the id is not present in the `Vfs`.
    pub fn file_path(&self, file_id: FileId) -> &VfsPath {
        self.interner.lookup(file_id)
    }

    /// The detected line-ending convention.
    pub fn line_endings(&self, file_id: FileId) -> Option<LineEndings> {
        match self.data.get(file_id.0 as usize)? {
            FileState::Exists { line_endings, .. } => *line_endings,
            FileState::Deleted | FileState::Excluded => None,
        }
    }

    /// Returns an iterator over the stored ids and their corresponding paths.
    ///
    /// This will skip deleted files.
    pub fn iter(&self) -> impl Iterator<Item = (FileId, &VfsPath)> + '_ {
        (0..self.data.len())
            .map(|it| FileId(it as u32))
            .filter(move |&file_id| matches!(self.get(file_id), FileState::Exists { .. }))
            .map(move |file_id| {
                let path = self.interner.lookup(file_id);
                (file_id, path)
            })
    }

    /// Update the `path` with the given `contents`. `None` means the file was deleted.
    ///
    /// Returns `true` if the file was modified, and saves the [change](ChangedFile).
    ///
    /// If the path does not currently exist in the `Vfs`, allocates a new
    /// [`FileId`] for it.
    pub fn set_file_contents(&mut self, path: VfsPath, contents: Option<Vec<u8>>) -> bool {
        let _p = span!(Level::INFO, "Vfs::set_file_contents").entered();
        let file_id = self.alloc_file_id(path);
        let state: FileState = self.get(file_id);

        let normalize_line_endings = |v: Vec<u8>| match String::from_utf8(v) {
            Ok(text) => {
                let (text, line_endings) = LineEndings::normalize(text);
                FileContents::Text { text, line_endings }
            }
            Err(err) => FileContents::Binary(err.into_bytes()),
        };

        let change = match (state, contents) {
            (FileState::Deleted, None) => return false,
            (FileState::Deleted, Some(v)) => {
                let contents = normalize_line_endings(v);
                let hash = hash_once::<FxHasher>(&contents);
                Change::Create(contents, hash)
            }
            (FileState::Exists { .. }, None) => Change::Delete,
            (FileState::Exists { hash, .. }, Some(v)) => {
                let contents = normalize_line_endings(v);
                let new_hash = hash_once::<FxHasher>(&contents);
                if new_hash == hash {
                    return false;
                }
                Change::Modify(contents, new_hash)
            }
            (FileState::Excluded, _) => return false,
        };

        let mut set_data = |change_kind: &Change| {
            self.data[file_id.0 as usize] = match change_kind {
                Change::Create(contents, hash) | Change::Modify(contents, hash) => {
                    let line_endings = match contents {
                        FileContents::Text { line_endings, .. } => Some(*line_endings),
                        FileContents::Binary(_) => None,
                    };
                    FileState::Exists { hash: *hash, line_endings }
                }
                Change::Delete => FileState::Deleted,
            };
        };

        let changed_file = ChangedFile { file_id, change };
        match self.changes.entry(file_id) {
            // two changes to the same file in one cycle, merge them appropriately
            Entry::Occupied(mut o) => {
                use Change::*;

                match (&mut o.get_mut().change, changed_file.change) {
                    // newer `Delete` wins
                    (change, Delete) => *change = Delete,
                    // merge `Create` with `Create` or `Modify`
                    (Create(prev, old_hash), Create(new, new_hash) | Modify(new, new_hash)) => {
                        *prev = new;
                        *old_hash = new_hash;
                    }
                    // collapse identical `Modify`es
                    (Modify(prev, old_hash), Modify(new, new_hash)) => {
                        *prev = new;
                        *old_hash = new_hash;
                    }
                    // equivalent to `Modify`
                    (change @ Delete, Create(new, new_hash)) => {
                        *change = Modify(new, new_hash);
                    }
                    // shouldn't occur, but collapse into `Create`
                    (change @ Delete, Modify(new, new_hash)) => {
                        stdx::never!();
                        *change = Create(new, new_hash);
                    }
                    // shouldn't occur, but keep the Create
                    (prev @ Modify(_, _), new @ Create(_, _)) => *prev = new,
                }
                set_data(&o.get().change);
            }
            Entry::Vacant(v) => set_data(&v.insert(changed_file).change),
        };

        true
    }

    /// Drains and returns all the changes in the `Vfs`.
    pub fn take_changes(&mut self) -> IndexMap<FileId, ChangedFile, BuildHasherDefault<FxHasher>> {
        mem::take(&mut self.changes)
    }

    /// Provides a panic-less way to verify file_id validity.
    pub fn exists(&self, file_id: FileId) -> bool {
        matches!(self.get(file_id), FileState::Exists { .. })
    }

    /// Returns the id associated with `path`
    ///
    /// - If `path` does not exist in the `Vfs`, allocates a new id for it, associated with a
    ///   deleted file;
    /// - Else, returns `path`'s id.
    ///
    /// Does not record a change.
    fn alloc_file_id(&mut self, path: VfsPath) -> FileId {
        let file_id = self.interner.intern(path);
        let idx = file_id.0 as usize;
        let len = self.data.len().max(idx + 1);
        self.data.resize(len, FileState::Deleted);
        file_id
    }

    /// Returns the status of the file associated with the given `file_id`.
    ///
    /// # Panics
    ///
    /// Panics if no file is associated to that id.
    fn get(&self, file_id: FileId) -> FileState {
        self.data[file_id.0 as usize]
    }

    /// We cannot ignore excluded files, because this will lead to errors when the client
    /// requests semantic information for them, so we instead mark them specially.
    pub fn insert_excluded_file(&mut self, path: VfsPath) {
        let file_id = self.alloc_file_id(path);
        self.data[file_id.0 as usize] = FileState::Excluded;
    }
}

impl fmt::Debug for Vfs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vfs").field("n_files", &self.data.len()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_endings_survive_draining_changes() {
        let mut vfs = Vfs::default();
        let path = VfsPath::new_virtual_path("/test.rs".into());
        assert_eq!(vfs.line_endings(FileId(0)), None);

        vfs.set_file_contents(path.clone(), Some(b"text\n".to_vec()));
        let (file_id, _) = vfs.file_id(&path).unwrap();
        vfs.set_file_contents(path.clone(), Some(b"text\r\n".to_vec()));
        assert_eq!(vfs.line_endings(file_id), Some(LineEndings::Dos));
        vfs.take_changes();
        assert_eq!(vfs.line_endings(file_id), Some(LineEndings::Dos));

        vfs.set_file_contents(path.clone(), Some(b"\xff\r\n".to_vec()));
        vfs.take_changes();
        assert_eq!(vfs.line_endings(file_id), None);

        vfs.set_file_contents(path.clone(), Some(b"text\n".to_vec()));
        vfs.take_changes();
        assert_eq!(vfs.line_endings(file_id), Some(LineEndings::Unix));

        vfs.set_file_contents(path.clone(), None);
        vfs.take_changes();
        assert_eq!(vfs.line_endings(file_id), None);

        vfs.set_file_contents(path.clone(), Some(b"text\r\n".to_vec()));
        vfs.take_changes();
        assert_eq!(vfs.line_endings(file_id), Some(LineEndings::Dos));
        vfs.insert_excluded_file(path);
        assert_eq!(vfs.line_endings(file_id), None);
    }

    #[test]
    fn changes_preserve_text_and_binary_contents() {
        let mut vfs = Vfs::default();
        let path = VfsPath::new_virtual_path("/test.rs".into());
        for (bytes, expected) in [
            (
                "α\r\n".as_bytes(),
                FileContents::Text { text: "α\n".into(), line_endings: LineEndings::Dos },
            ),
            (b"\xff\r\n".as_slice(), FileContents::Binary(b"\xff\r\n".to_vec())),
            (
                "β\n".as_bytes(),
                FileContents::Text { text: "β\n".into(), line_endings: LineEndings::Unix },
            ),
        ] {
            let bytes = bytes.to_vec();
            let allocation = bytes.as_ptr();
            assert!(vfs.set_file_contents(path.clone(), Some(bytes)));
            let changes = vfs.take_changes();
            assert_eq!(changes.len(), 1);
            let (Change::Create(contents, _) | Change::Modify(contents, _)) =
                &changes.values().next().unwrap().change
            else {
                panic!("expected file contents");
            };
            assert_eq!(*contents, expected);
            let contents_ptr = match contents {
                FileContents::Text { text, .. } => text.as_ptr(),
                FileContents::Binary(bytes) => bytes.as_ptr(),
            };
            assert_eq!(contents_ptr, allocation);
        }
    }

    #[test]
    fn coalesced_changes_keep_latest_line_endings() {
        for modify in [false, true] {
            for previous in [b"unix\n".as_slice(), &[0xff]] {
                let mut vfs = Vfs::default();
                let path = VfsPath::new_virtual_path("/test.rs".into());
                if modify {
                    vfs.set_file_contents(path.clone(), Some(b"initial\n".to_vec()));
                    vfs.take_changes();
                }
                vfs.set_file_contents(path.clone(), Some(previous.to_vec()));
                vfs.set_file_contents(path.clone(), Some(b"dos\r\n".to_vec()));

                let changes = vfs.take_changes();
                assert_eq!(changes.len(), 1);
                let file = changes.values().next().unwrap();
                assert_eq!(file.is_modified(), modify);
                let (Change::Create(contents, _) | Change::Modify(contents, _)) = &file.change
                else {
                    panic!("expected file contents");
                };
                assert_eq!(
                    contents,
                    &FileContents::Text { text: "dos\n".into(), line_endings: LineEndings::Dos }
                );
                assert!(!vfs.set_file_contents(path, Some(b"dos\r\n".to_vec())));
            }
        }
    }

    #[test]
    fn line_ending_only_changes_are_reported() {
        let mut vfs = Vfs::default();
        let path = VfsPath::new_virtual_path("/test.rs".into());
        vfs.set_file_contents(path.clone(), Some(b"text\n".to_vec()));
        vfs.take_changes();

        for (bytes, expected) in
            [(b"text\r\n".as_slice(), LineEndings::Dos), (b"text\n".as_slice(), LineEndings::Unix)]
        {
            assert!(vfs.set_file_contents(path.clone(), Some(bytes.to_vec())));
            let changes = vfs.take_changes();
            assert_eq!(changes.len(), 1);
            let Change::Modify(contents, _) = &changes.values().next().unwrap().change else {
                panic!("expected modification");
            };
            assert_eq!(
                contents,
                &FileContents::Text { text: "text\n".into(), line_endings: expected }
            );
            assert!(!vfs.set_file_contents(path.clone(), Some(bytes.to_vec())));
            assert!(vfs.take_changes().is_empty());
        }
    }
}
