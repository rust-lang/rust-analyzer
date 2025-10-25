//! Definition of `SolverDefId`

use hir_def::{
    AdtId, CallableDefId, ConstId, DefWithBodyId, EnumId, EnumVariantId, FunctionId,
    GeneralConstId, GenericDefId, ImplId, StaticId, StructId, TraitId, TypeAliasId, UnionId,
};
use rustc_type_ir::inherent;

use crate::db::{InternedClosureId, InternedCoroutineId, InternedOpaqueTyId};

use super::DbInterner;

#[derive(Debug, PartialOrd, Ord, Clone, Copy, PartialEq, Eq, Hash, salsa::Supertype)]
pub enum Ctor {
    Struct(StructId),
    Enum(EnumVariantId),
}

#[derive(PartialOrd, Ord, Clone, Copy, PartialEq, Eq, Hash, salsa::Supertype)]
pub enum SolverDefId<'db> {
    AdtId(AdtId),
    ConstId(ConstId),
    FunctionId(FunctionId),
    ImplId(ImplId),
    StaticId(StaticId),
    TraitId(TraitId),
    TypeAliasId(TypeAliasId),
    InternedClosureId(InternedClosureId),
    InternedCoroutineId(InternedCoroutineId),
    InternedOpaqueTyId(InternedOpaqueTyId<'db>),
    EnumVariantId(EnumVariantId),
    // FIXME(next-solver): Do we need the separation of `Ctor`? It duplicates some variants.
    Ctor(Ctor),
}

impl<'db> std::fmt::Debug for SolverDefId<'db> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let interner = DbInterner::conjure();
        let db = interner.db;
        match *self {
            SolverDefId::AdtId(AdtId::StructId(id)) => {
                f.debug_tuple("AdtId").field(&db.struct_signature(id).name.as_str()).finish()
            }
            SolverDefId::AdtId(AdtId::EnumId(id)) => {
                f.debug_tuple("AdtId").field(&db.enum_signature(id).name.as_str()).finish()
            }
            SolverDefId::AdtId(AdtId::UnionId(id)) => {
                f.debug_tuple("AdtId").field(&db.union_signature(id).name.as_str()).finish()
            }
            SolverDefId::ConstId(id) => f
                .debug_tuple("ConstId")
                .field(&db.const_signature(id).name.as_ref().map_or("_", |name| name.as_str()))
                .finish(),
            SolverDefId::FunctionId(id) => {
                f.debug_tuple("FunctionId").field(&db.function_signature(id).name.as_str()).finish()
            }
            SolverDefId::ImplId(id) => f.debug_tuple("ImplId").field(&id).finish(),
            SolverDefId::StaticId(id) => {
                f.debug_tuple("StaticId").field(&db.static_signature(id).name.as_str()).finish()
            }
            SolverDefId::TraitId(id) => {
                f.debug_tuple("TraitId").field(&db.trait_signature(id).name.as_str()).finish()
            }
            SolverDefId::TypeAliasId(id) => f
                .debug_tuple("TypeAliasId")
                .field(&db.type_alias_signature(id).name.as_str())
                .finish(),
            SolverDefId::InternedClosureId(id) => {
                f.debug_tuple("InternedClosureId").field(&id).finish()
            }
            SolverDefId::InternedCoroutineId(id) => {
                f.debug_tuple("InternedCoroutineId").field(&id).finish()
            }
            SolverDefId::InternedOpaqueTyId(id) => {
                f.debug_tuple("InternedOpaqueTyId").field(&id).finish()
            }
            SolverDefId::EnumVariantId(id) => {
                let parent_enum = id.loc(db).parent;
                f.debug_tuple("EnumVariantId")
                    .field(&format_args!(
                        "\"{}::{}\"",
                        db.enum_signature(parent_enum).name.as_str(),
                        parent_enum.enum_variants(db).variant_name_by_id(id).unwrap().as_str()
                    ))
                    .finish()
            }
            SolverDefId::Ctor(Ctor::Struct(id)) => {
                f.debug_tuple("Ctor").field(&db.struct_signature(id).name.as_str()).finish()
            }
            SolverDefId::Ctor(Ctor::Enum(id)) => {
                let parent_enum = id.loc(db).parent;
                f.debug_tuple("Ctor")
                    .field(&format_args!(
                        "\"{}::{}\"",
                        db.enum_signature(parent_enum).name.as_str(),
                        parent_enum.enum_variants(db).variant_name_by_id(id).unwrap().as_str()
                    ))
                    .finish()
            }
        }
    }
}

