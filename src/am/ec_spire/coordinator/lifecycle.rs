use crate::storage::relation_guard::HeapRelationGuard;

pub(super) struct SpireRelationLockGuard {
    relid: pg_sys::Oid,
    lockmode: pg_sys::LOCKMODE,
}

impl Drop for SpireRelationLockGuard {
    fn drop(&mut self) {
        // SAFETY: relid/lockmode were captured when the lock was acquired, and
        // Drop releases by OID without dereferencing a Relation pointer.
        unsafe { pg_sys::UnlockRelationOid(self.relid, self.lockmode) };
    }
}

pub(super) unsafe fn lock_publish_relation(
    index_relation: pg_sys::Relation,
) -> SpireRelationLockGuard {
    // Callers hold an open Relation for the guard lifetime. Capture the relid
    // before locking and unlock by relid so Drop never dereferences the pointer.
    // SAFETY: index_relation is live for the publish lock acquisition; only the
    // stable relation OID is copied out before locking.
    let relid = unsafe { (*index_relation).rd_id };
    // SAFETY: relid identifies the open index relation and the lock mode is the
    // fixed publish lock mode paired with SpireRelationLockGuard::drop.
    unsafe { pg_sys::LockRelationOid(relid, SPIRE_PUBLISH_LOCK_MODE) };
    SpireRelationLockGuard {
        relid,
        lockmode: SPIRE_PUBLISH_LOCK_MODE,
    }
}

unsafe fn open_spire_heap_relation_for_index(
    index_relation: pg_sys::Relation,
) -> Result<HeapRelationGuard, String> {
    // SAFETY: index_relation is live while resolving its owning heap relation.
    let index_oid = unsafe { (*index_relation).rd_id };
    // SAFETY: IndexGetRelation reads catalog metadata for the copied index OID
    // and does not retain the Relation pointer.
    let heap_oid = unsafe { pg_sys::IndexGetRelation(index_oid, false) };
    if heap_oid == pg_sys::InvalidOid {
        return Err("ec_spire maintenance could not resolve heap relation".to_owned());
    }
    HeapRelationGuard::try_access_share(heap_oid)
        .ok_or_else(|| "ec_spire maintenance failed to open heap relation".to_owned())
}

unsafe fn active_spire_maintenance_snapshot() -> Result<pg_sys::Snapshot, String> {
    // SAFETY: GetActiveSnapshot returns PostgreSQL backend-local snapshot state;
    // the caller checks for null before using it.
    let snapshot = unsafe { pg_sys::GetActiveSnapshot() };
    if snapshot.is_null() {
        return Err("ec_spire maintenance requires an active heap snapshot".to_owned());
    }
    Ok(snapshot)
}

pub(crate) fn register_gucs() {
    options::register_gucs();
}
