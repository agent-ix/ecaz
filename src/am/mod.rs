//! Access-method scaffolding for the future `tqhnsw` implementation.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::{offset_of, size_of};
use std::ptr;

use hnsw_rs::anndists::dist::distances::Distance;
use hnsw_rs::prelude::Hnsw;
use pgrx::{
    itemptr::item_pointer_get_both, pg_guard, pg_sys, varlena, AllocatedByRust, FromDatum, PgBox,
    PgTupleDesc,
};

use crate::quant::prod::PreparedQuery;

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
struct TqHnswReloptions {
    vl_len_: i32,
    m: i32,
    ef_construction: i32,
    build_source_column_offset: i32,
}

impl TqHnswReloptions {
    const DEFAULT: Self = Self {
        vl_len_: 0,
        m: TQHNSW_DEFAULT_M,
        ef_construction: TQHNSW_DEFAULT_EF_CONSTRUCTION,
        build_source_column_offset: 0,
    };
}

#[derive(Debug, Clone)]
struct TqHnswOptions {
    m: i32,
    ef_construction: i32,
    build_source_column: Option<String>,
}

impl TqHnswOptions {
    const DEFAULT: Self = Self {
        m: TQHNSW_DEFAULT_M,
        ef_construction: TQHNSW_DEFAULT_EF_CONSTRUCTION,
        build_source_column: None,
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

fn unsupported_build_only_error(operation: &str) -> ! {
    pgrx::error!("tqhnsw indexes are build-only for now: {operation} is not supported yet")
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

unsafe extern "C-unwind" fn tqhnsw_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut state = BuildState::new(index_relation);

            initialize_metadata_page(index_relation, state.initial_metadata());

            let heap_tuples = if state.options.build_source_column.is_some() {
                tqhnsw_build_scan_with_source(heap_relation, index_info, &mut state)
            } else {
                pg_sys::table_index_build_scan(
                    heap_relation,
                    index_relation,
                    index_info,
                    false,
                    false,
                    Some(tqhnsw_build_callback),
                    (&mut state as *mut BuildState).cast(),
                    ptr::null_mut(),
                )
            };
            let index_tuples = if state.heap_tuples.is_empty() {
                0.0
            } else {
                flush_build_state(index_relation, &state);
                state.heap_tuples.len() as f64
            };

            if heap_tuples != state.scanned_tuples as f64 {
                pgrx::error!(
                    "tqhnsw ambuild scanned {heap_tuples} heap tuples but observed {}",
                    state.scanned_tuples
                );
            }

            let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
            result.heap_tuples = heap_tuples;
            result.index_tuples = index_tuples;
            result.into_pg()
        })
    }
}

unsafe extern "C-unwind" fn tqhnsw_ambuildempty(index_relation: pg_sys::Relation) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = BuildState::new(index_relation);
            initialize_metadata_page(index_relation, state.initial_metadata());
        })
    }
}

unsafe extern "C-unwind" fn tqhnsw_aminsert(
    index_relation: pg_sys::Relation,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: pg_sys::ItemPointer,
    _heap_relation: pg_sys::Relation,
    _check_unique: pg_sys::IndexUniqueCheck::Type,
    _index_unchanged: bool,
    _index_info: *mut pg_sys::IndexInfo,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let heap_tid = decode_heap_tid(heap_tid);
            let tuple = build_heap_tuple(values, isnull, heap_tid);
            let options = relation_options(index_relation);
            let code_len = tuple.code.len();

            if let Some(source_column) = options.build_source_column {
                pgrx::error!(
                    "tqhnsw aminsert does not support build_source_column indexes yet: {source_column}"
                );
            }

            with_locked_metadata_page(index_relation, |metadata| {
                if metadata.dimensions == 0 && metadata.bits == 0 {
                    metadata.dimensions = tuple.dimensions;
                    metadata.bits = tuple.bits;
                    metadata.seed = tuple.seed;
                } else if tuple.dimensions != metadata.dimensions
                    || tuple.bits != metadata.bits
                    || tuple.seed != metadata.seed
                {
                    pgrx::error!(
                        "tqhnsw aminsert requires matching tqvector shape ({},{},{}) but got ({},{},{})",
                        metadata.dimensions,
                        metadata.bits,
                        metadata.seed,
                        tuple.dimensions,
                        tuple.bits,
                        tuple.seed
                    );
                }

                if let Some(element_tid) = find_duplicate_element_tid(
                    index_relation,
                    metadata.dimensions,
                    metadata.bits,
                    code_len,
                    &tuple.code,
                ) {
                    coalesce_duplicate_heap_tid(index_relation, element_tid, code_len, heap_tid);
                    return;
                }

                let element_tid = append_heap_tuple(index_relation, &tuple);
                if metadata.entry_point == page::ItemPointer::INVALID {
                    metadata.entry_point = element_tid;
                }
            });
            false
        })
    }
}

unsafe extern "C-unwind" fn tqhnsw_ambulkdelete(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
    _callback: pg_sys::IndexBulkDeleteCallback,
    _callback_state: *mut std::ffi::c_void,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe { pgrx::pgrx_extern_c_guard(|| tqhnsw_noop_vacuum_stats((*info).index, stats)) }
}

unsafe extern "C-unwind" fn tqhnsw_amvacuumcleanup(
    info: *mut pg_sys::IndexVacuumInfo,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    unsafe { pgrx::pgrx_extern_c_guard(|| tqhnsw_noop_vacuum_stats((*info).index, stats)) }
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
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            // Prefer explicit non-selection over accidental planner use until the scan
            // path is implemented.
            *index_startup_cost = f64::MAX;
            *index_total_cost = f64::MAX;
            *index_selectivity = 0.0;
            *index_correlation = 0.0;
            *index_pages = 0.0;
        })
    }
}

unsafe extern "C-unwind" fn tqhnsw_amoptions(
    reloptions: pg_sys::Datum,
    validate: bool,
) -> *mut pg_sys::bytea {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut relopts = pg_sys::local_relopts::default();

            pg_sys::init_local_reloptions(&mut relopts, size_of::<TqHnswReloptions>());
            pg_sys::add_local_int_reloption(
                &mut relopts,
                b"m\0".as_ptr().cast(),
                b"Maximum graph degree per layer.\0".as_ptr().cast(),
                TQHNSW_DEFAULT_M,
                TQHNSW_MIN_M,
                TQHNSW_MAX_M,
                offset_of!(TqHnswReloptions, m) as i32,
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
                offset_of!(TqHnswReloptions, ef_construction) as i32,
            );
            pg_sys::add_local_string_reloption(
                &mut relopts,
                b"build_source_column\0".as_ptr().cast(),
                b"Optional heap column name supplying raw real[] vectors for ambuild graph construction.\0"
                    .as_ptr()
                    .cast(),
                ptr::null(),
                None,
                None,
                offset_of!(TqHnswReloptions, build_source_column_offset) as i32,
            );
            pg_sys::build_local_reloptions(&mut relopts, reloptions, validate) as *mut pg_sys::bytea
        })
    }
}

unsafe extern "C-unwind" fn tqhnsw_amvalidate(_opclassoid: pg_sys::Oid) -> bool {
    unsafe { pgrx::pgrx_extern_c_guard(|| true) }
}

unsafe extern "C-unwind" fn tqhnsw_ambeginscan(
    index_relation: pg_sys::Relation,
    nkeys: std::ffi::c_int,
    norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let scan = pg_sys::RelationGetIndexScan(index_relation, nkeys, norderbys);
            if scan.is_null() {
                pgrx::error!("tqhnsw failed to allocate scan descriptor");
            }

            (*scan).opaque = PgBox::<TqScanOpaque>::alloc0().into_pg().cast();
            scan
        })
    }
}

