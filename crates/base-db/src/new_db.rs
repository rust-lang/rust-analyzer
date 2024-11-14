use std::hash::BuildHasherDefault;

use dashmap::DashMap;
use rustc_hash::{FxHashMap, FxHasher};
use salsa::Durability;
use span::EditionedFileId;
use syntax::{ast, Parse, SyntaxError};
use triomphe::Arc;

use crate::{
    CrateGraph, CrateId, CrateWorkspaceData, ReleaseChannel, SourceRoot, SourceRootId, Upcast,
};

#[salsa::input]
pub struct FileText {
    pub file_id: vfs::FileId,
    pub text: Arc<String>,
}

#[salsa::input]
pub struct SourceRootInput {
    pub source_root_id: SourceRootId,
    pub source_root: Arc<SourceRoot>,
}

/// Database which stores all significant input facts: source code and project
/// model. Everything else in rust-analyzer is derived from these queries.
#[db_ext_macro::query_group]
pub trait RootQueryDb: SourceDb + salsa::Database {
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
pub trait SourceDb: salsa::Database {
    /// Text of the file.
    fn file_text(&self, file_id: vfs::FileId) -> FileText;

    fn set_file_text(&self, file_id: vfs::FileId, text: Arc<String>);

    /// Contents of the source root.
    fn source_root(&self, id: vfs::FileId) -> SourceRootInput;

    /// Source root of the file.
    fn set_source_root_with_durability(
        &self,
        file_id: vfs::FileId,
        source_root_id: SourceRootId,
        source_root: Arc<SourceRoot>,
        durability: Durability,
    );
}

impl Upcast<dyn RootQueryDb> for Db {
    fn upcast(&self) -> &(dyn RootQueryDb + 'static) {
        self
    }
}

impl Upcast<dyn SourceDb> for Db {
    fn upcast(&self) -> &(dyn SourceDb + 'static) {
        self
    }
}

#[salsa::db]
pub struct Db {
    storage: salsa::Storage<Self>,
    files: DashMap<vfs::FileId, FileText, BuildHasherDefault<FxHasher>>,
    source_roots: DashMap<vfs::FileId, SourceRootInput, BuildHasherDefault<FxHasher>>,
}

impl Clone for Db {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            files: self.files.clone(),
            source_roots: self.source_roots.clone(),
        }
    }
}

#[salsa::db]
impl SourceDb for Db {
    fn file_text(&self, file_id: vfs::FileId) -> FileText {
        *self.files.get(&file_id).expect("Unable to fetch file; this is a bug")
    }

    fn set_file_text(&self, file_id: vfs::FileId, text: Arc<String>) {
        self.files.insert(file_id, FileText::new(self, file_id, text));
    }

    /// Source root of the file.
    fn source_root(&self, file_id: vfs::FileId) -> SourceRootInput {
        let source_root =
            self.source_roots.get(&file_id).expect("Unable to fetch source root id; this is a bug");

        *source_root
    }

    fn set_source_root_with_durability(
        &self,
        file_id: vfs::FileId,
        source_root_id: SourceRootId,
        source_root: Arc<SourceRoot>,
        durability: Durability,
    ) {
        let input =
            SourceRootInput::builder(source_root_id, source_root).durability(durability).new(self);
        self.source_roots.insert(file_id, input);
    }
}

#[salsa::db]
impl salsa::Database for Db {
    fn salsa_event(&self, _event: &dyn Fn() -> salsa::Event) {}
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
            db.source_root(root_file).source_root_id(db) == id
        })
        .collect::<Vec<_>>();
    crates.sort();
    crates.dedup();
    crates.into_iter().collect()
}
