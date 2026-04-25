use std::ffi::c_void;

use pgrx::{pg_sys, PgBox};

use super::page;

pub(super) unsafe extern "C-unwind" fn ec_ivf_ambulkdelete(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    _callback: pg_sys::IndexBulkDeleteCallback,
    _callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if info.is_null() {
                pgrx::error!("ec_ivf ambulkdelete requires vacuum info")
            }

            noop_vacuum_stats((*info).index, stats)
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_amvacuumcleanup(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if info.is_null() {
                pgrx::error!("ec_ivf amvacuumcleanup requires vacuum info")
            }

            noop_vacuum_stats((*info).index, stats)
        })
    }
}

unsafe fn noop_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let stats = if stats.is_null() {
        unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
    } else {
        stats
    };
    let metadata = unsafe { page::read_metadata_page(index_relation) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
        )
    };

    unsafe {
        (*stats).num_pages = block_count;
        (*stats).estimated_count = false;
        (*stats).num_index_tuples = metadata.total_live_tuples as f64;
    }

    stats
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_ec_ivf_vacuum_stats(
    index_oid: pg_sys::Oid,
) -> pg_sys::IndexBulkDeleteResult {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let mut info = PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
    info.index = index_relation;
    let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;

    let stats =
        unsafe { ec_ivf_ambulkdelete(info_ptr, std::ptr::null_mut(), None, std::ptr::null_mut()) };
    let stats = unsafe { ec_ivf_amvacuumcleanup(info_ptr, stats) };
    let result = unsafe { *stats };

    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}
