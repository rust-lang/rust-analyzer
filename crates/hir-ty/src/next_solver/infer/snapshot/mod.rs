//! Snapshotting in the infer ctxt of the next-trait-solver.

use ena::undo_log::UndoLogs;
use rustc_type_ir::UniverseIndex;
use tracing::{debug, instrument};

use super::InferCtxt;
use super::region_constraints::RegionSnapshot;

mod fudge;
pub(crate) mod undo_log;

pub(crate) use undo_log::StalledVarKey;
use undo_log::{Snapshot, UndoLog};

#[must_use = "once you start a snapshot, you should always consume it"]
pub struct CombinedSnapshot {
    pub(super) undo_snapshot: Snapshot,
    region_constraints_snapshot: RegionSnapshot,
    universe: UniverseIndex,
}

struct VariableLengths {
    region_constraints_len: usize,
    type_var_len: usize,
    int_var_len: usize,
    float_var_len: usize,
    const_var_len: usize,
}

impl<'db> InferCtxt<'db> {
    fn variable_lengths(&self) -> VariableLengths {
        let mut inner = self.inner.borrow_mut();
        VariableLengths {
            region_constraints_len: inner.unwrap_region_constraints().num_region_vars(),
            type_var_len: inner.type_variables().num_vars(),
            int_var_len: inner.int_unification_table().len(),
            float_var_len: inner.float_unification_table().len(),
            const_var_len: inner.const_unification_table().len(),
        }
    }

    pub fn in_snapshot(&self) -> bool {
        // The *inherent* method, not the always-`true` trait impl; see `InferCtxtUndoLogs`.
        self.inner.borrow().undo_log.in_snapshot()
    }

    pub fn num_open_snapshots(&self) -> usize {
        UndoLogs::<UndoLog<'db>>::num_open_snapshots(&self.inner.borrow_mut().undo_log)
    }

    /// The current length of the changed-inference-variables log, i.e. a cursor position from
    /// which [`Self::changed_vars_since`] later reports what changed. See
    /// [`undo_log::InferCtxtUndoLogs`].
    pub(crate) fn changed_vars_len(&self) -> usize {
        self.inner.borrow().undo_log.changed_vars_len()
    }

    /// Appends to `out` the variables that (potentially) changed since `cursor` (a position
    /// previously obtained from [`Self::changed_vars_len`]), then advances `cursor` past them.
    /// Reading is non-destructive, so any number of consumers can watch the same infer context
    /// with their own cursors.
    pub(crate) fn changed_vars_since(&self, cursor: &mut usize, out: &mut Vec<StalledVarKey>) {
        let inner = self.inner.borrow();
        out.extend_from_slice(inner.undo_log.changed_vars_since(*cursor));
        *cursor = inner.undo_log.changed_vars_len();
    }

    pub fn start_snapshot(&self) -> CombinedSnapshot {
        debug!("start_snapshot()");

        let mut inner = self.inner.borrow_mut();

        CombinedSnapshot {
            undo_snapshot: inner.undo_log.start_snapshot(),
            region_constraints_snapshot: inner.unwrap_region_constraints().start_snapshot(),
            universe: self.universe(),
        }
    }

    #[instrument(skip(self, snapshot), level = "debug")]
    pub fn rollback_to(&self, snapshot: CombinedSnapshot) {
        let CombinedSnapshot { undo_snapshot, region_constraints_snapshot, universe } = snapshot;

        self.universe.set(universe);

        let mut inner = self.inner.borrow_mut();
        inner.rollback_to(undo_snapshot);
        inner.unwrap_region_constraints().rollback_to(region_constraints_snapshot);
    }

    #[instrument(skip(self, snapshot), level = "debug")]
    pub fn commit_from(&self, snapshot: CombinedSnapshot) {
        let CombinedSnapshot { undo_snapshot, region_constraints_snapshot: _, universe: _ } =
            snapshot;

        self.inner.borrow_mut().commit(undo_snapshot);
    }

    /// Execute `f` and commit the bindings if closure `f` returns `Ok(_)`.
    #[instrument(skip(self, f), level = "debug")]
    pub fn commit_if_ok<T, E, F>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(&CombinedSnapshot) -> Result<T, E>,
    {
        let snapshot = self.start_snapshot();
        let r = f(&snapshot);
        debug!("commit_if_ok() -- r.is_ok() = {}", r.is_ok());
        match r {
            Ok(_) => {
                self.commit_from(snapshot);
            }
            Err(_) => {
                self.rollback_to(snapshot);
            }
        }
        r
    }