unsafe extern "C-unwind" fn tqhnsw_amrescan(
    scan: pg_sys::IndexScanDesc,
    keys: pg_sys::ScanKey,
    nkeys: std::ffi::c_int,
    orderbys: pg_sys::ScanKey,
    norderbys: std::ffi::c_int,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("tqhnsw amrescan received a null scan descriptor");
            }
            if nkeys != 0 || !keys.is_null() {
                pgrx::error!("tqhnsw scan does not support index quals yet");
            }
            if norderbys != 1 {
                pgrx::error!("tqhnsw scan currently requires exactly one ORDER BY query");
            }
            if orderbys.is_null() {
                pgrx::error!("tqhnsw amrescan received null order-by scan keys");
            }

            let orderby = &*orderbys;
            if (orderby.sk_flags as u32) & pg_sys::SK_ISNULL != 0 {
                pgrx::error!("tqhnsw scan query must not be NULL");
            }

            let query = Vec::<f32>::from_polymorphic_datum(
                orderby.sk_argument,
                false,
                pg_sys::FLOAT4ARRAYOID,
            )
            .unwrap_or_else(|| pgrx::error!("tqhnsw scan requires a real[] ORDER BY query"));
            if query.is_empty() {
                pgrx::error!("tqhnsw scan query must not be empty");
            }
            if query.len() > u16::MAX as usize {
                pgrx::error!(
                    "tqhnsw scan query dimension {} exceeds maximum {}",
                    query.len(),
                    u16::MAX
                );
            }

            let metadata = read_metadata_page((*scan).indexRelation);
            if metadata.dimensions != 0 && query.len() != metadata.dimensions as usize {
                pgrx::error!(
                    "tqhnsw scan query dimension mismatch: index dim {}, query dim {}",
                    metadata.dimensions,
                    query.len()
                );
            }

            (*scan).xs_recheck = false;
            (*scan).xs_recheckorderby = false;
            (*scan).xs_orderbyvals = ptr::null_mut();
            (*scan).xs_orderbynulls = ptr::null_mut();

            let opaque = &mut *(*scan).opaque.cast::<TqScanOpaque>();
            opaque.rescan_called = true;
            opaque.scan_dimensions = metadata.dimensions;
            opaque.scan_bits = metadata.bits;
            opaque.scan_code_len = if metadata.dimensions == 0 {
                0
            } else {
                crate::code_len(metadata.dimensions as usize, metadata.bits)
            };
            opaque.scan_block_count = pg_sys::RelationGetNumberOfBlocksInFork(
                (*scan).indexRelation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
            );
            store_scan_query(opaque, &query);
            store_scan_prepared_query(opaque, &query, &metadata);
            reset_scan_position(opaque);
        })
    }
}

unsafe extern "C-unwind" fn tqhnsw_amgettuple(
    scan: pg_sys::IndexScanDesc,
    direction: pg_sys::ScanDirection::Type,
) -> bool {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                pgrx::error!("tqhnsw amgettuple received a null scan descriptor");
            }

            let opaque_ptr = (*scan).opaque.cast::<TqScanOpaque>();
            if opaque_ptr.is_null() {
                pgrx::error!("tqhnsw amgettuple missing scan opaque state");
            }

            let opaque = &*opaque_ptr;
            if !opaque.rescan_called {
                pgrx::error!("tqhnsw amgettuple requires amrescan before scan execution");
            }
            if direction != pg_sys::ScanDirection::ForwardScanDirection {
                pgrx::error!("tqhnsw amgettuple only supports forward scan direction");
            }

            if opaque.scan_dimensions == 0 {
                return false;
            }

            let opaque = &mut *opaque_ptr;
            if let Some(heap_tid) =
                next_linear_scan_heap_tid((*scan).indexRelation, opaque, opaque.scan_code_len)
            {
                set_scan_heap_tid(scan, heap_tid);
                return true;
            }

            false
        })
    }
}

unsafe extern "C-unwind" fn tqhnsw_amendscan(scan: pg_sys::IndexScanDesc) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            if scan.is_null() {
                return;
            }

            let opaque = (*scan).opaque;
            if !opaque.is_null() {
                free_scan_prepared_query(&mut *opaque.cast::<TqScanOpaque>());
                free_scan_query(&mut *opaque.cast::<TqScanOpaque>());
                pg_sys::pfree(opaque);
                (*scan).opaque = ptr::null_mut();
            }
        })
    }
}

unsafe fn store_scan_query(opaque: &mut TqScanOpaque, query: &[f32]) {
    free_scan_query(opaque);

    let query_bytes = std::mem::size_of_val(query);
    let query_values = unsafe { pg_sys::palloc(query_bytes) }.cast::<f32>();
    if query_values.is_null() {
        pgrx::error!("tqhnsw failed to allocate scan query state");
    }

    unsafe {
        ptr::copy_nonoverlapping(query.as_ptr(), query_values, query.len());
    }
    opaque.query_dimensions = u16::try_from(query.len()).expect("query length should fit in u16");
    opaque.query_values = query_values;
}

unsafe fn free_scan_query(opaque: &mut TqScanOpaque) {
    if !opaque.query_values.is_null() {
        unsafe { pg_sys::pfree(opaque.query_values.cast()) };
        opaque.query_values = ptr::null_mut();
    }
    opaque.query_dimensions = 0;
}

fn store_scan_prepared_query(
    opaque: &mut TqScanOpaque,
    query: &[f32],
    metadata: &page::MetadataPage,
) {
    free_scan_prepared_query(opaque);
    if metadata.dimensions == 0 {
        return;
    }

    let prepared = crate::quant::prod::ProdQuantizer::cached(
        metadata.dimensions as usize,
        metadata.bits,
        metadata.seed,
    )
    .prepare_ip_query(query);
    opaque.prepared_query = Box::into_raw(Box::new(prepared));
}

fn free_scan_prepared_query(opaque: &mut TqScanOpaque) {
    if !opaque.prepared_query.is_null() {
        drop(unsafe { Box::from_raw(opaque.prepared_query) });
        opaque.prepared_query = ptr::null_mut();
    }
}

fn reset_scan_position(opaque: &mut TqScanOpaque) {
    opaque.next_block_number = page::FIRST_DATA_BLOCK_NUMBER;
    opaque.next_offset_number = 1;
    opaque.scan_exhausted = false;
    opaque.pending_heaptid_count = 0;
    opaque.pending_heaptid_index = 0;
    clear_scan_result_state(opaque);
}

fn store_pending_scan_heaptids(opaque: &mut TqScanOpaque, heaptids: &[page::ItemPointer]) {
    debug_assert!(heaptids.len() <= page::HEAPTID_INLINE_CAPACITY);

    opaque.pending_heaptids.fill(page::ItemPointer::INVALID);
    opaque.pending_heaptid_count =
        u8::try_from(heaptids.len()).expect("heap tid count should fit in u8");
    opaque.pending_heaptid_index = 0;

    for (index, tid) in heaptids.iter().copied().enumerate() {
        opaque.pending_heaptids[index] = tid;
    }
}

fn take_pending_scan_heap_tid(opaque: &mut TqScanOpaque) -> Option<page::ItemPointer> {
    if opaque.pending_heaptid_index >= opaque.pending_heaptid_count {
        return None;
    }

    let tid = opaque.pending_heaptids[opaque.pending_heaptid_index as usize];
    opaque.pending_heaptid_index += 1;
    if opaque.pending_heaptid_index >= opaque.pending_heaptid_count {
        opaque.pending_heaptid_count = 0;
        opaque.pending_heaptid_index = 0;
    }
    Some(tid)
}

fn clear_scan_result_state(opaque: &mut TqScanOpaque) {
    opaque.current_result_tid = page::ItemPointer::INVALID;
    opaque.current_result_score = 0.0;
    opaque.current_result_score_valid = false;
}

unsafe fn next_linear_scan_heap_tid(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    code_len: usize,
) -> Option<page::ItemPointer> {
    if let Some(heap_tid) = take_pending_scan_heap_tid(opaque) {
        return Some(heap_tid);
    }

    if opaque.scan_exhausted {
        return None;
    }

    if opaque.scan_block_count <= page::FIRST_DATA_BLOCK_NUMBER {
        opaque.scan_exhausted = true;
        clear_scan_result_state(opaque);
        return None;
    }

    for block_number in opaque.next_block_number..opaque.scan_block_count {
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let line_pointer_count = page_line_pointer_count(page_ptr);
        let offset_start = if block_number == opaque.next_block_number {
            opaque.next_offset_number.max(1)
        } else {
            1
        };

        for offset in offset_start..=line_pointer_count {
            let item_id = unsafe { &*page_item_id(page_ptr, offset) };
            if item_id.lp_flags() == 0 {
                continue;
            }

            let tuple_offset = item_id.lp_off() as usize;
            let tuple_len = item_id.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                pgrx::error!(
                    "tqhnsw found invalid tuple bounds while scanning block {block_number}"
                );
            }

            let tuple_bytes =
                unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
            if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
                continue;
            }

            let element = page::TqElementTuple::decode(tuple_bytes, code_len).unwrap_or_else(|e| {
                pgrx::error!("tqhnsw failed to decode scan element tuple: {e}")
            });
            if element.deleted || element.heaptids.is_empty() {
                continue;
            }

            opaque.next_block_number = block_number;
            debug_assert!(offset < u16::MAX, "scan offset should fit in page-local u16 range");
            opaque.next_offset_number = offset + 1;
            opaque.current_result_tid = page::ItemPointer {
                block_number,
                offset_number: offset,
            };
            opaque.current_result_score = 0.0;
            opaque.current_result_score_valid = false;

            store_pending_scan_heaptids(opaque, &element.heaptids);
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            return take_pending_scan_heap_tid(opaque);
        }

        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        opaque.next_block_number = block_number + 1;
        opaque.next_offset_number = 1;
    }

    opaque.scan_exhausted = true;
    clear_scan_result_state(opaque);
    None
}