impl<'db> From<AdtId> for SolverDefId<'db> {
    fn from(it: AdtId) -> SolverDefId<'db> {
        SolverDefId::AdtId(it)
    }
}
impl<'db> From<StructId> for SolverDefId<'db> {
    fn from(it: StructId) -> SolverDefId<'db> {
        SolverDefId::AdtId(AdtId::StructId(it))
    }
}
impl<'db> From<EnumId> for SolverDefId<'db> {
    fn from(it: EnumId) -> SolverDefId<'db> {
        SolverDefId::AdtId(AdtId::EnumId(it))
    }
}
impl<'db> From<UnionId> for SolverDefId<'db> {
    fn from(it: UnionId) -> SolverDefId<'db> {
        SolverDefId::AdtId(AdtId::UnionId(it))
    }
}
impl<'db> From<ConstId> for SolverDefId<'db> {
    fn from(it: ConstId) -> SolverDefId<'db> {
        SolverDefId::ConstId(it)
    }
}
impl<'db> From<FunctionId> for SolverDefId<'db> {
    fn from(it: FunctionId) -> SolverDefId<'db> {
        SolverDefId::FunctionId(it)
    }
}
impl<'db> From<ImplId> for SolverDefId<'db> {
    fn from(it: ImplId) -> SolverDefId<'db> {
        SolverDefId::ImplId(it)
    }
}
impl<'db> From<StaticId> for SolverDefId<'db> {
    fn from(it: StaticId) -> SolverDefId<'db> {
        SolverDefId::StaticId(it)
    }
}
impl<'db> From<TraitId> for SolverDefId<'db> {
    fn from(it: TraitId) -> SolverDefId<'db> {
        SolverDefId::TraitId(it)
    }
}
impl<'db> From<TypeAliasId> for SolverDefId<'db> {
    fn from(it: TypeAliasId) -> SolverDefId<'db> {
        SolverDefId::TypeAliasId(it)
    }
}
impl<'db> From<InternedClosureId> for SolverDefId<'db> {
    fn from(it: InternedClosureId) -> SolverDefId<'db> {
        SolverDefId::InternedClosureId(it)
    }
}
impl<'db> From<InternedCoroutineId> for SolverDefId<'db> {
    fn from(it: InternedCoroutineId) -> SolverDefId<'db> {
        SolverDefId::InternedCoroutineId(it)
    }
}
impl<'db> From<InternedOpaqueTyId<'db>> for SolverDefId<'db> {
    fn from(it: InternedOpaqueTyId<'db>) -> SolverDefId<'db> {
        SolverDefId::InternedOpaqueTyId(it)
    }
}
impl<'db> From<EnumVariantId> for SolverDefId<'db> {
    fn from(it: EnumVariantId) -> SolverDefId<'db> {
        SolverDefId::EnumVariantId(it)
    }
}
impl<'db> From<Ctor> for SolverDefId<'db> {
    fn from(it: Ctor) -> SolverDefId<'db> {
        SolverDefId::Ctor(it)
    }
}

impl<'db> From<GenericDefId> for SolverDefId<'db> {
    fn from(value: GenericDefId) -> Self {
        match value {
            GenericDefId::AdtId(adt_id) => SolverDefId::AdtId(adt_id),
            GenericDefId::ConstId(const_id) => SolverDefId::ConstId(const_id),
            GenericDefId::FunctionId(function_id) => SolverDefId::FunctionId(function_id),
            GenericDefId::ImplId(impl_id) => SolverDefId::ImplId(impl_id),
            GenericDefId::StaticId(static_id) => SolverDefId::StaticId(static_id),
            GenericDefId::TraitId(trait_id) => SolverDefId::TraitId(trait_id),
            GenericDefId::TypeAliasId(type_alias_id) => SolverDefId::TypeAliasId(type_alias_id),
        }
    }
}

impl<'db> From<GeneralConstId> for SolverDefId<'db> {
    #[inline]
    fn from(value: GeneralConstId) -> Self {
        match value {
            GeneralConstId::ConstId(const_id) => SolverDefId::ConstId(const_id),
            GeneralConstId::StaticId(static_id) => SolverDefId::StaticId(static_id),
        }
    }
}

impl<'db> From<DefWithBodyId> for SolverDefId<'db> {
    #[inline]
    fn from(value: DefWithBodyId) -> Self {
        match value {
            DefWithBodyId::FunctionId(id) => id.into(),
            DefWithBodyId::StaticId(id) => id.into(),
            DefWithBodyId::ConstId(id) => id.into(),
            DefWithBodyId::VariantId(id) => id.into(),
        }
    }
}

impl<'db> TryFrom<SolverDefId<'db>> for GenericDefId {
    type Error = ();

