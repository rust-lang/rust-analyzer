//! In rust, it is possible to have a value, a type and a macro with the same
//! name without conflicts.
//!
//! `PerNs` (per namespace) captures this.

use hir_expand::MacroDefId;

use crate::{item_scope::ItemInNs, visibility::Visibility, ModuleDefId};
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PerNs {
    pub types: Option<(ModuleDefId, Visibility)>,
    pub values: Option<(ModuleDefId, Visibility)>,
    pub macros: Option<(MacroDefId, Visibility)>,
}

impl Default for PerNs {
    fn default() -> Self {
        PerNs { types: None, values: None, macros: None }
    }
}

impl PerNs {
    pub fn none() -> PerNs {
        PerNs { types: None, values: None, macros: None }
    }

    pub fn values(t: ModuleDefId, v: Visibility) -> PerNs {
        PerNs { types: None, values: Some((t, v)), macros: None }
    }

    pub fn types(t: ModuleDefId, v: Visibility) -> PerNs {
        PerNs { types: Some((t, v)), values: None, macros: None }
    }

    pub fn both(types: ModuleDefId, values: ModuleDefId, v: Visibility) -> PerNs {
        PerNs { types: Some((types, v)), values: Some((values, v)), macros: None }
    }

    pub fn macros(macro_: MacroDefId, v: Visibility) -> PerNs {
        PerNs { types: None, values: None, macros: Some((macro_, v)) }
    }

    pub fn is_none(&self) -> bool {
        self.types.is_none() && self.values.is_none() && self.macros.is_none()
    }

    pub fn take_types(self) -> Option<ModuleDefId> {
        self.types.map(|it| it.0)
    }

    pub fn take_types_vis(self) -> Option<(ModuleDefId, Visibility)> {
        self.types
    }

    pub fn take_values(self) -> Option<ModuleDefId> {
        self.values.map(|it| it.0)
    }

    pub fn take_macros(self) -> Option<MacroDefId> {
        self.macros.map(|it| it.0)
    }

    pub fn filter_visibility(self, mut f: impl FnMut(Visibility) -> bool) -> PerNs {
        PerNs {
            types: self.types.filter(|(_, v)| f(*v)),
            values: self.values.filter(|(_, v)| f(*v)),
            macros: self.macros.filter(|(_, v)| f(*v)),
        }
    }

    pub fn with_visibility(self, vis: Visibility) -> PerNs {
        PerNs {
            types: self.types.map(|(it, _)| (it, vis)),
            values: self.values.map(|(it, _)| (it, vis)),
            macros: self.macros.map(|(it, _)| (it, vis)),
        }
    }

    pub fn or(self, other: PerNs) -> PerNs {
        PerNs {
            types: self.types.or(other.types),
            values: self.values.or(other.values),
            macros: self.macros.or(other.macros),
        }
    }

    pub fn iter_items(self) -> impl Iterator<Item = ItemInNs> {
        self.types
            .map(|it| ItemInNs::Types(it.0))
            .into_iter()
            .chain(self.values.map(|it| ItemInNs::Values(it.0)).into_iter())
            .chain(self.macros.map(|it| ItemInNs::Macros(it.0)).into_iter())
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Compressed(Repr);

#[derive(Clone, Eq, PartialEq)]
enum Repr {
    /// No namespace in use.
    None,
    /// Only in type namespace.
    Ty(ModuleDefId, Visibility),
    /// Only in value namespace.
    Val(ModuleDefId, Visibility),
    /// Same item in type and value namespaces.
    TyVal(ModuleDefId, Visibility),

    Other(Box<PerNs>),
}

impl Default for Compressed {
    fn default() -> Self {
        Compressed(Repr::None)
    }
}

impl From<PerNs> for Compressed {
    fn from(per_ns: PerNs) -> Self {
        let (t, v, m) = (per_ns.types, per_ns.values, per_ns.macros);
        match (t, v, m) {
            (None, None, None) => Compressed(Repr::None),
            (Some((t, v)), None, None) => Compressed(Repr::Ty(t, v)),
            (None, Some((val, vis)), None) => Compressed(Repr::Val(val, vis)),
            (Some((ty, tvis)), Some((val, vvis)), None) if ty == val && tvis == vvis => {
                Compressed(Repr::TyVal(ty, tvis))
            }
            _ => Compressed(Repr::Other(Box::new(per_ns))),
        }
    }
}

impl From<Compressed> for PerNs {
    fn from(comp: Compressed) -> Self {
        match comp.0 {
            Repr::None => PerNs::none(),
            Repr::Ty(t, v) => PerNs::types(t, v),
            Repr::Val(t, v) => PerNs::values(t, v),
            Repr::TyVal(t, v) => PerNs::both(t, t, v),
            Repr::Other(o) => *o,
        }
    }
}

impl<'a> From<&'a Compressed> for PerNs {
    fn from(comp: &'a Compressed) -> Self {
        match &comp.0 {
            Repr::None => PerNs::none(),
            Repr::Ty(t, v) => PerNs::types(*t, *v),
            Repr::Val(t, v) => PerNs::values(*t, *v),
            Repr::TyVal(t, v) => PerNs::both(*t, *t, *v),
            Repr::Other(o) => **o,
        }
    }
}

impl fmt::Debug for Compressed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        PerNs::from(self).fmt(f)
    }
}

/// `Compressed` is heavily used in `CrateDefMap`. Make sure its size doesn't unintentionally
/// increase, as that can have a somewhat large impact on memory usage.
#[test]
fn compressed_size() {
    assert_eq!(std::mem::size_of::<Compressed>(), 32);
}
