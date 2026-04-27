//! The home of `HirDatabase`, which is the Salsa database containing all the
//! type inference-related queries.

use base_db::{Crate, impl_intern_key, target::TargetLoadError};
use either::Either;
use hir_def::{
    AdtId, BuiltinDeriveImplId, CallableDefId, ConstId, ConstParamId, DefWithBodyId, EnumVariantId,
    ExpressionStoreOwnerId, FunctionId, GenericDefId, ImplId, LifetimeParamId, LocalFieldId,
    StaticId, TraitId, TypeAliasId, VariantId,
    builtin_derive::BuiltinDeriveImplMethod,
    db::DefDatabase,
    expr_store::ExpressionStore,
    hir::{ClosureKind, ExprId},
    layout::TargetDataLayout,
    signatures::{ConstSignature, StaticSignature},
};
use la_arena::ArenaMap;
use salsa::plumbing::AsId;
use span::Edition;
use stdx::impl_from;
use triomphe::Arc;

use crate::{
    ImplTraitId, TyDefId, ValueTyDefId,
    consteval::ConstEvalError,
    dyn_compatibility::DynCompatibilityViolation,
    layout::{Layout, LayoutError},
    lower::{Diagnostics, GenericDefaults},
    mir::{BorrowckResult, MirBody, MirLowerError},
    next_solver::{
        Allocation, EarlyBinder, GenericArgs, ParamEnv, PolyFnSig, StoredEarlyBinder,
        StoredGenericArgs, StoredTy, TraitRef, Ty, VariancesOf,
    },
    traits::{ParamEnvAndCrate, StoredParamEnvAndCrate},
};

#[query_group::query_group]
pub trait HirDatabase: DefDatabase + std::fmt::Debug {
    // region:mir

    // FXME: Collapse `mir_body_for_closure` into `mir_body`
    // and `monomorphized_mir_body_for_closure` into `monomorphized_mir_body`
    #[salsa::transparent]
    fn mir_body(&self, def: DefWithBodyId) -> Result<&MirBody, MirLowerError> {
        crate::mir::mir_body_query(self, def).as_ref().map_err(|err| err.clone())
    }

    #[salsa::transparent]
    fn mir_body_for_closure(&self, def: InternedClosureId) -> Result<&MirBody, MirLowerError> {
        crate::mir::mir_body_for_closure_query(self, def).as_ref().map_err(|err| err.clone())
    }

    #[salsa::transparent]
    fn monomorphized_mir_body(
        &self,
        def: DefWithBodyId,
        subst: StoredGenericArgs,
        env: StoredParamEnvAndCrate,
    ) -> Result<&MirBody, MirLowerError> {
        crate::mir::monomorphized_mir_body_query(self, def, subst, env)
            .as_ref()
            .map_err(|err| err.clone())
    }

    #[salsa::transparent]
    fn monomorphized_mir_body_for_closure(
        &self,
        def: InternedClosureId,
        subst: StoredGenericArgs,
        env: StoredParamEnvAndCrate,
    ) -> Result<&MirBody, MirLowerError> {
        crate::mir::monomorphized_mir_body_for_closure_query(self, def, subst, env)
            .as_ref()
            .map_err(|err| err.clone())
    }

    #[salsa::transparent]
    fn borrowck(&self, def: DefWithBodyId) -> Result<&[BorrowckResult], MirLowerError> {
        crate::mir::borrowck_query(self, def).as_ref().map(|it| &**it).map_err(|err| err.clone())
    }

