use pgrx::{pg_sys, AllocatedByRust, PgBox};

pub(crate) type IndexVacuumInfoBox = PgBox<pg_sys::IndexVacuumInfo, AllocatedByRust>;

pub(crate) fn alloc_index_bulk_delete_result() -> *mut pg_sys::IndexBulkDeleteResult {
    // SAFETY: PostgreSQL vacuum callbacks expect this stats struct in the
    // current memory context, zero-initialized, and returned to PostgreSQL.
    unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
}

pub(crate) fn alloc_index_vacuum_info() -> IndexVacuumInfoBox {
    // SAFETY: debug/test callers initialize the fields they hand to AM vacuum
    // callbacks before passing the struct across the PostgreSQL boundary.
    unsafe { PgBox::<pg_sys::IndexVacuumInfo>::alloc0() }
}
