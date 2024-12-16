//! The home of `HirDatabase`, which is the Salsa database containing all the
//! type inference-related queries.

use std::sync;

use base_db::{CrateId, Upcast, impl_intern_key, impl_wrapper};
use hir_def::{
    AdtId, BlockId, CallableDefId, ConstParamId, DefWithBodyId, EnumVariantId, FunctionId,
    GeneralConstId, GenericDefId, ImplId, LifetimeParamId, LocalFieldId, StaticId, TraitId,
    TypeAliasId, TypeOrConstParamId, VariantId, db::DefDatabase, hir::ExprId,
    layout::TargetDataLayout,
};
use la_arena::ArenaMap;
use salsa::plumbing::AsId;
use smallvec::SmallVec;
use triomphe::Arc;

use crate::{
    Binders, ClosureId, Const, FnDefId, ImplTraitId, ImplTraits, InferenceResult, Interner,
    PolyFnSig, Substitution, TraitEnvironment, TraitRef, Ty, TyDefId, ValueTyDefId, chalk_db,
    consteval::ConstEvalError,
    dyn_compatibility::DynCompatibilityViolation,
    layout::{Layout, LayoutError},
    lower::{Diagnostics, GenericDefaults, GenericPredicates},
    method_resolution::{InherentImpls, TraitImpls, TyFingerprint},
    mir::{BorrowckResult, MirBody, MirLowerError},
};
use hir_expand::name::Name;

#[db_ext_macro::query_group]
pub trait HirDatabase: DefDatabase + Upcast<dyn DefDatabase> + std::fmt::Debug {
    #[db_ext_macro::invoke(crate::infer::infer_query)]
    fn infer(&self, def: DefWithBodyId) -> Arc<InferenceResult>;

    // region:mir

    #[db_ext_macro::invoke(crate::mir::mir_body_query)]
    #[db_ext_macro::cycle(crate::mir::mir_body_recover)]
    fn mir_body(&self, def: DefWithBodyId) -> Result<Arc<MirBody>, MirLowerError>;

    #[db_ext_macro::invoke(crate::mir::mir_body_for_closure_query)]
    fn mir_body_for_closure(&self, def: ClosureId) -> Result<Arc<MirBody>, MirLowerError>;

    #[db_ext_macro::invoke(crate::mir::monomorphized_mir_body_query)]
    #[db_ext_macro::cycle(crate::mir::monomorphized_mir_body_recover)]
    fn monomorphized_mir_body(
        &self,
        def: DefWithBodyId,
        subst: Substitution,
        env: Arc<TraitEnvironment>,
    ) -> Result<Arc<MirBody>, MirLowerError>;

    #[db_ext_macro::invoke(crate::mir::monomorphized_mir_body_for_closure_query)]
    fn monomorphized_mir_body_for_closure(
        &self,
        def: ClosureId,
        subst: Substitution,
        env: Arc<TraitEnvironment>,
    ) -> Result<Arc<MirBody>, MirLowerError>;

    #[db_ext_macro::invoke(crate::mir::borrowck_query)]
    #[db_ext_macro::lru]
    fn borrowck(&self, def: DefWithBodyId) -> Result<Arc<[BorrowckResult]>, MirLowerError>;

    #[db_ext_macro::invoke(crate::consteval::const_eval_query)]
    #[db_ext_macro::cycle(crate::consteval::const_eval_recover)]
    fn const_eval(
        &self,
        def: GeneralConstId,
        subst: Substitution,
        trait_env: Option<Arc<TraitEnvironment>>,
    ) -> Result<Const, ConstEvalError>;

    #[db_ext_macro::invoke(crate::consteval::const_eval_static_query)]
    #[db_ext_macro::cycle(crate::consteval::const_eval_static_recover)]
    fn const_eval_static(&self, def: StaticId) -> Result<Const, ConstEvalError>;

    #[db_ext_macro::invoke(crate::consteval::const_eval_discriminant_variant)]
    #[db_ext_macro::cycle(crate::consteval::const_eval_discriminant_recover)]
    fn const_eval_discriminant(&self, def: EnumVariantId) -> Result<i128, ConstEvalError>;

    #[db_ext_macro::invoke(crate::method_resolution::lookup_impl_method_query)]
    fn lookup_impl_method(
        &self,
        env: Arc<TraitEnvironment>,
        func: FunctionId,
        fn_subst: Substitution,
    ) -> (FunctionId, Substitution);

    // endregion:mir

