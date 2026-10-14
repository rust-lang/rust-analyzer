use hir_def::GenericDefId;
use rustc_hash::FxHashSet;
use rustc_type_ir::{
    AliasTyKind, ConstKind, InferTy, Interner as _, OutlivesPredicate, TypeSuperVisitable as _,
    TypeVisitable as _, TypeVisitableExt as _, TypeVisitor, Upcast,
    inherent::{BoundExistentialPredicates as _, GenericArgs as _, IntoKind as _, Term as _},
};

use super::{
    BoundExistentialPredicates, Clause, ClauseKind, Const, Ctor, DbInterner, GenericArgs, Goal,
    ParamEnv, Pattern, PatternKind, Predicate, PredicateKind, Region, SolverDefId, Term, TermKind,
    TraitRef, Ty, TyKind,
};
use crate::{dyn_compatibility::object_region_bounds, lower::GenericPredicates};

pub(crate) fn unnormalized_obligations<'db>(
    interner: DbInterner<'db>,
    param_env: ParamEnv<'db>,
    term: Term<'db>,
) -> Option<Vec<Goal<'db, Predicate<'db>>>> {
    if term.is_infer() {
        return None;
    }
    stdx::always!(!term.has_escaping_bound_vars());

    let mut wf =
        WfPredicates { interner, param_env, out: Vec::new(), visited_tys: FxHashSet::default() };
    wf.add_wf_preds_for_term(term);
    if wf.out.is_empty() {
        return None;
    }
    Some(wf.out)
}

struct WfPredicates<'db> {
    interner: DbInterner<'db>,
    param_env: ParamEnv<'db>,
    out: Vec<Goal<'db, Predicate<'db>>>,
    visited_tys: FxHashSet<Ty<'db>>,
}

impl<'db> WfPredicates<'db> {
    fn push_goal(&mut self, predicate: impl Upcast<DbInterner<'db>, Predicate<'db>>) {
        self.out.push(Goal::new(self.interner, self.param_env, predicate));
    }

    fn push_clause(&mut self, clause: Clause<'db>) {
        self.out.push(Goal::new(self.interner, self.param_env, clause));
    }

    fn require_sized(&mut self, subty: Ty<'db>) {
        if !subty.has_escaping_bound_vars()
            && let Some(sized) = self.interner.lang_items().Sized
        {
            self.push_goal(TraitRef::new(self.interner, sized.into(), [subty]));
        }
    }

    fn add_wf_preds_for_term(&mut self, term: Term<'db>) {
        match term.kind() {
            TermKind::Ty(ty) => ty.visit_with(self),
            TermKind::Const(ct) => ct.visit_with(self),
        }
    }

    fn nominal_obligations(&mut self, def_id: SolverDefId<'db>, args: GenericArgs<'db>) {
        self.nominal_obligations_filtered(def_id, args, |_| true);
    }

