use pgrx::pg_sys;

use super::tqhnsw_noop_vacuum_stats;

pub(super) unsafe extern "C-unwind" fn tqhnsw_ambulkdelete(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    _callback: pg_sys::IndexBulkDeleteCallback,
    _callback_state: *mut std::ffi::c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe { pgrx::pgrx_extern_c_guard(|| tqhnsw_noop_vacuum_stats((*info).index, stats)) }
}

pub(super) unsafe extern "C-unwind" fn tqhnsw_amvacuumcleanup(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe { pgrx::pgrx_extern_c_guard(|| tqhnsw_noop_vacuum_stats((*info).index, stats)) }
}
