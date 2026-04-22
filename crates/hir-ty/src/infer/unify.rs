//! Unification and canonicalization logic.

use std::fmt;

use base_db::Crate;
use hir_def::{ExpressionStoreOwnerId, GenericParamId, TraitId};
use rustc_hash::FxHashSet;
use rustc_type_ir::{
    TyVid, TypeFoldable, TypeVisitableExt,
    inherent::{Const as _, GenericArg as _, IntoKind, Ty as _},
    solve::Certainty,
};
use smallvec::SmallVec;

use crate::{
    db::HirDatabase,
    next_solver::{
        Canonical, ClauseKind, Const, DbInterner, ErrorGuaranteed, GenericArg, GenericArgs,
        ParamEnv, Predicate, PredicateKind, Region, SolverDefId, Term, TraitRef, Ty, TyKind,
        TypingMode,
        fulfill::{FulfillmentCtxt, NextSolverError},
        infer::{
            DbInternerInferExt, InferCtxt, InferOk,
            at::At,
            snapshot::CombinedSnapshot,
            traits::{Obligation, ObligationCause, PredicateObligation},
        },
        inspect::{InspectConfig, InspectGoal, ProofTreeVisitor},
        obligation_ctxt::ObligationCtxt,
    },
    traits::ParamEnvAndCrate,
};

struct NestedObligationsForSelfTy<'a, 'db> {
    ctx: &'a InferenceTable<'db>,
    self_ty: TyVid,
    root_cause: &'a ObligationCause,
    obligations_for_self_ty: &'a mut SmallVec<[Obligation<'db, Predicate<'db>>; 4]>,
}

impl<'a, 'db> ProofTreeVisitor<'db> for NestedObligationsForSelfTy<'a, 'db> {
    type Result = ();

    fn config(&self) -> InspectConfig {
        // Using an intentionally low depth to minimize the chance of future
        // breaking changes in case we adapt the approach later on. This also
        // avoids any hangs for exponentially growing proof trees.
        InspectConfig { max_depth: 5 }
    }

    fn visit_goal(&mut self, inspect_goal: &InspectGoal<'_, 'db>) {
        // No need to walk into goal subtrees that certainly hold, since they
        // wouldn't then be stalled on an infer var.
        if inspect_goal.result() == Ok(Certainty::Yes) {
            return;
        }

        let db = self.ctx.interner();
        let goal = inspect_goal.goal();
        if self.ctx.predicate_has_self_ty(goal.predicate, self.self_ty) {
            self.obligations_for_self_ty.push(Obligation::new(
                db,
                self.root_cause.clone(),
                goal.param_env,
                goal.predicate,
            ));
        }

        // If there's a unique way to prove a given goal, recurse into
        // that candidate. This means that for `impl<F: FnOnce(u32)> Trait<F> for () {}`
        // and a `(): Trait<?0>` goal we recurse into the impl and look at
        // the nested `?0: FnOnce(u32)` goal.
        if let Some(candidate) = inspect_goal.unique_applicable_candidate() {
            candidate.visit_nested_no_probe(self)
        }
    }
}

/// Check if types unify.
///
/// Note that we consider placeholder types to unify with everything.
/// This means that there may be some unresolved goals that actually set bounds for the placeholder
/// type for the types to unify. For example `Option<T>` and `Option<U>` unify although there is
/// unresolved goal `T = U`.
pub fn could_unify<'db>(
    db: &'db dyn HirDatabase,
    env: ParamEnvAndCrate<'db>,
    tys: &Canonical<'db, (Ty<'db>, Ty<'db>)>,
) -> bool {
    could_unify_impl(db, env, tys, |ctxt| ctxt.try_evaluate_obligations())
}

/// Check if types unify eagerly making sure there are no unresolved goals.
///
/// This means that placeholder types are not considered to unify if there are any bounds set on
/// them. For example `Option<T>` and `Option<U>` do not unify as we cannot show that `T = U`
pub fn could_unify_deeply<'db>(
    db: &'db dyn HirDatabase,
    env: ParamEnvAndCrate<'db>,
    tys: &Canonical<'db, (Ty<'db>, Ty<'db>)>,
) -> bool {
    could_unify_impl(db, env, tys, |ctxt| ctxt.evaluate_obligations_error_on_ambiguity())
}

