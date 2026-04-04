//! Access-method scaffolding for the future `tqhnsw` implementation.

use std::ffi::c_void;
use std::mem::{offset_of, size_of};
use std::ptr;

use pgrx::{pg_guard, pg_sys, varlena::set_varsize_4b, AllocatedByRust, PgBox};

pub mod page;
pub mod wal;

const TQHNSW_DEFAULT_M: i32 = 8;
const TQHNSW_MIN_M: i32 = 2;
const TQHNSW_MAX_M: i32 = 100;
const TQHNSW_DEFAULT_EF_CONSTRUCTION: i32 = 64;
const TQHNSW_MIN_EF_CONSTRUCTION: i32 = 10;
const TQHNSW_MAX_EF_CONSTRUCTION: i32 = 1000;
const P_NEW: pg_sys::BlockNumber = u32::MAX;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct TqHnswOptions {
    vl_len_: i32,
    m: i32,
    ef_construction: i32,
}

impl TqHnswOptions {
    const DEFAULT: Self = Self {
        vl_len_: 0,
        m: TQHNSW_DEFAULT_M,
        ef_construction: TQHNSW_DEFAULT_EF_CONSTRUCTION,
    };
}

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

    amroutine.ambuild = Some(tqhnsw_ambuild);
    amroutine.ambuildempty = Some(tqhnsw_ambuildempty);
    amroutine.aminsert = Some(tqhnsw_aminsert);
    amroutine.aminsertcleanup = None;
    amroutine.ambulkdelete = Some(tqhnsw_ambulkdelete);
    amroutine.amvacuumcleanup = Some(tqhnsw_amvacuumcleanup);
    amroutine.amcanreturn = None;
    amroutine.amcostestimate = Some(tqhnsw_amcostestimate);
    amroutine.amoptions = Some(tqhnsw_amoptions);
    amroutine.amproperty = None;
    amroutine.ambuildphasename = None;
    amroutine.amvalidate = Some(tqhnsw_amvalidate);
    amroutine.amadjustmembers = None;
    amroutine.ambeginscan = Some(tqhnsw_ambeginscan);
    amroutine.amrescan = Some(tqhnsw_amrescan);
    amroutine.amgettuple = Some(tqhnsw_amgettuple);
    amroutine.amgetbitmap = None;
    amroutine.amendscan = Some(tqhnsw_amendscan);
    amroutine.ammarkpos = None;
    amroutine.amrestrpos = None;
    amroutine.amestimateparallelscan = None;
    amroutine.aminitparallelscan = None;
    amroutine.amparallelrescan = None;

    amroutine
}

#[pg_guard]
#[no_mangle]
pub unsafe extern "C-unwind" fn tqhnsw_handler(
    _fcinfo: pg_sys::FunctionCallInfo,
) -> pg_sys::Datum {
    pg_sys::Datum::from(build_tqhnsw_routine().into_pg())
}

#[no_mangle]
pub extern "C-unwind" fn pg_finfo_tqhnsw_handler() -> *const pg_sys::Pg_finfo_record {
    static API_V1: pg_sys::Pg_finfo_record = pg_sys::Pg_finfo_record { api_version: 1 };
    &API_V1
}

