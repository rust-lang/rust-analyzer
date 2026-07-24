//! Snapshotting in the infer ctxt of the next-trait-solver.

use ena::snapshot_vec as sv;
use ena::undo_log::{Rollback, UndoLogs};
use ena::unify as ut;
use rustc_type_ir::ConstVid;
use rustc_type_ir::FloatVid;
use rustc_type_ir::IntVid;
use rustc_type_ir::TyVid;
use tracing::debug;

use crate::next_solver::OpaqueTypeKey;
use crate::next_solver::infer::opaque_types::OpaqueHiddenType;
use crate::next_solver::infer::unify_key::ConstVidKey;
use crate::next_solver::infer::unify_key::RegionVidKey;
use crate::next_solver::infer::{InferCtxtInner, region_constraints, type_variable};

/// Identifies something a `FulfillmentCtxt` obligation can be stalled on: a specific inference
/// variable, or the opaque type storage as a whole (mirroring `GoalStalledOn::num_opaques`,
/// which isn't tied to any single variable).
///
/// Used both to index a parked obligation under the variables it is blocked on and, via
/// [`InferCtxtUndoLogs::changed_vars`], to report which of those have potentially changed, so
/// the obligation can be woken without re-scanning everything that's still genuinely stalled.
///
/// Variable keys always refer to the *unification root* at the time the key was created: `ena`
/// applies value changes to root slots, and a union touches both involved roots, so an event
/// stream keyed by root indices is complete as long as watchers re-normalize their keys
/// whenever they are woken (a union wakes the old root's watchers, which then re-register
/// under the new root).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum StalledVarKey {
    /// The `eq_relations` slot of this type variable changed (unified or instantiated).
    Ty(TyVid),
    /// The `sub_unification_table` slot of this type variable changed.
    TySubRoot(TyVid),
    Int(IntVid),
    Float(FloatVid),
    Const(ConstVid),
    /// The opaque type storage changed (an opaque type was registered, replaced, or drained).
    Opaques,
}

/// Best-effort extraction of the variable a given undo entry pertains to. Only entries that
/// represent a *value* changing (`SetElem`) carry a meaningful variable identity: `NewElem`
/// (a fresh, still-unconstrained variable was created) cannot wake anything, because no
/// obligation can have been registered as stalled on a variable that didn't exist yet.
fn changed_var_key(undo: &UndoLog<'_>) -> Option<StalledVarKey> {
    match undo {
        UndoLog::TypeVariables(type_variable::UndoLog::EqRelation(sv::UndoLog::SetElem(
            idx,
            _,
        ))) => Some(StalledVarKey::Ty(TyVid::from_u32(*idx as u32))),
        UndoLog::TypeVariables(type_variable::UndoLog::SubRelation(sv::UndoLog::SetElem(
            idx,
            _,
        ))) => Some(StalledVarKey::TySubRoot(TyVid::from_u32(*idx as u32))),
        UndoLog::IntUnificationTable(sv::UndoLog::SetElem(idx, _)) => {
            Some(StalledVarKey::Int(IntVid::from_u32(*idx as u32)))
        }
        UndoLog::FloatUnificationTable(sv::UndoLog::SetElem(idx, _)) => {
            Some(StalledVarKey::Float(FloatVid::from_u32(*idx as u32)))
        }
        UndoLog::ConstUnificationTable(sv::UndoLog::SetElem(idx, _)) => {
            Some(StalledVarKey::Const(ConstVid::from_u32(*idx as u32)))
        }
        UndoLog::OpaqueTypes(..) | UndoLog::DuplicateOpaqueType => Some(StalledVarKey::Opaques),
        _ => None,
    }
}

pub struct Snapshot {
    pub(crate) undo_len: usize,
}

