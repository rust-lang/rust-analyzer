//! Fulfill loop for next-solver.

use std::ops::ControlFlow;

use rustc_hash::{FxHashMap, FxHashSet};
use rustc_next_trait_solver::{
    delegate::SolverDelegate,
    solve::{GoalEvaluation, GoalStalledOn, HasChanged, SolverDelegateEvalExt},
};
use rustc_type_ir::{
    GenericArgKind, InferConst, InferCtxtLike, InferTy, Interner, TyVid, TypeSuperVisitable,
    TypeVisitable, TypeVisitableExt, TypeVisitor,
    inherent::{Const as _, IntoKind, OpaqueTypeStorageEntries as _, Ty as _},
    solve::{Certainty, Goal, NoSolution},
};
use smallvec::SmallVec;

use crate::{
    Span,
    next_solver::{
        Const, ConstKind, DbInterner, GenericArg, Predicate, SolverContext, SolverDefId, Ty,
        TyKind, TypingMode,
        infer::{
            InferCtxt, StalledVarKey,
            traits::{PredicateObligation, PredicateObligations},
        },
        inspect::ProofTreeVisitor,
    },
};

type PendingObligations<'db> =
    Vec<(PredicateObligation<'db>, Option<GoalStalledOn<DbInterner<'db>>>)>;

/// A trait engine using the new trait solver.
///
/// This is mostly identical to how `evaluate_all` works inside of the
/// solver, except that the requirements are slightly different.
///
/// Unlike `evaluate_all` it is possible to add new obligations later on
/// and we also have to track diagnostics information by using `Obligation`
/// instead of `Goal`.
///
/// It is also likely that we want to use slightly different datastructures
/// here as this will have to deal with far more root goals than `evaluate_all`.
#[derive(Debug)]
pub struct FulfillmentCtxt<'db> {
    obligations: ObligationStorage<'db>,

    /// The snapshot in which this context was created. Using the context
    /// outside of this snapshot leads to subtle bugs if the snapshot
    /// gets rolled back. Because of this we explicitly check that we only
    /// use the context in exactly this snapshot.
    usable_in_snapshot: usize,
    /// Read position into the infer context's changed-inference-variables log
    /// (see [`InferCtxt::changed_vars_since`]). Everything before this position has already
    /// been translated into wake-ups of the obligations parked in `obligations.watch`. The log
    /// is shared by every `FulfillmentCtxt` on the same `InferCtxt` but reading it is
    /// non-destructive, so short-lived contexts (nested probes, coercions, method resolution)
    /// don't steal wake signals from the long-lived, body-scoped one.
    changed_vars_cursor: usize,
    try_evaluate_obligations_scratch: PendingObligations<'db>,
    changed_vars_scratch: Vec<StalledVarKey>,
}

/// An ambiguous obligation parked on the variables of its `GoalStalledOn`, together with the
/// watch keys it is filed under in [`ObligationStorage::watch`].
#[derive(Debug, Clone)]
struct Slot<'db> {
    obligation: PredicateObligation<'db>,
    stalled_on: GoalStalledOn<DbInterner<'db>>,
    keys: SmallVec<[StalledVarKey; 4]>,
}

