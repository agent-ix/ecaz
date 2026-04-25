use pgrx::{pg_guard, pg_sys, AllocatedByRust, PgBox};

use super::{build, cost, insert, options, parallel, scan, vacuum};

fn build_ec_hnsw_routine() -> PgBox<pg_sys::IndexAmRoutine, AllocatedByRust> {
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
    // The callback surface is wired, but planner-visible parallel scans stay
    // disabled until Task 18 lands the shared coordinator and worker-local
    // traversal semantics.
    amroutine.amcanparallel = false;
    amroutine.amcanbuildparallel = true;
    amroutine.amcaninclude = false;
    amroutine.amusemaintenanceworkmem = true;
    amroutine.amsummarizing = false;
    amroutine.amparallelvacuumoptions = 0;
    amroutine.amkeytype = pg_sys::InvalidOid;

    amroutine.ambuild = Some(build::ec_hnsw_ambuild);
    amroutine.ambuildempty = Some(build::ec_hnsw_ambuildempty);
    amroutine.aminsert = Some(insert::ec_hnsw_aminsert);
    amroutine.aminsertcleanup = None;
    amroutine.ambulkdelete = Some(vacuum::ec_hnsw_ambulkdelete);
    amroutine.amvacuumcleanup = Some(vacuum::ec_hnsw_amvacuumcleanup);
    amroutine.amcanreturn = None;
    amroutine.amcostestimate = Some(cost::ec_hnsw_amcostestimate);
    #[cfg(feature = "pg18")]
    {
        amroutine.amgettreeheight = Some(cost::ec_hnsw_amgettreeheight);
    }
    amroutine.amoptions = Some(options::ec_hnsw_amoptions);
    amroutine.amproperty = None;
    amroutine.ambuildphasename = None;
    amroutine.amvalidate = Some(ec_hnsw_amvalidate);
    amroutine.amadjustmembers = None;
    amroutine.ambeginscan = Some(scan::ec_hnsw_ambeginscan);
    amroutine.amrescan = Some(scan::ec_hnsw_amrescan);
    amroutine.amgettuple = Some(scan::ec_hnsw_amgettuple);
    amroutine.amgetbitmap = None;
    amroutine.amendscan = Some(scan::ec_hnsw_amendscan);
    amroutine.ammarkpos = None;
    amroutine.amrestrpos = None;
    amroutine.amestimateparallelscan = Some(parallel::ec_amestimateparallelscan);
    amroutine.aminitparallelscan = Some(parallel::ec_aminitparallelscan);
    amroutine.amparallelrescan = Some(parallel::ec_amparallelrescan);
    #[cfg(feature = "pg18")]
    {
        amroutine.amtranslatestrategy = Some(cost::ec_hnsw_amtranslatestrategy);
        amroutine.amtranslatecmptype = Some(cost::ec_hnsw_amtranslatecmptype);
    }

    amroutine
}

unsafe extern "C-unwind" fn ec_hnsw_amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    unsafe { pgrx::pgrx_extern_c_guard(|| true) }
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn ec_hnsw_handler(
    _fcinfo: pg_sys::FunctionCallInfo,
) -> pg_sys::Datum {
    unsafe { pgrx::pgrx_extern_c_guard(|| pg_sys::Datum::from(build_ec_hnsw_routine().into_pg())) }
}

#[no_mangle]
pub extern "C-unwind" fn pg_finfo_ec_hnsw_handler() -> *const pg_sys::Pg_finfo_record {
    static API_V1: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
    &API_V1
}