    #[salsa::invoke(crate::consteval::const_eval)]
    #[salsa::transparent]
    fn const_eval<'db>(
        &'db self,
        def: ConstId,
        subst: GenericArgs<'db>,
        trait_env: Option<ParamEnvAndCrate<'db>>,
    ) -> Result<Allocation<'db>, ConstEvalError>;

    #[salsa::invoke(crate::consteval::const_eval_static)]
    #[salsa::transparent]
    fn const_eval_static<'db>(&'db self, def: StaticId) -> Result<Allocation<'db>, ConstEvalError>;

    #[salsa::invoke(crate::consteval::const_eval_discriminant_variant)]
    #[salsa::cycle(cycle_result = crate::consteval::const_eval_discriminant_cycle_result)]
    fn const_eval_discriminant(&self, def: EnumVariantId) -> Result<i128, ConstEvalError>;

    #[salsa::invoke(crate::method_resolution::lookup_impl_method_query)]
    #[salsa::transparent]
    fn lookup_impl_method<'db>(
        &'db self,
        env: ParamEnvAndCrate<'db>,
        func: FunctionId,
        fn_subst: GenericArgs<'db>,
    ) -> (Either<FunctionId, (BuiltinDeriveImplId, BuiltinDeriveImplMethod)>, GenericArgs<'db>);

    // endregion:mir

    #[salsa::invoke(crate::layout::layout_of_adt_query)]
    #[salsa::cycle(cycle_result = crate::layout::layout_of_adt_cycle_result)]
    fn layout_of_adt(
        &self,
        def: AdtId,
        args: StoredGenericArgs,
        trait_env: StoredParamEnvAndCrate,
    ) -> Result<Arc<Layout>, LayoutError>;

    #[salsa::invoke(crate::layout::layout_of_ty_query)]
    #[salsa::cycle(cycle_result = crate::layout::layout_of_ty_cycle_result)]
    fn layout_of_ty(
        &self,
        ty: StoredTy,
        env: StoredParamEnvAndCrate,
    ) -> Result<Arc<Layout>, LayoutError>;

    #[salsa::transparent]
    fn target_data_layout(&self, krate: Crate) -> Result<&TargetDataLayout, TargetLoadError> {
        crate::layout::target_data_layout_query(self, krate).as_ref().map_err(|err| err.clone())
    }

    #[salsa::invoke(crate::dyn_compatibility::dyn_compatibility_of_trait_query)]
    fn dyn_compatibility_of_trait(&self, trait_: TraitId) -> Option<DynCompatibilityViolation>;

    #[salsa::invoke(crate::lower::ty_query)]
    #[salsa::transparent]
    fn ty<'db>(&'db self, def: TyDefId) -> EarlyBinder<'db, Ty<'db>>;

    #[salsa::invoke(crate::lower::type_for_type_alias_with_diagnostics)]
    #[salsa::transparent]
    fn type_for_type_alias_with_diagnostics<'db>(
        &'db self,
        def: TypeAliasId,
    ) -> (EarlyBinder<'db, Ty<'db>>, Diagnostics);

    /// Returns the type of the value of the given constant, or `None` if the `ValueTyDefId` is
    /// a `StructId` or `EnumVariantId` with a record constructor.
    #[salsa::invoke(crate::lower::value_ty)]
    #[salsa::transparent]
    fn value_ty<'db>(&'db self, def: ValueTyDefId) -> Option<EarlyBinder<'db, Ty<'db>>>;

    #[salsa::invoke(crate::lower::impl_self_ty_with_diagnostics)]
    #[salsa::transparent]
    fn impl_self_ty_with_diagnostics<'db>(
        &'db self,
        def: ImplId,
    ) -> (EarlyBinder<'db, Ty<'db>>, Diagnostics);

    #[salsa::invoke(crate::lower::impl_self_ty_query)]
    #[salsa::transparent]
    fn impl_self_ty<'db>(&'db self, def: ImplId) -> EarlyBinder<'db, Ty<'db>>;

    #[salsa::invoke(crate::lower::const_param_ty_with_diagnostics)]
    #[salsa::transparent]
    fn const_param_ty_with_diagnostics<'db>(&'db self, def: ConstParamId)
    -> (Ty<'db>, Diagnostics);

    #[salsa::invoke(crate::lower::const_param_ty_query)]
    #[salsa::transparent]
    fn const_param_ty<'db>(&'db self, def: ConstParamId) -> Ty<'db>;

    #[salsa::invoke(crate::lower::impl_trait_with_diagnostics)]
    #[salsa::transparent]
    fn impl_trait_with_diagnostics<'db>(
        &'db self,
        def: ImplId,
    ) -> Option<(EarlyBinder<'db, TraitRef<'db>>, Diagnostics)>;

    #[salsa::invoke(crate::lower::impl_trait_query)]
    #[salsa::transparent]
    fn impl_trait<'db>(&'db self, def: ImplId) -> Option<EarlyBinder<'db, TraitRef<'db>>>;

    #[salsa::invoke(crate::lower::field_types_with_diagnostics_query)]
    #[salsa::transparent]
    fn field_types_with_diagnostics(
        &self,
        var: VariantId,
    ) -> &(ArenaMap<LocalFieldId, StoredEarlyBinder<StoredTy>>, Diagnostics);

    #[salsa::invoke(crate::lower::field_types_query)]
    #[salsa::transparent]
    fn field_types(&self, var: VariantId) -> &ArenaMap<LocalFieldId, StoredEarlyBinder<StoredTy>>;

    #[salsa::invoke(crate::lower::callable_item_signature)]
    #[salsa::transparent]
    fn callable_item_signature<'db>(
        &'db self,
        def: CallableDefId,
    ) -> EarlyBinder<'db, PolyFnSig<'db>>;

    #[salsa::invoke(crate::lower::trait_environment)]
    #[salsa::transparent]
    fn trait_environment<'db>(&'db self, def: ExpressionStoreOwnerId) -> ParamEnv<'db>;

    #[salsa::invoke(crate::lower::generic_defaults_with_diagnostics_query)]
    #[salsa::cycle(cycle_result = crate::lower::generic_defaults_with_diagnostics_cycle_result)]
    fn generic_defaults_with_diagnostics(
        &self,
        def: GenericDefId,
    ) -> (GenericDefaults, Diagnostics);

    /// This returns an empty list if no parameter has default.
    ///
    /// The binders of the returned defaults are only up to (not including) this parameter.
    #[salsa::invoke(crate::lower::generic_defaults_query)]
    #[salsa::transparent]
    fn generic_defaults(&self, def: GenericDefId) -> GenericDefaults;

    // Interned IDs for solver integration
    #[salsa::interned]
    fn intern_impl_trait_id(&self, id: ImplTraitId) -> InternedOpaqueTyId;

    #[salsa::invoke(crate::variance::variances_of)]
    #[salsa::transparent]
    fn variances_of<'db>(&'db self, def: GenericDefId) -> VariancesOf<'db>;
}