fn set_scan_heap_tid(scan: pg_sys::IndexScanDesc, heap_tid: page::ItemPointer) {
    unsafe {
        pgrx::itemptr::item_pointer_set_all(
            &mut (*scan).xs_heaptid,
            heap_tid.block_number,
            heap_tid.offset_number,
        );
    }
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn read_scan_query(opaque: &TqScanOpaque) -> Vec<f32> {
    if opaque.query_values.is_null() || opaque.query_dimensions == 0 {
        return Vec::new();
    }

    let query = unsafe {
        std::slice::from_raw_parts(opaque.query_values, opaque.query_dimensions as usize)
    };
    query.to_vec()
}

unsafe extern "C-unwind" fn tqhnsw_build_callback(
    _index: pg_sys::Relation,
    tid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut c_void,
) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = &mut *state.cast::<BuildState>();
            let heap_tid = decode_heap_tid(tid);
            let tuple = build_heap_tuple(values, isnull, heap_tid);
            state.push(tuple);
        })
    }
}

unsafe fn relation_options(index_relation: pg_sys::Relation) -> TqHnswOptions {
    let rd_options = unsafe { (*index_relation).rd_options };
    if rd_options.is_null() {
        return TqHnswOptions::DEFAULT;
    }

    let reloptions = unsafe { &*rd_options.cast::<TqHnswReloptions>() };
    let build_source_column = if reloptions.build_source_column_offset == 0 {
        None
    } else {
        let value_ptr = unsafe {
            rd_options
                .cast::<u8>()
                .add(reloptions.build_source_column_offset as usize)
                .cast::<std::ffi::c_char>()
        };
        let value = unsafe { std::ffi::CStr::from_ptr(value_ptr) }
            .to_str()
            .unwrap_or_else(|e| pgrx::error!("invalid tqhnsw build_source_column reloption: {e}"));
        if value.is_empty() {
            pgrx::error!("invalid tqhnsw build_source_column reloption: value must not be empty");
        }
        Some(value.to_owned())
    };

    TqHnswOptions {
        m: reloptions.m,
        ef_construction: reloptions.ef_construction,
        build_source_column,
    }
}

unsafe fn initialize_metadata_page(index_relation: pg_sys::Relation, metadata: page::MetadataPage) {
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
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    let special_size = (metadata_bytes.len() + 7) & !7;
    unsafe { pg_sys::PageInit(page, page_size, special_size) };
    unsafe { write_metadata_bytes(page, &metadata_bytes) };

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

unsafe fn write_metadata_bytes(page: pg_sys::Page, metadata_bytes: &[u8]) {
    let page_contents = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    unsafe {
        ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), page_contents, metadata_bytes.len());
    }
}

unsafe fn update_metadata_page(index_relation: pg_sys::Relation, metadata: page::MetadataPage) {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to open metadata buffer");
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    unsafe { write_metadata_bytes(page, &metadata_bytes) };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

unsafe fn with_locked_metadata_page<T>(
    index_relation: pg_sys::Relation,
    f: impl FnOnce(&mut page::MetadataPage) -> T,
) -> T {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to open metadata buffer");
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let raw_page = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let page_bytes = unsafe { std::slice::from_raw_parts(raw_page, page_size) };
    let mut metadata =
        page::MetadataPage::decode_page(page_bytes).expect("metadata page should decode");
    let result = f(&mut metadata);

    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page = unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    unsafe { write_metadata_bytes(page, &metadata_bytes) };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    result
}

unsafe fn append_heap_tuple(
    index_relation: pg_sys::Relation,
    tuple: &BuildTuple,
) -> page::ItemPointer {
    let neighbor_payload = page::TqNeighborTuple {
        count: 0,
        tids: Vec::new(),
    }
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode neighbor tuple: {e}"));
    let required_bytes = page::raw_tuple_storage_bytes(neighbor_payload.len())
        + page::raw_tuple_storage_bytes(page::TqElementTuple::encoded_len(tuple.code.len()));
    let mut staged_page =
        page::DataPage::new(page::FIRST_DATA_BLOCK_NUMBER, pg_sys::BLCKSZ as usize);
    staged_page
        .insert_raw_tuple(neighbor_payload.clone())
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to stage aminsert neighbor tuple: {e}"));
    if !staged_page.can_fit_raw_tuple(page::TqElementTuple::encoded_len(tuple.code.len())) {
        pgrx::error!(
            "tqhnsw aminsert does not yet support tuples that require more than one fresh data page"
        );
    }

    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks > page::FIRST_DATA_BLOCK_NUMBER {
        existing_blocks - 1
    } else {
        P_NEW
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
        pgrx::error!("tqhnsw failed to allocate data buffer for aminsert");
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    if target_block == P_NEW {
        unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };
    } else {
        let free_space = unsafe { pg_sys::PageGetFreeSpace(page_ptr) as usize };
        if free_space < required_bytes {
            std::mem::drop(wal_txn);
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            return unsafe {
                append_heap_tuple_to_new_page(index_relation, tuple, &neighbor_payload)
            };
        }
    }

    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };
    let neighbor_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            neighbor_payload.as_ptr().cast_mut().cast(),
            neighbor_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if neighbor_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write neighbor tuple during aminsert");
    }

    let element_payload = page::TqElementTuple {
        level: 0,
        deleted: false,
        heaptids: tuple.heap_tids.clone(),
        neighbortid: page::ItemPointer {
            block_number,
            offset_number: neighbor_offset,
        },
        code: tuple.code.clone(),
    }
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode element tuple: {e}"));
    let element_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            element_payload.as_ptr().cast_mut().cast(),
            element_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if element_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write element tuple during aminsert");
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    page::ItemPointer {
        block_number,
        offset_number: element_offset,
    }
}

unsafe fn append_heap_tuple_to_new_page(
    index_relation: pg_sys::Relation,
    tuple: &BuildTuple,
    neighbor_payload: &[u8],
) -> page::ItemPointer {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            P_NEW,
            pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("tqhnsw failed to allocate fallback data buffer for aminsert");
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };

    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };
    let neighbor_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            neighbor_payload.as_ptr().cast_mut().cast(),
            neighbor_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if neighbor_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write fallback neighbor tuple during aminsert");
    }

    let element_payload = page::TqElementTuple {
        level: 0,
        deleted: false,
        heaptids: tuple.heap_tids.clone(),
        neighbortid: page::ItemPointer {
            block_number,
            offset_number: neighbor_offset,
        },
        code: tuple.code.clone(),
    }
    .encode()
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode fallback element tuple: {e}"));
    let element_offset = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            element_payload.as_ptr().cast_mut().cast(),
            element_payload.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if element_offset == pg_sys::InvalidOffsetNumber {
        pgrx::error!("tqhnsw failed to write fallback element tuple during aminsert");
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    page::ItemPointer {
        block_number,
        offset_number: element_offset,
    }
}

unsafe fn find_duplicate_element_tid(
    index_relation: pg_sys::Relation,
    dimensions: u16,
    bits: u8,
    code_len: usize,
    code: &[u8],
) -> Option<page::ItemPointer> {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    if block_count <= page::FIRST_DATA_BLOCK_NUMBER {
        return None;
    }

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let line_pointer_count = page_line_pointer_count(page_ptr);

        for offset in 1..=line_pointer_count {
            let item_id = unsafe { &*page_item_id(page_ptr, offset) };
            if item_id.lp_flags() == 0 {
                continue;
            }

            let tuple_offset = item_id.lp_off() as usize;
            let tuple_len = item_id.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                pgrx::error!(
                    "tqhnsw found invalid tuple bounds while scanning block {block_number}"
                );
            }

            let tuple_bytes =
                unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
            if tuple_bytes.first().copied() != Some(page::TQ_ELEMENT_TAG) {
                continue;
            }

            let element = page::TqElementTuple::decode(tuple_bytes, code_len).unwrap_or_else(|e| {
                pgrx::error!("tqhnsw failed to decode candidate duplicate tuple: {e}")
            });
            if element.code == code {
                unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
                return Some(page::ItemPointer {
                    block_number,
                    offset_number: offset,
                });
            }
        }

        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    let _ = dimensions;
    let _ = bits;
    None
}

