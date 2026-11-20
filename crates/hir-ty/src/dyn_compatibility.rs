//! Compute the dyn-compatibility of a trait

use std::ops::ControlFlow;

use base_db::Crate;
use hir_def::{
    AssocItemId, ConstId, FunctionId, GenericDefId, HasModule, TraitId, TypeAliasId,
    TypeOrConstParamId, TypeParamId,
    attrs::AttrFlags,
    hir::generics::{GenericParams, LocalTypeOrConstParamId},
    nameres::assoc::TraitItems,
    signatures::{FunctionSignature, TraitFlags, TraitSignature},
    unstable_features::UnstableFeatures,
};
use rustc_type_ir::{
    AliasTyKind, BoundVarIndexKind, ClauseKind, DebruijnIndex, Interner as _, PredicatePolarity,
    TraitPredicate, TypeFoldable, TypeFolder, TypeSuperFoldable, TypeSuperVisitable as _,
    TypeVisitable, TypeVisitableExt, TypeVisitor, Upcast,
    elaborate::{self, elaborate},
    inherent::{Clause as _, IntoKind},
};
use tracing::{debug, instrument};
use triomphe::ThinArc;

use crate::{
    ImplTraitId,
    db::HirDatabase,
    lower::{GenericPredicates, SupertraitsInfo},
    next_solver::{
        AliasTy, Binder, Clause, Clauses, DbInterner, DefaultAny, EarlyBinder, GenericArgs,
        ParamEnv, ParamTy, Region, RegionKind, TraitRef, Ty, TyKind, TypingMode, Unnormalized,
        default_types,
        infer::{
            DbInternerInferExt,
            traits::{Obligation, ObligationCause},
        },
        mk_param,
    },
};

