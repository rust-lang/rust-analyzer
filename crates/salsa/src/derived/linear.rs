//! A derived storage that requires its key to be a monotonically increasing index.
// This is a copy paste from [`super::derived`], we could dedupe things with generics but I'm too
// lazy for that.
use crate::debug::TableEntry;
use crate::derived::slot::Slot;
use crate::derived::AlwaysMemoizeValue;
use crate::derived::MemoizationPolicy;
use crate::durability::Durability;
use crate::lru::Lru;
use crate::plumbing::DerivedQueryStorageOps;
use crate::plumbing::LruQueryStorageOps;
use crate::plumbing::QueryFunction;
use crate::plumbing::QueryStorageMassOps;
use crate::plumbing::QueryStorageOps;
use crate::runtime::StampedValue;
use crate::Runtime;
use crate::{Database, DatabaseKeyIndex, QueryDb, Revision};
use parking_lot::RwLock;
use std::marker::PhantomData;
use triomphe::Arc;

pub trait LinearIndex {
    fn index(&self) -> usize {
        self.as_u32() as usize
    }
    fn as_u32(&self) -> u32;
    fn from_u32(u32: u32) -> Self;
}

impl<T: crate::InternKey> LinearIndex for T {
    fn as_u32(&self) -> u32 {
        self.as_intern_id().as_u32()
    }

    fn from_u32(u32: u32) -> Self {
        Self::from_intern_id(crate::InternId::from(u32))
    }
}

/// Memoized queries store the result plus a list of the other queries
/// that they invoked. This means we can avoid recomputing them when
/// none of those inputs have changed.
pub type MemoizedLinearStorage<Q> = LinearStorage<Q, AlwaysMemoizeValue>;

/// Handles storage where the value is 'derived' by executing a
/// function (in contrast to "inputs").
pub struct LinearStorage<Q, MP>
where
    Q: QueryFunction,
    MP: MemoizationPolicy<Q>,
{
    group_index: u16,
    lru_list: Lru<Slot<Q, MP>>,
    slots: RwLock<Vec<Arc<Slot<Q, MP>>>>,
    policy: PhantomData<MP>,
}

impl<Q, MP> std::panic::RefUnwindSafe for LinearStorage<Q, MP>
where
    Q: QueryFunction,
    MP: MemoizationPolicy<Q>,
    Q::Key: std::panic::RefUnwindSafe,
    Q::Value: std::panic::RefUnwindSafe,
{
}

impl<Q, MP> LinearStorage<Q, MP>
where
    Q: QueryFunction,
    Q::Key: LinearIndex,
    MP: MemoizationPolicy<Q>,
{
    fn slot(&self, key: &Q::Key) -> Arc<Slot<Q, MP>> {
        let key_index = key.index();
        if let Some(v) = self.slots.read().get(key_index) {
            return v.clone();
        }

        let mut write = self.slots.write();
        match write.get_mut(key_index) {
            Some(entry) => entry.clone(),
            None => {
                write.resize_with(key_index, || Arc::new(Slot::new()));
                let slot = Arc::new(Slot::new());
                write.push(slot.clone());
                debug_assert!(write.len() == key_index + 1);
                slot
            }
        }
    }
}

impl<Q, MP> QueryStorageOps<Q> for LinearStorage<Q, MP>
where
    Q: QueryFunction,
    Q::Key: LinearIndex,
    MP: MemoizationPolicy<Q>,
{
    const CYCLE_STRATEGY: crate::plumbing::CycleRecoveryStrategy = Q::CYCLE_STRATEGY;

    fn new(group_index: u16) -> Self {
        LinearStorage {
            group_index,
            slots: RwLock::new(Vec::default()),
            lru_list: Default::default(),
            policy: PhantomData,
        }
    }

    fn fmt_index(
        &self,
        _db: &<Q as QueryDb<'_>>::DynDb,
        index: u32,
        fmt: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(
            fmt,
            "{}::{}({:?})",
            std::any::type_name::<Q>(),
            Q::QUERY_NAME,
            Q::Key::from_u32(index)
        )
    }

    fn maybe_changed_after(
        &self,
        db: &<Q as QueryDb<'_>>::DynDb,
        key_index: u32,
        revision: Revision,
    ) -> bool {
        debug_assert!(revision < db.salsa_runtime().current_revision());
        let slot = {
            let read = self.slots.read();
            let Some(slot) = read.get(key_index as usize) else {
                return false;
            };
            slot.clone()
        };
        slot.maybe_changed_after(
            db,
            revision,
            &Q::Key::from_u32(key_index),
            DatabaseKeyIndex {
                group_index: self.group_index,
                query_index: Q::QUERY_INDEX,
                key_index,
            },
        )
    }

    fn fetch(&self, db: &<Q as QueryDb<'_>>::DynDb, key: &Q::Key) -> Q::Value {
        db.unwind_if_cancelled();

        let slot = self.slot(key);
        let StampedValue { value, durability, changed_at } = slot.read(
            db,
            key,
            DatabaseKeyIndex {
                group_index: self.group_index,
                query_index: Q::QUERY_INDEX,
                key_index: key.as_u32(),
            },
        );

        if let Some(evicted) = self.lru_list.record_use(&slot) {
            evicted.evict();
        }

        db.salsa_runtime().report_query_read_and_unwind_if_cycle_resulted(
            DatabaseKeyIndex {
                group_index: self.group_index,
                query_index: Q::QUERY_INDEX,
                key_index: key.as_u32(),
            },
            durability,
            changed_at,
        );

        value
    }

    fn durability(&self, db: &<Q as QueryDb<'_>>::DynDb, key: &Q::Key) -> Durability {
        self.slot(key).durability(db)
    }

    fn entries<C>(&self, _db: &<Q as QueryDb<'_>>::DynDb) -> C
    where
        C: std::iter::FromIterator<TableEntry<Q::Key, Q::Value>>,
    {
        let slot_map = self.slots.read();
        slot_map
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| slot.as_table_entry(&Q::Key::from_u32(idx as u32)))
            .collect()
    }
}

impl<Q, MP> QueryStorageMassOps for LinearStorage<Q, MP>
where
    Q: QueryFunction,
    MP: MemoizationPolicy<Q>,
{
    fn purge(&self) {
        self.lru_list.purge();
        *self.slots.write() = Default::default();
    }
}

impl<Q, MP> LruQueryStorageOps for LinearStorage<Q, MP>
where
    Q: QueryFunction,
    MP: MemoizationPolicy<Q>,
{
    fn set_lru_capacity(&self, new_capacity: usize) {
        self.lru_list.set_lru_capacity(new_capacity);
    }
}

impl<Q, MP> DerivedQueryStorageOps<Q> for LinearStorage<Q, MP>
where
    Q: QueryFunction,
    Q::Key: LinearIndex,
    MP: MemoizationPolicy<Q>,
{
    fn invalidate(&self, runtime: &mut Runtime, key: &Q::Key) {
        runtime.with_incremented_revision(|new_revision| {
            let map_read = self.slots.read();

            if let Some(slot) = map_read.get(key.index()) {
                if let Some(durability) = slot.invalidate(new_revision) {
                    return Some(durability);
                }
            }

            None
        })
    }
}