    fn try_from(value: SolverDefId<'db>) -> Result<Self, Self::Error> {
        Ok(match value {
            SolverDefId::AdtId(adt_id) => GenericDefId::AdtId(adt_id),
            SolverDefId::ConstId(const_id) => GenericDefId::ConstId(const_id),
            SolverDefId::FunctionId(function_id) => GenericDefId::FunctionId(function_id),
            SolverDefId::ImplId(impl_id) => GenericDefId::ImplId(impl_id),
            SolverDefId::StaticId(static_id) => GenericDefId::StaticId(static_id),
            SolverDefId::TraitId(trait_id) => GenericDefId::TraitId(trait_id),
            SolverDefId::TypeAliasId(type_alias_id) => GenericDefId::TypeAliasId(type_alias_id),
            SolverDefId::InternedClosureId(_)
            | SolverDefId::InternedCoroutineId(_)
            | SolverDefId::InternedOpaqueTyId(_)
            | SolverDefId::EnumVariantId(_)
            | SolverDefId::Ctor(_) => return Err(()),
        })
    }
}

impl<'db> SolverDefId<'db> {
    #[inline]
    #[track_caller]
    pub fn expect_opaque_ty(self) -> InternedOpaqueTyId<'db> {
        match self {
            SolverDefId::InternedOpaqueTyId(it) => it,
            _ => panic!("expected opaque type, found {self:?}"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn expect_type_alias(self) -> TypeAliasId {
        match self {
            SolverDefId::TypeAliasId(it) => it,
            _ => panic!("expected type alias, found {self:?}"),
        }
    }
}

impl<'db> inherent::DefId<DbInterner<'db>> for SolverDefId<'db> {
    fn as_local(self) -> Option<SolverDefId<'db>> {
        Some(self)
    }
    fn is_local(self) -> bool {
        true
    }
}

macro_rules! declare_id_wrapper {
    ($name:ident, $wraps:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(pub $wraps);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Debug::fmt(&SolverDefId::from(self.0), f)
            }
        }

        impl From<$name> for $wraps {
            #[inline]
            fn from(value: $name) -> $wraps {
                value.0
            }
        }

        impl From<$wraps> for $name {
            #[inline]
            fn from(value: $wraps) -> $name {
                Self(value)
            }
        }

        impl<'db> From<$name> for SolverDefId<'db> {
            #[inline]
            fn from(value: $name) -> SolverDefId<'db> {
                value.0.into()
            }
        }

        impl<'db> TryFrom<SolverDefId<'db>> for $name {
            type Error = ();

            #[inline]
            fn try_from(value: SolverDefId<'db>) -> Result<Self, Self::Error> {
                match value {
                    SolverDefId::$wraps(it) => Ok(Self(it)),
                    _ => Err(()),
                }
            }
        }

        impl<'db> inherent::DefId<DbInterner<'db>> for $name {
            fn as_local(self) -> Option<SolverDefId<'db>> {
                Some(self.into())
            }
            fn is_local(self) -> bool {
                true
            }
        }
    };
}

declare_id_wrapper!(TraitIdWrapper, TraitId);
declare_id_wrapper!(TypeAliasIdWrapper, TypeAliasId);
declare_id_wrapper!(ClosureIdWrapper, InternedClosureId);
declare_id_wrapper!(CoroutineIdWrapper, InternedCoroutineId);
declare_id_wrapper!(AdtIdWrapper, AdtId);
declare_id_wrapper!(ImplIdWrapper, ImplId);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CallableIdWrapper(pub CallableDefId);

impl std::fmt::Debug for CallableIdWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}
impl From<CallableIdWrapper> for CallableDefId {
    #[inline]
    fn from(value: CallableIdWrapper) -> CallableDefId {
        value.0
    }
}
impl From<CallableDefId> for CallableIdWrapper {
    #[inline]
    fn from(value: CallableDefId) -> CallableIdWrapper {
        Self(value)
    }
}
impl<'db> From<CallableIdWrapper> for SolverDefId<'db> {
    #[inline]
    fn from(value: CallableIdWrapper) -> SolverDefId<'db> {
        match value.0 {
            CallableDefId::FunctionId(it) => it.into(),
            CallableDefId::StructId(it) => Ctor::Struct(it).into(),
            CallableDefId::EnumVariantId(it) => Ctor::Enum(it).into(),
        }
    }
}
impl<'db> TryFrom<SolverDefId<'db>> for CallableIdWrapper {
    type Error = ();
    #[inline]
    fn try_from(value: SolverDefId<'db>) -> Result<Self, Self::Error> {
        match value {
            SolverDefId::FunctionId(it) => Ok(Self(it.into())),
            SolverDefId::Ctor(Ctor::Struct(it)) => Ok(Self(it.into())),
            SolverDefId::Ctor(Ctor::Enum(it)) => Ok(Self(it.into())),
            _ => Err(()),
        }
    }
}
impl<'db> inherent::DefId<DbInterner<'db>> for CallableIdWrapper {
    fn as_local(self) -> Option<SolverDefId<'db>> {
        Some(self.into())
    }
    fn is_local(self) -> bool {
        true
    }
}
