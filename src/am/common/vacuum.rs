use pgrx::{pg_sys, AllocatedByRust, PgBox};

pub(crate) type IndexVacuumInfoBox = PgBox<pg_sys::IndexVacuumInfo, AllocatedByRust>;

pub(crate) struct IndexBulkDeleteResultAllocation {
    stats: *mut pg_sys::IndexBulkDeleteResult,
}

impl From<IndexBulkDeleteResultAllocation> for *mut pg_sys::IndexBulkDeleteResult {
    fn from(allocation: IndexBulkDeleteResultAllocation) -> Self {
        allocation.stats
    }
}

pub(crate) fn alloc_index_bulk_delete_result() -> IndexBulkDeleteResultAllocation {
    // SAFETY: PostgreSQL vacuum callbacks expect this stats struct in the
    // current memory context, zero-initialized, and returned to PostgreSQL.
    let stats = unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() };
    IndexBulkDeleteResultAllocation { stats }
}

pub(crate) fn alloc_index_vacuum_info() -> IndexVacuumInfoBox {
    // SAFETY: debug/test callers initialize the fields they hand to AM vacuum
    // callbacks before passing the struct across the PostgreSQL boundary.
    unsafe { PgBox::<pg_sys::IndexVacuumInfo>::alloc0() }
}
