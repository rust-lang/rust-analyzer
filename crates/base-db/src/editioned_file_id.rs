//! Defines [`EditionedFileId`], an interned wrapper around [`span::EditionedFileId`] that
//! is interned (so queries can take it) and stores only the underlying `span::EditionedFileId`.

use std::hash::Hash;

use salsa::Database;
use span::{Edition, File};
use syntax::{SyntaxError, ast};

use crate::SourceDatabase;

#[salsa::interned(debug, constructor = from_span_file_id, unsafe(no_lifetime), revisions = usize::MAX)]
#[derive(PartialOrd, Ord)]
pub struct EditionedFileId {
    #[returns(copy)]
    field: span::EditionedFileId,
}

#[salsa::tracked]
impl EditionedFileId {
    #[salsa::tracked(lru = 128, returns(clone))]
    pub fn parse(self, db: &dyn SourceDatabase) -> syntax::Parse<ast::SourceFile> {
        let _p = tracing::info_span!("parse", ?self).entered();
        let (file, edition) = self.unpack(db);
        let data = db.file_data(file);
        let text = data.text(db);
        ast::SourceFile::parse(text, edition)
    }

    // firewall query
    #[salsa::tracked(returns(as_deref))]
    pub fn parse_errors(self, db: &dyn SourceDatabase) -> Option<Box<[SyntaxError]>> {
        let errors = self.parse(db).errors();
        match &*errors {
            [] => None,
            [..] => Some(errors.into()),
        }
    }
}

impl EditionedFileId {
    #[inline]
    pub fn new(db: &dyn Database, file: File, edition: Edition) -> Self {
        Self::from_span_file_id(db, span::EditionedFileId::new(file, edition))
    }

    #[inline]
    pub fn current_edition(db: &dyn Database, file: File) -> Self {
        Self::from_span_file_id(db, span::EditionedFileId::current_edition(file))
    }

    #[inline]
    pub fn file(self, db: &dyn Database) -> File {
        self.field(db).file()
    }

    #[inline]
    pub fn span_file_id(self, db: &dyn Database) -> span::EditionedFileId {
        self.field(db)
    }

    #[inline]
    pub fn unpack(self, db: &dyn Database) -> (File, span::Edition) {
        self.field(db).unpack()
    }

    #[inline]
    pub fn edition(self, db: &dyn Database) -> Edition {
        self.field(db).edition()
    }
}