/// Obligations that are still ambiguous (`Certainty::Maybe`), indexed so that an inference
/// variable changing wakes exactly the obligations stalled on it, instead of every call to
/// `try_evaluate_obligations` rescanning everything still pending (which made large bodies
/// quadratic: O(calls) × O(pending)).
#[derive(Default, Debug, Clone)]
struct ObligationStorage<'db> {
    /// Obligations which resulted in an overflow in fulfillment itself.
    ///
    /// We cannot eagerly return these as error so we instead store them here
    /// to avoid recomputing them each time `try_evaluate_obligations` is called.
    /// This also allows us to return the correct `FulfillmentError` for them.
    overflowed: Vec<PredicateObligation<'db>>,
    /// Obligations registered from the outside and not yet evaluated in this context.
    /// Fully processed on the next `try_evaluate_obligations` call.
    fresh: Vec<PredicateObligation<'db>>,
    /// Obligations that evaluated to `Certainty::Maybe` but whose stall condition can't be
    /// attached to any watchable variable (no `stalled_on` at all, a `Fresh*` inference
    /// variable, or a stall set that had already been invalidated again by the time we got to
    /// park it). Re-evaluated on the first round of every call and after each round that made
    /// real progress — the same cadence at which the old code re-evaluated everything — but
    /// crucially they do *not* requeue rounds by themselves, which is what guarantees the
    /// fixpoint loop terminates once nothing is actually changing.
    unwatchable: PendingObligations<'db>,
    /// Ambiguous obligations parked on a known set of variables. `None` is a tombstone left
    /// behind by `wake`; slots are compacted once tombstones outnumber live entries, and
    /// `watch` is rebuilt from the keys stored in each slot at the same time.
    slab: Vec<Option<Slot<'db>>>,
    /// Number of `Some` entries in `slab`.
    live: usize,
    /// Maps a variable (or the opaque-type storage as a whole, see [`StalledVarKey::Opaques`])
    /// to the slab slots of obligations parked on it. An obligation is listed under each of its
    /// keys; waking it through any one of them empties its slot, turning the remaining entries
    /// into harmless tombstone hits.
    watch: FxHashMap<StalledVarKey, SmallVec<[usize; 2]>>,
}

/// Classifies every variable a [`GoalStalledOn`] is blocked on into a [`StalledVarKey`] for
/// indexing into [`ObligationStorage::watch`].
///
/// The caller must have just verified [`is_still_stalled`]: that check passing means every
/// tracked variable is an unresolved unification *root* (a resolved or re-rooted variable
/// counts as "changed"), so the variables can be used as watch keys directly. Mutations are
/// only observable on root slots, and a later union of the root itself fires a wake for it,
/// at which point the obligation re-registers under the merged root. Returns `None` if some
/// tracked variable isn't watchable (`Fresh*` inference variables, which no unification table
/// observes, and which `is_changed_arg` should already have rejected) — the caller must then
/// fall back to periodic rechecking rather than risk the obligation never being woken.
fn stalled_var_keys<'db>(
    stalled_on: &GoalStalledOn<DbInterner<'db>>,
) -> Option<SmallVec<[StalledVarKey; 4]>> {
    let mut keys =
        SmallVec::with_capacity(stalled_on.stalled_vars.len() + stalled_on.sub_roots.len() + 1);
    for arg in &stalled_on.stalled_vars {
        match arg.kind() {
            // Lifetimes can never stall a goal; the solver never puts one in `stalled_vars`.
            GenericArgKind::Lifetime(_) => {}
            GenericArgKind::Type(ty) => match ty.kind() {
                TyKind::Infer(InferTy::TyVar(vid)) => keys.push(StalledVarKey::Ty(vid)),
                TyKind::Infer(InferTy::IntVar(vid)) => keys.push(StalledVarKey::Int(vid)),
                TyKind::Infer(InferTy::FloatVar(vid)) => keys.push(StalledVarKey::Float(vid)),
                _ => return None,
            },
            GenericArgKind::Const(ct) => match ct.kind() {
                ConstKind::Infer(InferConst::Var(vid)) => keys.push(StalledVarKey::Const(vid)),
                _ => return None,
            },
        }
    }
    keys.extend(stalled_on.sub_roots.iter().map(|&vid| StalledVarKey::TySubRoot(vid)));
    // `num_opaques` is a condition of every `GoalStalledOn`, not tied to a specific variable.
    keys.push(StalledVarKey::Opaques);
    Some(keys)
}

impl<'db> ObligationStorage<'db> {
    fn register_fresh(&mut self, obligation: PredicateObligation<'db>) {
        self.fresh.push(obligation);
    }

    /// Files an obligation that evaluated to `Certainty::Maybe`. If its stall condition is
    /// intact and watchable it is parked in `slab`/`watch` and only ever revisited when one of
    /// its variables is reported changed; otherwise it goes to `unwatchable` for periodic
    /// rechecking.
    fn park(
        &mut self,
        infcx: &InferCtxt<'db>,
        obligation: PredicateObligation<'db>,
        stalled_on: Option<GoalStalledOn<DbInterner<'db>>>,
    ) {
        match stalled_on {
            // Only park an obligation whose stall condition currently holds: `is_still_stalled`
            // is an absolute check ("no tracked variable is resolved or re-rooted"), so a stall
            // set that is already invalidated here would never validate again, while the wake
            // events for its variables may already have been consumed — parking it could leave
            // it asleep forever. Such obligations must stay on the periodic recheck path.
            Some(stalled_on) if is_still_stalled(infcx, &stalled_on) => {
                self.park_verified(obligation, stalled_on)
            }
            stalled_on => self.unwatchable.push((obligation, stalled_on)),
        }
    }

