use hir_expand::{HirFileId, MacroCallKind, MacroDefId};
use ra_syntax::ast;

use crate::{
    db::DefDatabase,
    nameres::{BuiltinShadowMode, CrateDefMap},
    path::Path,
    AstId, LocalModuleId,
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
        ast_id: AstId<ast::MacroCall>,
        path: &Path,
        module: LocalModuleId,
    ) -> Option<HirFileId> {
        let resolver = MacroResolver { crate_def_map: &self, module };

        resolver.resolve(db, ast_id, path)
    }
}
