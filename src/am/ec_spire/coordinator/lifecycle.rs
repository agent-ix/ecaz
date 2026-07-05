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

/// Safe handle-based variant of [`lock_publish_relation`].
pub(super) fn lock_publish_relation_handle(
    index_relation: crate::storage::relation::RelationHandle,
) -> SpireRelationLockGuard {
    // SAFETY: `RelationHandle` is a non-null pointer for a live relation.
    unsafe { lock_publish_relation(index_relation.as_ptr()) }
}

/// # Safety
/// Callers hold an open Relation for the guard lifetime. `relid` is captured
/// before locking and unlocked by relid in `SpireRelationLockGuard::drop`,
/// so Drop never dereferences the pointer.
pub(super) unsafe fn lock_publish_relation(
    index_relation: pg_sys::Relation,
) -> SpireRelationLockGuard {
    let index_relation_handle = std::ptr::NonNull::new(index_relation)
        .unwrap_or_else(|| pgrx::error!("ec_spire publish lock needs a valid index relation"));
    let relid = crate::storage::relation::relation_oid_handle(index_relation_handle);
    pg_sys::LockRelationOid(relid, SPIRE_PUBLISH_LOCK_MODE);
    SpireRelationLockGuard {
        relid,
        lockmode: SPIRE_PUBLISH_LOCK_MODE,
    }
}

/// # Safety
/// Caller passes the live SPIRE index relation whose heap relation OID is
/// resolved and opened under AccessShareLock.
unsafe fn open_spire_heap_relation_for_index(
    index_relation: pg_sys::Relation,
) -> Result<HeapRelationGuard, String> {
    let index_relation = std::ptr::NonNull::new(index_relation)
        .ok_or_else(|| "ec_spire maintenance needs a valid index relation".to_owned())?;
    let heap_oid = crate::storage::relation::index_heap_relation_oid_handle(index_relation);
    if heap_oid == pg_sys::InvalidOid {
        return Err("ec_spire maintenance could not resolve heap relation".to_owned());
    }
    HeapRelationGuard::try_access_share(heap_oid)
        .ok_or_else(|| "ec_spire maintenance failed to open heap relation".to_owned())
}

/// # Safety
/// Caller is in a backend with the active snapshot stack populated by
/// PostgreSQL for the current maintenance operation.
unsafe fn active_spire_maintenance_snapshot() -> Result<pg_sys::Snapshot, String> {
    crate::storage::snapshot_guard::active_snapshot()
        .ok_or_else(|| "ec_spire maintenance requires an active heap snapshot".to_owned())
}

pub(crate) fn register_gucs() {
    options::register_gucs();
}