    /// Like [`Self::park`], for callers that have just checked `is_still_stalled` themselves.
    fn park_verified(
        &mut self,
        obligation: PredicateObligation<'db>,
        stalled_on: GoalStalledOn<DbInterner<'db>>,
    ) {
        let Some(keys) = stalled_var_keys(&stalled_on) else {
            self.unwatchable.push((obligation, Some(stalled_on)));
            return;
        };
        self.maybe_compact();
        let slot = self.slab.len();
        for &key in &keys {
            self.watch.entry(key).or_default().push(slot);
        }
        self.slab.push(Some(Slot { obligation, stalled_on, keys }));
        self.live += 1;
    }

    /// Moves every obligation whose slab slot is watched under `key` into `out`, leaving
    /// tombstones behind.
    fn wake(&mut self, key: StalledVarKey, out: &mut PendingObligations<'db>) {
        let Some(slots) = self.watch.remove(&key) else { return };
        for idx in slots {
            if let Some(slot) = self.slab[idx].take() {
                self.live -= 1;
                out.push((slot.obligation, Some(slot.stalled_on)));
            }
        }
    }

    /// Rebuilds `slab`/`watch` without tombstones once they outnumber live entries, so storage
    /// stays proportional to what is actually parked instead of growing for the lifetime of
    /// the context.
    fn maybe_compact(&mut self) {
        if self.slab.len() < 64 || 2 * self.live > self.slab.len() {
            return;
        }
        self.slab.retain(Option::is_some);
        self.watch.clear();
        for (idx, slot) in self.slab.iter().flatten().enumerate() {
            for &key in &slot.keys {
                self.watch.entry(key).or_default().push(idx);
            }
        }
    }

    /// Removes and returns every pending obligation matching `pred`, across all three buckets.
    fn extract_pending_if(
        &mut self,
        mut pred: impl FnMut(&PredicateObligation<'db>) -> bool,
    ) -> PredicateObligations<'db> {
        let mut result: PredicateObligations<'db> =
            self.fresh.extract_if(.., |o| pred(o)).collect();
        result.extend(self.unwatchable.extract_if(.., |(o, _)| pred(o)).map(|(o, _)| o));
        for slot in self.slab.iter_mut() {
            if let Some(taken) = slot.take_if(|slot| pred(&slot.obligation)) {
                self.live -= 1;
                result.push(taken.obligation);
            }
        }
        result
    }