    #[db_ext_macro::invoke(crate::layout::layout_of_adt_query)]
    #[db_ext_macro::cycle(crate::layout::layout_of_adt_recover)]
    fn layout_of_adt(
        &self,
        def: AdtId,
        subst: Substitution,
        env: Arc<TraitEnvironment>,
    ) -> Result<Arc<Layout>, LayoutError>;

    #[db_ext_macro::invoke(crate::layout::layout_of_ty_query)]
    #[db_ext_macro::cycle(crate::layout::layout_of_ty_recover)]
    fn layout_of_ty(&self, ty: Ty, env: Arc<TraitEnvironment>) -> Result<Arc<Layout>, LayoutError>;

    #[db_ext_macro::invoke(crate::layout::target_data_layout_query)]
    fn target_data_layout(&self, krate: CrateId) -> Result<Arc<TargetDataLayout>, Arc<str>>;

    #[db_ext_macro::invoke(crate::dyn_compatibility::dyn_compatibility_of_trait_query)]
    fn dyn_compatibility_of_trait(&self, trait_: TraitId) -> Option<DynCompatibilityViolation>;

    #[db_ext_macro::invoke(crate::lower::ty_query)]
    #[db_ext_macro::cycle(crate::lower::ty_recover)]
    fn ty(&self, def: TyDefId) -> Binders<Ty>;

    #[db_ext_macro::invoke(crate::lower::type_for_type_alias_with_diagnostics_query)]
    fn type_for_type_alias_with_diagnostics(&self, def: TypeAliasId) -> (Binders<Ty>, Diagnostics);

    /// Returns the type of the value of the given constant, or `None` if the `ValueTyDefId` is
    /// a `StructId` or `EnumVariantId` with a record constructor.
    #[db_ext_macro::invoke(crate::lower::value_ty_query)]
    fn value_ty(&self, def: ValueTyDefId) -> Option<Binders<Ty>>;

    #[db_ext_macro::invoke(crate::lower::impl_self_ty_with_diagnostics_query)]
    #[db_ext_macro::cycle(crate::lower::impl_self_ty_with_diagnostics_recover)]
    fn impl_self_ty_with_diagnostics(&self, def: ImplId) -> (Binders<Ty>, Diagnostics);

    #[db_ext_macro::invoke(crate::lower::impl_self_ty_query)]
    fn impl_self_ty(&self, def: ImplId) -> Binders<Ty>;

    #[db_ext_macro::invoke(crate::lower::const_param_ty_with_diagnostics_query)]
    fn const_param_ty_with_diagnostics(&self, def: ConstParamId) -> (Ty, Diagnostics);

    #[db_ext_macro::invoke(crate::lower::const_param_ty_query)]
    fn const_param_ty(&self, def: ConstParamId) -> Ty;

    #[db_ext_macro::invoke(crate::lower::impl_trait_with_diagnostics_query)]
    fn impl_trait_with_diagnostics(&self, def: ImplId) -> Option<(Binders<TraitRef>, Diagnostics)>;

    #[db_ext_macro::invoke(crate::lower::impl_trait_query)]
    fn impl_trait(&self, def: ImplId) -> Option<Binders<TraitRef>>;

    #[db_ext_macro::invoke(crate::lower::field_types_with_diagnostics_query)]
    fn field_types_with_diagnostics(
        &self,
        var: VariantId,
    ) -> (Arc<ArenaMap<LocalFieldId, Binders<Ty>>>, Diagnostics);

    #[db_ext_macro::invoke(crate::lower::field_types_query)]
    fn field_types(&self, var: VariantId) -> Arc<ArenaMap<LocalFieldId, Binders<Ty>>>;

    #[db_ext_macro::invoke(crate::lower::callable_item_sig)]
    fn callable_item_signature(&self, def: CallableDefId) -> PolyFnSig;

    #[db_ext_macro::invoke(crate::lower::return_type_impl_traits)]
    fn return_type_impl_traits(&self, def: FunctionId) -> Option<Arc<Binders<ImplTraits>>>;

    #[db_ext_macro::invoke(crate::lower::type_alias_impl_traits)]
    fn type_alias_impl_traits(&self, def: TypeAliasId) -> Option<Arc<Binders<ImplTraits>>>;

    #[db_ext_macro::invoke(crate::lower::generic_predicates_for_param_query)]
    #[db_ext_macro::cycle(crate::lower::generic_predicates_for_param_recover)]
    fn generic_predicates_for_param(
        &self,
        def: GenericDefId,
        param_id: TypeOrConstParamId,
        assoc_name: Option<Name>,
    ) -> GenericPredicates;