    fn nominal_obligations_filtered(
        &mut self,
        def_id: SolverDefId<'db>,
        args: GenericArgs<'db>,
        mut keep: impl FnMut(&Clause<'db>) -> bool,
    ) {
        if let SolverDefId::TraitId(trait_id) = def_id
            && self.interner.lang_items().Sized.is_some_and(|sized| sized == trait_id)
        {
            return;
        }

        let db = self.interner.db;
        if let SolverDefId::InternedOpaqueTyId(opaque) = def_id {
            for clause in opaque
                .predicates(db)
                .iter_instantiated_copied(self.interner, args.as_slice())
                .map(|clause| clause.skip_norm_wip())
                .filter(|clause| !clause.has_escaping_bound_vars())
                .filter(|clause| keep(clause))
            {
                self.push_clause(clause);
            }
            return;
        }

        if matches!(
            def_id,
            SolverDefId::InternedClosureId(_)
                | SolverDefId::InternedCoroutineId(_)
                | SolverDefId::InternedCoroutineClosureId(_)
                | SolverDefId::EnumVariantId(_)
        ) {
            return;
        }

        if let SolverDefId::AnonConstId(anon_const) = def_id {
            let loc = anon_const.loc(db);
            if loc.allow_using_generic_params {
                let owner = loc.owner.generic_def(db);
                self.push_instantiated_predicates(owner, args, &mut keep);
            }
            return;
        }

        let generic_def: GenericDefId = match def_id {
            SolverDefId::Ctor(Ctor::Struct(id)) => id.into(),
            SolverDefId::Ctor(Ctor::Enum(variant)) => variant.loc(db).parent.into(),
            _ => match def_id.try_into() {
                Ok(def) => def,
                Err(()) => return,
            },
        };
        self.push_instantiated_predicates(generic_def, args, &mut keep);
    }

    fn push_instantiated_predicates(
        &mut self,
        def: GenericDefId,
        args: GenericArgs<'db>,
        keep: &mut impl FnMut(&Clause<'db>) -> bool,
    ) {
        let predicates = GenericPredicates::query_all(self.interner.db, def);
        for clause in predicates
            .iter_instantiated(self.interner, args.as_slice())
            .map(|clause| clause.skip_norm_wip())
            .filter(|clause| !clause.has_escaping_bound_vars())
            .filter(|clause| keep(clause))
        {
            self.push_clause(clause);
        }
    }

    fn add_wf_preds_for_dyn_ty(
        &mut self,
        ty: Ty<'db>,
        bounds: BoundExistentialPredicates<'db>,
        region: Region<'db>,
    ) {
        if !bounds.has_escaping_bound_vars() && !region.has_escaping_bound_vars() {
            for implicit_bound in object_region_bounds(self.interner, bounds) {
                self.push_goal(ClauseKind::RegionOutlives(OutlivesPredicate(
                    region,
                    implicit_bound,
                )));
            }
        }

        if !bounds.as_slice().is_empty()
            && let Some(principal) = bounds.principal()
        {
            let principal_def_id = principal.skip_binder().def_id;
            self.push_goal(PredicateKind::DynCompatible(principal_def_id));

            let args = principal.skip_binder().with_self_ty(self.interner, ty).args;
            self.nominal_obligations_filtered(
                principal_def_id.0.into(),
                args,
                &mut |clause: &Clause<'_>| {
                    matches!(
                        clause.as_predicate().kind().skip_binder(),
                        PredicateKind::Clause(ClauseKind::ConstArgHasType(ct, _))
                            if matches!(
                                ct.kind(),
                                ConstKind::Param(..) | ConstKind::Placeholder(..)
                            )
                    )
                },
            );
        }

        if !ty.has_escaping_bound_vars() {
            for projection in bounds.projection_bounds() {
                let item_def_id = projection.item_def_id();
                let projection = projection.with_self_ty(self.interner, ty);
                let Some(projection) = projection.no_bound_vars() else {
                    continue;
                };
                if let Some(ct) = projection.term.as_const() {
                    let assoc_const_ty = self
                        .interner
                        .type_of(item_def_id.0.into())
                        .instantiate(self.interner, projection.projection_term.args)
                        .skip_norm_wip();
                    self.push_goal(ClauseKind::ConstArgHasType(ct, assoc_const_ty));
                }
            }
        }
    }

    fn add_wf_preds_for_pat_ty(&mut self, base_ty: Ty<'db>, pat: Pattern<'db>) {
        match *pat.inner() {
            PatternKind::Range { start, end } => {
                self.push_goal(ClauseKind::ConstArgHasType(start, base_ty));
                self.push_goal(ClauseKind::ConstArgHasType(end, base_ty));
            }
            PatternKind::Or(pats) => {
                for pat in pats.as_slice().iter() {
                    self.add_wf_preds_for_pat_ty(base_ty, *pat);
                }
            }
            PatternKind::NotNull => {}
        }
    }
}

impl<'db> TypeVisitor<DbInterner<'db>> for WfPredicates<'db> {
    type Result = ();