    fn iter_pending(&self) -> impl Iterator<Item = &PredicateObligation<'db>> {
        self.fresh
            .iter()
            .chain(self.unwatchable.iter().map(|(o, _)| o))
            .chain(self.slab.iter().flatten().map(|slot| &slot.obligation))
    }

    fn clone_pending(&self) -> PredicateObligations<'db> {
        let mut obligations: PredicateObligations<'db> = self.iter_pending().cloned().collect();
        obligations.extend(self.overflowed.iter().cloned());
        obligations
    }

    fn drain_pending(&mut self) -> impl Iterator<Item = PredicateObligation<'db>> {
        self.watch.clear();
        self.live = 0;
        self.fresh
            .drain(..)
            .chain(self.unwatchable.drain(..).map(|(o, _)| o))
            .chain(self.slab.drain(..).flatten().map(|slot| slot.obligation))
    }

    /// Debug-builds-only safety net for the event-driven wake machinery. Called after a
    /// round's wake-ups have been applied: every obligation still parked must have an intact
    /// stall condition, because anything that could invalidate one records a changed-variable
    /// event, and consuming that event wakes the obligation. An entry failing this check has
    /// missed a wake-up and could keep a stale ambiguity (i.e. wrong inference results)
    /// forever. Also verifies the `live` counter, which the compaction heuristic relies on.
    fn assert_no_missed_wakes(&self, infcx: &InferCtxt<'db>) {
        if cfg!(debug_assertions) {
            stdx::never!(
                self.live != self.slab.iter().flatten().count(),
                "`live` out of sync with the slab"
            );
            for slot in self.slab.iter().flatten() {
                stdx::never!(
                    !is_still_stalled(infcx, &slot.stalled_on),
                    "parked obligation missed a wake-up; its stall condition no longer holds"
                );
            }
        }
    }

    fn on_fulfillment_overflow(&mut self, infcx: &InferCtxt<'db>) {
        // Overflow is rare enough that unconditionally re-evaluating the parked obligations
        // (like the old flat storage did) and dropping their stall state is fine; whatever is
        // retained ends up in `unwatchable` (its `stalled_on` is `take`n below) and gets
        // rechecked from scratch on the next call.
        self.unwatchable.extend(
            self.slab.drain(..).flatten().map(|slot| (slot.obligation, Some(slot.stalled_on))),
        );
        self.watch.clear();
        self.live = 0;
        infcx.probe(|_| {
            // IMPORTANT: we must not use solve any inference variables in the obligations
            // as this is all happening inside of a probe. We use a probe to make sure
            // we get all obligations involved in the overflow. We pretty much check: if
            // we were to do another step of `try_evaluate_obligations`, which goals would
            // change.
            let delegate = <&SolverContext<'db>>::from(infcx);
            let has_changed =
                |o: &PredicateObligation<'db>,
                 stalled_on: Option<GoalStalledOn<DbInterner<'db>>>| {
                    let result =
                        delegate.evaluate_root_goal(o.as_goal(), o.cause.span(), stalled_on);
                    matches!(result, Ok(GoalEvaluation { has_changed: HasChanged::Yes, .. }))
                };

            self.overflowed.extend(self.fresh.extract_if(.., |o| has_changed(o, None)));
            self.overflowed.extend(
                self.unwatchable
                    .extract_if(.., |(o, stalled_on)| has_changed(o, stalled_on.take()))
                    .map(|(o, _)| o),
            );
        })
    }
}

impl<'db> FulfillmentCtxt<'db> {
    pub fn new(infcx: &InferCtxt<'db>) -> FulfillmentCtxt<'db> {
        FulfillmentCtxt {
            obligations: Default::default(),
            usable_in_snapshot: infcx.num_open_snapshots(),
            // Nothing is parked yet, so variable changes before this point are irrelevant.
            changed_vars_cursor: infcx.changed_vars_len(),
            try_evaluate_obligations_scratch: Default::default(),
            changed_vars_scratch: Default::default(),
        }
    }
}

impl<'db> FulfillmentCtxt<'db> {
    #[tracing::instrument(level = "trace", skip(self, infcx))]
    pub(crate) fn register_predicate_obligation(
        &mut self,
        infcx: &InferCtxt<'db>,
        obligation: PredicateObligation<'db>,
    ) {
        assert_eq!(self.usable_in_snapshot, infcx.num_open_snapshots());
        self.obligations.register_fresh(obligation);
    }

    pub(crate) fn register_predicate_obligations(
        &mut self,
        infcx: &InferCtxt<'db>,
        obligations: impl IntoIterator<Item = PredicateObligation<'db>>,
    ) {
        assert_eq!(self.usable_in_snapshot, infcx.num_open_snapshots());
        obligations.into_iter().for_each(|obligation| self.obligations.register_fresh(obligation));
    }

    pub(crate) fn collect_remaining_errors(
        &mut self,
        _infcx: &InferCtxt<'db>,
    ) -> Vec<NextSolverError<'db>> {
        let mut errors: Vec<_> =
            self.obligations.drain_pending().map(NextSolverError::Ambiguity).collect();
        errors.extend(self.obligations.overflowed.drain(..).map(NextSolverError::Overflow));
        errors
    }