    #[db_ext_macro::invoke(crate::lower::generic_predicates_query)]
    fn generic_predicates(&self, def: GenericDefId) -> GenericPredicates;

    #[db_ext_macro::invoke(crate::lower::generic_predicates_without_parent_with_diagnostics_query)]
    fn generic_predicates_without_parent_with_diagnostics(
        &self,
        def: GenericDefId,
    ) -> (GenericPredicates, Diagnostics);

    #[db_ext_macro::invoke(crate::lower::generic_predicates_without_parent_query)]
    fn generic_predicates_without_parent(&self, def: GenericDefId) -> GenericPredicates;

    #[db_ext_macro::invoke(crate::lower::trait_environment_for_body_query)]
    #[db_ext_macro::transparent]
    fn trait_environment_for_body(&self, def: DefWithBodyId) -> Arc<TraitEnvironment>;

    #[db_ext_macro::invoke(crate::lower::trait_environment_query)]
    fn trait_environment(&self, def: GenericDefId) -> Arc<TraitEnvironment>;

    #[db_ext_macro::invoke(crate::lower::generic_defaults_with_diagnostics_query)]
    #[db_ext_macro::cycle(crate::lower::generic_defaults_with_diagnostics_recover)]
    fn generic_defaults_with_diagnostics(&self, def: GenericDefId)
    -> GenericDefaultsAndDiagnostics;

    #[db_ext_macro::invoke(crate::lower::generic_defaults_query)]
    fn generic_defaults(&self, def: GenericDefId) -> GenericDefaults;

    #[db_ext_macro::invoke(InherentImpls::inherent_impls_in_crate_query)]
    fn inherent_impls_in_crate(&self, krate: CrateId) -> Arc<InherentImpls>;

    #[db_ext_macro::invoke(InherentImpls::inherent_impls_in_block_query)]
    fn inherent_impls_in_block(&self, block: BlockId) -> Option<Arc<InherentImpls>>;

    /// Collects all crates in the dependency graph that have impls for the
    /// given fingerprint. This is only used for primitive types and types
    /// annotated with `rustc_has_incoherent_inherent_impls`; for other types
    /// we just look at the crate where the type is defined.
    #[db_ext_macro::invoke(crate::method_resolution::incoherent_inherent_impl_crates)]
    fn incoherent_inherent_impl_crates(
        &self,
        krate: CrateId,
        fp: TyFingerprint,
    ) -> SmallVec<[CrateId; 2]>;

    #[db_ext_macro::invoke(TraitImpls::trait_impls_in_crate_query)]
    fn trait_impls_in_crate(&self, krate: CrateId) -> Arc<TraitImpls>;

    #[db_ext_macro::invoke(TraitImpls::trait_impls_in_block_query)]
    fn trait_impls_in_block(&self, block: BlockId) -> Option<Arc<TraitImpls>>;

    #[db_ext_macro::invoke(TraitImpls::trait_impls_in_deps_query)]
    fn trait_impls_in_deps(&self, krate: CrateId) -> Arc<[Arc<TraitImpls>]>;

    // Interned IDs for Chalk integration
    #[db_ext_macro::interned(CallableDefIdWrapper)]
    fn intern_callable_def(&self, callable_def: CallableDefId) -> InternedCallableDefId;

    #[db_ext_macro::interned(TypeOrConstParamIdWrapper)]
    fn intern_type_or_const_param_id(
        &self,
        param_id: TypeOrConstParamId,
    ) -> InternedTypeOrConstParamId;

    #[db_ext_macro::interned(LifetimeParamIdWrapper)]
    fn intern_lifetime_param_id(&self, param_id: LifetimeParamId) -> InternedLifetimeParamId;

    #[db_ext_macro::interned(ImplTraitIdWrapper)]
    fn intern_impl_trait_id(&self, id: ImplTraitId) -> InternedOpaqueTyId;

    #[db_ext_macro::interned(InternedClosureWrapper)]
    fn intern_closure(&self, id: InternedClosure) -> InternedClosureId;

    #[db_ext_macro::interned(InternedCoroutineWrapper)]
    fn intern_coroutine(&self, id: InternedCoroutine) -> InternedCoroutineId;

    #[db_ext_macro::invoke(chalk_db::associated_ty_data_query)]
    fn associated_ty_data(
        &self,
        id: chalk_db::AssocTypeId,
    ) -> sync::Arc<chalk_db::AssociatedTyDatum>;