unsafe fn coalesce_duplicate_heap_tid(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    code_len: usize,
    heap_tid: page::ItemPointer,
) {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            element_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!(
            "tqhnsw failed to open duplicate element block {}",
            element_tid.block_number
        );
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) }
            .cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let item_id = unsafe { &*page_item_id(page_ptr, element_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        pgrx::error!("tqhnsw duplicate element tuple slot is unused");
    }
    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        pgrx::error!(
            "tqhnsw found invalid duplicate tuple bounds on block {}",
            element_tid.block_number
        );
    }

    let tuple_bytes = unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
    let mut element = page::TqElementTuple::decode(tuple_bytes, code_len)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode duplicate element tuple: {e}"));
    if element.heaptids.contains(&heap_tid) {
        unsafe { wal_txn.finish() };
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        return;
    }
    if element.heaptids.len() >= page::HEAPTID_INLINE_CAPACITY {
        pgrx::error!(
            "tqhnsw aminsert supports at most {} duplicate heap tids per encoded vector",
            page::HEAPTID_INLINE_CAPACITY
        );
    }
    element.heaptids.push(heap_tid);
    let encoded = element
        .encode()
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to encode coalesced element tuple: {e}"));
    if encoded.len() != tuple_len {
        pgrx::error!(
            "tqhnsw duplicate element tuple size changed from {} to {}",
            tuple_len,
            encoded.len()
        );
    }
    unsafe {
        ptr::copy_nonoverlapping(encoded.as_ptr(), page_ptr.add(tuple_offset), encoded.len());
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

unsafe fn tqhnsw_noop_vacuum_stats(
    index_relation: pg_sys::Relation,
    stats: *mut pg_sys::IndexBulkDeleteResult,
) -> *mut pg_sys::IndexBulkDeleteResult {
    let stats = if stats.is_null() {
        unsafe { PgBox::<pg_sys::IndexBulkDeleteResult>::alloc0().into_pg() }
    } else {
        stats
    };

    unsafe {
        (*stats).num_pages = pg_sys::RelationGetNumberOfBlocksInFork(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
        );
        (*stats).estimated_count = false;
        (*stats).num_index_tuples = count_element_tuples(index_relation) as f64;
    }

    stats
}

unsafe fn count_element_tuples(index_relation: pg_sys::Relation) -> usize {
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let mut count = 0_usize;

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
        let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let line_pointer_count = page_line_pointer_count(page_ptr);

        for offset in 1..=line_pointer_count {
            let item_id = unsafe { &*page_item_id(page_ptr, offset) };
            if item_id.lp_flags() == 0 {
                continue;
            }

            let tuple_offset = item_id.lp_off() as usize;
            let tuple_len = item_id.lp_len() as usize;
            if tuple_offset + tuple_len > page_size {
                pgrx::error!(
                    "tqhnsw found invalid tuple bounds while counting vacuum tuples on block {block_number}"
                );
            }

            let tuple_bytes =
                unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
            if tuple_bytes.first().copied() == Some(page::TQ_ELEMENT_TAG) {
                count += 1;
            }
        }

        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    count
}

unsafe fn page_item_id(page_ptr: *mut u8, offset: u16) -> *const pg_sys::ItemIdData {
    unsafe {
        page_ptr
            .add(
                page::PAGE_HEADER_BYTES + ((offset - 1) as usize * size_of::<pg_sys::ItemIdData>()),
            )
            .cast::<pg_sys::ItemIdData>()
    }
}

fn page_line_pointer_count(page_ptr: *mut u8) -> u16 {
    let page_header = page_ptr.cast::<pg_sys::PageHeaderData>();
    ((unsafe { (*page_header).pd_lower } as usize - size_of::<pg_sys::PageHeaderData>())
        / size_of::<pg_sys::ItemIdData>()) as u16
}

#[derive(Debug, Clone)]
struct BuildTuple {
    heap_tids: Vec<page::ItemPointer>,
    dimensions: u16,
    bits: u8,
    seed: u64,
    code: Vec<u8>,
    source_vector: Option<Vec<f32>>,
    source_count: usize,
}

#[derive(Debug)]
struct BuildState {
    options: TqHnswOptions,
    page_size: usize,
    scanned_tuples: usize,
    heap_tuples: Vec<BuildTuple>,
    dimensions: Option<u16>,
    bits: Option<u8>,
    seed: Option<u64>,
}

#[derive(Debug, Clone)]
struct HnswBuildNode {
    level: u8,
    neighbors: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
struct BuildCodeDistance {
    dimensions: usize,
    bits: u8,
    seed: u64,
    score_offset: f32,
}

impl BuildCodeDistance {
    fn new(dimensions: usize, bits: u8, seed: u64) -> Self {
        let quantizer = crate::quant::prod::ProdQuantizer::cached(dimensions, bits, seed);
        let max_abs_centroid = quantizer
            .codebook
            .iter()
            .map(|value| value.abs())
            .fold(0.0_f32, f32::max);

        Self {
            dimensions,
            bits,
            seed,
            score_offset: dimensions as f32 * max_abs_centroid * max_abs_centroid,
        }
    }
}

impl Distance<u8> for BuildCodeDistance {
    fn eval(&self, va: &[u8], vb: &[u8]) -> f32 {
        self.score_offset
            - crate::score_code_inner_product(self.dimensions, self.bits, self.seed, va, vb)
    }
}

#[derive(Debug, Clone, Copy)]
struct BuildVectorDistance {
    score_offset: f32,
}

#[repr(C)]
#[derive(Debug)]
struct TqScanOpaque {
    rescan_called: bool,
    query_dimensions: u16,
    query_values: *mut f32,
    prepared_query: *mut PreparedQuery,
    scan_dimensions: u16,
    scan_bits: u8,
    scan_code_len: usize,
    scan_block_count: u32,
    current_result_tid: page::ItemPointer,
    current_result_score: f32,
    current_result_score_valid: bool,
    next_block_number: u32,
    next_offset_number: u16,
    scan_exhausted: bool,
    pending_heaptids: [page::ItemPointer; page::HEAPTID_INLINE_CAPACITY],
    pending_heaptid_count: u8,
    pending_heaptid_index: u8,
}

impl Default for TqScanOpaque {
    fn default() -> Self {
        Self {
            rescan_called: false,
            query_dimensions: 0,
            query_values: ptr::null_mut(),
            prepared_query: ptr::null_mut(),
            scan_dimensions: 0,
            scan_bits: 0,
            scan_code_len: 0,
            scan_block_count: 0,
            current_result_tid: page::ItemPointer::INVALID,
            current_result_score: 0.0,
            current_result_score_valid: false,
            next_block_number: page::FIRST_DATA_BLOCK_NUMBER,
            next_offset_number: 1,
            scan_exhausted: false,
            pending_heaptids: [page::ItemPointer::INVALID; page::HEAPTID_INLINE_CAPACITY],
            pending_heaptid_count: 0,
            pending_heaptid_index: 0,
        }
    }
}

#[cfg(any(test, feature = "pg_test"))]
type HeapTidCoords = (u32, u16);

impl Distance<f32> for BuildVectorDistance {
    fn eval(&self, va: &[f32], vb: &[f32]) -> f32 {
        self.score_offset - score_source_inner_product(va, vb)
    }
}

impl BuildState {
    fn new(index_relation: pg_sys::Relation) -> Self {
        let options = unsafe { relation_options(index_relation) };
        Self {
            options,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 0,
            heap_tuples: Vec::new(),
            dimensions: None,
            bits: None,
            seed: None,
        }
    }

    fn initial_metadata(&self) -> page::MetadataPage {
        page::MetadataPage {
            m: u16::try_from(self.options.m).expect("validated m should fit into u16"),
            ef_construction: u16::try_from(self.options.ef_construction)
                .expect("validated ef_construction should fit into u16"),
            entry_point: page::ItemPointer::INVALID,
            dimensions: 0,
            bits: 0,
            max_level: 0,
            seed: 0,
        }
    }

    fn push(&mut self, tuple: BuildTuple) {
        self.scanned_tuples += tuple.heap_tids.len();

        match (self.dimensions, self.bits, self.seed) {
            (None, None, None) => {
                self.dimensions = Some(tuple.dimensions);
                self.bits = Some(tuple.bits);
                self.seed = Some(tuple.seed);
                if !page::element_tuple_fits_on_page(tuple.code.len(), self.page_size) {
                    pgrx::error!(
                        "tqhnsw element tuple for dim {} bits {} does not fit on a page",
                        tuple.dimensions,
                        tuple.bits
                    );
                }
            }
            (Some(dimensions), Some(bits), Some(seed)) => {
                if tuple.dimensions != dimensions || tuple.bits != bits || tuple.seed != seed {
                    pgrx::error!(
                        "tqhnsw ambuild requires a single tqvector shape; saw ({},{},{}) after ({},{},{})",
                        tuple.dimensions,
                        tuple.bits,
                        tuple.seed,
                        dimensions,
                        bits,
                        seed
                    );
                }
            }
            _ => unreachable!("shape tracking must be initialized together"),
        }

        if let Some(existing) = self
            .heap_tuples
            .iter_mut()
            .find(|existing| existing.code == tuple.code)
        {
            if existing.heap_tids.len() + tuple.heap_tids.len() > page::HEAPTID_INLINE_CAPACITY {
                pgrx::error!(
                    "tqhnsw ambuild supports at most {} duplicate heap tids per encoded vector",
                    page::HEAPTID_INLINE_CAPACITY
                );
            }
            existing.heap_tids.extend(tuple.heap_tids);
            match (&mut existing.source_vector, tuple.source_vector) {
                (Some(existing_source), Some(tuple_source)) => {
                    if existing.source_count == 0 || tuple.source_count == 0 {
                        pgrx::error!(
                            "tqhnsw build_source_column representatives must have non-zero counts"
                        );
                    }
                    if existing_source.len() != tuple_source.len() {
                        pgrx::error!(
                            "tqhnsw build_source_column representative length mismatch: {} vs {}",
                            existing_source.len(),
                            tuple_source.len()
                        );
                    }
                    average_source_representatives(
                        existing_source,
                        existing.source_count,
                        &tuple_source,
                        tuple.source_count,
                    );
                    existing.source_count += tuple.source_count;
                }
                (None, Some(tuple_source)) => {
                    existing.source_vector = Some(tuple_source);
                    existing.source_count = tuple.source_count;
                }
                _ => {}
            }
            return;
        }

        self.heap_tuples.push(tuple);
    }
}

unsafe fn decode_heap_tid(tid: pg_sys::ItemPointer) -> page::ItemPointer {
    if tid.is_null() {
        pgrx::error!("tqhnsw ambuild received a null heap tid");
    }
    let (block_number, offset_number) = item_pointer_get_both(unsafe { *tid });
    page::ItemPointer {
        block_number,
        offset_number,
    }
}

unsafe fn build_heap_tuple(
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: page::ItemPointer,
) -> BuildTuple {
    if values.is_null() || isnull.is_null() {
        pgrx::error!("tqhnsw ambuild received null tuple value arrays");
    }
    if unsafe { *isnull } {
        pgrx::error!("tqhnsw does not support NULL indexed values");
    }

    let datum = unsafe { *values };
    if datum.is_null() {
        pgrx::error!("tqhnsw ambuild received a null tqvector datum");
    }

    let original = datum.cast_mut_ptr::<std::ffi::c_void>().cast::<pg_sys::varlena>();
    let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
    let is_copy = !std::ptr::eq(varlena, original);
    let bytes = unsafe { varlena::varlena_to_byte_slice(varlena) }.to_vec();
    if is_copy {
        unsafe { pg_sys::pfree(varlena.cast()) };
    }

    let (dimensions, bits, seed, _, code) = crate::unpack(&bytes)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw ambuild found invalid tqvector: {e}"));

    BuildTuple {
        heap_tids: vec![heap_tid],
        dimensions,
        bits,
        seed,
        code: code.to_vec(),
        source_vector: None,
        source_count: 0,
    }
}

unsafe fn build_heap_tuple_with_source(
    vector_datum: pg_sys::Datum,
    heap_tid: page::ItemPointer,
    source_vector: Vec<f32>,
) -> BuildTuple {
    if vector_datum.is_null() {
        pgrx::error!("tqhnsw ambuild received a null tqvector datum");
    }

    let original = vector_datum
        .cast_mut_ptr::<std::ffi::c_void>()
        .cast::<pg_sys::varlena>();
    let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
    let is_copy = !std::ptr::eq(varlena, original);
    let bytes = unsafe { varlena::varlena_to_byte_slice(varlena) }.to_vec();
    if is_copy {
        unsafe { pg_sys::pfree(varlena.cast()) };
    }

    let (dimensions, bits, seed, _, code) = crate::unpack(&bytes)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw ambuild found invalid tqvector: {e}"));

    if source_vector.is_empty() {
        pgrx::error!("tqhnsw build_source_column arrays must not be empty");
    }
    if source_vector.len() != dimensions as usize {
        pgrx::error!(
            "tqhnsw build_source_column dimension mismatch: source dim {} vs tqvector dim {}",
            source_vector.len(),
            dimensions
        );
    }

    BuildTuple {
        heap_tids: vec![heap_tid],
        dimensions,
        bits,
        seed,
        code: code.to_vec(),
        source_vector: Some(source_vector),
        source_count: 1,
    }
}

fn average_source_representatives(
    existing: &mut [f32],
    existing_count: usize,
    incoming: &[f32],
    incoming_count: usize,
) {
    assert_eq!(existing.len(), incoming.len());
    assert!(existing_count > 0);
    assert!(incoming_count > 0);

    let total_count = existing_count + incoming_count;
    for (existing_value, incoming_value) in existing.iter_mut().zip(incoming.iter()) {
        *existing_value = ((*existing_value * existing_count as f32)
            + (*incoming_value * incoming_count as f32))
            / total_count as f32;
    }
}

unsafe fn tqhnsw_build_scan_with_source(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    state: &mut BuildState,
) -> f64 {
    let source_column = state
        .options
        .build_source_column
        .clone()
        .expect("source scan should only run when build_source_column is configured");
    let index_attnum = unsafe { source_build_index_attnum(index_info) };
    let source_attnum = unsafe { resolve_source_attnum(heap_relation, &source_column) };
    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let att = tuple_desc
        .get(source_attnum as usize - 1)
        .expect("resolved build source attribute should exist");
    if att.attisdropped {
        pgrx::error!("tqhnsw build_source_column \"{source_column}\" references a dropped column");
    }
    if att.atttypid != pg_sys::FLOAT4ARRAYOID {
        pgrx::error!(
            "tqhnsw build_source_column \"{source_column}\" must be real[], got type oid {}",
            u32::from(att.atttypid)
        );
    }

    let slot = unsafe {
        pg_sys::MakeSingleTupleTableSlot(
            (*heap_relation).rd_att,
            pg_sys::table_slot_callbacks(heap_relation),
        )
    };
    if slot.is_null() {
        pgrx::error!("tqhnsw ambuild failed to allocate heap scan slot");
    }

    let snapshot = unsafe { pg_sys::RegisterSnapshot(pg_sys::GetLatestSnapshot()) };
    unsafe { pg_sys::PushActiveSnapshot(snapshot) };
    let scan = unsafe {
        pg_sys::heap_beginscan(
            heap_relation,
            snapshot,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            pg_sys::ScanOptions::SO_TYPE_SEQSCAN
                | pg_sys::ScanOptions::SO_ALLOW_PAGEMODE
                | pg_sys::ScanOptions::SO_ALLOW_STRAT
                | pg_sys::ScanOptions::SO_ALLOW_SYNC,
        )
    };
    if scan.is_null() {
        unsafe {
            pg_sys::UnregisterSnapshot(snapshot);
            pg_sys::ExecDropSingleTupleTableSlot(slot);
        }
        pgrx::error!("tqhnsw ambuild failed to begin heap scan");
    }

    let mut scanned_tuples = 0.0_f64;
    while unsafe {
        pg_sys::heap_getnextslot(scan, pg_sys::ScanDirection::ForwardScanDirection, slot)
    } {
        scanned_tuples += 1.0;
        let heap_tid = unsafe { decode_slot_tid(slot) };
        let vector_datum =
            unsafe { required_slot_datum(slot, index_attnum, "indexed tqvector column") };
        let source_datum =
            unsafe { required_slot_datum(slot, source_attnum, "tqhnsw build_source_column") };
        let source_vector = unsafe {
            Vec::<f32>::from_polymorphic_datum(source_datum, false, pg_sys::FLOAT4ARRAYOID)
        }
        .unwrap_or_else(|| {
            pgrx::error!("tqhnsw build_source_column \"{source_column}\" cannot be NULL")
        });

        let tuple = unsafe { build_heap_tuple_with_source(vector_datum, heap_tid, source_vector) };
        state.push(tuple);
    }

    unsafe {
        pg_sys::heap_endscan(scan);
        pg_sys::PopActiveSnapshot();
        pg_sys::UnregisterSnapshot(snapshot);
        pg_sys::ExecDropSingleTupleTableSlot(slot);
    }
    scanned_tuples
}

unsafe fn source_build_index_attnum(index_info: *mut pg_sys::IndexInfo) -> i32 {
    if index_info.is_null() {
        pgrx::error!("tqhnsw ambuild received a null IndexInfo");
    }
    let index_info = unsafe { &*index_info };
    if index_info.ii_NumIndexAttrs != 1 || index_info.ii_NumIndexKeyAttrs != 1 {
        pgrx::error!("tqhnsw build_source_column currently supports single-column indexes only");
    }
    if !index_info.ii_Expressions.is_null() {
        pgrx::error!("tqhnsw build_source_column does not support expression indexes yet");
    }
    if !index_info.ii_Predicate.is_null() {
        pgrx::error!("tqhnsw build_source_column does not support partial indexes yet");
    }

    let attnum = i32::from(index_info.ii_IndexAttrNumbers[0]);
    if attnum <= 0 {
        pgrx::error!("tqhnsw build_source_column requires a base heap column index key");
    }
    attnum
}

unsafe fn resolve_source_attnum(heap_relation: pg_sys::Relation, source_column: &str) -> i32 {
    let source_column = std::ffi::CString::new(source_column).unwrap_or_else(|_| {
        pgrx::error!("tqhnsw build_source_column contains an invalid NUL byte")
    });
    let attnum = unsafe { pg_sys::get_attnum((*heap_relation).rd_id, source_column.as_ptr()) };
    let attnum = i32::from(attnum);
    if attnum <= 0 {
        pgrx::error!(
            "tqhnsw build_source_column \"{}\" does not name a user column on the heap relation",
            source_column.to_string_lossy()
        );
    }
    attnum
}

unsafe fn decode_slot_tid(slot: *mut pg_sys::TupleTableSlot) -> page::ItemPointer {
    let heap_tid = unsafe { (*slot).tts_tid };
    let tid = pg_sys::ItemPointerData {
        ip_blkid: heap_tid.ip_blkid,
        ip_posid: heap_tid.ip_posid,
    };
    let (block_number, offset_number) = item_pointer_get_both(tid);
    page::ItemPointer {
        block_number,
        offset_number,
    }
}

unsafe fn required_slot_datum(
    slot: *mut pg_sys::TupleTableSlot,
    attnum: i32,
    label: &str,
) -> pg_sys::Datum {
    if unsafe { (*slot).tts_nvalid } < attnum as i16 {
        unsafe { pg_sys::slot_getsomeattrs_int(slot, attnum) };
    }
    let attr_index = usize::try_from(attnum - 1).expect("attribute number should be positive");
    if unsafe { *(*slot).tts_isnull.add(attr_index) } {
        pgrx::error!("tqhnsw does not support NULL {label}");
    }
    unsafe { *(*slot).tts_values.add(attr_index) }
}

unsafe fn flush_build_state(index_relation: pg_sys::Relation, state: &BuildState) {
    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions");
    let bits = state.bits.expect("non-empty build should record bits");
    let mut data_pages = page::DataPageChain::new(state.page_size);
    let mut element_tids = Vec::with_capacity(state.heap_tuples.len());
    let graph_nodes = build_hnsw_graph(state);

    for (idx, tuple) in state.heap_tuples.iter().enumerate() {
        let element_tid = data_pages
            .insert_element(&page::TqElementTuple {
                level: graph_nodes[idx].level,
                deleted: false,
                heaptids: tuple.heap_tids.clone(),
                neighbortid: page::ItemPointer::INVALID,
                code: tuple.code.clone(),
            })
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to stage element tuple: {e}"));
        element_tids.push(element_tid);
    }

    for (idx, element_tid) in element_tids.iter().copied().enumerate() {
        let neighbor_refs = graph_nodes[idx]
            .neighbors
            .iter()
            .map(|neighbor_idx| element_tids[*neighbor_idx])
            .collect::<Vec<_>>();

        let neighbor_tid = data_pages
            .insert_neighbor(&page::TqNeighborTuple {
                count: neighbor_refs.len() as u16,
                tids: neighbor_refs,
            })
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to stage neighbor tuple: {e}"));
        let mut element = data_pages
            .read_element(element_tid, state.heap_tuples[idx].code.len())
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to read staged element tuple: {e}"));
        element.neighbortid = neighbor_tid;
        data_pages
            .update_element(element_tid, &element)
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to backfill element tuple: {e}"));
    }

    let entry_point = choose_entry_point(&element_tids, &graph_nodes, state)
        .unwrap_or(page::ItemPointer::INVALID);
    let max_level = graph_nodes.iter().map(|node| node.level).max().unwrap_or(0);

    unsafe { write_data_pages(index_relation, &data_pages) };
    unsafe {
        initialize_metadata_page(
            index_relation,
            page::MetadataPage {
                m: u16::try_from(state.options.m).expect("validated m should fit into u16"),
                ef_construction: u16::try_from(state.options.ef_construction)
                    .expect("validated ef_construction should fit into u16"),
                entry_point,
                dimensions,
                bits,
                max_level,
                seed: state.seed.expect("non-empty build should record seed"),
            },
        )
    };
}

fn build_hnsw_graph(state: &BuildState) -> Vec<HnswBuildNode> {
    if state.heap_tuples.len() <= 1 {
        return vec![
            HnswBuildNode {
                level: 0,
                neighbors: Vec::new(),
            };
            state.heap_tuples.len()
        ];
    }

    if state
        .heap_tuples
        .iter()
        .all(|tuple| tuple.source_vector.is_some())
    {
        return build_hnsw_graph_from_source(state);
    }

    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions") as usize;
    let bits = state.bits.expect("non-empty build should record bits");
    let seed = state.seed.expect("non-empty build should record seed");
    let m = usize::try_from(state.options.m).expect("validated m should be non-negative");
    let max_level_cap = page::max_level_that_fits(
        u16::try_from(state.options.m).expect("validated m should fit into u16"),
        state.page_size,
    );
    let max_layer = usize::from(max_level_cap).saturating_add(1).max(1);
    let hnsw = Hnsw::new(
        m,
        state.heap_tuples.len(),
        max_layer,
        usize::try_from(state.options.ef_construction)
            .expect("validated ef_construction should be non-negative"),
        BuildCodeDistance::new(dimensions, bits, seed),
    );

    for (origin_id, tuple) in state.heap_tuples.iter().enumerate() {
        hnsw.insert((&tuple.code, origin_id));
    }

    let mut nodes = vec![
        HnswBuildNode {
            level: 0,
            neighbors: Vec::new(),
        };
        state.heap_tuples.len()
    ];
    for point in hnsw.get_point_indexation().get_layer_iterator(0) {
        let origin_id = point.get_origin_id();
        let level = point.get_point_id().0.min(max_level_cap);
        let neighbors = flatten_point_neighbors(origin_id, level, &point.get_neighborhood_id());
        nodes[origin_id] = HnswBuildNode { level, neighbors };
    }

    nodes
}

fn build_hnsw_graph_from_source(state: &BuildState) -> Vec<HnswBuildNode> {
    let m = usize::try_from(state.options.m).expect("validated m should be non-negative");
    let max_level_cap = page::max_level_that_fits(
        u16::try_from(state.options.m).expect("validated m should fit into u16"),
        state.page_size,
    );
    let max_layer = usize::from(max_level_cap).saturating_add(1).max(1);
    let score_offset = state
        .heap_tuples
        .iter()
        .map(|tuple| {
            tuple
                .source_vector
                .as_ref()
                .expect("source graph build requires source vectors")
                .iter()
                .map(|value| value * value)
                .sum::<f32>()
        })
        .fold(0.0_f32, f32::max);
    let hnsw = Hnsw::new(
        m,
        state.heap_tuples.len(),
        max_layer,
        usize::try_from(state.options.ef_construction)
            .expect("validated ef_construction should be non-negative"),
        BuildVectorDistance { score_offset },
    );

    for (origin_id, tuple) in state.heap_tuples.iter().enumerate() {
        let source = tuple
            .source_vector
            .as_ref()
            .expect("source graph build requires source vectors");
        hnsw.insert((source.as_slice(), origin_id));
    }

    let mut nodes = vec![
        HnswBuildNode {
            level: 0,
            neighbors: Vec::new(),
        };
        state.heap_tuples.len()
    ];
    for point in hnsw.get_point_indexation().get_layer_iterator(0) {
        let origin_id = point.get_origin_id();
        let level = point.get_point_id().0.min(max_level_cap);
        let neighbors = flatten_point_neighbors(origin_id, level, &point.get_neighborhood_id());
        nodes[origin_id] = HnswBuildNode { level, neighbors };
    }

    nodes
}

fn build_scored_neighbor_graph(state: &BuildState) -> Vec<Vec<usize>> {
    if state.heap_tuples.len() <= 1 || state.options.m <= 0 {
        return vec![Vec::new(); state.heap_tuples.len()];
    }

    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions") as usize;
    let bits = state.bits.expect("non-empty build should record bits");
    let seed = state.seed.expect("non-empty build should record seed");
    let max_degree = usize::try_from(state.options.m)
        .expect("validated m should be non-negative")
        .min(state.heap_tuples.len() - 1);
    let mut graph = Vec::with_capacity(state.heap_tuples.len());

    for (idx, tuple) in state.heap_tuples.iter().enumerate() {
        let mut candidates = state
            .heap_tuples
            .iter()
            .enumerate()
            .filter(|(candidate_idx, _)| *candidate_idx != idx)
            .map(|(candidate_idx, candidate)| {
                (
                    candidate_idx,
                    crate::score_code_inner_product(
                        dimensions,
                        bits,
                        seed,
                        &tuple.code,
                        &candidate.code,
                    ),
                )
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|(left_idx, left_score), (right_idx, right_score)| {
            right_score
                .total_cmp(left_score)
                .then_with(|| left_idx.cmp(right_idx))
        });
        graph.push(
            candidates
                .into_iter()
                .take(max_degree)
                .map(|(candidate_idx, _)| candidate_idx)
                .collect(),
        );
    }

    graph
}

fn flatten_point_neighbors(
    origin_id: usize,
    level: u8,
    neighbors_per_layer: &[Vec<hnsw_rs::hnsw::Neighbour>],
) -> Vec<usize> {
    let mut seen = HashSet::new();
    let mut flattened = Vec::new();

    for layer in 0..=usize::from(level) {
        if let Some(layer_neighbors) = neighbors_per_layer.get(layer) {
            for neighbor in layer_neighbors {
                if neighbor.d_id != origin_id && seen.insert(neighbor.d_id) {
                    flattened.push(neighbor.d_id);
                }
            }
        }
    }

    flattened
}

fn score_source_inner_product(left: &[f32], right: &[f32]) -> f32 {
    left.iter().zip(right.iter()).map(|(l, r)| l * r).sum()
}

fn choose_entry_point(
    element_tids: &[page::ItemPointer],
    graph_nodes: &[HnswBuildNode],
    state: &BuildState,
) -> Option<page::ItemPointer> {
    if element_tids.is_empty() {
        return None;
    }

    let max_level = graph_nodes.iter().map(|node| node.level).max().unwrap_or(0);
    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions") as usize;
    let bits = state.bits.expect("non-empty build should record bits");
    let seed = state.seed.expect("non-empty build should record seed");

    (0..state.heap_tuples.len())
        .filter(|idx| graph_nodes[*idx].level == max_level)
        .max_by(|left_idx, right_idx| {
            compare_entry_point_candidates(
                *left_idx,
                *right_idx,
                graph_nodes,
                state,
                dimensions,
                bits,
                seed,
            )
        })
        .map(|idx| element_tids[idx])
}

fn compare_entry_point_candidates(
    left_idx: usize,
    right_idx: usize,
    graph_nodes: &[HnswBuildNode],
    state: &BuildState,
    dimensions: usize,
    bits: u8,
    seed: u64,
) -> Ordering {
    let left_score = entry_point_score(left_idx, graph_nodes, state, dimensions, bits, seed);
    let right_score = entry_point_score(right_idx, graph_nodes, state, dimensions, bits, seed);
    left_score
        .total_cmp(&right_score)
        .then_with(|| right_idx.cmp(&left_idx))
}

fn entry_point_score(
    idx: usize,
    graph_nodes: &[HnswBuildNode],
    state: &BuildState,
    dimensions: usize,
    bits: u8,
    seed: u64,
) -> f32 {
    let source_vectors = state
        .heap_tuples
        .iter()
        .all(|tuple| tuple.source_vector.is_some());
    graph_nodes[idx]
        .neighbors
        .iter()
        .map(|neighbor_idx| {
            if source_vectors {
                score_source_inner_product(
                    state.heap_tuples[idx]
                        .source_vector
                        .as_ref()
                        .expect("source-scored entry point requires source vectors"),
                    state.heap_tuples[*neighbor_idx]
                        .source_vector
                        .as_ref()
                        .expect("source-scored entry point requires source vectors"),
                )
            } else {
                crate::score_code_inner_product(
                    dimensions,
                    bits,
                    seed,
                    &state.heap_tuples[idx].code,
                    &state.heap_tuples[*neighbor_idx].code,
                )
            }
        })
        .sum()
}

unsafe fn write_data_pages(index_relation: pg_sys::Relation, data_pages: &page::DataPageChain) {
    for staged_page in data_pages.pages() {
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                P_NEW,
                pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
                ptr::null_mut(),
            )
        };
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            pgrx::error!(
                "tqhnsw failed to allocate data buffer for block {}",
                staged_page.block_number()
            );
        }

        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
        let page_ptr =
            unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
        unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };

        for tuple in staged_page.tuples() {
            let offset = unsafe {
                pg_sys::PageAddItemExtended(
                    page_ptr,
                    tuple.as_ptr().cast_mut().cast(),
                    tuple.len(),
                    pg_sys::InvalidOffsetNumber,
                    0,
                )
            };
            if offset == pg_sys::InvalidOffsetNumber {
                pgrx::error!(
                    "tqhnsw failed to write tuple to block {}",
                    staged_page.block_number()
                );
            }
        }

        unsafe { wal_txn.finish() };
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DebugIndexDataPage {
    pub block_number: u32,
    pub tuples: Vec<Vec<u8>>,
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_index_pages(
    index_oid: pg_sys::Oid,
) -> (u32, page::MetadataPage, Vec<DebugIndexDataPage>) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };

    let metadata = unsafe { read_metadata_page(index_relation) };
    let mut data_pages = Vec::new();
    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        data_pages.push(unsafe { read_data_page(index_relation, block_number) });
    }

    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (block_count, metadata, data_pages)
}