    pub(crate) fn try_evaluate_obligations(
        &mut self,
        infcx: &InferCtxt<'db>,
    ) -> Vec<NextSolverError<'db>> {
        assert_eq!(self.usable_in_snapshot, infcx.num_open_snapshots());
        self.try_evaluate_obligations_scratch.clear();
        let mut errors = Vec::new();
        // Whether to re-evaluate the `unwatchable` obligations this round: once per call, and
        // again after every round that made real inference progress (the cadence at which the
        // old code re-evaluated everything still pending). Never merely because the previous
        // round's batch was non-empty — an obligation that stays ambiguous without an
        // attachable stall set would then requeue itself forever.
        let mut recheck_unwatchable = true;
        loop {
            // Gather this round's work: obligations that were newly registered, obligations we
            // can't watch (periodically), and obligations parked on an inference variable that
            // changed since we last looked — whether the change came from our own previous
            // round, from solver entry points between calls, or from a nested fulfillment
            // context sharing this infer context.
            let batch = &mut self.try_evaluate_obligations_scratch;
            batch.extend(self.obligations.fresh.drain(..).map(|o| (o, None)));
            if recheck_unwatchable {
                batch.append(&mut self.obligations.unwatchable);
            }
            if self.obligations.watch.is_empty() {
                // Nothing is parked on any variable (every slab entry has at least the
                // `Opaques` watch key), so skip scanning the changed-variable events and just
                // advance past them.
                self.changed_vars_cursor = infcx.changed_vars_len();
            } else {
                self.changed_vars_scratch.clear();
                infcx.changed_vars_since(
                    &mut self.changed_vars_cursor,
                    &mut self.changed_vars_scratch,
                );
                for &key in &self.changed_vars_scratch {
                    self.obligations.wake(key, batch);
                }
            }
            self.obligations.assert_no_missed_wakes(infcx);
            if batch.is_empty() {
                break;
            }
            recheck_unwatchable = false;

            for (mut obligation, stalled_on) in self.try_evaluate_obligations_scratch.drain(..) {
                // If this goal stalled on a specific set of inference variables before and none
                // of them has really changed (the wake-up was spurious — e.g. caused by a
                // mutation that was rolled back since), evaluating it again is guaranteed to
                // produce the same `Maybe` result (goal evaluation is pure), so park it again
                // without touching the solver.
                let stalled_on = match stalled_on {
                    Some(stalled_on) if is_still_stalled(infcx, &stalled_on) => {
                        self.obligations.park_verified(obligation, stalled_on);
                        continue;
                    }
                    stalled_on => stalled_on,
                };

                if obligation.recursion_depth >= infcx.interner.recursion_limit() {
                    self.obligations.on_fulfillment_overflow(infcx);
                    // Only return true errors that we have accumulated while processing.
                    return errors;
                }

                let goal = obligation.as_goal();
                let delegate = <&SolverContext<'db>>::from(infcx);
                if let Some(certainty) =
                    delegate.compute_goal_fast_path(goal, obligation.cause.span())
                {
                    match certainty {
                        Certainty::Yes => {}
                        Certainty::Maybe { .. } => {
                            self.obligations.park(
                                infcx,
                                obligation,
                                fast_path_stalled_on(infcx, goal),
                            );
                        }
                    }
                    continue;
                }

                let result = delegate.evaluate_root_goal(goal, obligation.cause.span(), stalled_on);
                infcx.inspect_evaluated_obligation(&obligation, &result, || {
                    Some(
                        delegate.evaluate_root_goal_for_proof_tree(goal, obligation.cause.span()).1,
                    )
                });
                let GoalEvaluation { goal: _, certainty, has_changed, stalled_on } = match result {
                    Ok(result) => result,
                    Err(NoSolution) => {
                        errors.push(NextSolverError::TrueError(obligation));
                        continue;
                    }
                };

                if has_changed == HasChanged::Yes {
                    // We increment the recursion depth here to track the number of times
                    // this goal has resulted in inference progress. This doesn't precisely
                    // model the way that we track recursion depth in the old solver due
                    // to the fact that we only process root obligations, but it is a good
                    // approximation and should only result in fulfillment overflow in
                    // pathological cases.
                    obligation.recursion_depth += 1;
                    recheck_unwatchable = true;
                }

                match certainty {
                    Certainty::Yes => {}
                    Certainty::Maybe { .. } => self.obligations.park(infcx, obligation, stalled_on),
                }
            }
        }