pub type DynCompatibilityViolations = ThinArc<(), DynCompatibilityViolation>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DynCompatibilityViolation {
    /// Trait is marked `#[rustc_dyn_incompatible_trait]`.
    ExplicitlyDynIncompatible,

    /// `Self: Sized` declared on the trait.
    SizedSelf,

    /// Supertrait reference references `Self` an in illegal location
    /// (e.g., `trait Foo : Bar<Self>`).
    SupertraitSelf,

    // Supertrait has a non-lifetime `for<T>` binder.
    SupertraitNonLifetimeBinder,

    // Trait has a `const Trait` supertrait.
    SupertraitConst,

    /// Method has something illegal.
    Method(FunctionId, MethodViolation),

    /// Associated constant is faulty.
    AssocConst(ConstId, AssocConstViolation),

    /// Generic associated type (GAT).
    GenericAssocTy(TypeAliasId),

    // This doesn't exist in rustc, but added for better visualization
    HasNonCompatibleSuperTrait(TraitId, DynCompatibilityViolations),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssocConstViolation {
    /// Unstable feature `min_generic_const_args` wasn't enabled.
    FeatureNotEnabled,

    /// Not defined as a type-level associated const.
    NonType,

    /// Has own generic parameters (GAC).
    Generic,

    /// Its type mentions the `Self` type parameter.
    TypeReferencesSelf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MethodViolation {
    /// e.g., `fn foo()`
    StaticMethod,

    /// e.g., `fn foo(&self, x: Self)`
    ReferencesSelfInput,

    /// e.g., `fn foo(&self) -> Self`
    ReferencesSelfOutput,

    /// e.g., `fn foo(&self) -> impl Sized`
    ReferencesImplTraitInTrait,

    /// e.g., `async fn foo(&self)`
    AsyncFn,

    /// e.g., `fn foo(&self) where Self: Clone`
    WhereClauseReferencesSelf,

    /// e.g., `fn foo<A>()`
    Generic,

    /// e.g., `fn (mut ap: ...)`
    CVariadic,

    /// the method's receiver (`self` argument) can't be dispatched on
    UndispatchableReceiver,
}

struct LazyFeatures<'db> {
    krate: Crate,
    features: Option<&'db UnstableFeatures>,
}

impl<'db> LazyFeatures<'db> {
    fn get(&mut self, db: &'db dyn HirDatabase) -> &'db UnstableFeatures {
        self.features.get_or_insert_with(|| UnstableFeatures::query(db, self.krate))
    }
}

#[salsa::tracked]
pub fn dyn_compatibility_violations(
    db: &dyn HirDatabase,
    trait_: TraitId,
) -> Option<DynCompatibilityViolations> {
    let mut violations = Vec::new();
    for &super_trait in &SupertraitsInfo::query(db, trait_).direct_supertraits {
        if let Some(supertrait_violations) = dyn_compatibility_violations(db, super_trait) {
            violations.push(DynCompatibilityViolation::HasNonCompatibleSuperTrait(
                super_trait,
                supertrait_violations.clone(),
            ));
        }
    }

    let krate = trait_.krate(db);
    let interner = DbInterner::new_with(db, krate);
    let mut features = LazyFeatures { krate, features: None };
    dyn_compatibility_violations_for_trait(interner, &mut features, trait_, &mut violations);

    if violations.is_empty() {
        None
    } else {
        Some(ThinArc::from_header_and_iter((), violations.into_iter()))
    }
}

#[instrument(level = "debug", skip(interner, features), ret)]
fn dyn_compatibility_violations_for_trait<'db>(
    interner: DbInterner<'db>,
    features: &mut LazyFeatures<'db>,
    trait_def_id: TraitId,
    violations: &mut Vec<DynCompatibilityViolation>,
) {
    let db = interner.db;

    // Check assoc items for violations.
    let trait_items = trait_def_id.trait_items(db);
    for &(_, item) in &trait_items.items {
        dyn_compatibility_violations_for_assoc_item(
            interner,
            features,
            trait_def_id,
            item,
            violations,
        );
    }

    // Check the trait itself.
    if trait_has_sized_self(db, trait_def_id) {
        violations.push(DynCompatibilityViolation::SizedSelf);
    } else if AttrFlags::query(db, trait_def_id.into())
        .contains(AttrFlags::RUSTC_DYN_INCOMPATIBLE_TRAIT)
    {
        violations.push(DynCompatibilityViolation::ExplicitlyDynIncompatible);
    }

    if clauses_reference_self(interner, trait_def_id) {
        violations.push(DynCompatibilityViolation::SupertraitSelf);
    }
    if bounds_reference_self(interner, trait_def_id, trait_items) {
        violations.push(DynCompatibilityViolation::SupertraitSelf);
    }
    if super_clauses_have_non_lifetime_binders(interner, trait_def_id) {
        violations.push(DynCompatibilityViolation::SupertraitNonLifetimeBinder);
    }
    if super_clauses_are_unconditionally_const(interner, trait_def_id) {
        violations.push(DynCompatibilityViolation::SupertraitConst);
    }
}

fn clauses_reference_self(interner: DbInterner<'_>, trait_def_id: TraitId) -> bool {
    let trait_ref = Binder::dummy(TraitRef::identity(interner, trait_def_id.into()));
    let clauses = GenericPredicates::query_own(interner.db, trait_def_id.into());
    clauses.skip_binder().map(|clause| clause.instantiate_supertrait(interner, trait_ref)).any(
        |clause| {
            // Super clauses cannot allow self projections, since they're
            // impossible to make into existential bounds without eager resolution
            // or something.
            // e.g. `trait A: B<Item = Self::Assoc>`.
            predicate_references_self(interner, trait_def_id, clause, AllowSelfProjections::No)
        },
    )
}

fn bounds_reference_self(
    interner: DbInterner<'_>,
    trait_def_id: TraitId,
    trait_items: &TraitItems,
) -> bool {
    trait_items
        .items
        .iter()
        // We're only looking at associated type bounds
        .filter_map(|&(_, item)| match item {
            AssocItemId::TypeAliasId(it) => Some(it),
            AssocItemId::FunctionId(_) | AssocItemId::ConstId(_) => None,
        })
        // Ignore GATs with `Self: Sized`
        .filter(|&item| !generics_require_sized_self(interner.db, item.into()))
        .flat_map(|item| {
            interner
                .db
                .type_alias_bounds(item)
                .iter_identity_copied()
                .map(Unnormalized::skip_norm_wip)
        })
        .any(|clause| {
            // Item bounds *can* have self projections, since they never get
            // their self type erased.
            predicate_references_self(interner, trait_def_id, clause, AllowSelfProjections::Yes)
        })
}

fn predicate_references_self<'db>(
    interner: DbInterner<'db>,
    trait_def_id: TraitId,
    predicate: Clause<'db>,
    allow_self_projections: AllowSelfProjections,
) -> bool {
    match predicate.kind().skip_binder() {
        ClauseKind::Trait(ref data) => {
            // In the case of a trait predicate, we can skip the "self" type.
            data.trait_ref.args[1..].iter().any(|&arg| {
                contains_illegal_self_type_reference(
                    interner,
                    trait_def_id,
                    arg,
                    allow_self_projections,
                )
            })
        }
        ClauseKind::Projection(ref data) => {
            // And similarly for projections. This should be redundant with
            // the previous check because any projection should have a
            // matching `Trait` predicate with the same inputs, but we do
            // the check to be safe.
            //
            // It's also won't be redundant if we allow type-generic associated
            // types for trait objects.
            //
            // Note that we *do* allow projection *outputs* to contain
            // `Self` (i.e., `trait Foo: Bar<Output=Self::Result> { type Result; }`),
            // we just require the user to specify *both* outputs
            // in the object type (i.e., `dyn Foo<Output=(), Result=()>`).
            //
            // This is ALT2 in issue #56288, see that for discussion of the
            // possible alternatives.
            data.projection_term.args[1..].iter().any(|&arg| {
                contains_illegal_self_type_reference(
                    interner,
                    trait_def_id,
                    arg,
                    allow_self_projections,
                )
            })
        }
        ClauseKind::ConstArgHasType(_ct, ty) => {
            contains_illegal_self_type_reference(interner, trait_def_id, ty, allow_self_projections)
        }

        ClauseKind::WellFormed(..)
        | ClauseKind::TypeOutlives(..)
        | ClauseKind::RegionOutlives(..)
        | ClauseKind::HostEffect(..)
        | ClauseKind::UnstableFeature(_) => false,

        // FIXME(generic_const_exprs): this can mention `Self`
        ClauseKind::ConstEvaluatable(..) => false,
    }
}

