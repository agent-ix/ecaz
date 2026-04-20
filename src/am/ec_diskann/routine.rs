use std::ffi::c_void;

use pgrx::{pg_guard, pg_sys, AllocatedByRust, PgBox};

use super::{ambuild, options};

fn build_ec_diskann_routine() -> PgBox<pg_sys::IndexAmRoutine, AllocatedByRust> {
    // SAFETY: `IndexAmRoutine` is a PostgreSQL Node type and must be allocated
    // with the corresponding node tag.
    let mut amroutine =
        unsafe { PgBox::<pg_sys::IndexAmRoutine>::alloc_node(pg_sys::NodeTag::T_IndexAmRoutine) };

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
    amroutine.amcanbuildparallel = false;
    amroutine.amcaninclude = false;
    amroutine.amusemaintenanceworkmem = true;
    amroutine.amsummarizing = false;
    amroutine.amparallelvacuumoptions = 0;
    amroutine.amkeytype = pg_sys::InvalidOid;

    amroutine.ambuild = Some(ambuild::ec_diskann_ambuild);
    amroutine.ambuildempty = Some(ambuild::ec_diskann_ambuildempty);
    amroutine.aminsert = Some(ec_diskann_aminsert);
    amroutine.aminsertcleanup = None;
    amroutine.ambulkdelete = Some(ec_diskann_ambulkdelete);
    amroutine.amvacuumcleanup = Some(ec_diskann_amvacuumcleanup);
    amroutine.amcanreturn = None;
    amroutine.amcostestimate = Some(ec_diskann_amcostestimate);
    amroutine.amoptions = Some(options::ec_diskann_amoptions);
    amroutine.amproperty = None;
    amroutine.ambuildphasename = None;
    amroutine.amvalidate = Some(ec_diskann_amvalidate);
    amroutine.amadjustmembers = None;
    amroutine.ambeginscan = Some(ec_diskann_ambeginscan);
    amroutine.amrescan = Some(ec_diskann_amrescan);
    amroutine.amgettuple = Some(ec_diskann_amgettuple);
    amroutine.amgetbitmap = None;
    amroutine.amendscan = Some(ec_diskann_amendscan);
    amroutine.ammarkpos = None;
    amroutine.amrestrpos = None;
    amroutine.amestimateparallelscan = None;
    amroutine.aminitparallelscan = None;
    amroutine.amparallelrescan = None;

    amroutine
}

unsafe extern "C-unwind" fn ec_diskann_aminsert(
    _index_relation: pg_sys::Relation,
    _values: *mut pg_sys::Datum,
    _isnull: *mut bool,
    _heap_tid: pg_sys::ItemPointer,
    _heap_relation: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    _index_info: *mut pg_sys::IndexInfo,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("ec_diskann aminsert is not yet implemented (task 17 phase 4)");
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_ambulkdelete(
    _info: *mut pg_sys::IndexVacuumInfo,
    _stats: *mut pg_sys::IndexBulkDeleteResult,
    _callback: pg_sys::IndexBulkDeleteCallback,
    _callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("ec_diskann ambulkdelete is not yet implemented (task 17 phase 5)");
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_amvacuumcleanup(
    _info: *mut pg_sys::IndexVacuumInfo,
    _stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("ec_diskann amvacuumcleanup is not yet implemented (task 17 phase 5)");
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_amcostestimate(
    _root: *mut pg_sys::PlannerInfo,
    _path: *mut pg_sys::IndexPath,
    _loop_count: f64,
    index_startup_cost: *mut pg_sys::Cost,
    index_total_cost: *mut pg_sys::Cost,
    index_selectivity: *mut pg_sys::Selectivity,
    index_correlation: *mut f64,
    index_pages: *mut f64,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            // Phase 1A: surface a prohibitive cost so the planner never
            // picks ec_diskann. Phase 6 replaces this with a real cost
            // model once scan (phase 3) lands.
            *index_startup_cost = pg_sys::disable_cost;
            *index_total_cost = pg_sys::disable_cost;
            *index_selectivity = 1.0;
            *index_correlation = 0.0;
            *index_pages = 0.0;
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_ambeginscan(
    _index_relation: pg_sys::Relation,
    _nkeys: std::ffi::c_int,
    _norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("ec_diskann ambeginscan is not yet implemented (task 17 phase 3)");
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_amrescan(
    _scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    _nkeys: std::ffi::c_int,
    _orderbys: pg_sys::ScanKey,
    _norderbys: std::ffi::c_int,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("ec_diskann amrescan is not yet implemented (task 17 phase 3)");
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_amgettuple(
    _scan: pg_sys::IndexScanDesc,
    _direction: pg_sys::ScanDirection::Type,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("ec_diskann amgettuple is not yet implemented (task 17 phase 3)");
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_amendscan(_scan: pg_sys::IndexScanDesc) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("ec_diskann amendscan is not yet implemented (task 17 phase 3)");
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    unsafe { pgrx::pgrx_extern_c_guard(|| true) }
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn ec_diskann_handler(
    _fcinfo: pg_sys::FunctionCallInfo,
) -> pg_sys::Datum {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| pg_sys::Datum::from(build_ec_diskann_routine().into_pg()))
    }
}

#[no_mangle]
pub extern "C-unwind" fn pg_finfo_ec_diskann_handler() -> *const pg_sys::Pg_finfo_record {
    static API_V1: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
    &API_V1
}
