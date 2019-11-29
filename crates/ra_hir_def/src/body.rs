//! Defines `Body`: a lowered representation of bodies of functions, statics and
//! consts.
mod lower;
mod source_map;
pub mod scope;

use std::{ops::Index, sync::Arc};

use hir_expand::{
    hygiene::Hygiene, AstId, HirFileId, InFile, MacroDefId, MacroFileKind,
};
use ra_arena::Arena;
use ra_syntax::{ast, AstNode};

use crate::{
    db::DefDatabase,
    expr::{Expr, ExprId, Pat, PatId},
    nameres::CrateDefMap,
    path::Path,
    DefWithBodyId, ModuleId,
};
pub use source_map::{
    ExprPtr, ExprSource, PatPtr, PatSource,
    BodySourceMap,
};
pub(crate) use source_map::BodyWithSourceMap;

struct Expander {
    crate_def_map: Arc<CrateDefMap>,
    current_file_id: HirFileId,
    hygiene: Hygiene,
    module: ModuleId,
}

impl Expander {
    fn new(db: &impl DefDatabase, current_file_id: HirFileId, module: ModuleId) -> Expander {
        let crate_def_map = db.crate_def_map(module.krate);
        let hygiene = Hygiene::new(db, current_file_id);
        Expander { crate_def_map, current_file_id, hygiene, module }
    }

    fn enter_expand(
        &mut self,
        db: &impl DefDatabase,
        macro_call: ast::MacroCall,
    ) -> Option<(Mark, ast::Expr)> {
        let ast_id = AstId::new(
            self.current_file_id,
            db.ast_id_map(self.current_file_id).ast_id(&macro_call),
        );

        if let Some(path) = macro_call.path().and_then(|path| self.parse_path(path)) {
            if let Some(def) = self.resolve_path_as_macro(db, &path) {
                let call_id = def.as_call_id(db, ast_id);
                let file_id = call_id.as_file(MacroFileKind::Expr);
                if let Some(node) = db.parse_or_expand(file_id) {
                    if let Some(expr) = ast::Expr::cast(node) {
                        log::debug!("macro expansion {:#?}", expr.syntax());

                        let mark = Mark { file_id: self.current_file_id };
                        self.hygiene = Hygiene::new(db, file_id);
                        self.current_file_id = file_id;

                        return Some((mark, expr));
                    }
                }
            }
        }

        // FIXME: Instead of just dropping the error from expansion
        // report it
        None
    }

    fn exit(&mut self, db: &impl DefDatabase, mark: Mark) {
        self.hygiene = Hygiene::new(db, mark.file_id);
        self.current_file_id = mark.file_id;
        std::mem::forget(mark);
    }

    fn to_source<T>(&self, value: T) -> InFile<T> {
        InFile { file_id: self.current_file_id, value }
    }

    fn parse_path(&mut self, path: ast::Path) -> Option<Path> {
        Path::from_src(path, &self.hygiene)
    }

    fn resolve_path_as_macro(&self, db: &impl DefDatabase, path: &Path) -> Option<MacroDefId> {
        self.crate_def_map.resolve_path(db, self.module.local_id, path).0.take_macros()
    }
}

struct Mark {
    file_id: HirFileId,
}

impl Drop for Mark {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            panic!("dropped mark")
        }
    }
}

/// The body of an item (function, const etc.).
#[derive(Debug, Eq, PartialEq)]
pub struct Body {
    pub exprs: Arena<ExprId, Expr>,
    pub pats: Arena<PatId, Pat>,
    /// The patterns for the function's parameters. While the parameter types are
    /// part of the function signature, the patterns are not (they don't change
    /// the external type of the function).
    ///
    /// If this `Body` is for the body of a constant, this will just be
    /// empty.
    pub params: Vec<PatId>,
    /// The `ExprId` of the actual body expression.
    pub body_expr: ExprId,
}

impl Body {
    pub(crate) fn body_with_source_map_query(
        db: &impl DefDatabase,
        def: DefWithBodyId,
    ) -> (Arc<Body>, Arc<BodySourceMap>) {
        let (body, source_map) = lower::lower(db, def);
        (Arc::new(body), Arc::new(source_map))
    }

    pub(crate) fn body_query(db: &impl DefDatabase, def: DefWithBodyId) -> Arc<Body> {
        db.body_with_source_map(def).0
    }
}

impl Index<ExprId> for Body {
    type Output = Expr;

    fn index(&self, expr: ExprId) -> &Expr {
        &self.exprs[expr]
    }
}

impl Index<PatId> for Body {
    type Output = Pat;

    fn index(&self, pat: PatId) -> &Pat {
        &self.pats[pat]
    }
}