/// Records the "undo" data for a single operation that affects some form of inference variable.
#[derive(Clone)]
pub(crate) enum UndoLog<'db> {
    DuplicateOpaqueType,
    OpaqueTypes(OpaqueTypeKey<'db>, Option<OpaqueHiddenType<'db>>),
    TypeVariables(type_variable::UndoLog<'db>),
    ConstUnificationTable(sv::UndoLog<ut::Delegate<ConstVidKey<'db>>>),
    IntUnificationTable(sv::UndoLog<ut::Delegate<IntVid>>),
    FloatUnificationTable(sv::UndoLog<ut::Delegate<FloatVid>>),
    RegionConstraintCollector(region_constraints::UndoLog<'db>),
    RegionUnificationTable(sv::UndoLog<ut::Delegate<RegionVidKey<'db>>>),
    PushTypeOutlivesConstraint,
    PushRegionAssumption,
}

macro_rules! impl_from {
    ($($ctor:ident ($ty:ty),)*) => {
        $(
        impl<'db> From<$ty> for UndoLog<'db> {
            fn from(x: $ty) -> Self {
                UndoLog::$ctor(x.into())
            }
        }
        )*
    }
}

// Upcast from a single kind of "undoable action" to the general enum
impl_from! {
    RegionConstraintCollector(region_constraints::UndoLog<'db>),

    TypeVariables(sv::UndoLog<ut::Delegate<type_variable::TyVidEqKey<'db>>>),
    TypeVariables(sv::UndoLog<ut::Delegate<type_variable::TyVidSubKey>>),
    TypeVariables(type_variable::UndoLog<'db>),
    IntUnificationTable(sv::UndoLog<ut::Delegate<IntVid>>),
    FloatUnificationTable(sv::UndoLog<ut::Delegate<FloatVid>>),

    ConstUnificationTable(sv::UndoLog<ut::Delegate<ConstVidKey<'db>>>),

    RegionUnificationTable(sv::UndoLog<ut::Delegate<RegionVidKey<'db>>>),
}

/// The Rollback trait defines how to rollback a particular action.
impl<'db> Rollback<UndoLog<'db>> for InferCtxtInner<'db> {
    fn reverse(&mut self, undo: UndoLog<'db>) {
        match undo {
            UndoLog::DuplicateOpaqueType => self.opaque_type_storage.pop_duplicate_entry(),
            UndoLog::OpaqueTypes(key, idx) => self.opaque_type_storage.remove(key, idx),
            UndoLog::TypeVariables(undo) => self.type_variable_storage.reverse(undo),
            UndoLog::ConstUnificationTable(undo) => self.const_unification_storage.reverse(undo),
            UndoLog::IntUnificationTable(undo) => self.int_unification_storage.reverse(undo),
            UndoLog::FloatUnificationTable(undo) => self.float_unification_storage.reverse(undo),
            UndoLog::RegionConstraintCollector(undo) => {
                self.region_constraint_storage.as_mut().unwrap().reverse(undo)
            }
            UndoLog::RegionUnificationTable(undo) => {
                self.region_constraint_storage.as_mut().unwrap().unification_table.reverse(undo)
            }
            UndoLog::PushTypeOutlivesConstraint => {
                let popped = self.region_obligations.pop();
                assert!(popped.is_some(), "pushed region constraint but could not pop it");
            }
            UndoLog::PushRegionAssumption => {
                let popped = self.region_assumptions.pop();
                assert!(popped.is_some(), "pushed region assumption but could not pop it");
            }
        }
    }
}

/// The combined undo log for all the various unification tables. For each change to the storage
/// for any kind of inference variable, we record an UndoLog entry in the vector here.
#[derive(Clone, Default)]
pub(crate) struct InferCtxtUndoLogs<'db> {
    logs: Vec<UndoLog<'db>>,
    num_open_snapshots: usize,
    /// Variables observed to have (potentially) changed, for waking `FulfillmentCtxt`
    /// obligations that were parked on them.
    ///
    /// Unlike `logs`, this is appended to unconditionally, not just while a snapshot is open,
    /// and it is never truncated by rollback: over-reporting a variable as changed (e.g. one
    /// whose change was later rolled back, or a mere path-compression write) only causes an
    /// obligation to be needlessly re-checked, never a missed wake-up. Consumers each keep a
    /// cursor into this log (see `FulfillmentCtxt`) rather than draining it, so several
    /// consumers can share one infer context without stealing each other's wake signals.
    changed_vars: Vec<StalledVarKey>,
}

/// The UndoLogs trait defines how we undo a particular kind of action (of type T). We can undo any
/// action that is convertible into an UndoLog (per the From impls above).
impl<'db, T> UndoLogs<T> for InferCtxtUndoLogs<'db>
where
    UndoLog<'db>: From<T>,
{
    #[inline]
    fn num_open_snapshots(&self) -> usize {
        self.num_open_snapshots
    }

    /// Deliberately claims a snapshot is always open. `ena`'s mutation methods
    /// (`SnapshotVec::{set, update}` and everything in `unify` built on them) consult this
    /// *before* calling [`Self::push`] and skip the call entirely when it returns false, which
    /// would make `changed_vars` silently miss every mutation performed while no snapshot is
    /// open. Answering `true` routes all mutations through `push`/`extend`, which record the
    /// changed variable unconditionally and store the actual undo entry only when a snapshot
    /// is really open (`num_open_snapshots > 0`), preserving the original rollback behavior.
    #[inline]
    fn in_snapshot(&self) -> bool {
        true
    }

    #[inline]
    fn push(&mut self, undo: T) {
        let undo = undo.into();
        if let Some(key) = changed_var_key(&undo) {
            self.changed_vars.push(key);
        }
        if self.num_open_snapshots > 0 {
            self.logs.push(undo)
        }
    }

    fn clear(&mut self) {
        // Note that `changed_vars` is intentionally left alone: it is append-only for the
        // lifetime of the infer context so that consumer cursors stay valid.
        self.logs.clear();
        self.num_open_snapshots = 0;
    }

    // `extend` is left at its provided default, which forwards to `push` element-wise.
}

impl<'db> InferCtxtInner<'db> {
    pub fn rollback_to(&mut self, snapshot: Snapshot) {
        debug!("rollback_to({})", snapshot.undo_len);
        self.undo_log.assert_open_snapshot(&snapshot);

        while self.undo_log.logs.len() > snapshot.undo_len {
            let undo = self.undo_log.logs.pop().unwrap();
            self.reverse(undo);
        }

        self.type_variable_storage.finalize_rollback();

        if self.undo_log.num_open_snapshots == 1 {
            // After the root snapshot the undo log should be empty.
            assert!(snapshot.undo_len == 0);
            assert!(self.undo_log.logs.is_empty());
        }

        self.undo_log.num_open_snapshots -= 1;
    }

    pub fn commit(&mut self, snapshot: Snapshot) {
        debug!("commit({})", snapshot.undo_len);

        if self.undo_log.num_open_snapshots == 1 {
            // The root snapshot. It's safe to clear the undo log because
            // there's no snapshot further out that we might need to roll back
            // to.
            assert!(snapshot.undo_len == 0);
            self.undo_log.logs.clear();
        }

        self.undo_log.num_open_snapshots -= 1;
    }
}

impl<'db> InferCtxtUndoLogs<'db> {
    /// Whether a snapshot is really open. This inherent method shadows the
    /// [`UndoLogs::in_snapshot`] trait method, which deliberately always answers `true` (see
    /// the trait impl above for why); anything wanting the real state must call this one.
    pub(crate) fn in_snapshot(&self) -> bool {
        self.num_open_snapshots > 0
    }

    pub(crate) fn start_snapshot(&mut self) -> Snapshot {
        self.num_open_snapshots += 1;
        Snapshot { undo_len: self.logs.len() }
    }

    pub(crate) fn region_constraints_in_snapshot(
        &self,
        s: &Snapshot,
    ) -> impl Iterator<Item = &'_ region_constraints::UndoLog<'db>> + Clone {
        self.logs[s.undo_len..].iter().filter_map(|log| match log {
            UndoLog::RegionConstraintCollector(log) => Some(log),
            _ => None,
        })
    }

    pub(crate) fn opaque_types_in_snapshot(&self, s: &Snapshot) -> bool {
        self.logs[s.undo_len..].iter().any(|log| matches!(log, UndoLog::OpaqueTypes(..)))
    }

    fn assert_open_snapshot(&self, snapshot: &Snapshot) {
        // Failures here may indicate a failure to follow a stack discipline.
        assert!(self.logs.len() >= snapshot.undo_len);
        assert!(self.num_open_snapshots > 0);
    }

    /// The current length of the changed-variables log. Positions before this are in the past;
    /// a consumer that has processed everything up to `len` can later ask for
    /// [`Self::changed_vars_since`] that position to see only what changed in between.
    pub(crate) fn changed_vars_len(&self) -> usize {
        self.changed_vars.len()
    }

    /// The variables that (potentially) changed since `cursor`, a position previously obtained
    /// from [`Self::changed_vars_len`]. The log is append-only, so this is stable across
    /// snapshots, rollbacks, and other consumers reading their own cursors.
    pub(crate) fn changed_vars_since(&self, cursor: usize) -> &[StalledVarKey] {
        &self.changed_vars[cursor.min(self.changed_vars.len())..]
    }

    /// Records an opaque-type-storage change that doesn't go through the undo machinery
    /// (draining the storage via `take_opaque_types` shrinks the entry count without pushing
    /// any undo entry).
    pub(crate) fn mark_opaques_changed(&mut self) {
        self.changed_vars.push(StalledVarKey::Opaques);
    }
}

impl<'db> std::ops::Index<usize> for InferCtxtUndoLogs<'db> {
    type Output = UndoLog<'db>;

    fn index(&self, key: usize) -> &Self::Output {
        &self.logs[key]
    }
}

impl<'db> std::ops::IndexMut<usize> for InferCtxtUndoLogs<'db> {
    fn index_mut(&mut self, key: usize) -> &mut Self::Output {
        &mut self.logs[key]
    }
}