fn super_clauses_have_non_lifetime_binders(
    interner: DbInterner<'_>,
    trait_def_id: TraitId,
) -> bool {
    interner
        .explicit_super_predicates_of(trait_def_id.into())
        .iter_identity()
        .map(Unnormalized::skip_norm_wip)
        .any(|(clause, _)| clause.has_non_region_bound_vars())
}

/// Checks for `const Trait` supertraits. We're okay with `[const] Trait`,
/// supertraits since for a non-const instantiation of that trait, the
/// conditionally-const supertrait is also not required to be const.
fn super_clauses_are_unconditionally_const(
    interner: DbInterner<'_>,
    trait_def_id: TraitId,
) -> bool {
    interner
        .explicit_super_predicates_of(trait_def_id.into())
        .iter_identity()
        .map(Unnormalized::skip_norm_wip)
        .any(|(clause, _)| matches!(clause.kind().skip_binder(), ClauseKind::HostEffect(_)))
}

fn trait_has_sized_self(db: &dyn HirDatabase, trait_def_id: TraitId) -> bool {
    generics_require_sized_self(db, trait_def_id.into())
}

#[salsa::tracked(returns(copy))]
pub fn generics_require_sized_self(db: &dyn HirDatabase, def_id: GenericDefId) -> bool {
    let interner = DbInterner::new_with(db, def_id.krate(db));
    let Some(sized_def_id) = interner.lang_items().Sized else {
        return false; /* No Sized trait, can't require it! */
    };

    // Search for a clause like `Self: Sized` amongst the trait bounds.
    let clauses = GenericPredicates::query_own_explicit(db, def_id)
        .iter_identity()
        .map(Unnormalized::skip_norm_wip);
    elaborate(interner, clauses).any(|clause| match clause.kind().skip_binder() {
        ClauseKind::Trait(ref trait_pred) => {
            trait_pred.def_id().0 == sized_def_id && trait_pred.self_ty().is_param(0)
        }
        ClauseKind::RegionOutlives(_)
        | ClauseKind::TypeOutlives(_)
        | ClauseKind::Projection(_)
        | ClauseKind::ConstArgHasType(_, _)
        | ClauseKind::WellFormed(_)
        | ClauseKind::ConstEvaluatable(_)
        | ClauseKind::UnstableFeature(_)
        | ClauseKind::HostEffect(..) => false,
    })
}