    /// Execute `f` then unroll any bindings it creates.
    #[instrument(skip(self, f), level = "debug")]
    pub fn probe<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&CombinedSnapshot) -> R,
    {
        let snapshot = self.start_snapshot();
        let r = f(&snapshot);
        self.rollback_to(snapshot);
        r
    }

    /// Scan the constraints produced since `snapshot` and check whether
    /// we added any region constraints.
    pub fn region_constraints_added_in_snapshot(&self, snapshot: &CombinedSnapshot) -> bool {
        self.inner
            .borrow_mut()
            .unwrap_region_constraints()
            .region_constraints_added_in_snapshot(&snapshot.undo_snapshot)
    }

    pub fn opaque_types_added_in_snapshot(&self, snapshot: &CombinedSnapshot) -> bool {
        self.inner.borrow().undo_log.opaque_types_in_snapshot(&snapshot.undo_snapshot)
    }
}

#[cfg(test)]
mod tests {
    use rustc_type_ir::{TypingMode, inherent::Ty as _};
    use test_fixture::WithFixture;

    use crate::{
        next_solver::{
            DbInterner, Ty,
            infer::{DbInternerInferExt, InferCtxt, StalledVarKey},
        },
        test_db::TestDB,
    };

    fn with_infcx(f: impl FnOnce(&InferCtxt<'_>)) {
        let (db, file_id) = TestDB::with_single_file("fn f() {}");
        crate::attach_db(&db, || {
            let krate = db.module_for_file(file_id.file_id(&db)).krate(&db);
            let interner = DbInterner::new_with(&db, krate);
            let infcx = interner.infer_ctxt().build(TypingMode::non_body_analysis());
            f(&infcx);
        });
    }

    fn new_ty_var(infcx: &InferCtxt<'_>) -> rustc_type_ir::TyVid {
        infcx.next_ty_vid(crate::Span::Dummy)
    }

    fn changed_since(infcx: &InferCtxt<'_>, cursor: &mut usize) -> Vec<StalledVarKey> {
        let mut out = Vec::new();
        infcx.changed_vars_since(cursor, &mut out);
        out
    }

    /// A mutation performed while *no* snapshot is open must still be reported: `ena` gates
    /// its undo-log `push` calls on `in_snapshot()`, which `InferCtxtUndoLogs` deliberately
    /// short-circuits to `true` to keep this log complete. A lost event here would leave a
    /// `FulfillmentCtxt` obligation parked on the variable with a stale `Certainty::Maybe`
    /// forever, i.e. wrong inference results rather than a crash.
    #[test]
    fn zero_snapshot_mutation_is_reported() {
        with_infcx(|infcx| {
            assert_eq!(infcx.num_open_snapshots(), 0);
            let vid = new_ty_var(infcx);
            let mut cursor = infcx.changed_vars_len();
            infcx
                .inner
                .borrow_mut()
                .type_variables()
                .instantiate(vid, Ty::new_bool(infcx.interner));
            assert!(changed_since(infcx, &mut cursor).contains(&StalledVarKey::Ty(vid)));
        });
    }

    /// Unioning two variables must be observable through *both* variables' identities, and
    /// instantiating the surviving root afterwards must be observable through that root: a
    /// watcher keyed on either original variable is woken by the union, re-registers under
    /// the merged root, and is then woken again by the instantiation.
    #[test]
    fn union_then_instantiate_root_is_reported() {
        with_infcx(|infcx| {
            let v1 = new_ty_var(infcx);
            let v2 = new_ty_var(infcx);
            let mut cursor = infcx.changed_vars_len();
            infcx.inner.borrow_mut().type_variables().equate(v1, v2);
            let after_union = changed_since(infcx, &mut cursor);
            assert!(after_union.contains(&StalledVarKey::Ty(v1)));
            assert!(after_union.contains(&StalledVarKey::Ty(v2)));

            let root = infcx.root_var(v1);
            assert_eq!(root, infcx.root_var(v2));
            infcx
                .inner
                .borrow_mut()
                .type_variables()
                .instantiate(root, Ty::new_bool(infcx.interner));
            assert!(changed_since(infcx, &mut cursor).contains(&StalledVarKey::Ty(root)));
        });
    }

    /// Changes made inside a probe are rolled back, but the wake events they produced must
    /// stick around: waking an obligation spuriously is harmless, missing a wake is not, and
    /// a consumer may only get to read the log after the rollback already happened.
    #[test]
    fn rolled_back_mutation_still_reported() {
        with_infcx(|infcx| {
            let vid = new_ty_var(infcx);
            let mut cursor = infcx.changed_vars_len();
            infcx.probe(|_| {
                infcx
                    .inner
                    .borrow_mut()
                    .type_variables()
                    .instantiate(vid, Ty::new_bool(infcx.interner));
            });
            assert!(changed_since(infcx, &mut cursor).contains(&StalledVarKey::Ty(vid)));
        });
    }
}
