use pgrx::pg_sys;

pub(crate) struct IndexRelationGuard {
    relation: pg_sys::Relation,
    lockmode: pg_sys::LOCKMODE,
}

impl IndexRelationGuard {
    pub(crate) fn open(
        index_oid: pg_sys::Oid,
        lockmode: pg_sys::LOCKMODE,
        caller: &'static str,
    ) -> Self {
        Self::try_open(index_oid, lockmode)
            .unwrap_or_else(|| pgrx::error!("{caller} could not open index relation"))
    }

    pub(crate) fn try_open(index_oid: pg_sys::Oid, lockmode: pg_sys::LOCKMODE) -> Option<Self> {
        // SAFETY: PostgreSQL owns the relation cache entry returned by
        // `index_open`; this guard owns the matching close for `lockmode`.
        let relation = unsafe { pg_sys::index_open(index_oid, lockmode) };
        if relation.is_null() {
            return None;
        }
        Some(Self { relation, lockmode })
    }

    pub(crate) fn access_share(index_oid: pg_sys::Oid, caller: &'static str) -> Self {
        Self::open(
            index_oid,
            pg_sys::AccessShareLock as pg_sys::LOCKMODE,
            caller,
        )
    }

    pub(crate) fn try_access_share(index_oid: pg_sys::Oid) -> Option<Self> {
        Self::try_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE)
    }

    pub(crate) fn as_ptr(&self) -> pg_sys::Relation {
        self.relation
    }
}

impl Drop for IndexRelationGuard {
    fn drop(&mut self) {
        // SAFETY: `relation` was returned by `index_open` in
        // `IndexRelationGuard::try_open`; this guard owns the matching close.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe { pg_sys::index_close(self.relation, self.lockmode) };
    }
}

pub(crate) struct HeapRelationGuard {
    relation: pg_sys::Relation,
    lockmode: pg_sys::LOCKMODE,
}

impl HeapRelationGuard {
    pub(crate) fn try_open(relation_oid: pg_sys::Oid, lockmode: pg_sys::LOCKMODE) -> Option<Self> {
        // SAFETY: PostgreSQL owns the relation cache entry returned by
        // `table_open`; this guard owns the matching close for `lockmode`.
        let relation = unsafe { pg_sys::table_open(relation_oid, lockmode) };
        if relation.is_null() {
            return None;
        }
        Some(Self { relation, lockmode })
    }

    pub(crate) fn try_access_share(relation_oid: pg_sys::Oid) -> Option<Self> {
        Self::try_open(relation_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE)
    }

    pub(crate) fn as_ptr(&self) -> pg_sys::Relation {
        self.relation
    }
}

impl Drop for HeapRelationGuard {
    fn drop(&mut self) {
        // SAFETY: `relation` was returned by `table_open` in
        // `HeapRelationGuard::try_open`; this guard owns the matching close.
        // SAFETY: pgrx ERROR paths must unwind Rust frames so Drop runs;
        // re-audit on pgrx bumps or pg_guard behavior changes.
        unsafe { pg_sys::table_close(self.relation, self.lockmode) };
    }
}