#[instrument(level = "debug", skip(interner, features), ret)]
fn dyn_compatibility_violations_for_assoc_item<'db>(
    interner: DbInterner<'db>,
    features: &mut LazyFeatures<'db>,
    trait_def_id: TraitId,
    item: AssocItemId,
    violations: &mut Vec<DynCompatibilityViolation>,
) {
    let db = interner.db;

    // FIXME: `final` assoc functions don't prevent a trait from being dyn-compatible
    // if db.defaultness(item.def_id).is_final() {
    //     return;
    // }

    // Any item that has a `Self: Sized` requisite is otherwise exempt from the regulations.
    if generics_require_sized_self(db, item.into()) {
        return;
    }

    match item {
        AssocItemId::ConstId(item) => {
            // We will permit type associated consts if they are explicitly mentioned in the
            // trait object type. We can't check this here, as here we only check if it is
            // guaranteed to not be possible.

            let mut push_violation =
                |violation| violations.push(DynCompatibilityViolation::AssocConst(item, violation));

            let features = features.get(db);
            if features.gca_min_const_items {
                if !GenericParams::of(db, item.into()).is_empty() {
                    push_violation(AssocConstViolation::Generic);
                } else if !AttrFlags::query(db, item.into()).contains(AttrFlags::RUSTC_ALWAYS_GCA)
                    && !features.gca_const_items
                {
                    push_violation(AssocConstViolation::NonType);
                }

                let ty = Binder::dummy(
                    db.value_ty(item.into()).unwrap().instantiate_identity().skip_norm_wip(),
                );
                if contains_illegal_self_type_reference(
                    interner,
                    trait_def_id,
                    ty,
                    AllowSelfProjections::Yes,
                ) {
                    push_violation(AssocConstViolation::TypeReferencesSelf);
                }
            } else {
                push_violation(AssocConstViolation::FeatureNotEnabled);
            }
        }
        AssocItemId::FunctionId(item) => {
            virtual_call_violations_for_method(interner, trait_def_id, item, violations)
        }
        AssocItemId::TypeAliasId(item) => {
            if !GenericParams::of(db, item.into()).is_empty() {
                violations.push(DynCompatibilityViolation::GenericAssocTy(item));
            }
            // We will permit associated types if they are explicitly mentioned in the trait
            // object type. We can't check this here, as here we only check if it is
            // guaranteed to not be possible.
        }
    }
}

/// Returns `Some(_)` if this method cannot be called on a trait
/// object; this does not necessarily imply that the enclosing trait
/// is dyn-incompatible, because the method might have a where clause
/// `Self: Sized`.
fn virtual_call_violations_for_method<'db>(
    interner: DbInterner<'db>,
    trait_def_id: TraitId,
    method: FunctionId,
    violations: &mut Vec<DynCompatibilityViolation>,
) {
    let db = interner.db;
    let mut push_violation =
        |violation| violations.push(DynCompatibilityViolation::Method(method, violation));

    let sig = db.callable_item_signature(method.into()).instantiate_identity().skip_norm_wip();
    let data = FunctionSignature::of(db, method);

    // The method's first parameter must be named `self`
    if !data.has_self_param() {
        push_violation(MethodViolation::StaticMethod);
        // Not having `self` parameter messes up the later checks,
        // so we need to return instead of pushing
        return;
    }

    for &input_ty in sig.skip_binder().inputs().iter().skip(1) {
        if contains_illegal_self_type_reference(
            interner,
            trait_def_id,
            sig.rebind(input_ty),
            AllowSelfProjections::Yes,
        ) {
            push_violation(MethodViolation::ReferencesSelfInput);
        }
    }
    if contains_illegal_self_type_reference(
        interner,
        trait_def_id,
        sig.output(),
        AllowSelfProjections::Yes,
    ) {
        push_violation(MethodViolation::ReferencesSelfOutput);
    }
    if let Some(error) = contains_illegal_impl_trait_in_trait(interner, method, data, sig.output())
    {
        push_violation(error);
    }
    if sig.skip_binder().c_variadic() {
        push_violation(MethodViolation::CVariadic);
    }

    // We can't monomorphize things like `fn foo<A>(...)`.
    if data.generic_params.len_type_or_consts() != 0 {
        push_violation(MethodViolation::Generic);
    }

    let receiver_ty = interner.liberate_late_bound_regions(method.into(), sig.input(0));

    let clauses = GenericPredicates::query(db, method.into());

    // `self: Self` can't be dispatched on.
    // However, this is considered dyn compatible. We allow it as a special case here.
    // FIXME(mikeyhew) get rid of this `if` statement once `receiver_is_dispatchable` allows
    // `Receiver: Unsize<Receiver[Self => dyn Trait]>`.
    if !receiver_ty.is_param(0) {
        if !receiver_is_dispatchable(interner, trait_def_id, method, receiver_ty, clauses) {
            push_violation(MethodViolation::UndispatchableReceiver);
        } else {
            // We confirm that the `receiver_is_dispatchable` is accurate later,
            // see `check_receiver_correct`. It should be kept in sync with this code.
        }
    }

    // NOTE: This check happens last, because it results in a lint, and not a
    // hard error.
    if clauses.own_predicates().skip_binder().any(|clause| {
        // dyn Trait is okay:
        //
        //     trait Trait {
        //         fn f(&self) where Self: 'static;
        //     }
        //
        // because a trait object can't claim to live longer than the concrete
        // type. If the lifetime bound holds on dyn Trait then it's guaranteed
        // to hold as well on the concrete type.
        if clause.as_type_outlives_clause().is_some() {
            return false;
        }

        // dyn Trait is okay:
        //
        //     auto trait AutoTrait {}
        //
        //     trait Trait {
        //         fn f(&self) where Self: AutoTrait;
        //     }
        //
        // because `impl AutoTrait for dyn Trait` is disallowed by coherence.
        // Traits with a default impl are implemented for a trait object if and
        // only if the autotrait is one of the trait object's trait bounds, like
        // in `dyn Trait + AutoTrait`. This guarantees that trait objects only
        // implement auto traits if the underlying type does as well.
        if let ClauseKind::Trait(TraitPredicate {
            trait_ref: pred_trait_ref,
            polarity: PredicatePolarity::Positive,
        }) = clause.kind().skip_binder()
            && pred_trait_ref.self_ty().is_param(0)
            && TraitSignature::of(db, pred_trait_ref.def_id.0).flags.contains(TraitFlags::AUTO)
        {
            return false;
        }

        contains_illegal_self_type_reference(
            interner,
            trait_def_id,
            clause,
            AllowSelfProjections::Yes,
        )
    }) {
        push_violation(MethodViolation::WhereClauseReferencesSelf);
    }
}