fn could_unify_impl<'db>(
    db: &'db dyn HirDatabase,
    env: ParamEnvAndCrate<'db>,
    tys: &Canonical<'db, (Ty<'db>, Ty<'db>)>,
    select: for<'a> fn(&mut ObligationCtxt<'a, 'db>) -> Vec<NextSolverError<'db>>,
) -> bool {
    let interner = DbInterner::new_with(db, env.krate);
    let infcx = interner.infer_ctxt().build(TypingMode::PostAnalysis);
    let cause = ObligationCause::dummy();
    let at = infcx.at(&cause, env.param_env);
    let ((ty1_with_vars, ty2_with_vars), _) = infcx.instantiate_canonical(tys);
    let mut ctxt = ObligationCtxt::new(&infcx);
    let can_unify = at
        .eq(ty1_with_vars, ty2_with_vars)
        .map(|infer_ok| ctxt.register_infer_ok_obligations(infer_ok))
        .is_ok();
    can_unify && select(&mut ctxt).is_empty()
}

#[derive(Clone)]
pub(crate) struct InferenceTable<'db> {
    pub(crate) db: &'db dyn HirDatabase,
    pub(crate) param_env: ParamEnv<'db>,
    pub(crate) infer_ctxt: InferCtxt<'db>,
    pub(super) fulfillment_cx: FulfillmentCtxt<'db>,
    pub(super) diverging_type_vars: FxHashSet<Ty<'db>>,
}

impl<'db> InferenceTable<'db> {
    /// Inside hir-ty you should use this for inference only, and always pass `owner`.
    /// Outside it, always pass `owner = None`.
    pub(crate) fn new(
        db: &'db dyn HirDatabase,
        trait_env: ParamEnv<'db>,
        krate: Crate,
        owner: ExpressionStoreOwnerId,
    ) -> Self {
        let interner = DbInterner::new_with(db, krate);
        let typing_mode = TypingMode::typeck_for_body(interner, owner.into());
        let infer_ctxt = interner.infer_ctxt().build(typing_mode);
        InferenceTable {
            db,
            param_env: trait_env,
            fulfillment_cx: FulfillmentCtxt::new(&infer_ctxt),
            infer_ctxt,
            diverging_type_vars: FxHashSet::default(),
        }
    }

