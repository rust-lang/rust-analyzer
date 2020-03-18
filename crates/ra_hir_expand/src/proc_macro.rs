//! Proc Macro Expander stub

use crate::{db::AstDatabase, LazyMacroId, MacroCallKind, MacroCallLoc};
use ra_db::CrateId;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ProcMacroExpander {
    krate: CrateId,
}

impl ProcMacroExpander {
    pub fn new(krate: CrateId) -> ProcMacroExpander {
        ProcMacroExpander { krate }
    }

    pub fn expand(
        &self,
        db: &dyn AstDatabase,
        id: LazyMacroId,
        tt: &tt::Subtree,
    ) -> Result<tt::Subtree, mbe::ExpandError> {
        let name = self.macro_name(db, id).ok_or_else(|| mbe::ExpandError::ConversionError)?;
        let proc_macro = db.crate_graph()[self.krate]
            .proc_macro
            .clone()
            .ok_or_else(|| mbe::ExpandError::ConversionError)?;
        proc_macro.custom_derive(tt, &name)
    }

    fn macro_name(&self, db: &dyn AstDatabase, id: LazyMacroId) -> Option<String> {
        let loc: MacroCallLoc = db.lookup_intern_macro(id);
        match loc.kind {
            MacroCallKind::FnLike(_) => None,
            MacroCallKind::Attr(_, name) => Some(name),
        }
    }
}