/// Performs a type instantiation to produce the version of `receiver_ty` when `Self = self_ty`.
/// For example, for `receiver_ty = Rc<Self>` and `self_ty = Foo`, returns `Rc<Foo>`.
fn receiver_for_self_ty<'db>(
    interner: DbInterner<'db>,
    receiver_ty: Ty<'db>,
    self_ty: Ty<'db>,
    method_def_id: FunctionId,
) -> Ty<'db> {
    debug!("receiver_for_self_ty({:?}, {:?}, {:?})", receiver_ty, self_ty, method_def_id);
    let args = GenericArgs::for_item(interner, method_def_id.into(), |index, param, _, _| {
        if index == 0 { self_ty.into() } else { mk_param(interner, index, param) }
    });

    let result = EarlyBinder::bind(receiver_ty).instantiate(interner, args).skip_norm_wip();
    debug!(
        "receiver_for_self_ty({:?}, {:?}, {:?}) = {:?}",
        receiver_ty, self_ty, method_def_id, result
    );
    result
}

/// Checks the method's receiver (the `self` argument) can be dispatched on when `Self` is a
/// trait object. We require that `DispatchableFromDyn` be implemented for the receiver type
/// in the following way:
/// - let `Receiver` be the type of the `self` argument, i.e `Self`, `&Self`, `Rc<Self>`,
/// - require the following bound:
///
///   ```ignore (not-rust)
///   Receiver[Self => T]: DispatchFromDyn<Receiver[Self => dyn Trait]>
///   ```
///
///   where `Foo[X => Y]` means "the same type as `Foo`, but with `X` replaced with `Y`"
///   (instantiation notation).
///
/// Some examples of receiver types and their required obligation:
/// - `&'a mut self` requires `&'a mut Self: DispatchFromDyn<&'a mut dyn Trait>`,
/// - `self: Rc<Self>` requires `Rc<Self>: DispatchFromDyn<Rc<dyn Trait>>`,
/// - `self: Pin<Box<Self>>` requires `Pin<Box<Self>>: DispatchFromDyn<Pin<Box<dyn Trait>>>`.
///
/// The only case where the receiver is not dispatchable, but is still a valid receiver
/// type (just not dyn compatible), is when there is more than one level of pointer indirection.
/// E.g., `self: &&Self`, `self: &Rc<Self>`, `self: Box<Box<Self>>`. In these cases, there
/// is no way, or at least no inexpensive way, to coerce the receiver from the version where
/// `Self = dyn Trait` to the version where `Self = T`, where `T` is the unknown erased type
/// contained by the trait object, because the object that needs to be coerced is behind
/// a pointer.
///
/// If lowering already produced an error in the receiver type, we conservatively treat it as
/// undispatchable instead of asking the solver.
///
/// In practice, we cannot use `dyn Trait` explicitly in the obligation because it would result in
/// a new check that `Trait` is dyn-compatible, creating a cycle.
/// Instead, we emulate a placeholder by introducing a new type parameter `U` such that
/// `Self: Unsize<U>` and `U: Trait + MetaSized`, and use `U` in place of `dyn Trait`.
///
/// Written as a chalk-style query:
/// ```ignore (not-rust)
/// forall (U: Trait + MetaSized) {
///     if (Self: Unsize<U>) {
///         Receiver: DispatchFromDyn<Receiver[Self => U]>
///     }
/// }
/// ```
/// for `self: &'a mut Self`, this means `&'a mut Self: DispatchFromDyn<&'a mut U>`
/// for `self: Rc<Self>`, this means `Rc<Self>: DispatchFromDyn<Rc<U>>`
/// for `self: Pin<Box<Self>>`, this means `Pin<Box<Self>>: DispatchFromDyn<Pin<Box<U>>>`
//
// FIXME(mikeyhew) when unsized receivers are implemented as part of unsized rvalues, add this
// fallback query: `Receiver: Unsize<Receiver[Self => U]>` to support receivers like
// `self: Wrapper<Self>`.
fn receiver_is_dispatchable<'db>(
    interner: DbInterner<'db>,
    trait_def_id: TraitId,
    method: FunctionId,
    receiver_ty: Ty<'db>,
    clauses: &GenericPredicates,
) -> bool {
    debug!("receiver_is_dispatchable: method = {:?}, receiver_ty = {:?}", method, receiver_ty);

    let lang_items = interner.lang_items();
    let (Some(unsize_did), Some(dispatch_from_dyn_did)) =
        (lang_items.Unsize, lang_items.DispatchFromDyn)
    else {
        debug!("receiver_is_dispatchable: Missing `Unsize` or `DispatchFromDyn` traits");
        return true;
    };

    // the type `U` in the query
    // use a bogus type parameter to mimic a forall(U) query using u32::MAX for now.
    let self_param_id = TypeParamId::from_unchecked(TypeOrConstParamId {
        parent: trait_def_id.into(),
        local_id: LocalTypeOrConstParamId::from_raw(la_arena::RawIdx::from_u32(0)),
    });
    let self_param_ty =
        Ty::new(interner, rustc_type_ir::TyKind::Param(ParamTy { index: 0, id: self_param_id }));
    // FIXME: That seems problematic to fake a generic param like that?
    let unsized_self_ty = Ty::new_param(interner, self_param_id, u32::MAX);

    // `Receiver[Self => U]`
    let unsized_receiver_ty = receiver_for_self_ty(interner, receiver_ty, unsized_self_ty, method);

    // create a modified param env, with `Self: Unsize<U>` and `U: Trait` (and all of
    // its supertraits) added to caller bounds. `U: MetaSized` is already implied here.
    let param_env = {
        // N.B. We generally want to emulate the construction of the `unnormalized_param_env`
        // in the param-env query here. The fact that we don't just start with the clauses
        // in the param-env of the method is because those are already normalized, and mixing
        // normalized and unnormalized copies of predicates in `normalize_param_env_or_error`
        // will cause ambiguity that the user can't really avoid.
        //
        // We leave out certain complexities of the param-env query here. Specifically, we:
        // 1. Do not add `[const]` bounds since there are no `dyn const Trait`s.
        // 2. Do not add RPITIT self projection bounds for defaulted methods, since we
        //    are not constructing a param-env for "inside" of the body of the defaulted
        //    method, so we don't really care about projecting to a specific RPIT type,
        //    and because RPITITs are not dyn compatible (yet).
        let mut clauses: Vec<_> =
            clauses.all_predicates().iter_identity().map(Unnormalized::skip_norm_wip).collect();

        // Self: Unsize<U>
        let unsize_predicate =
            TraitRef::new(interner, unsize_did.into(), [self_param_ty, unsized_self_ty]);
        clauses.push(unsize_predicate.upcast(interner));

        // U: Trait<Arg1, ..., ArgN>
        let args = GenericArgs::for_item(interner, trait_def_id.into(), |index, param, _, _| {
            if index == 0 { unsized_self_ty.into() } else { mk_param(interner, index, param) }
        });
        let trait_predicate = TraitRef::new_from_args(interner, trait_def_id.into(), args);
        clauses.push(trait_predicate.upcast(interner));

        let meta_sized_predicate = {
            let Some(meta_sized_did) = lang_items.MetaSized else { return true };
            TraitRef::new(interner, meta_sized_did.into(), [unsized_self_ty])
        };
        clauses.push(meta_sized_predicate.upcast(interner));

        // FIXME: We need to do something with normalization here, but I'm not sure what the status of ParamEnv normalization
        // with the new solver is. Revisit when things are more clear.
        // normalize_param_env_or_error(
        //     db,
        //     ty::ParamEnv::new(db.mk_clauses(&clauses)),
        //     ObligationCause::dummy_with_span(db.def_span(method.def_id)),
        // )
        ParamEnv { clauses: Clauses::new_from_slice(&clauses) }
    };

    // Receiver: DispatchFromDyn<Receiver[Self => U]>
    let obligation = {
        let predicate = TraitRef::new(
            interner,
            dispatch_from_dyn_did.into(),
            [receiver_ty, unsized_receiver_ty],
        );

        Obligation::new(interner, ObligationCause::dummy(), param_env, predicate)
    };

    let infcx = interner.infer_ctxt().build(TypingMode::non_body_analysis());
    // the receiver is dispatchable iff the obligation holds
    infcx.predicate_must_hold_modulo_regions(&obligation)
}

