use std::ffi::c_void;

use pgrx::pg_sys;

pub(super) unsafe extern "C-unwind" fn ec_spire_ambulkdelete(
    _info: *mut pg_sys::IndexVacuumInfo,
    _stats: *mut pg_sys::IndexBulkDeleteResult,
    _callback: pg_sys::IndexBulkDeleteCallback,
    _callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("ambulkdelete")) }
}

pub(super) unsafe extern "C-unwind" fn ec_spire_amvacuumcleanup(
    _info: *mut pg_sys::IndexVacuumInfo,
    _stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe { pgrx::pgrx_extern_c_guard(|| super::not_implemented("amvacuumcleanup")) }
}