#[test]
fn hir_database_is_dyn_compatible() {
    fn _assert_dyn_compatible(_: &dyn HirDatabase) {}
}

#[salsa_macros::interned(no_lifetime, debug, revisions = usize::MAX)]
#[derive(PartialOrd, Ord)]
pub struct InternedLifetimeParamId {
    /// This stores the param and its index.
    pub loc: (LifetimeParamId, u32),
}

#[salsa_macros::interned(no_lifetime, debug, revisions = usize::MAX)]
#[derive(PartialOrd, Ord)]
pub struct InternedConstParamId {
    pub loc: ConstParamId,
}

#[salsa_macros::interned(no_lifetime, debug, revisions = usize::MAX)]
#[derive(PartialOrd, Ord)]
pub struct InternedOpaqueTyId {
    pub loc: ImplTraitId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedClosure {
    pub owner: ExpressionStoreOwnerId,
    pub expr: ExprId,
    pub kind: ClosureKind,
}

#[salsa_macros::interned(constructor = new_impl, no_lifetime, debug, revisions = usize::MAX)]
#[derive(PartialOrd, Ord)]
pub struct InternedClosureId {
    pub loc: InternedClosure,
}

impl InternedClosureId {
    #[inline]
    pub fn new(db: &dyn HirDatabase, loc: InternedClosure) -> Self {
        if cfg!(debug_assertions) {
            let store = ExpressionStore::of(db, loc.owner);
            let expr = &store[loc.expr];
            assert!(
                matches!(
                    expr,
                    hir_def::hir::Expr::Closure {
                        closure_kind: hir_def::hir::ClosureKind::Closure,
                        ..
                    }
                ),
                "expected a closure, found {expr:?}"
            );
        }

        Self::new_impl(db, loc)
    }
}

#[salsa_macros::interned(constructor = new_impl, no_lifetime, debug, revisions = usize::MAX)]
#[derive(PartialOrd, Ord)]
pub struct InternedCoroutineId {
    pub loc: InternedClosure,
}

impl InternedCoroutineId {
    #[inline]
    pub fn new(db: &dyn HirDatabase, loc: InternedClosure) -> Self {
        if cfg!(debug_assertions) {
            let store = ExpressionStore::of(db, loc.owner);
            let expr = &store[loc.expr];
            assert!(
                matches!(
                    expr,
                    hir_def::hir::Expr::Closure {
                        closure_kind: hir_def::hir::ClosureKind::OldCoroutine(_)
                            | hir_def::hir::ClosureKind::Coroutine { .. },
                        ..
                    }
                ),
                "expected a coroutine, found {expr:?}"
            );
        }

        Self::new_impl(db, loc)
    }
}

#[salsa_macros::interned(constructor = new_impl, no_lifetime, debug, revisions = usize::MAX)]
#[derive(PartialOrd, Ord)]
pub struct InternedCoroutineClosureId {
    pub loc: InternedClosure,
}

impl InternedCoroutineClosureId {
    #[inline]
    pub fn new(db: &dyn HirDatabase, loc: InternedClosure) -> Self {
        if cfg!(debug_assertions) {
            let store = ExpressionStore::of(db, loc.owner);
            let expr = &store[loc.expr];
            assert!(
                matches!(
                    expr,
                    hir_def::hir::Expr::Closure {
                        closure_kind: hir_def::hir::ClosureKind::CoroutineClosure(_),
                        ..
                    }
                ),
                "expected a coroutine closure, found {expr:?}"
            );
        }

        Self::new_impl(db, loc)
    }
}

/// An anonymous const expression that appears in a type position (e.g., array lengths,
/// const generic arguments like `{ N + 1 }`). Unlike named constants, these don't have
/// their own `Body` — their expressions live in the parent's signature `ExpressionStore`.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct AnonConstLoc {
    /// The owner store containing this expression.
    pub owner: ExpressionStoreOwnerId,
    /// The ExprId within the owner's ExpressionStore that is the root
    /// of this anonymous const expression.
    pub expr: ExprId,
    pub ty: StoredTy,
}
impl_intern_key!(AnonConstId, AnonConstLoc);

