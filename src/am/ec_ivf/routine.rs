use pgrx::{pg_guard, pg_sys, AllocatedByRust, PgBox};

use super::{build, insert, options, scan, vacuum};

fn build_ec_ivf_routine() -> PgBox<pg_sys::IndexAmRoutine, AllocatedByRust> {
    // SAFETY: `IndexAmRoutine` is a PostgreSQL Node type and must be allocated
    // with the corresponding node tag.
    let mut amroutine =
        unsafe { PgBox::<pg_sys::IndexAmRoutine>::alloc_node(pg_sys::NodeTag::T_IndexAmRoutine) };

    amroutine.amstrategies = 1;
    amroutine.amsupport = 1;
    amroutine.amoptsprocnum = 0;

    amroutine.amcanorder = false;
    amroutine.amcanorderbyop = true;
    #[cfg(feature = "pg18")]
    {
        amroutine.amconsistentordering = true;
    }
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

    amroutine.ambuild = Some(build::ec_ivf_ambuild);
    amroutine.ambuildempty = Some(build::ec_ivf_ambuildempty);
    amroutine.aminsert = Some(insert::ec_ivf_aminsert);
    amroutine.aminsertcleanup = None;
    amroutine.ambulkdelete = Some(vacuum::ec_ivf_ambulkdelete);
    amroutine.amvacuumcleanup = Some(vacuum::ec_ivf_amvacuumcleanup);
    amroutine.amcanreturn = None;
    amroutine.amcostestimate = Some(ec_ivf_amcostestimate);
    amroutine.amoptions = Some(options::ec_ivf_amoptions);
    amroutine.amproperty = None;
    amroutine.ambuildphasename = None;
    amroutine.amvalidate = Some(ec_ivf_amvalidate);
    amroutine.amadjustmembers = None;
    amroutine.ambeginscan = Some(scan::ec_ivf_ambeginscan);
    amroutine.amrescan = Some(scan::ec_ivf_amrescan);
    amroutine.amgettuple = Some(scan::ec_ivf_amgettuple);
    amroutine.amgetbitmap = None;
    amroutine.amendscan = Some(scan::ec_ivf_amendscan);
    amroutine.ammarkpos = None;
    amroutine.amrestrpos = None;

    amroutine
}

unsafe extern "C-unwind" fn ec_ivf_amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    unsafe { pgrx::pgrx_extern_c_guard(|| true) }
}

unsafe extern "C-unwind" fn ec_ivf_amcostestimate(
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
            *index_startup_cost = f64::MAX;
            *index_total_cost = f64::MAX;
            *index_selectivity = 1.0;
            *index_correlation = 0.0;
            *index_pages = 0.0;
        })
    }
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn ec_ivf_handler(_fcinfo: pg_sys::FunctionCallInfo) -> pg_sys::Datum {
    unsafe { pgrx::pgrx_extern_c_guard(|| pg_sys::Datum::from(build_ec_ivf_routine().into_pg())) }
}

#[no_mangle]
pub extern "C-unwind" fn pg_finfo_ec_ivf_handler() -> *const pg_sys::Pg_finfo_record {
    static API_V1: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
    &API_V1
}