unsafe extern "C-unwind" fn tqhnsw_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    _index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe { initialize_metadata_page(index_relation) };

    let heap_tuples = unsafe {
        pg_sys::table_index_build_scan(
            heap_relation,
            index_relation,
            _index_info,
            false,
            false,
            Some(tqhnsw_build_callback),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    if heap_tuples > 0.0 {
        pgrx::error!("tqhnsw ambuild for non-empty tables is not implemented yet");
    }

    let mut result = unsafe { PgBox::<pg_sys::IndexBuildResult>::alloc0() };
    result.heap_tuples = heap_tuples;
    result.index_tuples = 0.0;
    result.into_pg()
}

unsafe extern "C-unwind" fn tqhnsw_ambuildempty(index_relation: pg_sys::Relation) {
    unsafe { initialize_metadata_page(index_relation) };
}

unsafe extern "C-unwind" fn tqhnsw_aminsert(
    _index_relation: pg_sys::Relation,
    _values: *mut pg_sys::Datum,
    _isnull: *mut bool,
    _heap_tid: pg_sys::ItemPointer,
    _heap_relation: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    _index_info: *mut pg_sys::IndexInfo,
) -> bool {
    pgrx::error!("tqhnsw aminsert is not implemented yet")
}

unsafe extern "C-unwind" fn tqhnsw_ambulkdelete(
    _info: *mut pg_sys::IndexVacuumInfo,
    _stats: *mut pg_sys::IndexBulkDeleteResult,
    _callback: pg_sys::IndexBulkDeleteCallback,
    _callback_state: *mut std::ffi::c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    pgrx::error!("tqhnsw ambulkdelete is not implemented yet")
}

unsafe extern "C-unwind" fn tqhnsw_amvacuumcleanup(
    _info: *mut pg_sys::IndexVacuumInfo,
    _stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    pgrx::error!("tqhnsw amvacuumcleanup is not implemented yet")
}

unsafe extern "C-unwind" fn tqhnsw_amcostestimate(
    _root: *mut pg_sys::PlannerInfo,
    _path: *mut pg_sys::IndexPath,
    _loop_count: f64,
    index_startup_cost: *mut pg_sys::Cost,
    index_total_cost: *mut pg_sys::Cost,
    index_selectivity: *mut pg_sys::Selectivity,
    index_correlation: *mut f64,
    index_pages: *mut f64,
) {
    // Prefer explicit non-selection over accidental planner use until the scan
    // path is implemented.
    unsafe {
        *index_startup_cost = f64::MAX;
        *index_total_cost = f64::MAX;
        *index_selectivity = 0.0;
        *index_correlation = 0.0;
        *index_pages = 0.0;
    }
}

unsafe extern "C-unwind" fn tqhnsw_amoptions(
    reloptions: pg_sys::Datum,
    validate: bool,
) -> *mut pg_sys::bytea {
    let mut relopts = pg_sys::local_relopts::default();

    unsafe {
        pg_sys::init_local_reloptions(&mut relopts, size_of::<TqHnswOptions>());
        pg_sys::add_local_int_reloption(
            &mut relopts,
            b"m\0".as_ptr().cast(),
            b"Maximum graph degree per layer.\0".as_ptr().cast(),
            TQHNSW_DEFAULT_M,
            TQHNSW_MIN_M,
            TQHNSW_MAX_M,
            offset_of!(TqHnswOptions, m) as i32,
        );
        pg_sys::add_local_int_reloption(
            &mut relopts,
            b"ef_construction\0".as_ptr().cast(),
            b"Candidate list width used during graph construction.\0"
                .as_ptr()
                .cast(),
            TQHNSW_DEFAULT_EF_CONSTRUCTION,
            TQHNSW_MIN_EF_CONSTRUCTION,
            TQHNSW_MAX_EF_CONSTRUCTION,
            offset_of!(TqHnswOptions, ef_construction) as i32,
        );
        pg_sys::build_local_reloptions(&mut relopts, reloptions, validate) as *mut pg_sys::bytea
    }
}

unsafe extern "C-unwind" fn tqhnsw_amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    true
}

unsafe extern "C-unwind" fn tqhnsw_ambeginscan(
    _index_relation: pg_sys::Relation,
    _nkeys: std::ffi::c_int,
    _norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    pgrx::error!("tqhnsw ambeginscan is not implemented yet")
}

unsafe extern "C-unwind" fn tqhnsw_amrescan(
    _scan: pg_sys::IndexScanDesc,
    _keys: pg_sys::ScanKey,
    _nkeys: std::ffi::c_int,
    _orderbys: pg_sys::ScanKey,
    _norderbys: std::ffi::c_int,
) {
    pgrx::error!("tqhnsw amrescan is not implemented yet")
}

