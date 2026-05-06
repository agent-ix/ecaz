pub(super) struct SpireRelationLockGuard {
    relid: pg_sys::Oid,
    lockmode: pg_sys::LOCKMODE,
}

impl Drop for SpireRelationLockGuard {
    fn drop(&mut self) {
        unsafe { pg_sys::UnlockRelationOid(self.relid, self.lockmode) };
    }
}

pub(super) unsafe fn lock_publish_relation(
    index_relation: pg_sys::Relation,
) -> SpireRelationLockGuard {
    // Callers hold an open Relation for the guard lifetime. Capture the relid
    // before locking and unlock by relid so Drop never dereferences the pointer.
    let relid = unsafe { (*index_relation).rd_id };
    unsafe { pg_sys::LockRelationOid(relid, SPIRE_PUBLISH_LOCK_MODE) };
    SpireRelationLockGuard {
        relid,
        lockmode: SPIRE_PUBLISH_LOCK_MODE,
    }
}

struct SpireHeapRelationGuard {
    relation: pg_sys::Relation,
}

impl SpireHeapRelationGuard {
    unsafe fn open_for_index(index_relation: pg_sys::Relation) -> Result<Self, String> {
        let index_oid = unsafe { (*index_relation).rd_id };
        let heap_oid = unsafe { pg_sys::IndexGetRelation(index_oid, false) };
        if heap_oid == pg_sys::InvalidOid {
            return Err("ec_spire maintenance could not resolve heap relation".to_owned());
        }
        let relation =
            unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        if relation.is_null() {
            return Err("ec_spire maintenance failed to open heap relation".to_owned());
        }
        Ok(Self { relation })
    }

    fn relation(&self) -> pg_sys::Relation {
        self.relation
    }
}

impl Drop for SpireHeapRelationGuard {
    fn drop(&mut self) {
        unsafe { pg_sys::table_close(self.relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    }
}

struct SpireHeapSlotGuard {
    slot: *mut pg_sys::TupleTableSlot,
}

impl SpireHeapSlotGuard {
    unsafe fn new(heap_relation: pg_sys::Relation) -> Result<Self, String> {
        let slot = unsafe {
            pg_sys::MakeSingleTupleTableSlot(
                (*heap_relation).rd_att,
                pg_sys::table_slot_callbacks(heap_relation),
            )
        };
        if slot.is_null() {
            return Err("ec_spire maintenance failed to allocate a heap tuple slot".to_owned());
        }
        Ok(Self { slot })
    }

    fn as_ptr(&self) -> *mut pg_sys::TupleTableSlot {
        self.slot
    }
}

impl Drop for SpireHeapSlotGuard {
    fn drop(&mut self) {
        unsafe { pg_sys::ExecDropSingleTupleTableSlot(self.slot) };
    }
}

unsafe fn active_spire_maintenance_snapshot() -> Result<pg_sys::Snapshot, String> {
    let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if snapshot.is_null() {
        return Err("ec_spire maintenance requires an active heap snapshot".to_owned());
    }
    Ok(snapshot)
}

pub(crate) fn register_gucs() {
    options::register_gucs();
}