#[derive(Copy, Clone)]
enum AllowSelfProjections {
    Yes,
    No,
}

/// Check if the given value contains illegal `Self` references.
///
/// This is somewhat subtle. In general, we want to forbid references to `Self` in the
/// argument and return types, since the value of `Self` is erased.
///
/// However, there is one exception: It is ok to reference `Self` in order to access an
/// associated type of the current trait, since we retain the value of those associated
/// types in the trait object type itself.
///
/// The same thing holds for associated consts under feature `min_generic_const_args`.
///
/// ```rust,ignore (example)
/// trait SuperTrait {
///     type X;
/// }
///
/// trait Trait : SuperTrait {
///     type Y;
///     fn foo(&self, x: Self) // bad
///     fn foo(&self) -> Self // bad
///     fn foo(&self) -> Option<Self> // bad
///     fn foo(&self) -> Self::Y // OK, desugars to next example
///     fn foo(&self) -> <Self as Trait>::Y // OK
///     fn foo(&self) -> Self::X // OK, desugars to next example
///     fn foo(&self) -> <Self as SuperTrait>::X // OK
/// }
/// ```
///
/// However, it is not as simple as allowing `Self` in a projected
/// type, because there are illegal ways to use `Self` as well:
///
/// ```rust,ignore (example)
/// trait Trait : SuperTrait {
///     ...
///     fn foo(&self) -> <Self as SomeOtherTrait>::X;
/// }
/// ```
///
/// Here we will not have the type of `X` recorded in the
/// object type, and we cannot resolve `Self as SomeOtherTrait`
/// without knowing what `Self` is.
fn contains_illegal_self_type_reference<'db, T: TypeVisitable<DbInterner<'db>>>(
    interner: DbInterner<'db>,
    trait_def_id: TraitId,
    value: T,
    allow_self_projections: AllowSelfProjections,
) -> bool {
    value
        .visit_with(&mut IllegalSelfTypeVisitor {
            interner,
            trait_def_id,
            supertraits: None,
            allow_self_projections,
            types: default_types(),
        })
        .is_break()
}