unsafe extern "C-unwind" fn tqhnsw_amgettuple(
    _scan: pg_sys::IndexScanDesc,
    _direction: pg_sys::ScanDirection::Type,
) -> bool {
    pgrx::error!("tqhnsw amgettuple is not implemented yet")
}

unsafe extern "C-unwind" fn tqhnsw_amendscan(_scan: pg_sys::IndexScanDesc) {
    pgrx::error!("tqhnsw amendscan is not implemented yet")
}

unsafe extern "C-unwind" fn tqhnsw_build_callback(
    _index: pg_sys::Relation,
    _tid: pg_sys::ItemPointer,
    _values: *mut pg_sys::Datum,
    _isnull: *mut bool,
    _tuple_is_alive: bool,
    _state: *mut c_void,
) {
}

unsafe fn relation_options(index_relation: pg_sys::Relation) -> TqHnswOptions {
    let mut options = TqHnswOptions::DEFAULT;
    unsafe {
        set_varsize_4b(
            (&mut options as *mut TqHnswOptions).cast::<pg_sys::varlena>(),
            size_of::<TqHnswOptions>() as i32,
        );
    }

    let relid = unsafe { (*index_relation).rd_id };
    let reloptions = pgrx::Spi::get_one_with_args::<Vec<String>>(
        "SELECT reloptions FROM pg_class WHERE oid = $1",
        &[unsafe { pgrx::datum::DatumWithOid::new(relid, pg_sys::OIDOID) }],
    )
    .unwrap_or_else(|e| pgrx::error!("failed to read tqhnsw reloptions: {e}"));

    if let Some(reloptions) = reloptions {
        for reloption in reloptions {
            if let Some(value) = reloption.strip_prefix("m=") {
                options.m = value
                    .parse::<i32>()
                    .unwrap_or_else(|e| pgrx::error!("invalid tqhnsw m reloption: {e}"));
            } else if let Some(value) = reloption.strip_prefix("ef_construction=") {
                options.ef_construction = value.parse::<i32>().unwrap_or_else(|e| {
                    pgrx::error!("invalid tqhnsw ef_construction reloption: {e}")
                });
            }
        }
    }

    options
}

unsafe fn initialize_metadata_page(index_relation: pg_sys::Relation) {
    let options = unsafe { relation_options(index_relation) };
    let metadata = page::MetadataPage {
        m: u16::try_from(options.m).expect("validated m should fit into u16"),
        ef_construction: u16::try_from(options.ef_construction)
            .expect("validated ef_construction should fit into u16"),
        entry_point: page::ItemPointer::INVALID,
        dimensions: 0,
        bits: 0,
        max_level: 0,
    };

    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks == 0 {
        P_NEW
    } else {
        page::METADATA_BLOCK_NUMBER
    };
    let read_mode = if target_block == P_NEW {
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK
    } else {
        pg_sys::ReadBufferMode::RBM_NORMAL
    };
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            target_block,
            read_mode,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to allocate metadata buffer");
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    let special_size = (metadata_bytes.len() + 7) & !7;
    unsafe { pg_sys::PageInit(page, page_size, special_size) };
    let page_contents = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    unsafe {
        ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), page_contents, metadata_bytes.len());
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_index_metadata(index_oid: pg_sys::Oid) -> (u32, i32, i32, page::MetadataPage) {
    let index_relation = unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let options = unsafe { relation_options(index_relation) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };

    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };

    let metadata = {
        let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
        let page = unsafe { wal_txn.register_buffer(buffer, 0) };
        let contents = unsafe { pg_sys::PageGetContents(page) }.cast::<u8>();
        let contents_slice =
            unsafe { std::slice::from_raw_parts(contents, page::PAGE_HEADER_BYTES) };
        let decoded = page::MetadataPage::decode_contents(contents_slice)
            .expect("metadata page should decode");
        drop(wal_txn);
        decoded
    };

    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    (block_count, options.m, options.ef_construction, metadata)
}
