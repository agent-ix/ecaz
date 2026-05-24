use std::ptr::NonNull;

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

pub(crate) fn copy_index_bulk_delete_result(
    stats: NonNull<pg_sys::IndexBulkDeleteResult>,
) -> pg_sys::IndexBulkDeleteResult {
    // SAFETY: `NonNull` proves the callback returned stats exists for this
    // immediate copy; no PostgreSQL-owned pointer is retained.
    unsafe { *stats.as_ptr() }
}

fn with_index_bulk_delete_result_mut<R>(
    mut stats: NonNull<pg_sys::IndexBulkDeleteResult>,
    f: impl FnOnce(&mut pg_sys::IndexBulkDeleteResult) -> R,
) -> R {
    // SAFETY: `NonNull` proves the vacuum callback stats struct exists for
    // the immediate mutation; callers do not retain the mutable borrow.
    unsafe { f(stats.as_mut()) }
}

pub(crate) fn add_index_bulk_delete_tuples_removed(
    stats: NonNull<pg_sys::IndexBulkDeleteResult>,
    removed_heap_tids: u64,
) {
    with_index_bulk_delete_result_mut(stats, |stats| {
        stats.tuples_removed += removed_heap_tids as f64;
    });
}

pub(crate) fn set_index_bulk_delete_summary(
    stats: NonNull<pg_sys::IndexBulkDeleteResult>,
    num_pages: pg_sys::BlockNumber,
    live_tuples: u64,
) {
    with_index_bulk_delete_result_mut(stats, |stats| {
        stats.num_pages = num_pages;
        stats.estimated_count = false;
        stats.num_index_tuples = live_tuples as f64;
    });
}