struct IllegalSelfTypeVisitor<'db> {
    interner: DbInterner<'db>,
    trait_def_id: TraitId,
    supertraits: Option<Vec<TraitRef<'db>>>,
    allow_self_projections: AllowSelfProjections,
    types: &'db DefaultAny<'db>,
}

impl<'db> IllegalSelfTypeVisitor<'db> {
    fn is_supertrait_of_current_trait(&mut self, trait_ref: TraitRef<'db>) -> bool {
        let interner = self.interner;
        // Compute supertraits of current trait lazily.
        let supertraits = self.supertraits.get_or_insert_with(|| {
            elaborate::supertraits(
                interner,
                Binder::dummy(TraitRef::identity(interner, self.trait_def_id.into())),
            )
            .map(|trait_ref| {
                interner.erase_and_anonymize_regions(
                    interner.instantiate_bound_regions_with_erased(trait_ref),
                )
            })
            .collect()
        });

        // Determine whether the given trait ref is in fact a supertrait of the current trait.
        // In that case, any derived projections are legal, because the term will be specified
        // in the trait object type.
        // Note that we can just use direct equality here because all of these types are part of
        // the formal parameter listing, and hence there should be no inference variables.
        let trait_ref = trait_ref.fold_with(&mut EraseEscapingBoundRegions {
            interner: self.interner,
            binder: rustc_type_ir::INNERMOST,
            types: self.types,
        });
        supertraits.contains(&trait_ref)
    }
}

