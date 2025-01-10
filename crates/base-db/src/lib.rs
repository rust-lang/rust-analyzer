//! base_db defines basic database traits. The concrete DB is defined by ide.
// FIXME: Rename this crate, base db is non descriptive
mod change;
mod input;

use std::hash::BuildHasherDefault;

pub use crate::{
    change::FileChange,
    input::{
        CrateData, CrateDisplayName, CrateGraph, CrateId, CrateName, CrateOrigin, Dependency, Env,
        LangCrateOrigin, ProcMacroPaths, ReleaseChannel, SourceRoot, SourceRootId,
        TargetLayoutLoadResult,
    },
};
use dashmap::{mapref::entry::Entry, DashMap};
pub use db_ext_macro::{self};
use rustc_hash::{FxHashMap, FxHasher};
pub use salsa::{self};
use salsa::{Durability, Setter};
pub use semver::{BuildMetadata, Prerelease, Version, VersionReq};
use syntax::{ast, Parse, SyntaxError};
use triomphe::Arc;
pub use vfs::{file_set::FileSet, AnchoredPath, AnchoredPathBuf, VfsPath};
use vfs::{AbsPathBuf, FileId};

#[macro_export]
macro_rules! impl_intern_key {
    ($name:ident) => {
        impl $crate::salsa::plumbing::AsId for $name {
            fn as_id(&self) -> $crate::salsa::Id {
                self.0
            }
        }

        impl $crate::salsa::plumbing::FromId for $name {
            fn from_id(id: salsa::Id) -> Self {
                $name(id)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_wrapper {
    ($id:ident, $loc:ident, $intern:ident) => {
        #[salsa::interned_sans_lifetime(id = $id)]
        pub struct $intern {
            pub loc: $loc,
        }
    };
}

pub trait Upcast<T: ?Sized> {
    fn upcast(&self) -> &T;
}

pub const DEFAULT_FILE_TEXT_LRU_CAP: u16 = 16;
pub const DEFAULT_PARSE_LRU_CAP: u16 = 128;
pub const DEFAULT_BORROWCK_LRU_CAP: u16 = 2024;

/// Crate related data shared by the whole workspace.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CrateWorkspaceData {
    /// The working directory to run proc-macros in. This is usually the workspace root of cargo workspaces.
    pub proc_macro_cwd: Option<AbsPathBuf>,
    // FIXME: Consider removing this, making HirDatabase::target_data_layout an input query
    pub data_layout: TargetLayoutLoadResult,
    /// Toolchain version used to compile the crate.
    pub toolchain: Option<Version>,
}

#[derive(Debug, Default)]
pub struct Files {
    files: Arc<DashMap<vfs::FileId, FileText, BuildHasherDefault<FxHasher>>>,
    source_roots: Arc<DashMap<SourceRootId, SourceRootInput, BuildHasherDefault<FxHasher>>>,
    file_source_roots: Arc<DashMap<vfs::FileId, FileSourceRootInput, BuildHasherDefault<FxHasher>>>,
}

impl Files {
    pub fn file_text(&self, file_id: vfs::FileId) -> FileText {
        *self.files.get(&file_id).expect("Unable to fetch file; this is a bug")
    }

    pub fn set_file_text(&self, db: &mut dyn SourceDatabase, file_id: vfs::FileId, text: &str) {
        let files = Arc::clone(&self.files);
        match files.entry(file_id) {
            Entry::Occupied(mut occupied) => {
                occupied.get_mut().set_text(db).to(Arc::from(text));
            }
            Entry::Vacant(vacant) => {
                let text = FileText::new(db, Arc::from(text), file_id);
                vacant.insert(text);
            }
        };
    }

    pub fn set_file_text_with_durability(
        &self,
        db: &mut dyn SourceDatabase,
        file_id: vfs::FileId,
        text: &str,
        durability: Durability,
    ) {
        let files = Arc::clone(&self.files);
        match files.entry(file_id) {
            Entry::Occupied(mut occupied) => {
                occupied.get_mut().set_text(db).to(Arc::from(text));
            }
            Entry::Vacant(vacant) => {
                let text =
                    FileText::builder(Arc::from(text), file_id).durability(durability).new(db);
                vacant.insert(text);
            }
        };
    }

    /// Source root of the file.
    pub fn source_root(&self, source_root_id: SourceRootId) -> SourceRootInput {
        let source_root = self
            .source_roots
            .get(&source_root_id)
            .expect("Unable to fetch source root id; this is a bug");

        *source_root
    }

    pub fn set_source_root_with_durability(
        &self,
        db: &mut dyn SourceDatabase,
        source_root_id: SourceRootId,
        source_root: Arc<SourceRoot>,
        durability: Durability,
    ) {
        let source_roots = Arc::clone(&self.source_roots);
        match source_roots.entry(source_root_id) {
            Entry::Occupied(mut occupied) => {
                occupied.get_mut().set_source_root(db).to(source_root);
            }
            Entry::Vacant(vacant) => {
                let source_root =
                    SourceRootInput::builder(source_root).durability(durability).new(db);
                vacant.insert(source_root);
            }
        };
    }

    pub fn file_source_root(&self, id: vfs::FileId) -> FileSourceRootInput {
        let file_source_root = self
            .file_source_roots
            .get(&id)
            .expect("Unable to fetch FileSourceRootInput; this is a bug");
        *file_source_root
    }

    pub fn set_file_source_root_with_durability(
        &self,
        db: &mut dyn SourceDatabase,
        id: vfs::FileId,
        source_root_id: SourceRootId,
        durability: Durability,
    ) {
        let file_source_roots = Arc::clone(&self.file_source_roots);
        // let db = self;
        match file_source_roots.entry(id) {
            Entry::Occupied(mut occupied) => {
                occupied.get_mut().set_source_root_id(db).to(source_root_id);
            }
            Entry::Vacant(vacant) => {
                let file_source_root =
                    FileSourceRootInput::builder(source_root_id).durability(durability).new(db);
                vacant.insert(file_source_root);
            }
        };
    }
}

#[salsa::interned_sans_lifetime]
pub struct EditionedFileId {
    pub file_id: FileText,
    pub editioned_file_id: span::EditionedFileId,
}

#[salsa::input]
pub struct FileText {
    pub text: Arc<str>,
    pub file_id: vfs::FileId,
}

#[salsa::input]
pub struct FileSourceRootInput {
    pub source_root_id: SourceRootId,
}

#[salsa::input]
pub struct SourceRootInput {
    pub source_root: Arc<SourceRoot>,
}

/// Database which stores all significant input facts: source code and project
/// model. Everything else in rust-analyzer is derived from these queries.
#[db_ext_macro::query_group]
pub trait RootQueryDb: SourceDatabase + salsa::Database {
    /// Parses the file into the syntax tree.
    #[db_ext_macro::invoke_actual(parse)]
    #[db_ext_macro::lru(128)]
    fn parse(&self, file_id: EditionedFileId) -> Parse<ast::SourceFile>;

    /// Returns the set of errors obtained from parsing the file including validation errors.
    fn parse_errors(&self, file_id: EditionedFileId) -> Option<Arc<[SyntaxError]>>;

    /// The crate graph.
    #[db_ext_macro::input]
    fn crate_graph(&self) -> Arc<CrateGraph>;

    #[db_ext_macro::input]
    fn crate_workspace_data(&self) -> Arc<FxHashMap<CrateId, Arc<CrateWorkspaceData>>>;

    #[db_ext_macro::transparent]
    fn toolchain_channel(&self, krate: CrateId) -> Option<ReleaseChannel>;

    /// Crates whose root file is in `id`.
    fn source_root_crates(&self, id: SourceRootId) -> Arc<[CrateId]>;

    #[db_ext_macro::transparent]
    fn relevant_crates(&self, file_id: FileId) -> Arc<[CrateId]>;
}

#[salsa::db]
pub trait SourceDatabase: salsa::Database {
    /// Text of the file.
    fn file_text(&self, file_id: vfs::FileId) -> FileText;

    fn set_file_text(&mut self, file_id: vfs::FileId, text: &str);

    fn set_file_text_with_durability(
        &mut self,
        file_id: vfs::FileId,
        text: &str,
        durability: Durability,
    );

    /// Contents of the source root.
    fn source_root(&self, id: SourceRootId) -> SourceRootInput;

    fn file_source_root(&self, id: vfs::FileId) -> FileSourceRootInput;

    fn set_file_source_root_with_durability(
        &mut self,
        id: vfs::FileId,
        source_root_id: SourceRootId,
        durability: Durability,
    );

    /// Source root of the file.
    fn set_source_root_with_durability(
        &mut self,
        source_root_id: SourceRootId,
        source_root: Arc<SourceRoot>,
        durability: Durability,
    );

    fn resolve_path(&self, path: AnchoredPath<'_>) -> Option<FileId> {
        // FIXME: this *somehow* should be platform agnostic...
        let source_root = self.file_source_root(path.anchor);
        let source_root = self.source_root(source_root.source_root_id(self));
        source_root.source_root(self).resolve_path(path)
    }
}

fn toolchain_channel(db: &dyn RootQueryDb, krate: CrateId) -> Option<ReleaseChannel> {
    db.crate_workspace_data()
        .get(&krate)?
        .toolchain
        .as_ref()
        .and_then(|v| ReleaseChannel::from_str(&v.pre))
}

fn parse(db: &dyn RootQueryDb, file_id: EditionedFileId) -> Parse<ast::SourceFile> {
    let _p = tracing::info_span!("parse", ?file_id).entered();
    let (text, editioned_file_id) = (file_id.file_id(db), file_id.editioned_file_id(db));
    let (_, edition) = editioned_file_id.unpack();
    ast::SourceFile::parse(&text.text(db), edition)
}

fn parse_errors(db: &dyn RootQueryDb, file_id: EditionedFileId) -> Option<Arc<[SyntaxError]>> {
    let errors = db.parse(file_id).errors();
    match &*errors {
        [] => None,
        [..] => Some(errors.into()),
    }
}

fn source_root_crates(db: &dyn RootQueryDb, id: SourceRootId) -> Arc<[CrateId]> {
    let graph = db.crate_graph();
    let mut crates = graph
        .iter()
        .filter(|&krate| {
            let root_file = graph[krate].root_file_id;
            db.file_source_root(root_file).source_root_id(db) == id
        })
        .collect::<Vec<_>>();
    crates.sort();
    crates.dedup();
    crates.into_iter().collect()
}

fn relevant_crates(db: &dyn RootQueryDb, file_id: FileId) -> Arc<[CrateId]> {
    let _p = tracing::info_span!("relevant_crates").entered();

    let source_root = db.file_source_root(file_id);
    db.source_root_crates(source_root.source_root_id(db))
}
