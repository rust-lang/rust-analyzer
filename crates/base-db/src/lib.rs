//! base_db defines basic database traits. The concrete DB is defined by ide.
// FIXME: Rename this crate, base db is non descriptive
mod change;
mod input;

pub use crate::{
    change::FileChange,
    input::{
        CrateData, CrateDisplayName, CrateGraph, CrateId, CrateName, CrateOrigin, Dependency, Env,
        LangCrateOrigin, ProcMacroPaths, ReleaseChannel, SourceRoot, SourceRootId,
        TargetLayoutLoadResult,
    },
};
pub use db_ext_macro::{self};
pub use ra_salsa::InternValueTrivial;
use rustc_hash::FxHashMap;
use salsa::Durability;
pub use salsa::{self};
pub use semver::{BuildMetadata, Prerelease, Version, VersionReq};
use span::EditionedFileId;
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

#[salsa::input]
pub struct FileText {
    pub file_id: vfs::FileId,
    pub text: Arc<str>,
}

#[salsa::input]
pub struct FileSourceRootInput {
    pub file_id: vfs::FileId,
    pub source_root_id: SourceRootId,
}

#[salsa::input]
pub struct SourceRootInput {
    pub source_root_id: SourceRootId,
    pub source_root: Arc<SourceRoot>,
}

/// Database which stores all significant input facts: source code and project
/// model. Everything else in rust-analyzer is derived from these queries.
#[db_ext_macro::query_group]
pub trait RootQueryDb: SourceDatabase + salsa::Database {
    /// Parses the file into the syntax tree.
    // #[db_ext_macro::lru]
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
}

#[salsa::db]
pub trait SourceDatabase: salsa::Database {
    /// Text of the file.
    fn file_text(&self, file_id: vfs::FileId) -> FileText;

    fn set_file_text(&self, file_id: vfs::FileId, text: &str);

    fn set_file_text_with_durability(
        &self,
        file_id: vfs::FileId,
        text: &str,
        durability: Durability,
    );

    /// Contents of the source root.
    fn source_root(&self, id: SourceRootId) -> SourceRootInput;

    fn file_source_root(&self, id: vfs::FileId) -> FileSourceRootInput;

    fn set_file_source_root_with_durability(
        &self,
        id: vfs::FileId,
        source_root_id: SourceRootId,
        durability: Durability,
    );

    /// Source root of the file.
    fn set_source_root_with_durability(
        &self,
        source_root_id: SourceRootId,
        source_root: Arc<SourceRoot>,
        durability: Durability,
    );

    fn resolve_path(&self, path: AnchoredPath<'_>) -> Option<FileId>;

    /// Crates whose root's source root is the same as the source root of `file_id`
    fn relevant_crates(&self, file_id: FileId) -> Arc<[CrateId]>;
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
    let (file_id, edition) = file_id.unpack();
    let text = db.file_text(file_id);
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