    #[db_ext_macro::invoke(chalk_db::trait_datum_query)]
    fn trait_datum(
        &self,
        krate: CrateId,
        trait_id: chalk_db::TraitId,
    ) -> sync::Arc<chalk_db::TraitDatum>;

    #[db_ext_macro::invoke(chalk_db::adt_datum_query)]
    fn adt_datum(
        &self,
        krate: CrateId,
        struct_id: chalk_db::AdtId,
    ) -> sync::Arc<chalk_db::AdtDatum>;

    #[db_ext_macro::invoke(chalk_db::impl_datum_query)]
    fn impl_datum(
        &self,
        krate: CrateId,
        impl_id: chalk_db::ImplId,
    ) -> sync::Arc<chalk_db::ImplDatum>;

    #[db_ext_macro::invoke(chalk_db::fn_def_datum_query)]
    fn fn_def_datum(&self, fn_def_id: FnDefId) -> sync::Arc<chalk_db::FnDefDatum>;

    #[db_ext_macro::invoke(chalk_db::fn_def_variance_query)]
    fn fn_def_variance(&self, fn_def_id: FnDefId) -> chalk_db::Variances;

    #[db_ext_macro::invoke(chalk_db::adt_variance_query)]
    fn adt_variance(&self, adt_id: chalk_db::AdtId) -> chalk_db::Variances;

    #[db_ext_macro::invoke(crate::variance::variances_of)]
    #[db_ext_macro::cycle(crate::variance::variances_of_cycle)]
    fn variances_of(&self, def: GenericDefId) -> Option<Arc<[crate::variance::Variance]>>;

    #[db_ext_macro::invoke(chalk_db::associated_ty_value_query)]
    fn associated_ty_value(
        &self,
        krate: CrateId,
        id: chalk_db::AssociatedTyValueId,
    ) -> sync::Arc<chalk_db::AssociatedTyValue>;

    #[db_ext_macro::invoke(crate::traits::normalize_projection_query)]
    #[db_ext_macro::transparent]
    fn normalize_projection(
        &self,
        projection: crate::ProjectionTy,
        env: Arc<TraitEnvironment>,
    ) -> Ty;

    #[db_ext_macro::invoke(crate::traits::trait_solve_query)]
    fn trait_solve(
        &self,
        krate: CrateId,
        block: Option<BlockId>,
        goal: crate::Canonical<crate::InEnvironment<crate::Goal>>,
    ) -> Option<crate::Solution>;

    #[db_ext_macro::invoke(chalk_db::program_clauses_for_chalk_env_query)]
    fn program_clauses_for_chalk_env(
        &self,
        krate: CrateId,
        block: Option<BlockId>,
        env: chalk_ir::Environment<Interner>,
    ) -> chalk_ir::ProgramClauses<Interner>;
}

#[test]
fn hir_database_is_dyn_compatible() {
    fn _assert_dyn_compatible(_: &dyn HirDatabase) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InternedTypeOrConstParamId(salsa::Id);
impl_intern_key!(InternedTypeOrConstParamId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InternedLifetimeParamId(salsa::Id);
impl_intern_key!(InternedLifetimeParamId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InternedConstParamId(salsa::Id);
impl_intern_key!(InternedConstParamId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InternedOpaqueTyId(salsa::Id);
impl_intern_key!(InternedOpaqueTyId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InternedClosureId(salsa::Id);
impl_intern_key!(InternedClosureId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InternedClosure(pub DefWithBodyId, pub ExprId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InternedCoroutineId(salsa::Id);
impl_intern_key!(InternedCoroutineId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InternedCoroutine(pub DefWithBodyId, pub ExprId);

/// This exists just for Chalk, because Chalk just has a single `FnDefId` where
/// we have different IDs for struct and enum variant constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct InternedCallableDefId(pub salsa::Id);
impl_intern_key!(InternedCallableDefId);

impl_wrapper!(InternedCallableDefId, CallableDefId, CallableDefIdWrapper);
impl_wrapper!(
    InternedTypeOrConstParamId,
    TypeOrConstParamId,
    TypeOrConstParamIdWrapper
);
impl_wrapper!(
    InternedLifetimeParamId,
    LifetimeParamId,
    LifetimeParamIdWrapper
);
impl_wrapper!(InternedOpaqueTyId, ImplTraitId, ImplTraitIdWrapper);
impl_wrapper!(InternedClosureId, InternedClosure, InternedClosureWrapper);
impl_wrapper!(
    InternedCoroutineId,
    InternedCoroutine,
    InternedCoroutineWrapper
);
