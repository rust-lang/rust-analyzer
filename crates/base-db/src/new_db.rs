//! //! `base_db`` defines basic database traits.
//!
//! This module has a similar purpose as [`crate::SourceDatabase`], in that
//! it bridges to non-Salsa managed data. This module should not be used until
//! all of rust-analyzer has moved to the new Salsa.

use std::hash::BuildHasherDefault;

use dashmap::DashMap;
use rustc_hash::FxHasher;
use salsa::{Accumulator, Durability, Event};
use span::Edition;
use syntax::{ast, Parse, SyntaxError};
use triomphe::Arc;

use crate::{CrateId, ReleaseChannel};

#[salsa::tracked]
pub struct ParsedFile<'db> {
    pub file: Parse<ast::SourceFile>,
}

#[salsa::input]
pub struct SourceFile {
    #[return_ref]
    text: String,
}

/// Returns the set of errors obtained from parsing the file.
#[salsa::accumulator]
pub struct ParseError {
    pub error: SyntaxError,
}

#[salsa::input]
pub struct CrateGraph {
    pub crate_graph: Arc<crate::input::CrateGraph>,
}

#[salsa::input]
pub struct CrateWorkspaceData {
    #[return_ref]
    data: Arc<rustc_hash::FxHashMap<CrateId, Arc<crate::CrateWorkspaceData>>>,
}

/// Database which stores all significant input facts: source code and project
/// model. Everything else in rust-analyzer is derived from these queries.
#[salsa::db]
pub trait Db: salsa::Database {
    /// Text of the file.
    fn file_text(&self, file_id: vfs::FileId) -> SourceFile;

    fn parse_errors(&self, file: SourceFile) -> Option<Vec<ParseError>>;

    /// The crate graph.
    fn crate_graph(&self) -> CrateGraph;

    fn crate_workspace_data(&self) -> CrateWorkspaceData;

    fn toolchain_channel(&self, krate: CrateId) -> Option<ReleaseChannel>;
}

#[salsa::tracked]
pub fn parse<'db>(db: &'db dyn Db, file: SourceFile) -> ParsedFile<'db> {
    let _p = tracing::info_span!("parse", ?file).entered();
    let edition = Edition::CURRENT; // TODO: use the actual edition.
    let text = file.text(db);

    let parsed = ast::SourceFile::parse(text, edition);
    for error in parsed.errors() {
        ParseError { error }.accumulate(db);
    }

    ParsedFile::new(db, parsed)
}

#[salsa::db]
pub struct SourceDatabase {
    storage: salsa::Storage<Self>,
    files: DashMap<vfs::FileId, SourceFile, BuildHasherDefault<FxHasher>>,
    crate_graph: CrateGraph,
    workspace_data: CrateWorkspaceData,
}

impl SourceDatabase {
    pub fn set_file_text(&self, file_id: vfs::FileId, text: &str, durability: Durability) {
        let file = SourceFile::builder(text.to_owned()).durability(durability).new(self);
        self.files.insert(file_id, file);
    }
}

#[salsa::db]
impl salsa::Database for SourceDatabase {
    fn salsa_event(&self, _event: &dyn Fn() -> Event) {}
}

#[salsa::db]
impl Db for SourceDatabase {
    fn file_text(&self, file_id: vfs::FileId) -> SourceFile {
        *self.files.get(&file_id).expect("Unable to get non-existent file")
    }

    fn parse_errors(&self, file: SourceFile) -> Option<Vec<ParseError>> {
        let _parsed = parse(self, file);
        let errors = parse::accumulated::<ParseError>(self, file);

        if errors.is_empty() {
            None
        } else {
            Some(errors)
        }
    }

    fn crate_graph(&self) -> CrateGraph {
        self.crate_graph
    }

    fn crate_workspace_data(&self) -> CrateWorkspaceData {
        self.workspace_data
    }

    fn toolchain_channel(&self, krate: CrateId) -> Option<ReleaseChannel> {
        let db = self;

        db.crate_workspace_data()
            .data(db)
            .get(&krate)?
            .toolchain
            .as_ref()
            .and_then(|v| ReleaseChannel::from_str(&v.pre))
    }
}
