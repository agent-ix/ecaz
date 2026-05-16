use pgrx::{pg_guard, pg_sys, AllocatedByRust, PgBox};

use super::{build, cost, insert, options, scan, vacuum};

fn build_ec_spire_routine() -> PgBox<pg_sys::IndexAmRoutine, AllocatedByRust> {
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
    amroutine.amcaninclude = true;
    amroutine.amusemaintenanceworkmem = true;
    amroutine.amsummarizing = false;
    amroutine.amparallelvacuumoptions = 0;
    amroutine.amkeytype = pg_sys::InvalidOid;

    amroutine.ambuild = Some(build::ec_spire_ambuild);
    amroutine.ambuildempty = Some(build::ec_spire_ambuildempty);
    amroutine.aminsert = Some(insert::ec_spire_aminsert);
    amroutine.aminsertcleanup = None;
    amroutine.ambulkdelete = Some(vacuum::ec_spire_ambulkdelete);
    amroutine.amvacuumcleanup = Some(vacuum::ec_spire_amvacuumcleanup);
    amroutine.amcanreturn = None;
    amroutine.amcostestimate = Some(cost::ec_spire_amcostestimate);
    #[cfg(feature = "pg18")]
    {
        amroutine.amgettreeheight = Some(cost::ec_spire_amgettreeheight);
    }
    amroutine.amoptions = Some(options::ec_spire_amoptions);
    amroutine.amproperty = None;
    amroutine.ambuildphasename = None;
    amroutine.amvalidate = Some(ec_spire_amvalidate);
    amroutine.amadjustmembers = None;
    amroutine.ambeginscan = Some(scan::ec_spire_ambeginscan);
    amroutine.amrescan = Some(scan::ec_spire_amrescan);
    amroutine.amgettuple = Some(scan::ec_spire_amgettuple);
    amroutine.amgetbitmap = None;
    amroutine.amendscan = Some(scan::ec_spire_amendscan);
    amroutine.ammarkpos = None;
    amroutine.amrestrpos = None;
    amroutine.amestimateparallelscan = None;
    amroutine.aminitparallelscan = None;
    amroutine.amparallelrescan = None;
    #[cfg(feature = "pg18")]
    {
        amroutine.amtranslatestrategy = Some(cost::ec_spire_amtranslatestrategy);
        amroutine.amtranslatecmptype = Some(cost::ec_spire_amtranslatecmptype);
    }

    amroutine
}

unsafe extern "C-unwind" fn ec_spire_amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    true
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn ec_spire_handler(
    _fcinfo: pg_sys::FunctionCallInfo,
) -> pg_sys::Datum {
    // `#[pg_guard]` is the pgrx boundary guard for this PostgreSQL callback.
    pg_sys::Datum::from(build_ec_spire_routine().into_pg())
}

#[no_mangle]
pub extern "C-unwind" fn pg_finfo_ec_spire_handler() -> *const pg_sys::Pg_finfo_record {
    static API_V1: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
    &API_V1
}
