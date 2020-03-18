//! Proc Macro Expander stub

use crate::{db::AstDatabase, LazyMacroId, MacroCallKind, MacroCallLoc};
use mbe::{ExpandError, ProcMacroError};
use ra_db::CrateId;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ProcMacroExpander {
    krate: CrateId,
}

macro_rules! err {
    ($fmt:literal, $($tt:tt),*) => {
        ExpandError::ProcMacroError(ProcMacroError::Unknown(format!($fmt, $($tt),*)))
    };
    ($fmt:literal) => {
        ExpandError::ProcMacroError(ProcMacroError::Unknown($fmt.to_string()))
    }
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
        let name = self.macro_name(db, id).ok_or_else(|| err!("Fail to find name in macro"))?;

        let tt = remove_derive_atr(tt, &name)
            .ok_or_else(|| err!("Fail to remove attr in macro {}", name))?;
        let proc_macro = db.crate_graph()[self.krate]
            .proc_macro
            .clone()
            .ok_or_else(|| err!("Fail to find krate in proc macro"))?;
        proc_macro.custom_derive(&tt, &name).map_err(|err| {
            match err {
                ProcMacroError::Dummy => (),
                _ => {
                    eprintln!("Proc macro expansion error : {:?}", err);
                }
            }
            err.into()
        })
    }

    fn macro_name(&self, db: &dyn AstDatabase, id: LazyMacroId) -> Option<String> {
        let loc: MacroCallLoc = db.lookup_intern_macro(id);
        match loc.kind {
            MacroCallKind::FnLike(_) => None,
            MacroCallKind::Attr(_, name) => Some(name),
        }
    }
}

fn remove_derive_atr(tt: &tt::Subtree, _name: &str) -> Option<tt::Subtree> {
    // FIXME: better handling
    // We have a bug in mbe such that the following logic is wrong
    // // We assume the first 2 tokens are #[derive(name)]
    // if tt.token_trees.len() > 2 {
    //     let mut tt = tt.clone();
    //     tt.token_trees.remove(0);
    //     tt.token_trees.remove(0);
    //     return Some(tt);
    // }
    if tt.token_trees.len() > 2 {
        let mut tt = tt.clone();
        tt.token_trees.remove(0);
        while tt.token_trees.len() > 0 {
            let curr = tt.token_trees.remove(0);
            if let tt::TokenTree::Leaf(tt::Leaf::Punct(punct)) = curr {
                if punct.char == ']' {
                    break;
                }
            }
        }

        return Some(tt);
    }

    None
}