    #[inline]
    pub(crate) fn interner(&self) -> DbInterner<'db> {
        self.infer_ctxt.interner
    }

    pub(crate) fn type_is_copy_modulo_regions(&self, ty: Ty<'db>) -> bool {
        self.infer_ctxt.type_is_copy_modulo_regions(self.param_env, ty)
    }

    pub(crate) fn type_is_sized_modulo_regions(&self, ty: Ty<'db>) -> bool {
        self.infer_ctxt.type_is_sized_modulo_regions(self.param_env, ty)
    }

    pub(crate) fn type_is_use_cloned_modulo_regions(&self, ty: Ty<'db>) -> bool {
        self.infer_ctxt.type_is_use_cloned_modulo_regions(self.param_env, ty)
    }

    pub(crate) fn type_var_is_sized(&self, self_ty: TyVid) -> bool {
        let Some(sized_did) = self.interner().lang_items().Sized else {
            return true;
        };
        self.obligations_for_self_ty(self_ty).into_iter().any(|obligation| {
            match obligation.predicate.kind().skip_binder() {
                PredicateKind::Clause(ClauseKind::Trait(data)) => data.def_id().0 == sized_did,
                _ => false,
            }
        })
    }

    pub(super) fn obligations_for_self_ty(
        &self,
        self_ty: TyVid,
    ) -> SmallVec<[Obligation<'db, Predicate<'db>>; 4]> {
        let obligations = self.fulfillment_cx.pending_obligations();
        let mut obligations_for_self_ty = SmallVec::new();
        for obligation in obligations {
            let mut visitor = NestedObligationsForSelfTy {
                ctx: self,
                self_ty,
                obligations_for_self_ty: &mut obligations_for_self_ty,
                root_cause: &obligation.cause,
            };

            let goal = obligation.as_goal();
            self.infer_ctxt.visit_proof_tree(goal, &mut visitor);
        }

        obligations_for_self_ty.retain_mut(|obligation| {
            obligation.predicate = self.infer_ctxt.resolve_vars_if_possible(obligation.predicate);
            !obligation.predicate.has_placeholders()
        });
        obligations_for_self_ty
    }

    fn predicate_has_self_ty(&self, predicate: Predicate<'db>, expected_vid: TyVid) -> bool {
        match predicate.kind().skip_binder() {
            PredicateKind::Clause(ClauseKind::Trait(data)) => {
                self.type_matches_expected_vid(expected_vid, data.self_ty())
            }
            PredicateKind::Clause(ClauseKind::Projection(data)) => {
                self.type_matches_expected_vid(expected_vid, data.projection_term.self_ty())
            }
            PredicateKind::Clause(ClauseKind::ConstArgHasType(..))
            | PredicateKind::Subtype(..)
            | PredicateKind::Coerce(..)
            | PredicateKind::Clause(ClauseKind::RegionOutlives(..))
            | PredicateKind::Clause(ClauseKind::TypeOutlives(..))
            | PredicateKind::Clause(ClauseKind::WellFormed(..))
            | PredicateKind::DynCompatible(..)
            | PredicateKind::NormalizesTo(..)
            | PredicateKind::AliasRelate(..)
            | PredicateKind::Clause(ClauseKind::ConstEvaluatable(..))
            | PredicateKind::ConstEquate(..)
            | PredicateKind::Clause(ClauseKind::HostEffect(..))
            | PredicateKind::Clause(ClauseKind::UnstableFeature(_))
            | PredicateKind::Ambiguous => false,
        }
    }

    fn type_matches_expected_vid(&self, expected_vid: TyVid, ty: Ty<'db>) -> bool {
        let ty = self.shallow_resolve(ty);

        match ty.kind() {
            TyKind::Infer(rustc_type_ir::TyVar(found_vid)) => {
                self.infer_ctxt.root_var(expected_vid) == self.infer_ctxt.root_var(found_vid)
            }
            _ => false,
        }
    }

    pub(super) fn set_diverging(&mut self, ty: Ty<'db>) {
        self.diverging_type_vars.insert(ty);
    }

    pub(crate) fn next_ty_var(&self) -> Ty<'db> {
        self.infer_ctxt.next_ty_var()
    }

    pub(crate) fn next_const_var(&self) -> Const<'db> {
        self.infer_ctxt.next_const_var()
    }

    pub(crate) fn next_int_var(&self) -> Ty<'db> {
        self.infer_ctxt.next_int_var()
    }

    pub(crate) fn next_float_var(&self) -> Ty<'db> {
        self.infer_ctxt.next_float_var()
    }

    pub(crate) fn new_maybe_never_var(&mut self) -> Ty<'db> {
        let var = self.next_ty_var();
        self.set_diverging(var);
        var
    }

    pub(crate) fn next_region_var(&self) -> Region<'db> {
        self.infer_ctxt.next_region_var()
    }

    pub(crate) fn next_var_for_param(&self, id: GenericParamId) -> GenericArg<'db> {
        self.infer_ctxt.next_var_for_param(id)
    }

    pub(crate) fn resolve_completely<T>(&mut self, value: T) -> T
    where
        T: TypeFoldable<DbInterner<'db>>,
    {
        let value = self.infer_ctxt.resolve_vars_if_possible(value);

        let mut goals = vec![];

        // FIXME(next-solver): Handle `goals`.

        value.fold_with(&mut resolve_completely::Resolver::new(self, true, &mut goals))
    }

    pub(crate) fn at<'a>(&'a self, cause: &'a ObligationCause) -> At<'a, 'db> {
        self.infer_ctxt.at(cause, self.param_env)
    }

    pub(crate) fn shallow_resolve(&self, ty: Ty<'db>) -> Ty<'db> {
        self.infer_ctxt.shallow_resolve(ty)
    }

    pub(crate) fn resolve_vars_if_possible<T: TypeFoldable<DbInterner<'db>>>(&self, t: T) -> T {
        self.infer_ctxt.resolve_vars_if_possible(t)
    }

    pub(crate) fn resolve_vars_with_obligations<T>(&mut self, t: T) -> T
    where
        T: rustc_type_ir::TypeFoldable<DbInterner<'db>>,
    {
        if !t.has_non_region_infer() {
            return t;
        }

        let t = self.infer_ctxt.resolve_vars_if_possible(t);

        if !t.has_non_region_infer() {
            return t;
        }

        self.select_obligations_where_possible();
        self.infer_ctxt.resolve_vars_if_possible(t)
    }

    /// Create a `GenericArgs` full of infer vars for `def`.
    pub(crate) fn fresh_args_for_item(&self, def: SolverDefId) -> GenericArgs<'db> {
        self.infer_ctxt.fresh_args_for_item(def)
    }

    /// Try to resolve `ty` to a structural type, normalizing aliases.
    ///
    /// In case there is still ambiguity, the returned type may be an inference
    /// variable. This is different from `structurally_resolve_type` which errors
    /// in this case.
    pub(crate) fn try_structurally_resolve_type(&mut self, ty: Ty<'db>) -> Ty<'db> {
        if let TyKind::Alias(..) = ty.kind() {
            let result = self
                .infer_ctxt
                .at(&ObligationCause::misc(), self.param_env)
                .structurally_normalize_ty(ty, &mut self.fulfillment_cx);
            match result {
                Ok(normalized_ty) => normalized_ty,
                Err(_errors) => Ty::new_error(self.interner(), ErrorGuaranteed),
            }
        } else {
            self.resolve_vars_with_obligations(ty)
        }
    }

    pub(crate) fn snapshot(&mut self) -> CombinedSnapshot {
        self.infer_ctxt.start_snapshot()
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn rollback_to(&mut self, snapshot: CombinedSnapshot) {
        self.infer_ctxt.rollback_to(snapshot);
    }

    pub(crate) fn commit_if_ok<T, E>(
        &mut self,
        f: impl FnOnce(&mut InferenceTable<'db>) -> Result<T, E>,
    ) -> Result<T, E> {
        let snapshot = self.snapshot();
        let result = f(self);
        match result {
            Ok(_) => self.infer_ctxt.commit_from(snapshot),
            Err(_) => self.rollback_to(snapshot),
        }
        result
    }

    pub(crate) fn register_bound(&mut self, ty: Ty<'db>, def_id: TraitId, cause: ObligationCause) {
        if !ty.references_non_lt_error() {
            let trait_ref = TraitRef::new(self.interner(), def_id.into(), [ty]);
            self.register_predicate(Obligation::new(
                self.interner(),
                cause,
                self.param_env,
                trait_ref,
            ));
        }
    }

    pub(crate) fn register_infer_ok<T>(&mut self, infer_ok: InferOk<'db, T>) -> T {
        let InferOk { value, obligations } = infer_ok;
        self.register_predicates(obligations);
        value
    }

    pub(crate) fn select_obligations_where_possible(&mut self) {
        self.fulfillment_cx.try_evaluate_obligations(&self.infer_ctxt);
    }

    pub(super) fn register_predicate(&mut self, obligation: PredicateObligation<'db>) {
        if obligation.has_escaping_bound_vars() {
            panic!("escaping bound vars in predicate {:?}", obligation);
        }

        self.fulfillment_cx.register_predicate_obligation(&self.infer_ctxt, obligation);
    }

    pub(crate) fn register_predicates<I>(&mut self, obligations: I)
    where
        I: IntoIterator<Item = PredicateObligation<'db>>,
    {
        obligations.into_iter().for_each(|obligation| {
            self.register_predicate(obligation);
        });
    }

    /// checking later, during regionck, that `arg` is well-formed.
    pub(crate) fn register_wf_obligation(&mut self, term: Term<'db>, cause: ObligationCause) {
        self.register_predicate(Obligation::new(
            self.interner(),
            cause,
            self.param_env,
            ClauseKind::WellFormed(term),
        ));
    }

    /// Registers obligations that all `args` are well-formed.
    pub(crate) fn add_wf_bounds(&mut self, args: GenericArgs<'db>) {
        for term in args.iter().filter_map(|it| it.as_term()) {
            self.register_wf_obligation(term, ObligationCause::new());
        }
    }

    pub(super) fn insert_type_vars<T>(&mut self, ty: T) -> T
    where
        T: TypeFoldable<DbInterner<'db>>,
    {
        self.infer_ctxt.insert_type_vars(ty)
    }

    /// Whenever you lower a user-written type, you should call this.
    pub(crate) fn process_user_written_ty(&mut self, ty: Ty<'db>) -> Ty<'db> {
        self.process_remote_user_written_ty(ty)
        // FIXME: Register a well-formed obligation.
    }

    /// The difference of this method from `process_user_written_ty()` is that this method doesn't register a well-formed obligation,
    /// while `process_user_written_ty()` should (but doesn't currently).
    pub(crate) fn process_remote_user_written_ty(&mut self, ty: Ty<'db>) -> Ty<'db> {
        let ty = self.insert_type_vars(ty);
        // See https://github.com/rust-lang/rust/blob/cdb45c87e2cd43495379f7e867e3cc15dcee9f93/compiler/rustc_hir_typeck/src/fn_ctxt/mod.rs#L487-L495:
        // Even though the new solver only lazily normalizes usually, here we eagerly normalize so that not everything needs
        // to normalize before inspecting the `TyKind`.
        // FIXME(next-solver): We should not deeply normalize here, only shallowly.
        self.try_structurally_resolve_type(ty)
    }

    /// Replaces ConstScalar::Unknown by a new type var, so we can maybe still infer it.
    pub(super) fn insert_const_vars_shallow(&mut self, c: Const<'db>) -> Const<'db> {
        if c.is_ct_error() { self.next_const_var() } else { c }
    }
}

impl fmt::Debug for InferenceTable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InferenceTable")
            .field("name", &self.infer_ctxt.inner.borrow().type_variable_storage)
            .field("fulfillment_cx", &self.fulfillment_cx)
            .finish()
    }
}

mod resolve_completely {
    use rustc_type_ir::{DebruijnIndex, Flags, TypeFolder, TypeSuperFoldable};

    use crate::{
        infer::unify::InferenceTable,
        next_solver::{
            Const, DbInterner, Goal, Predicate, Region, Term, Ty,
            infer::{resolve::ReplaceInferWithError, traits::ObligationCause},
            normalize::deeply_normalize_with_skipped_universes_and_ambiguous_coroutine_goals,
        },
    };

    pub(super) struct Resolver<'a, 'db> {
        ctx: &'a mut InferenceTable<'db>,
        /// Whether we should normalize, disabled when resolving predicates.
        should_normalize: bool,
        nested_goals: &'a mut Vec<Goal<'db, Predicate<'db>>>,
    }

    impl<'a, 'db> Resolver<'a, 'db> {
        pub(super) fn new(
            ctx: &'a mut InferenceTable<'db>,
            should_normalize: bool,
            nested_goals: &'a mut Vec<Goal<'db, Predicate<'db>>>,
        ) -> Resolver<'a, 'db> {
            Resolver { ctx, nested_goals, should_normalize }
        }

        fn handle_term<T>(
            &mut self,
            value: T,
            outer_exclusive_binder: impl FnOnce(T) -> DebruijnIndex,
        ) -> T
        where
            T: Into<Term<'db>> + TypeSuperFoldable<DbInterner<'db>> + Copy,
        {
            let value = if self.should_normalize {
                let cause = ObligationCause::new();
                let at = self.ctx.at(&cause);
                let universes = vec![None; outer_exclusive_binder(value).as_usize()];
                match deeply_normalize_with_skipped_universes_and_ambiguous_coroutine_goals(
                    at, value, universes,
                ) {
                    Ok((value, goals)) => {
                        self.nested_goals.extend(goals);
                        value
                    }
                    Err(_errors) => {
                        // FIXME: Report the error.
                        value
                    }
                }
            } else {
                value
            };

            value.fold_with(&mut ReplaceInferWithError::new(self.ctx.interner()))
        }
    }

    impl<'cx, 'db> TypeFolder<DbInterner<'db>> for Resolver<'cx, 'db> {
        fn cx(&self) -> DbInterner<'db> {
            self.ctx.interner()
        }

        fn fold_region(&mut self, r: Region<'db>) -> Region<'db> {
            if r.is_var() { Region::error(self.ctx.interner()) } else { r }
        }

        fn fold_ty(&mut self, ty: Ty<'db>) -> Ty<'db> {
            self.handle_term(ty, |it| it.outer_exclusive_binder())
        }

        fn fold_const(&mut self, ct: Const<'db>) -> Const<'db> {
            self.handle_term(ct, |it| it.outer_exclusive_binder())
        }

        fn fold_predicate(&mut self, predicate: Predicate<'db>) -> Predicate<'db> {
            assert!(
                !self.should_normalize,
                "normalizing predicates in writeback is not generally sound"
            );
            predicate.super_fold_with(self)
        }
    }
}