        errors
    }

    pub(crate) fn evaluate_obligations_error_on_ambiguity(
        &mut self,
        infcx: &InferCtxt<'db>,
    ) -> Vec<NextSolverError<'db>> {
        let errors = self.try_evaluate_obligations(infcx);
        if !errors.is_empty() {
            return errors;
        }

        self.collect_remaining_errors(infcx)
    }

    pub(crate) fn pending_obligations(&self) -> PredicateObligations<'db> {
        self.obligations.clone_pending()
    }

    pub(crate) fn drain_stalled_obligations_for_coroutines(
        &mut self,
        infcx: &InferCtxt<'db>,
    ) -> PredicateObligations<'db> {
        let stalled_coroutines = match infcx.typing_mode_raw().assert_not_erased() {
            TypingMode::Analysis { defining_opaque_types_and_generators } => {
                defining_opaque_types_and_generators
            }
            TypingMode::Coherence
            | TypingMode::Borrowck { defining_opaque_types: _ }
            | TypingMode::PostBorrowckAnalysis { defined_opaque_types: _ }
            | TypingMode::PostAnalysis => return Default::default(),
        };
        let stalled_coroutines = stalled_coroutines.as_slice();

        if stalled_coroutines.is_empty() {
            return Default::default();
        }

        let is_stalled_on_coroutine = |obl: &PredicateObligation<'db>| {
            infcx.probe(|_| {
                infcx
                    .visit_proof_tree(
                        obl.as_goal(),
                        &mut StalledOnCoroutines {
                            stalled_coroutines,
                            span: obl.cause.span(),
                            cache: Default::default(),
                        },
                    )
                    .is_break()
            })
        };

        self.obligations.extract_pending_if(is_stalled_on_coroutine)
    }
}

/// Mirrors the stalled-goal fast path that `evaluate_goal_raw` runs internally on every
/// evaluation: if none of the tracked variables changed (and the opaque type storage didn't
/// grow) since `stalled_on` was recorded, the goal is guaranteed to still evaluate to the same
/// `Maybe` result. Running this cheap check *before* touching the solver at all is what makes
/// `try_evaluate_obligations` proportional to the obligations actually woken since the last
/// call rather than to the total pending set.
fn is_still_stalled<'db>(
    infcx: &InferCtxt<'db>,
    stalled_on: &GoalStalledOn<DbInterner<'db>>,
) -> bool {
    !infcx.disable_trait_solver_fast_paths()
        && !stalled_on.stalled_vars.iter().any(|&value| infcx.is_changed_arg(value))
        && !stalled_on.sub_roots.iter().any(|&vid| infcx.sub_unification_table_root_var(vid) != vid)
        && !infcx.opaque_types_storage_num_entries().needs_reevaluation(stalled_on.num_opaques)
}

