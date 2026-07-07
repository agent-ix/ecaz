use std::{ffi::c_void, ptr};

use pgrx::{pg_guard, pg_sys};

use crate::am::common::{
    callback::pg_am_callback,
    routine::{alloc_index_am_routine, IndexAmRoutineBox},
    vacuum::{alloc_index_bulk_delete_result, set_index_bulk_delete_summary},
};

use super::{ambuild, cost, options};

fn build_ec_distann_routine() -> IndexAmRoutineBox {
    let mut amroutine = alloc_index_am_routine();

    amroutine.amstrategies = 1;
    amroutine.amsupport = 1;
    amroutine.amoptsprocnum = 0;

    amroutine.amcanorder = false;
    amroutine.amcanorderbyop = true;
    amroutine.amcanbackward = false;
    amroutine.amcanunique = false;
    amroutine.amcanmulticol = false;
    amroutine.amoptionalkey = true;
    amroutine.amsearcharray = false;
    amroutine.amsearchnulls = false;
    amroutine.amstorage = false;
    amroutine.amclusterable = false;
    amroutine.ampredlocks = false;
    amroutine.amcanparallel = false;
    // The FR-077 sharded build (M1) owns build parallelism; the monolithic
    // M0 build is single-process.
    amroutine.amcanbuildparallel = false;
    amroutine.amcaninclude = false;
    amroutine.amusemaintenanceworkmem = true;
    amroutine.amsummarizing = false;
    amroutine.amparallelvacuumoptions = 0;
    amroutine.amkeytype = pg_sys::InvalidOid;

    amroutine.ambuild = Some(ambuild::ec_distann_ambuild);
    amroutine.ambuildempty = Some(ambuild::ec_distann_ambuildempty);
    amroutine.aminsert = Some(ec_distann_aminsert);
    amroutine.aminsertcleanup = None;
    amroutine.ambulkdelete = Some(ec_distann_ambulkdelete);
    amroutine.amvacuumcleanup = Some(ec_distann_amvacuumcleanup);
    amroutine.amcanreturn = None;
    amroutine.amcostestimate = Some(cost::ec_distann_amcostestimate);
    amroutine.amoptions = Some(options::ec_distann_amoptions);
    amroutine.amproperty = None;
    amroutine.ambuildphasename = None;
    amroutine.amvalidate = Some(ec_distann_amvalidate);
    amroutine.amadjustmembers = None;
    amroutine.ambeginscan = Some(ec_distann_ambeginscan);
    amroutine.amrescan = Some(ec_distann_amrescan);
    amroutine.amgettuple = Some(ec_distann_amgettuple);
    amroutine.amgetbitmap = None;
    amroutine.amendscan = Some(ec_distann_amendscan);
    amroutine.ammarkpos = None;
    amroutine.amrestrpos = None;
    amroutine.amestimateparallelscan = None;
    amroutine.aminitparallelscan = None;
    amroutine.amparallelrescan = None;

    amroutine
}

unsafe extern "C-unwind" fn ec_distann_aminsert(
    _index_relation: pg_sys::Relation,
    _values: *mut pg_sys::Datum,
    _isnull: *mut bool,
    _heap_tid: pg_sys::ItemPointer,
    _heap_relation: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    _index_info: *mut pg_sys::IndexInfo,
) -> bool {
    pg_am_callback!({
        pgrx::error!(
            "ec_distann aminsert is not implemented yet: the FR-083 delta-buffer DML slice lands later in Task 162"
        );
    })
}

unsafe extern "C-unwind" fn ec_distann_ambulkdelete(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    _callback: pg_sys::IndexBulkDeleteCallback,
    _callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    pg_am_callback!({
        // No graph-node records exist yet, so there is nothing to delete;
        // the D10 tombstone path arrives with the FR-076 record slice.
        ec_distann_noop_vacuum_stats((*info).index, stats)
            .unwrap_or_else(|e| pgrx::error!("ec_distann ambulkdelete failed: {e}"))
    })
}

unsafe extern "C-unwind" fn ec_distann_amvacuumcleanup(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    pg_am_callback!({
        ec_distann_noop_vacuum_stats((*info).index, stats)
            .unwrap_or_else(|e| pgrx::error!("ec_distann amvacuumcleanup failed: {e}"))
    })
}

unsafe fn ec_distann_noop_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> Result<*mut pg_sys::IndexBulkDeleteResult, String> {
    let stats = if stats.is_null() {
        alloc_index_bulk_delete_result().into()
    } else {
        stats
    };
    let stats_handle =
        ptr::NonNull::new(stats).ok_or_else(|| "ec_distann vacuum stats is null".to_owned())?;
    let index_relation_handle = ptr::NonNull::new(index_relation)
        .ok_or_else(|| "ec_distann vacuum stats needs a valid index relation".to_owned())?;
    let block_count = crate::storage::relation::main_fork_block_count_handle(index_relation_handle);
    // Live tuple accounting starts when FR-076 graph-node records exist.
    set_index_bulk_delete_summary(stats_handle, block_count, 0);

    Ok(stats)
}

unsafe extern "C-unwind" fn ec_distann_ambeginscan(
    _index_relation: pg_sys::Relation,
    _nkeys: std::ffi::c_int,
    _norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    pg_am_callback!({
        pgrx::error!(
            "ec_distann scans are not implemented yet: the FR-081 local hop-round loop lands later in Task 162"
        );
    })
}

unsafe extern "C-unwind" fn ec_distann_amrescan(
    _scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    _nkeys: std::ffi::c_int,
    _orderbys: pg_sys::ScanKey,
    _norderbys: std::ffi::c_int,
) {
    pg_am_callback!({
        pgrx::error!(
            "ec_distann scans are not implemented yet: the FR-081 local hop-round loop lands later in Task 162"
        );
    })
}

unsafe extern "C-unwind" fn ec_distann_amgettuple(
    _scan: pg_sys::IndexScanDesc,
    _direction: pg_sys::ScanDirection::Type,
) -> bool {
    pg_am_callback!({
        pgrx::error!(
            "ec_distann scans are not implemented yet: the FR-081 local hop-round loop lands later in Task 162"
        );
    })
}

unsafe extern "C-unwind" fn ec_distann_amendscan(_scan: pg_sys::IndexScanDesc) {
    pg_am_callback!({})
}

#[pg_guard]
unsafe extern "C-unwind" fn ec_distann_amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    true
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn ec_distann_handler(
    _fcinfo: pg_sys::FunctionCallInfo,
) -> pg_sys::Datum {
    // `#[pg_guard]` is the pgrx boundary guard for this PostgreSQL callback.
    pg_sys::Datum::from(build_ec_distann_routine().into_pg())
}

#[no_mangle]
pub extern "C-unwind" fn pg_finfo_ec_distann_handler() -> *const pg_sys::Pg_finfo_record {
    static API_V1: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
    &API_V1
}