impl<'db> TypeVisitor<DbInterner<'db>> for IllegalSelfTypeVisitor<'db> {
    type Result = ControlFlow<()>;

    fn visit_ty(&mut self, ty: Ty<'db>) -> Self::Result {
        match ty.kind() {
            TyKind::Param(_) => {
                if ty.is_param(GenericParams::SELF_PARAM_ID_IN_SELF.into_raw().into_u32()) {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            }
            // FIXME: Once we implement RPITIT.
            // TyKind::Alias(_, AliasTy { kind: AliasTyKind::Projection { def_id }, .. })
            //     if self.db.is_impl_trait_in_trait(*def_id) =>
            // {
            //     // We'll deny these later in their own pass
            //     ControlFlow::Continue(())
            // }
            TyKind::Alias(proj @ AliasTy { kind: AliasTyKind::Projection { .. }, .. }) => {
                match self.allow_self_projections {
                    AllowSelfProjections::Yes => {
                        // Only walk contained types if the parent trait is not a supertrait.
                        if self.is_supertrait_of_current_trait(proj.trait_ref(self.interner)) {
                            ControlFlow::Continue(())
                        } else {
                            ty.super_visit_with(self)
                        }
                    }
                    AllowSelfProjections::No => ty.super_visit_with(self),
                }
            }
            _ => ty.super_visit_with(self),
        }
    }

    // FIXME: Should implement this once we sync rustc_type_ir enough and potentially implement GCA.
    // fn visit_const(&mut self, ct: Const<'db>) -> Self::Result {
    //     let ct = self.db.expand_abstract_consts(ct);

    //     match ct.kind() {
    //         ty::ConstKind::Alias(
    //             _,
    //             ty::AliasConst { kind: ty::AliasConstKind::Projection { def_id }, args, .. },
    //         ) if self.db.features().min_generic_const_args() => {
    //             match self.allow_self_projections {
    //                 AllowSelfProjections::Yes => {
    //                     let trait_def_id = self.db.parent(def_id);
    //                     let trait_ref = TraitRef::from_assoc(self.interner, trait_def_id, args);

    //                     // Only walk contained consts if the parent trait is not a supertrait.
    //                     if self.is_supertrait_of_current_trait(trait_ref) {
    //                         ControlFlow::Continue(())
    //                     } else {
    //                         ct.super_visit_with(self)
    //                     }
    //                 }
    //                 AllowSelfProjections::No => ct.super_visit_with(self),
    //             }
    //         }
    //         _ => ct.super_visit_with(self),
    //     }
    // }
}

struct EraseEscapingBoundRegions<'db> {
    interner: DbInterner<'db>,
    binder: DebruijnIndex,
    types: &'db DefaultAny<'db>,
}

impl<'db> TypeFolder<DbInterner<'db>> for EraseEscapingBoundRegions<'db> {
    fn cx(&self) -> DbInterner<'db> {
        self.interner
    }

    fn fold_binder<T>(&mut self, t: Binder<'db, T>) -> Binder<'db, T>
    where
        T: TypeFoldable<DbInterner<'db>>,
    {
        self.binder.shift_in(1);
        let result = t.super_fold_with(self);
        self.binder.shift_out(1);
        result
    }

    fn fold_region(&mut self, r: Region<'db>) -> Region<'db> {
        if let RegionKind::ReBound(BoundVarIndexKind::Bound(debruijn), _) = r.kind()
            && debruijn < self.binder
        {
            r
        } else {
            self.types.regions.erased
        }
    }
}

fn contains_illegal_impl_trait_in_trait<'db>(
    interner: DbInterner<'db>,
    fn_def_id: FunctionId,
    data: &FunctionSignature,
    ty: Binder<'db, Ty<'db>>,
) -> Option<MethodViolation> {
    if !ty.has_aliases() {
        return None;
    }

    let ty = interner.liberate_late_bound_regions(fn_def_id.into(), ty);

    if data.is_async() {
        // Rendering the error as a separate `async-specific` message is better.
        Some(MethodViolation::AsyncFn)
    } else {
        ty.visit_with(&mut IllegalRpititVisitor { db: interner.db }).break_value()
    }
}

struct IllegalRpititVisitor<'db> {
    db: &'db dyn HirDatabase,
}

impl<'db> TypeVisitor<DbInterner<'db>> for IllegalRpititVisitor<'db> {
    type Result = ControlFlow<MethodViolation>;

    fn visit_ty(&mut self, ty: Ty<'db>) -> Self::Result {
        if let TyKind::Alias(AliasTy { kind: AliasTyKind::Opaque { def_id }, .. }) = ty.kind()
            && matches!(def_id.0.loc(self.db), ImplTraitId::ReturnTypeImplTrait(..))
        {
            ControlFlow::Break(MethodViolation::ReferencesImplTraitInTrait)
        } else {
            ty.super_visit_with(self)
        }
    }
}