/// Best-effort reconstruction of a [`GoalStalledOn`] for goals resolved by
/// [`crate::next_solver::solver::SolverContext::compute_goal_fast_path`]'s `Certainty::Maybe`
/// result.
///
/// That fast path (unlike the real solver) does not build a `GoalStalledOn`, so without this
/// we would lose all stall-tracking for the obligations it intercepts, forcing them through the
/// fast path again on every single call to `try_evaluate_obligations` for the rest of the body.
/// We over-approximate by collecting every inference variable mentioned in the goal's
/// predicate: extra entries only cause spurious wake-ups and rechecks on unrelated changes,
/// which is wasteful but never unsound, whereas missing one could permanently mask real
/// progress.
fn fast_path_stalled_on<'db>(
    infcx: &InferCtxt<'db>,
    goal: Goal<DbInterner<'db>, Predicate<'db>>,
) -> Option<GoalStalledOn<DbInterner<'db>>> {
    struct CollectInferVars<'a, 'db> {
        infcx: &'a InferCtxt<'db>,
        stalled_vars: Vec<GenericArg<'db>>,
        sub_roots: Vec<TyVid>,
    }

    impl<'db> TypeVisitor<DbInterner<'db>> for CollectInferVars<'_, 'db> {
        type Result = ();

        fn visit_ty(&mut self, ty: Ty<'db>) {
            // Record variables under their current unification root: the interned predicate
            // can mention variables that have since been unioned into another root, and
            // `is_still_stalled` treats a non-root variable as always-changed, so recording
            // such a variable raw would invalidate the stall set on arrival (the obligation
            // would never park).
            match ty.kind() {
                TyKind::Infer(InferTy::TyVar(vid)) => {
                    let root = self.infcx.root_var(vid);
                    self.stalled_vars.push(Ty::new_var(self.infcx.interner, root).into());
                    self.sub_roots.push(self.infcx.sub_unification_table_root_var(vid));
                }
                TyKind::Infer(InferTy::IntVar(vid)) => {
                    let root = self.infcx.root_int_var(vid);
                    self.stalled_vars.push(Ty::new_int_var(self.infcx.interner, root).into());
                }
                TyKind::Infer(InferTy::FloatVar(vid)) => {
                    let root = self.infcx.root_float_var(vid);
                    self.stalled_vars.push(Ty::new_float_var(self.infcx.interner, root).into());
                }
                TyKind::Infer(_) => self.stalled_vars.push(ty.into()),
                _ if ty.has_infer() => ty.super_visit_with(self),
                _ => {}
            }
        }

        fn visit_const(&mut self, ct: Const<'db>) {
            match ct.kind() {
                ConstKind::Infer(InferConst::Var(vid)) => {
                    let root = self.infcx.root_const_var(vid);
                    self.stalled_vars.push(Const::new_var(self.infcx.interner, root).into());
                }
                _ if ct.has_infer() => ct.super_visit_with(self),
                _ => {}
            }
        }
    }

    let mut collector = CollectInferVars { infcx, stalled_vars: Vec::new(), sub_roots: Vec::new() };
    goal.predicate.visit_with(&mut collector);
    if collector.stalled_vars.is_empty() {
        // We couldn't pin down what this goal is blocked on (e.g. the ambiguity came from the
        // param-env rather than the predicate); fall back to always rechecking it.
        return None;
    }
    Some(GoalStalledOn {
        num_opaques: infcx.opaque_types_storage_num_entries().opaque_type_count(),
        stalled_vars: collector.stalled_vars,
        sub_roots: collector.sub_roots,
        stalled_certainty: Certainty::AMBIGUOUS,
    })
}

/// Detect if a goal is stalled on a coroutine that is owned by the current typeck root.
///
/// This function can (erroneously) fail to detect a predicate, i.e. it doesn't need to
/// be complete. However, this will lead to ambiguity errors, so we want to make it
/// accurate.
///
/// This function can be also return false positives, which will lead to poor diagnostics
/// so we want to keep this visitor *precise* too.
pub struct StalledOnCoroutines<'a, 'db> {
    pub stalled_coroutines: &'a [SolverDefId<'db>],
    pub span: Span,
    pub cache: FxHashSet<Ty<'db>>,
}

impl<'db> ProofTreeVisitor<'db> for StalledOnCoroutines<'_, 'db> {
    type Result = ControlFlow<()>;

    fn span(&self) -> Span {
        self.span
    }

    fn visit_goal(&mut self, inspect_goal: &super::inspect::InspectGoal<'_, 'db>) -> Self::Result {
        inspect_goal.goal().predicate.visit_with(self)?;

        if let Some(candidate) = inspect_goal.unique_applicable_candidate() {
            candidate.visit_nested_no_probe(self)
        } else {
            ControlFlow::Continue(())
        }
    }
}

impl<'db> TypeVisitor<DbInterner<'db>> for StalledOnCoroutines<'_, 'db> {
    type Result = ControlFlow<()>;

    fn visit_ty(&mut self, ty: Ty<'db>) -> Self::Result {
        if !self.cache.insert(ty) {
            return ControlFlow::Continue(());
        }

        if let TyKind::Coroutine(def_id, _) = ty.kind()
            && self.stalled_coroutines.contains(&def_id.into())
        {
            ControlFlow::Break(())
        } else if ty.has_coroutines() {
            ty.super_visit_with(self)
        } else {
            ControlFlow::Continue(())
        }
    }
}

#[derive(Debug, Clone)]
pub enum NextSolverError<'db> {
    TrueError(PredicateObligation<'db>),
    Ambiguity(PredicateObligation<'db>),
    Overflow(PredicateObligation<'db>),
}

impl NextSolverError<'_> {
    #[inline]
    pub fn is_true_error(&self) -> bool {
        matches!(self, NextSolverError::TrueError(_))
    }
}