unsafe fn read_metadata_page(index_relation: pg_sys::Relation) -> page::MetadataPage {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let raw_page = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let page_bytes = unsafe { std::slice::from_raw_parts(raw_page, page_size) };
    let metadata =
        page::MetadataPage::decode_page(page_bytes).expect("metadata page should decode");
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    metadata
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn read_data_page(
    index_relation: pg_sys::Relation,
    block_number: u32,
) -> DebugIndexDataPage {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let raw_page = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let page_header = raw_page.cast::<pg_sys::PageHeaderData>();
    let line_pointer_count = ((unsafe { (*page_header).pd_lower } as usize
        - size_of::<pg_sys::PageHeaderData>())
        / size_of::<pg_sys::ItemIdData>()) as u16;

    let mut tuples = Vec::with_capacity(line_pointer_count as usize);
    for offset in 1..=line_pointer_count {
        let item_id_ptr = unsafe {
            raw_page
                .add(
                    page::PAGE_HEADER_BYTES
                        + ((offset - 1) as usize * size_of::<pg_sys::ItemIdData>()),
                )
                .cast::<pg_sys::ItemIdData>()
        };
        let item_id = unsafe { &*item_id_ptr };
        if item_id.lp_flags() == 0 {
            continue;
        }
        let tuple_offset = item_id.lp_off() as usize;
        let tuple_len = item_id.lp_len() as usize;
        if tuple_offset + tuple_len > page_size {
            pgrx::error!("tqhnsw debug read found invalid tuple bounds on block {block_number}");
        }
        tuples.push(
            unsafe { std::slice::from_raw_parts(raw_page.add(tuple_offset), tuple_len) }.to_vec(),
        );
    }

    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    DebugIndexDataPage {
        block_number,
        tuples,
    }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_index_metadata(
    index_oid: pg_sys::Oid,
) -> (u32, i32, i32, page::MetadataPage) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let options = unsafe { relation_options(index_relation) };
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let metadata = unsafe { read_metadata_page(index_relation) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };

    (block_count, options.m, options.ef_construction, metadata)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_vacuum_stats(index_oid: pg_sys::Oid) -> pg_sys::IndexBulkDeleteResult {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let mut info = PgBox::<pg_sys::IndexVacuumInfo>::alloc0();
    info.index = index_relation;
    let info_ptr = (&mut *info) as *mut pg_sys::IndexVacuumInfo;

    let stats = unsafe { tqhnsw_ambulkdelete(info_ptr, ptr::null_mut(), None, ptr::null_mut()) };
    let stats = unsafe { tqhnsw_amvacuumcleanup(info_ptr, stats) };
    let result = unsafe { *stats };

    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_begin_end_scan(index_oid: pg_sys::Oid) -> (bool, bool) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };
    let has_opaque = unsafe { !(*scan).opaque.is_null() };

    unsafe { tqhnsw_amendscan(scan) };
    let cleared_opaque = unsafe { (*scan).opaque.is_null() };

    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (has_opaque, cleared_opaque)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_end_scan_twice(index_oid: pg_sys::Oid) -> (bool, bool, bool) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };
    let has_opaque = unsafe { !(*scan).opaque.is_null() };

    unsafe { tqhnsw_amendscan(scan) };
    let cleared_after_first = unsafe { (*scan).opaque.is_null() };

    unsafe { tqhnsw_amendscan(scan) };
    let cleared_after_second = unsafe { (*scan).opaque.is_null() };

    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (has_opaque, cleared_after_first, cleared_after_second)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_query_dimensions(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (bool, u16, Vec<f32>, u16, u8, usize, u32, bool, usize, usize) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let result = (
        opaque.rescan_called,
        opaque.query_dimensions,
        read_scan_query(opaque),
        opaque.scan_dimensions,
        opaque.scan_bits,
        opaque.scan_code_len,
        opaque.scan_block_count,
        !opaque.prepared_query.is_null(),
        opaque
            .prepared_query
            .as_ref()
            .map(|prepared| prepared.lut.len())
            .unwrap_or(0),
        opaque
            .prepared_query
            .as_ref()
            .map(|prepared| prepared.sq.len())
            .unwrap_or(0),
    );

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_overwrites_query_dimensions(
    index_oid: pg_sys::Oid,
    first_query: Vec<f32>,
    second_query: Vec<f32>,
) -> (bool, u16, Vec<f32>, u16, u8, usize, u32, bool, usize, usize) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut first_orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(first_query)
            .expect("first query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut first_orderby, 1) };

    let mut second_orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(second_query)
            .expect("second query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut second_orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let result = (
        opaque.rescan_called,
        opaque.query_dimensions,
        read_scan_query(opaque),
        opaque.scan_dimensions,
        opaque.scan_bits,
        opaque.scan_code_len,
        opaque.scan_block_count,
        !opaque.prepared_query.is_null(),
        opaque
            .prepared_query
            .as_ref()
            .map(|prepared| prepared.lut.len())
            .unwrap_or(0),
        opaque
            .prepared_query
            .as_ref()
            .map(|prepared| prepared.sq.len())
            .unwrap_or(0),
    );

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_null_query(index_oid: pg_sys::Oid) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_flags: pg_sys::SK_ISNULL as i32,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_with_index_qual(index_oid: pg_sys::Oid, query: Vec<f32>) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 1, 1) };

    let mut key = pg_sys::ScanKeyData::default();
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, &mut key, 1, &mut orderby, 1) };
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_with_multiple_orderbys(index_oid: pg_sys::Oid, query: Vec<f32>) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 2) };

    let datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderbys = [
        pg_sys::ScanKeyData {
            sk_argument: datum,
            ..Default::default()
        },
        pg_sys::ScanKeyData {
            sk_argument: datum,
            ..Default::default()
        },
    ];
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, orderbys.as_mut_ptr(), 2) };
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_without_rescan(index_oid: pg_sys::Oid) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) };
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_after_rescan(index_oid: pg_sys::Oid, query: Vec<f32>) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };
    unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) };
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_after_rescan_result(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> bool {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };
    let result = unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) };

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_scan_heap_tids(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> Vec<HeapTidCoords> {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let mut tids = Vec::new();
    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {
        let (block_number, offset_number) =
            pgrx::itemptr::item_pointer_get_both(unsafe { (*scan).xs_heaptid });
        tids.push((block_number, offset_number));
    }

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    tids
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_exhaustion_state(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (Vec<HeapTidCoords>, bool, bool) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let mut tids = Vec::new();
    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {
        tids.push(item_pointer_get_both(unsafe { (*scan).xs_heaptid }));
    }

    let exhausted_once = unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) };
    let exhausted_twice = unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) };

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (tids, exhausted_once, exhausted_twice)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_current_result_state(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (bool, HeapTidCoords, bool, bool, HeapTidCoords, bool) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let before_found = opaque.current_result_tid != page::ItemPointer::INVALID;
    let before_tid = (
        opaque.current_result_tid.block_number,
        opaque.current_result_tid.offset_number,
    );
    let before_score = opaque.current_result_score_valid;

    let found = unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) };
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let after_tid = (
        opaque.current_result_tid.block_number,
        opaque.current_result_tid.offset_number,
    );
    let after_score = opaque.current_result_score_valid;

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (before_found, before_tid, before_score, found, after_tid, after_score)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_backward_after_rescan(index_oid: pg_sys::Oid, query: Vec<f32>) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };
    unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::BackwardScanDirection) };
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_rescan_after_exhaustion(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (Vec<HeapTidCoords>, Vec<HeapTidCoords>) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let mut first_pass = Vec::new();
    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {
        first_pass.push(item_pointer_get_both(unsafe { (*scan).xs_heaptid }));
    }

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1) };

    let mut rescanned = Vec::new();
    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {
        rescanned.push(item_pointer_get_both(unsafe { (*scan).xs_heaptid }));
    }

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (first_pass, rescanned)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_rescan_after_partial(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (HeapTidCoords, Vec<HeapTidCoords>) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let found_first =
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) };
    assert!(
        found_first,
        "partial scan should yield at least one heap tid"
    );
    let first_tid = item_pointer_get_both(unsafe { (*scan).xs_heaptid });

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1) };

    let mut tids = Vec::new();
    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {
        tids.push(item_pointer_get_both(unsafe { (*scan).xs_heaptid }));
    }

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (first_tid, tids)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded_code(vector: &[f32], bits: u8, seed: u64) -> Vec<u8> {
        let quantizer = crate::quant::prod::ProdQuantizer::cached(vector.len(), bits, seed);
        let encoded = quantizer.encode(vector);
        let mut code = encoded.mse_packed;
        code.extend_from_slice(&encoded.qjl_packed);
        code
    }

    #[test]
    fn scored_neighbor_graph_prefers_similarity_over_insert_order() {
        let seed = 42_u64;
        let bits = 8_u8;
        let tuples = vec![
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 1,
                }],
                dimensions: 8,
                bits,
                seed,
                code: encoded_code(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 2,
                }],
                dimensions: 8,
                bits,
                seed,
                code: encoded_code(&[0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 3,
                }],
                dimensions: 8,
                bits,
                seed,
                code: encoded_code(&[0.98, 0.02, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
        ];
        let state = BuildState {
            options: TqHnswOptions {
                m: 1,
                ef_construction: 32,
                build_source_column: None,
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 3,
            heap_tuples: tuples,
            dimensions: Some(8),
            bits: Some(bits),
            seed: Some(seed),
        };

        let graph = build_scored_neighbor_graph(&state);

        assert_eq!(graph.len(), 3);
        assert_eq!(graph[0], vec![2]);
        assert_eq!(graph[2], vec![0]);
    }

    #[test]
    fn hnsw_graph_builds_for_small_dataset() {
        let seed = 42_u64;
        let bits = 4_u8;
        let tuples = vec![
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 1,
                }],
                dimensions: 4,
                bits,
                seed,
                code: encoded_code(&[1.0, 0.0, 0.5, -1.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 2,
                }],
                dimensions: 4,
                bits,
                seed,
                code: encoded_code(&[0.0, 1.0, 0.25, -0.5], bits, seed),
                source_vector: None,
                source_count: 0,
            },
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 3,
                }],
                dimensions: 4,
                bits,
                seed,
                code: encoded_code(&[-1.0, 0.5, 0.0, 1.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
        ];
        let state = BuildState {
            options: TqHnswOptions {
                m: 10,
                ef_construction: 90,
                build_source_column: None,
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 3,
            heap_tuples: tuples,
            dimensions: Some(4),
            bits: Some(bits),
            seed: Some(seed),
        };

        let nodes = build_hnsw_graph(&state);

        assert_eq!(nodes.len(), 3);
        assert!(nodes.iter().any(|node| !node.neighbors.is_empty()));
    }

    #[test]
    fn source_scored_entry_point_prefers_raw_vectors() {
        let seed = 42_u64;
        let bits = 4_u8;
        let state = BuildState {
            options: TqHnswOptions {
                m: 2,
                ef_construction: 64,
                build_source_column: Some("source".to_owned()),
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 3,
            heap_tuples: vec![
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: 1,
                    }],
                    dimensions: 2,
                    bits,
                    seed,
                    code: vec![0x00, 0x00],
                    source_vector: Some(vec![1.0, 0.0]),
                    source_count: 1,
                },
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: 2,
                    }],
                    dimensions: 2,
                    bits,
                    seed,
                    code: vec![0xff, 0xff],
                    source_vector: Some(vec![0.9, 0.1]),
                    source_count: 1,
                },
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: 3,
                    }],
                    dimensions: 2,
                    bits,
                    seed,
                    code: vec![0x00, 0x01],
                    source_vector: Some(vec![-1.0, 0.0]),
                    source_count: 1,
                },
            ],
            dimensions: Some(2),
            bits: Some(bits),
            seed: Some(seed),
        };

        let graph_nodes = vec![
            HnswBuildNode {
                level: 0,
                neighbors: vec![1],
            },
            HnswBuildNode {
                level: 0,
                neighbors: vec![2],
            },
            HnswBuildNode {
                level: 0,
                neighbors: vec![1],
            },
        ];
        let element_tids = vec![
            page::ItemPointer {
                block_number: 2,
                offset_number: 1,
            },
            page::ItemPointer {
                block_number: 2,
                offset_number: 2,
            },
            page::ItemPointer {
                block_number: 2,
                offset_number: 3,
            },
        ];

        let entry_point = choose_entry_point(&element_tids, &graph_nodes, &state)
            .expect("entry point should exist");
        assert_eq!(entry_point, element_tids[0]);
    }

    #[test]
    fn average_source_representative_weights_by_duplicate_count() {
        let mut representative = vec![1.0, 0.0];
        average_source_representatives(&mut representative, 1, &[0.0, 1.0], 1);
        assert_eq!(representative, vec![0.5, 0.5]);

        average_source_representatives(&mut representative, 2, &[1.0, 1.0], 2);
        assert_eq!(representative, vec![0.75, 0.75]);
    }
}
