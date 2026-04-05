use pgrx::{pg_guard, pg_sys, AllocatedByRust, PgBox};

use super::{
    build, cost, options, scan, tqhnsw_aminsert, vacuum,
};

fn build_tqhnsw_routine() -> PgBox<pg_sys::IndexAmRoutine, AllocatedByRust> {
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

    amroutine.ambuild = Some(build::tqhnsw_ambuild);
    amroutine.ambuildempty = Some(build::tqhnsw_ambuildempty);
    amroutine.aminsert = Some(tqhnsw_aminsert);
    amroutine.aminsertcleanup = None;
    amroutine.ambulkdelete = Some(vacuum::tqhnsw_ambulkdelete);
    amroutine.amvacuumcleanup = Some(vacuum::tqhnsw_amvacuumcleanup);
    amroutine.amcanreturn = None;
    amroutine.amcostestimate = Some(cost::tqhnsw_amcostestimate);
    amroutine.amoptions = Some(options::tqhnsw_amoptions);
    amroutine.amproperty = None;
    amroutine.ambuildphasename = None;
    amroutine.amvalidate = Some(tqhnsw_amvalidate);
    amroutine.amadjustmembers = None;
    amroutine.ambeginscan = Some(scan::tqhnsw_ambeginscan);
    amroutine.amrescan = Some(scan::tqhnsw_amrescan);
    amroutine.amgettuple = Some(scan::tqhnsw_amgettuple);
    amroutine.amgetbitmap = None;
    amroutine.amendscan = Some(scan::tqhnsw_amendscan);
    amroutine.ammarkpos = None;
    amroutine.amrestrpos = None;
    amroutine.amestimateparallelscan = None;
    amroutine.aminitparallelscan = None;
    amroutine.amparallelrescan = None;

    amroutine
}

unsafe extern "C-unwind" fn tqhnsw_amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    unsafe { pgrx::pgrx_extern_c_guard(|| true) }
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn tqhnsw_handler(_fcinfo: pg_sys::FunctionCallInfo) -> pg_sys::Datum {
    unsafe { pgrx::pgrx_extern_c_guard(|| pg_sys::Datum::from(build_tqhnsw_routine().into_pg())) }
}

#[no_mangle]
pub extern "C-unwind" fn pg_finfo_tqhnsw_handler() -> *const pg_sys::Pg_finfo_record {
    static API_V1: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
    &API_V1
}
