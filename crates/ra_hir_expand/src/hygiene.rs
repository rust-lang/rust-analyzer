//! This modules handles hygiene information.
//!
//! Specifically, `ast` + `Hygiene` allows you to create a `Name`. Note that, at
//! this moment, this is horribly incomplete and handles only `$crate`.
use either::Either;
use ra_db::CrateId;
use ra_syntax::ast;

use crate::{
    db::AstDatabase,
    name::{AsName, Name},
    HirFileId, HirFileIdRepr, MacroCallId, MacroDefKind,
};

#[derive(Debug)]
pub struct Hygiene {
    // This is what `$crate` expands to
    def_crate: Option<CrateId>,
    local_inner_macros: bool,
}

impl Hygiene {
    pub fn new(db: &dyn AstDatabase, file_id: HirFileId) -> Hygiene {
        let mut local_inner_macros = false;
        let def_crate = match file_id.0 {
            HirFileIdRepr::FileId(_) => None,
            HirFileIdRepr::MacroFile(macro_file) => match macro_file.macro_call_id {
                MacroCallId::LazyMacro(id) => {
                    let loc = db.lookup_intern_macro(id);
                    local_inner_macros = loc.def.local_inner_macros(db);
                    match loc.def.kind {
                        MacroDefKind::Declarative { .. } => loc.def.krate,
                        MacroDefKind::BuiltIn(_) => None,
                        MacroDefKind::BuiltInDerive(_) => None,
                        MacroDefKind::BuiltInEager(_) => None,
                        MacroDefKind::CustomDerive(_) => None,
                    }
                }
                MacroCallId::EagerMacro(_id) => None,
            },
        };

        Hygiene { def_crate, local_inner_macros }
    }

    pub fn new_unhygienic() -> Hygiene {
        Hygiene { def_crate: None, local_inner_macros: false }
    }

    /// Whether paths to macros should be treated as starting with a `$crate` segment.
    pub fn implicit_dollar_crate(&self) -> Option<CrateId> {
        if self.local_inner_macros {
            self.def_crate
        } else {
            None
        }
    }

    // FIXME: this should just return name
    pub fn name_ref_to_name(&self, name_ref: ast::NameRef) -> Either<Name, CrateId> {
        if let Some(def_crate) = self.def_crate {
            if name_ref.text() == "$crate" {
                return Either::Right(def_crate);
            }
        }
        Either::Left(name_ref.as_name())
    }
}
