use std::ffi::c_void;

use pgrx::{pg_guard, pg_sys, AllocatedByRust, PgBox};

use super::options;

fn build_tqdiskann_routine() -> PgBox<pg_sys::IndexAmRoutine, AllocatedByRust> {
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

    amroutine.ambuild = Some(tqdiskann_ambuild);
    amroutine.ambuildempty = Some(tqdiskann_ambuildempty);
    amroutine.aminsert = Some(tqdiskann_aminsert);
    amroutine.aminsertcleanup = None;
    amroutine.ambulkdelete = Some(tqdiskann_ambulkdelete);
    amroutine.amvacuumcleanup = Some(tqdiskann_amvacuumcleanup);
    amroutine.amcanreturn = None;
    amroutine.amcostestimate = Some(tqdiskann_amcostestimate);
    amroutine.amoptions = Some(options::tqdiskann_amoptions);
    amroutine.amproperty = None;
    amroutine.ambuildphasename = None;
    amroutine.amvalidate = Some(tqdiskann_amvalidate);
    amroutine.amadjustmembers = None;
    amroutine.ambeginscan = Some(tqdiskann_ambeginscan);
    amroutine.amrescan = Some(tqdiskann_amrescan);
    amroutine.amgettuple = Some(tqdiskann_amgettuple);
    amroutine.amgetbitmap = None;
    amroutine.amendscan = Some(tqdiskann_amendscan);
    amroutine.ammarkpos = None;
    amroutine.amrestrpos = None;
    amroutine.amestimateparallelscan = None;
    amroutine.aminitparallelscan = None;
    amroutine.amparallelrescan = None;

    amroutine
}

unsafe extern "C-unwind" fn tqdiskann_ambuild(
    _heap_relation: pg_sys::Relation,
    _index_relation: pg_sys::Relation,
    _index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("tqdiskann ambuild is not yet implemented (task 17 phase 2)");
        })
    }
}

unsafe extern "C-unwind" fn tqdiskann_ambuildempty(_index_relation: pg_sys::Relation) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("tqdiskann ambuildempty is not yet implemented (task 17 phase 2)");
        })
    }
}

unsafe extern "C-unwind" fn tqdiskann_aminsert(
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
            pgrx::error!("tqdiskann aminsert is not yet implemented (task 17 phase 4)");
        })
    }
}

unsafe extern "C-unwind" fn tqdiskann_ambulkdelete(
    _info: *mut pg_sys::IndexVacuumInfo,
    _stats: *mut pg_sys::IndexBulkDeleteResult,
    _callback: pg_sys::IndexBulkDeleteCallback,
    _callback_state: *mut c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("tqdiskann ambulkdelete is not yet implemented (task 17 phase 5)");
        })
    }
}

unsafe extern "C-unwind" fn tqdiskann_amvacuumcleanup(
    _info: *mut pg_sys::IndexVacuumInfo,
    _stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("tqdiskann amvacuumcleanup is not yet implemented (task 17 phase 5)");
        })
    }
}

unsafe extern "C-unwind" fn tqdiskann_amcostestimate(
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
            // picks tqdiskann. Phase 6 replaces this with a real cost
            // model once scan (phase 3) lands.
            *index_startup_cost = pg_sys::disable_cost;
            *index_total_cost = pg_sys::disable_cost;
            *index_selectivity = 1.0;
            *index_correlation = 0.0;
            *index_pages = 0.0;
        })
    }
}

unsafe extern "C-unwind" fn tqdiskann_ambeginscan(
    _index_relation: pg_sys::Relation,
    _nkeys: std::ffi::c_int,
    _norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("tqdiskann ambeginscan is not yet implemented (task 17 phase 3)");
        })
    }
}

unsafe extern "C-unwind" fn tqdiskann_amrescan(
    _scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    _nkeys: std::ffi::c_int,
    _orderbys: pg_sys::ScanKey,
    _norderbys: std::ffi::c_int,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("tqdiskann amrescan is not yet implemented (task 17 phase 3)");
        })
    }
}

unsafe extern "C-unwind" fn tqdiskann_amgettuple(
    _scan: pg_sys::IndexScanDesc,
    _direction: pg_sys::ScanDirection::Type,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("tqdiskann amgettuple is not yet implemented (task 17 phase 3)");
        })
    }
}

unsafe extern "C-unwind" fn tqdiskann_amendscan(_scan: pg_sys::IndexScanDesc) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            pgrx::error!("tqdiskann amendscan is not yet implemented (task 17 phase 3)");
        })
    }
}

unsafe extern "C-unwind" fn tqdiskann_amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    unsafe { pgrx::pgrx_extern_c_guard(|| true) }
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn tqdiskann_handler(
    _fcinfo: pg_sys::FunctionCallInfo,
) -> pg_sys::Datum {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| pg_sys::Datum::from(build_tqdiskann_routine().into_pg()))
    }
}

#[no_mangle]
pub extern "C-unwind" fn pg_finfo_tqdiskann_handler() -> *const pg_sys::Pg_finfo_record {
    static API_V1: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
    &API_V1
}
