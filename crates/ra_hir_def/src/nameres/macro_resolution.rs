//! This module resolves a macro with path to a file.
use hir_expand::{hygiene::Hygiene, HirFileId, MacroCallKind, MacroDefId};
use ra_syntax::ast;

use crate::{
    db::DefDatabase,
    nameres::{BuiltinShadowMode, CrateDefMap},
    path::{Path, PathKind},
    AstId, InFile, LocalModuleId,
};

pub struct MacroResolver<'a> {
    crate_def_map: &'a CrateDefMap,
    module: LocalModuleId,
}

impl<'a> MacroResolver<'a> {
    fn resolve(
        &self,
        db: &impl DefDatabase,
        ast_id: AstId<ast::MacroCall>,
        path: &Path,
    ) -> Option<HirFileId> {
        let def = self.resolve_path_as_macro(db, &path)?;
        let call_id = def.as_call_id(db, MacroCallKind::FnLike(ast_id));
        let file_id = call_id.as_file();
        Some(file_id)
    }

    fn resolve_path_as_macro(&self, db: &impl DefDatabase, path: &Path) -> Option<MacroDefId> {
        self.crate_def_map
            .resolve_path(db, self.module, path, BuiltinShadowMode::Other)
            .0
            .take_macros()
    }
}

impl CrateDefMap {
    pub(crate) fn resolve_macro_as_file(
        &self,
        db: &impl DefDatabase,
        macro_call: InFile<ast::MacroCall>,
        hygiene: &Hygiene,
        module: LocalModuleId,
        force_self: bool,
    ) -> Option<HirFileId> {
        let ast_id = macro_call.clone().map(|it| db.ast_id_map(macro_call.file_id).ast_id(&it));
        let mut path = macro_call.value.path().and_then(|path| Path::from_src(path, &hygiene))?;
        // We rewrite simple path `macro_name` to `self::macro_name` to force resolve in module scope only.
        if force_self && path.is_ident() {
            path.kind = PathKind::Self_;
        }

        let resolver = MacroResolver { crate_def_map: &self, module };
        resolver.resolve(db, ast_id, &path)
    }
}
