use ra_syntax::ast;

use crate::ids::HirFileId;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Source<T> {
    pub file_id: HirFileId,
    pub ast: T,
}

pub enum ModuleSource {
    SourceFile(ast::SourceFile),
    Module(ast::Module),
}