    fn visit_ty(&mut self, t: Ty<'db>) {
        if !self.visited_tys.insert(t) {
            return;
        }

        let interner = self.interner;
        match t.kind() {
            TyKind::Bool
            | TyKind::Char
            | TyKind::Int(..)
            | TyKind::Uint(..)
            | TyKind::Float(..)
            | TyKind::Error(_)
            | TyKind::Str
            | TyKind::CoroutineWitness(..)
            | TyKind::Never
            | TyKind::Param(_)
            | TyKind::Bound(..)
            | TyKind::Placeholder(..)
            | TyKind::Foreign(..) => {}

            TyKind::Infer(InferTy::IntVar(_) | InferTy::FloatVar(_)) => {}

            TyKind::Slice(subty) => {
                self.require_sized(subty);
            }

            TyKind::Array(subty, len) => {
                self.require_sized(subty);
                self.push_goal(ClauseKind::ConstArgHasType(
                    len,
                    interner.default_types().types.usize,
                ));
            }

            TyKind::Pat(base_ty, pat) => {
                self.require_sized(base_ty);
                self.add_wf_preds_for_pat_ty(base_ty, pat);
            }

            TyKind::Tuple(tys) => {
                if let Some((_, rest)) = tys.as_slice().split_last() {
                    for &elem in rest {
                        self.require_sized(elem);
                    }
                }
            }

            TyKind::RawPtr(..) => {}

            TyKind::Alias(alias) => match alias.kind {
                AliasTyKind::Projection { def_id } => {
                    self.nominal_obligations(def_id.0.into(), alias.args);
                }
                AliasTyKind::Free { def_id } => {
                    self.nominal_obligations(def_id.0.into(), alias.args);
                }
                AliasTyKind::Opaque { def_id } => {
                    self.nominal_obligations(def_id.into(), alias.args);
                }
                AliasTyKind::Inherent { .. } => {
                    // FIXME: needs impl-format args via selection; only the
                    // args are visited. Misses some ill-formed types, never
                    // false-positives.
                }
            },

            TyKind::Adt(def, args) => {
                self.nominal_obligations(SolverDefId::AdtId(def.def_id()), args);
            }

            TyKind::FnDef(did, args) => {
                if !args.has_escaping_bound_vars() {
                    let sig =
                        interner.fn_sig(did).instantiate(interner, args.as_slice()).skip_norm_wip();
                    if let Some(output) = sig.output().no_bound_vars() {
                        output.visit_with(self);
                    }
                }

                self.nominal_obligations(did.into(), args);
            }

            TyKind::Ref(region, rty, _) => {
                if !region.has_escaping_bound_vars() && !rty.has_escaping_bound_vars() {
                    self.push_goal(ClauseKind::TypeOutlives(OutlivesPredicate(rty, region)));
                }
            }

            TyKind::Coroutine(..) => {}

            TyKind::Closure(_, args) => {
                args.as_closure().tupled_upvars_ty().visit_with(self);
                return;
            }

            TyKind::CoroutineClosure(_, args) => {
                args.as_coroutine_closure().tupled_upvars_ty().visit_with(self);
                return;
            }

            TyKind::FnPtr(..) => {}

            TyKind::UnsafeBinder(binder) => {
                if let Some(inner) = binder.no_bound_vars()
                    && let Some(no_drop) = interner.lang_items().BikeshedGuaranteedNoDrop
                {
                    self.push_goal(TraitRef::new(interner, no_drop.into(), [inner]));
                }
            }

            TyKind::Dynamic(bounds, region) => {
                self.add_wf_preds_for_dyn_ty(t, bounds, region);
            }

            TyKind::Infer(_) => {
                self.push_goal(ClauseKind::WellFormed(t.into()));
            }
        }

        t.super_visit_with(self);
    }

    fn visit_const(&mut self, c: Const<'db>) {
        match c.kind() {
            ConstKind::Unevaluated(uv) => {
                if !c.has_escaping_bound_vars() {
                    self.push_goal(ClauseKind::ConstEvaluatable(c));
                    self.nominal_obligations(uv.def.into(), uv.args);
                }
            }
            ConstKind::Infer(_) => {
                self.push_goal(ClauseKind::WellFormed(c.into()));
            }
            ConstKind::Expr(_) => {
                // FIXME(generic_const_exprs): bounds such as `typeof(N): Add`
                // for `Expr(N + 1)` are not verified. No goal is emitted: the
                // solver cannot evaluate consts of this kind.
            }
            ConstKind::Error(_)
            | ConstKind::Param(_)
            | ConstKind::Bound(..)
            | ConstKind::Placeholder(..) => {}
            ConstKind::Value(_) => {}
        }

        c.super_visit_with(self);
    }

    fn visit_predicate(&mut self, _p: Predicate<'db>) {
        stdx::never!("predicates should not be checked for well-formedness");
    }
}