/// A constant, which might appears as a const item, an anonymous const block in expressions
/// or patterns, or as a constant in types with const generics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, salsa_macros::Supertype)]
pub enum GeneralConstId {
    ConstId(ConstId),
    StaticId(StaticId),
    AnonConstId(AnonConstId),
}

impl_from!(ConstId, StaticId, AnonConstId for GeneralConstId);

impl GeneralConstId {
    pub fn generic_def(self, db: &dyn HirDatabase) -> Option<GenericDefId> {
        match self {
            GeneralConstId::ConstId(it) => Some(it.into()),
            GeneralConstId::StaticId(it) => Some(it.into()),
            GeneralConstId::AnonConstId(it) => Some(it.loc(db).owner.generic_def(db)),
        }
    }

    pub fn name(self, db: &dyn DefDatabase) -> String {
        match self {
            GeneralConstId::StaticId(it) => {
                StaticSignature::of(db, it).name.display(db, Edition::CURRENT).to_string()
            }
            GeneralConstId::ConstId(const_id) => {
                ConstSignature::of(db, const_id).name.as_ref().map_or_else(
                    || "_".to_owned(),
                    |name| name.display(db, Edition::CURRENT).to_string(),
                )
            }
            GeneralConstId::AnonConstId(_) => "{anon const}".to_owned(),
        }
    }
}
