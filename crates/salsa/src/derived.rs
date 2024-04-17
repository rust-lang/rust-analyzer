use crate::debug::TableEntry;
use crate::durability::Durability;
use crate::hash::FxIndexMap;
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

mod slot;
use slot::Slot;

pub(crate) mod linear;

/// Memoized queries store the result plus a list of the other queries
/// that they invoked. This means we can avoid recomputing them when
/// none of those inputs have changed.
pub type MemoizedStorage<Q> = DerivedStorage<Q, AlwaysMemoizeValue>;

/// "Dependency" queries just track their dependencies and not the
/// actual value (which they produce on demand). This lessens the
/// storage requirements.
pub type DependencyStorage<Q> = DerivedStorage<Q, NeverMemoizeValue>;

/// Handles storage where the value is 'derived' by executing a
/// function (in contrast to "inputs").
pub struct DerivedStorage<Q, MP>
where
    Q: QueryFunction,
    MP: MemoizationPolicy<Q>,
{
    group_index: u16,
    lru_list: Lru<Slot<Q, MP>>,
    slot_map: RwLock<FxIndexMap<Q::Key, Arc<Slot<Q, MP>>>>,
    policy: PhantomData<MP>,
}

impl<Q, MP> std::panic::RefUnwindSafe for DerivedStorage<Q, MP>
where
    Q: QueryFunction,
    MP: MemoizationPolicy<Q>,
    Q::Key: std::panic::RefUnwindSafe,
    Q::Value: std::panic::RefUnwindSafe,
{
}

pub trait MemoizationPolicy<Q>: Send + Sync
where
    Q: QueryFunction,
{
    fn should_memoize_value(key: &Q::Key) -> bool;

    fn memoized_value_eq(old_value: &Q::Value, new_value: &Q::Value) -> bool;
}

pub enum AlwaysMemoizeValue {}
impl<Q> MemoizationPolicy<Q> for AlwaysMemoizeValue
where
    Q: QueryFunction,
    Q::Value: Eq,
{
    fn should_memoize_value(_key: &Q::Key) -> bool {
        true
    }

    fn memoized_value_eq(old_value: &Q::Value, new_value: &Q::Value) -> bool {
        old_value == new_value
    }
}

pub enum NeverMemoizeValue {}
impl<Q> MemoizationPolicy<Q> for NeverMemoizeValue
where
    Q: QueryFunction,
{
    fn should_memoize_value(_key: &Q::Key) -> bool {
        false
    }

    fn memoized_value_eq(_old_value: &Q::Value, _new_value: &Q::Value) -> bool {
        panic!("cannot reach since we never memoize")
    }
}

impl<Q, MP> DerivedStorage<Q, MP>
where
    Q: QueryFunction,
    MP: MemoizationPolicy<Q>,
{
    fn slot(&self, key: &Q::Key) -> (u32, Arc<Slot<Q, MP>>) {
        if let Some((idx, _, v)) = self.slot_map.read().get_full(key) {
            return (idx as u32, v.clone());
        }

        let mut write = self.slot_map.write();
        let entry = write.entry(key.clone());
        (entry.index() as u32, entry.or_insert_with(|| Arc::new(Slot::new())).clone())
    }
}

impl<Q, MP> QueryStorageOps<Q> for DerivedStorage<Q, MP>
where
    Q: QueryFunction,
    MP: MemoizationPolicy<Q>,
{
    const CYCLE_STRATEGY: crate::plumbing::CycleRecoveryStrategy = Q::CYCLE_STRATEGY;

    fn new(group_index: u16) -> Self {
        DerivedStorage {
            group_index,
            slot_map: RwLock::new(FxIndexMap::default()),
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
        let slot_map = self.slot_map.read();
        let key = slot_map.get_index(index as usize).unwrap().0;
        write!(fmt, "{}::{}({:?})", std::any::type_name::<Q>(), Q::QUERY_NAME, key)
    }

    fn maybe_changed_after(
        &self,
        db: &<Q as QueryDb<'_>>::DynDb,
        key_index: u32,
        revision: Revision,
    ) -> bool {
        debug_assert!(revision < db.salsa_runtime().current_revision());
        let (key, slot) = {
            let read = self.slot_map.read();
            let Some((key, slot)) = read.get_index(key_index as usize) else {
                return false;
            };
            (key.clone(), slot.clone())
        };
        slot.maybe_changed_after(
            db,
            revision,
            &key,
            DatabaseKeyIndex {
                group_index: self.group_index,
                query_index: Q::QUERY_INDEX,
                key_index,
            },
        )
    }

    fn fetch(&self, db: &<Q as QueryDb<'_>>::DynDb, key: &Q::Key) -> Q::Value {
        db.unwind_if_cancelled();

        let (key_index, slot) = self.slot(key);
        let StampedValue { value, durability, changed_at } = slot.read(
            db,
            key,
            DatabaseKeyIndex {
                group_index: self.group_index,
                query_index: Q::QUERY_INDEX,
                key_index,
            },
        );

        if let Some(evicted) = self.lru_list.record_use(&slot) {
            evicted.evict();
        }

        db.salsa_runtime().report_query_read_and_unwind_if_cycle_resulted(
            DatabaseKeyIndex {
                group_index: self.group_index,
                query_index: Q::QUERY_INDEX,
                key_index,
            },
            durability,
            changed_at,
        );

        value
    }

    fn durability(&self, db: &<Q as QueryDb<'_>>::DynDb, key: &Q::Key) -> Durability {
        self.slot(key).1.durability(db)
    }

    fn entries<C>(&self, _db: &<Q as QueryDb<'_>>::DynDb) -> C
    where
        C: std::iter::FromIterator<TableEntry<Q::Key, Q::Value>>,
    {
        let slot_map = self.slot_map.read();
        slot_map.iter().filter_map(|(key, slot)| slot.as_table_entry(key)).collect()
    }
}

impl<Q, MP> QueryStorageMassOps for DerivedStorage<Q, MP>
where
    Q: QueryFunction,
    MP: MemoizationPolicy<Q>,
{
    fn purge(&self) {
        self.lru_list.purge();
        *self.slot_map.write() = Default::default();
    }
}

impl<Q, MP> LruQueryStorageOps for DerivedStorage<Q, MP>
where
    Q: QueryFunction,
    MP: MemoizationPolicy<Q>,
{
    fn set_lru_capacity(&self, new_capacity: usize) {
        self.lru_list.set_lru_capacity(new_capacity);
    }
}

impl<Q, MP> DerivedQueryStorageOps<Q> for DerivedStorage<Q, MP>
where
    Q: QueryFunction,
    MP: MemoizationPolicy<Q>,
{
    fn invalidate(&self, runtime: &mut Runtime, key: &Q::Key) {
        runtime.with_incremented_revision(|new_revision| {
            let map_read = self.slot_map.read();

            if let Some(slot) = map_read.get(key) {
                if let Some(durability) = slot.invalidate(new_revision) {
                    return Some(durability);
                }
            }

            None
        })
    }
}
