use pgrx::{pg_guard, pg_sys, AllocatedByRust, PgBox};

use super::{build, insert, options, scan, vacuum};

fn build_symphony_routine() -> PgBox<pg_sys::IndexAmRoutine, AllocatedByRust> {
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

    amroutine.ambuild = Some(build::symphony_ambuild);
    amroutine.ambuildempty = Some(build::symphony_ambuildempty);
    amroutine.aminsert = Some(insert::symphony_aminsert);
    amroutine.aminsertcleanup = None;
    amroutine.ambulkdelete = Some(vacuum::symphony_ambulkdelete);
    amroutine.amvacuumcleanup = Some(vacuum::symphony_amvacuumcleanup);
    amroutine.amcanreturn = None;
    amroutine.amcostestimate = None;
    #[cfg(feature = "pg18")]
    {
        amroutine.amgettreeheight = None;
    }
    amroutine.amoptions = Some(options::symphony_amoptions);
    amroutine.amproperty = None;
    amroutine.ambuildphasename = None;
    amroutine.amvalidate = Some(symphony_amvalidate);
    amroutine.amadjustmembers = None;
    amroutine.ambeginscan = Some(scan::symphony_ambeginscan);
    amroutine.amrescan = Some(scan::symphony_amrescan);
    amroutine.amgettuple = Some(scan::symphony_amgettuple);
    amroutine.amgetbitmap = None;
    amroutine.amendscan = Some(scan::symphony_amendscan);
    amroutine.ammarkpos = None;
    amroutine.amrestrpos = None;
    amroutine.amestimateparallelscan = None;
    amroutine.aminitparallelscan = None;
    amroutine.amparallelrescan = None;
    #[cfg(feature = "pg18")]
    {
        amroutine.amtranslatestrategy = None;
        amroutine.amtranslatecmptype = None;
    }

    amroutine
}

unsafe extern "C-unwind" fn symphony_amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    unsafe { pgrx::pgrx_extern_c_guard(|| true) }
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn symphony_handler(
    _fcinfo: pg_sys::FunctionCallInfo,
) -> pg_sys::Datum {
    unsafe { pgrx::pgrx_extern_c_guard(|| pg_sys::Datum::from(build_symphony_routine().into_pg())) }
}

#[no_mangle]
pub extern "C-unwind" fn pg_finfo_symphony_handler() -> *const pg_sys::Pg_finfo_record {
    static API_V1: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
    &API_V1
}
