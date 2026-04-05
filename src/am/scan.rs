use std::ptr;

use pgrx::{pg_sys, FromDatum, PgBox};

use crate::quant::prod::PreparedQuery;

use super::graph;
use super::page;

pub(super) unsafe extern "C-unwind" fn tqhnsw_ambeginscan(
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

pub(super) unsafe extern "C-unwind" fn tqhnsw_amrescan(
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

            let metadata = super::read_metadata_page((*scan).indexRelation);
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
            opaque.scan_seed = metadata.seed;
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
            initialize_scan_entry_candidate(
                (*scan).indexRelation,
                (*scan).heapRelation,
                opaque,
                &metadata,
            );
        })
    }
}

pub(super) unsafe extern "C-unwind" fn tqhnsw_amgettuple(
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
            if let Some(heap_tid) = next_linear_scan_heap_tid(
                (*scan).indexRelation,
                (*scan).heapRelation,
                opaque,
                opaque.scan_code_len,
            ) {
                set_scan_heap_tid(scan, heap_tid);
                return true;
            }

            false
        })
    }
}

pub(super) unsafe extern "C-unwind" fn tqhnsw_amendscan(scan: pg_sys::IndexScanDesc) {
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
    clear_scan_candidate_state(opaque);
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
    update_current_scan_result_heap_tid(opaque, tid);
    Some(tid)
}

fn clear_scan_result_state(opaque: &mut TqScanOpaque) {
    opaque.current_result = CurrentScanResult::default();
}

fn clear_scan_candidate_state(opaque: &mut TqScanOpaque) {
    opaque.candidate_frontier = CandidateFrontier::default();
    opaque.candidate_frontier_head = u8::MAX;
}

unsafe fn initialize_scan_entry_candidate(
    index_relation: pg_sys::Relation,
    _heap_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    metadata: &page::MetadataPage,
) {
    clear_scan_candidate_state(opaque);
    if metadata.dimensions == 0 || metadata.entry_point == page::ItemPointer::INVALID {
        return;
    }

    let entry = unsafe { graph::load_graph_element(index_relation, metadata.entry_point, opaque.scan_code_len) };
    if entry.deleted || entry.heaptids.is_empty() {
        return;
    }

    opaque.candidate_frontier.candidates[0] = ScanCandidate {
        element_tid: entry.tid,
        score: score_scan_element_result(opaque, entry.gamma, &entry.code),
        score_valid: true,
    };

    let (_, neighbors) = unsafe { graph::load_graph_adjacency(index_relation, entry.tid, opaque.scan_code_len) };
    if let Some(candidate) = select_successor_candidate(&neighbors.tids, |neighbor_tid| {
        let neighbor = unsafe {
            graph::load_graph_element(index_relation, neighbor_tid, opaque.scan_code_len)
        };
        if neighbor.deleted || neighbor.heaptids.is_empty() {
            return None;
        }

        Some(ScanCandidate {
            element_tid: neighbor.tid,
            score: score_scan_element_result(opaque, neighbor.gamma, &neighbor.code),
            score_valid: true,
        })
    }) {
        opaque.candidate_frontier.candidates[1] = candidate;
    }

    recompute_candidate_frontier_head(opaque);
}

fn recompute_candidate_frontier_head(opaque: &mut TqScanOpaque) {
    let candidates = &opaque.candidate_frontier.candidates;
    opaque.candidate_frontier_head = match (
        candidates[0].score_valid,
        candidates[1].score_valid,
    ) {
        (false, false) => u8::MAX,
        (true, false) => 0,
        (false, true) => 1,
        (true, true) => {
            if candidates[0].score <= candidates[1].score {
                0
            } else {
                1
            }
        }
    };
}

fn consume_candidate_frontier_head(opaque: &mut TqScanOpaque) -> Option<ScanCandidate> {
    let head = match opaque.candidate_frontier_head {
        0 | 1 => opaque.candidate_frontier_head as usize,
        u8::MAX => return None,
        other => {
            debug_assert!(
                false,
                "candidate frontier head should only be 0, 1, or u8::MAX; got {other}"
            );
            return None;
        }
    };

    let consumed = opaque.candidate_frontier.candidates[head];
    opaque.candidate_frontier.candidates[head] = ScanCandidate::default();
    recompute_candidate_frontier_head(opaque);
    Some(consumed)
}

fn select_successor_candidate<F>(
    neighbor_tids: &[page::ItemPointer],
    mut candidate_for_tid: F,
) -> Option<ScanCandidate>
where
    F: FnMut(page::ItemPointer) -> Option<ScanCandidate>,
{
    for neighbor_tid in neighbor_tids.iter().copied() {
        if neighbor_tid == page::ItemPointer::INVALID {
            continue;
        }

        if let Some(candidate) = candidate_for_tid(neighbor_tid) {
            return Some(candidate);
        }
    }

    None
}

unsafe fn next_linear_scan_heap_tid(
    index_relation: pg_sys::Relation,
    _heap_relation: pg_sys::Relation,
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
        clear_scan_candidate_state(opaque);
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
        let line_pointer_count = super::page_line_pointer_count(page_ptr);
        let offset_start = if block_number == opaque.next_block_number {
            opaque.next_offset_number.max(1)
        } else {
            1
        };

        for offset in offset_start..=line_pointer_count {
            let item_id = unsafe { &*super::page_item_id(page_ptr, offset) };
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
            debug_assert!(
                offset < u16::MAX,
                "scan offset should fit in page-local u16 range"
            );
            opaque.next_offset_number = offset + 1;
            set_current_scan_result(
                opaque,
                page::ItemPointer {
                    block_number,
                    offset_number: offset,
                },
                score_scan_element_result(opaque, element.gamma, &element.code),
            );

            store_pending_scan_heaptids(opaque, &element.heaptids);
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            return take_pending_scan_heap_tid(opaque);
        }

        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        opaque.next_block_number = block_number + 1;
        opaque.next_offset_number = 1;
    }

    opaque.scan_exhausted = true;
    clear_scan_candidate_state(opaque);
    clear_scan_result_state(opaque);
    None
}

unsafe fn score_scan_element_result(opaque: &TqScanOpaque, gamma: f32, code_bytes: &[u8]) -> f32 {
    if opaque.prepared_query.is_null() {
        pgrx::error!("tqhnsw scan scoring requires a prepared query");
    }

    let quantizer = crate::quant::prod::ProdQuantizer::cached(
        opaque.scan_dimensions as usize,
        opaque.scan_bits,
        opaque.scan_seed,
    );
    let prepared_query = unsafe { &*opaque.prepared_query };
    -quantizer.score_ip_from_parts(prepared_query, gamma, code_bytes)
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

fn set_current_scan_result(
    opaque: &mut TqScanOpaque,
    element_tid: page::ItemPointer,
    score: f32,
) {
    opaque.current_result = CurrentScanResult {
        element_tid,
        heap_tid: page::ItemPointer::INVALID,
        score,
        score_valid: true,
    };
}

fn update_current_scan_result_heap_tid(opaque: &mut TqScanOpaque, heap_tid: page::ItemPointer) {
    if opaque.current_result.element_tid != page::ItemPointer::INVALID {
        opaque.current_result.heap_tid = heap_tid;
    }
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn read_scan_query(opaque: &TqScanOpaque) -> Vec<f32> {
    if opaque.query_values.is_null() || opaque.query_dimensions == 0 {
        return Vec::new();
    }

    let query =
        unsafe { std::slice::from_raw_parts(opaque.query_values, opaque.query_dimensions as usize) };
    query.to_vec()
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct CurrentScanResult {
    element_tid: page::ItemPointer,
    heap_tid: page::ItemPointer,
    score: f32,
    score_valid: bool,
}

impl Default for CurrentScanResult {
    fn default() -> Self {
        Self {
            element_tid: page::ItemPointer::INVALID,
            heap_tid: page::ItemPointer::INVALID,
            score: 0.0,
            score_valid: false,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ScanCandidate {
    element_tid: page::ItemPointer,
    score: f32,
    score_valid: bool,
}

impl Default for ScanCandidate {
    fn default() -> Self {
        Self {
            element_tid: page::ItemPointer::INVALID,
            score: 0.0,
            score_valid: false,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct CandidateFrontier {
    candidates: [ScanCandidate; 2],
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
    scan_seed: u64,
    scan_code_len: usize,
    scan_block_count: u32,
    candidate_frontier: CandidateFrontier,
    candidate_frontier_head: u8,
    current_result: CurrentScanResult,
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
            scan_seed: 0,
            scan_code_len: 0,
            scan_block_count: 0,
            candidate_frontier: CandidateFrontier::default(),
            candidate_frontier_head: u8::MAX,
            current_result: CurrentScanResult::default(),
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
pub(crate) type HeapTidCoords = (u32, u16);

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateSlot = (bool, HeapTidCoords, f32);

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontier = [DebugCandidateSlot; 2];

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontierLifecycle = (
    u8,
    DebugCandidateFrontier,
    u8,
    DebugCandidateFrontier,
    u8,
    DebugCandidateFrontier,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontierConsume =
    (u8, DebugCandidateFrontier, u8, DebugCandidateFrontier, u8, DebugCandidateFrontier);

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
        tids.push(pgrx::itemptr::item_pointer_get_both(unsafe { (*scan).xs_heaptid }));
    }

    let exhausted_once =
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) };
    let exhausted_twice =
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) };

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (tids, exhausted_once, exhausted_twice)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_current_result_state(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (bool, HeapTidCoords, bool, f32, bool, HeapTidCoords, bool, f32) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let before_found = opaque.current_result.element_tid != page::ItemPointer::INVALID;
    let before_tid = (
        opaque.current_result.element_tid.block_number,
        opaque.current_result.element_tid.offset_number,
    );
    let before_score = opaque.current_result.score_valid;
    let before_score_value = opaque.current_result.score;

    let found = unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) };
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let after_tid = (
        opaque.current_result.element_tid.block_number,
        opaque.current_result.element_tid.offset_number,
    );
    let after_score = opaque.current_result.score_valid;
    let after_score_value = opaque.current_result.score;

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        before_found,
        before_tid,
        before_score,
        before_score_value,
        found,
        after_tid,
        after_score,
        after_score_value,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_entry_candidate_state(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (bool, HeapTidCoords, f32, bool, HeapTidCoords, f32) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let before_valid = opaque.candidate_frontier.candidates[0].score_valid;
    let before_tid = (
        opaque.candidate_frontier.candidates[0].element_tid.block_number,
        opaque.candidate_frontier.candidates[0].element_tid.offset_number,
    );
    let before_score = opaque.candidate_frontier.candidates[0].score;

    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {}

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let after_valid = opaque.candidate_frontier.candidates[0].score_valid;
    let after_tid = (
        opaque.candidate_frontier.candidates[0].element_tid.block_number,
        opaque.candidate_frontier.candidates[0].element_tid.offset_number,
    );
    let after_score = opaque.candidate_frontier.candidates[0].score;

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        before_valid,
        before_tid,
        before_score,
        after_valid,
        after_tid,
        after_score,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_successor_candidate_state(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (
    HeapTidCoords,
    Vec<HeapTidCoords>,
    bool,
    HeapTidCoords,
    f32,
) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let metadata = unsafe { super::read_metadata_page(index_relation) };
    let entry_tid = (metadata.entry_point.block_number, metadata.entry_point.offset_number);
    let entry_neighbors = unsafe { super::debug_entry_point_neighbor_tids(index_oid) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let successor_valid = opaque.candidate_frontier.candidates[1].score_valid;
    let successor_tid = (
        opaque.candidate_frontier.candidates[1].element_tid.block_number,
        opaque.candidate_frontier.candidates[1].element_tid.offset_number,
    );
    let successor_score = opaque.candidate_frontier.candidates[1].score;

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        entry_tid,
        entry_neighbors,
        successor_valid,
        successor_tid,
        successor_score,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_candidate_frontier(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (u8, DebugCandidateFrontier) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let frontier = opaque.candidate_frontier.candidates.map(|candidate| {
        (
            candidate.score_valid,
            (candidate.element_tid.block_number, candidate.element_tid.offset_number),
            candidate.score,
        )
    });
    let head = opaque.candidate_frontier_head;

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (head, frontier)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_candidate_frontier_head_lifecycle(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugCandidateFrontierLifecycle {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let before_head = opaque.candidate_frontier_head;
    let before_frontier = opaque.candidate_frontier.candidates.map(|candidate| {
        (
            candidate.score_valid,
            (candidate.element_tid.block_number, candidate.element_tid.offset_number),
            candidate.score,
        )
    });

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "frontier-head lifecycle helper requires a first tuple"
    );
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let partial_head = opaque.candidate_frontier_head;
    let partial_frontier = opaque.candidate_frontier.candidates.map(|candidate| {
        (
            candidate.score_valid,
            (candidate.element_tid.block_number, candidate.element_tid.offset_number),
            candidate.score,
        )
    });

    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {}

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let exhausted_head = opaque.candidate_frontier_head;
    let exhausted_frontier = opaque.candidate_frontier.candidates.map(|candidate| {
        (
            candidate.score_valid,
            (candidate.element_tid.block_number, candidate.element_tid.offset_number),
            candidate.score,
        )
    });

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        before_head,
        before_frontier,
        partial_head,
        partial_frontier,
        exhausted_head,
        exhausted_frontier,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_consume_candidate_frontier_head(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugCandidateFrontierConsume {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &mut *(*scan).opaque.cast::<TqScanOpaque>() };
    let before_head = opaque.candidate_frontier_head;
    let before_frontier = opaque.candidate_frontier.candidates.map(|candidate| {
        (
            candidate.score_valid,
            (candidate.element_tid.block_number, candidate.element_tid.offset_number),
            candidate.score,
        )
    });

    let first_consumed = consume_candidate_frontier_head(opaque);
    debug_assert_eq!(first_consumed.is_some(), before_head != u8::MAX);
    let after_first_head = opaque.candidate_frontier_head;
    let after_first_frontier = opaque.candidate_frontier.candidates.map(|candidate| {
        (
            candidate.score_valid,
            (candidate.element_tid.block_number, candidate.element_tid.offset_number),
            candidate.score,
        )
    });

    consume_candidate_frontier_head(opaque);
    let after_second_head = opaque.candidate_frontier_head;
    let after_second_frontier = opaque.candidate_frontier.candidates.map(|candidate| {
        (
            candidate.score_valid,
            (candidate.element_tid.block_number, candidate.element_tid.offset_number),
            candidate.score,
        )
    });

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        before_head,
        before_frontier,
        after_first_head,
        after_first_frontier,
        after_second_head,
        after_second_frontier,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_entry_candidate_lifecycle(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (
    bool,
    HeapTidCoords,
    f32,
    bool,
    HeapTidCoords,
    f32,
    bool,
    HeapTidCoords,
    f32,
) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let before_valid = opaque.candidate_frontier.candidates[0].score_valid;
    let before_tid = (
        opaque.candidate_frontier.candidates[0].element_tid.block_number,
        opaque.candidate_frontier.candidates[0].element_tid.offset_number,
    );
    let before_score = opaque.candidate_frontier.candidates[0].score;

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "entry-candidate lifecycle helper requires a first tuple"
    );
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let partial_valid = opaque.candidate_frontier.candidates[0].score_valid;
    let partial_tid = (
        opaque.candidate_frontier.candidates[0].element_tid.block_number,
        opaque.candidate_frontier.candidates[0].element_tid.offset_number,
    );
    let partial_score = opaque.candidate_frontier.candidates[0].score;

    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {}

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let exhausted_valid = opaque.candidate_frontier.candidates[0].score_valid;
    let exhausted_tid = (
        opaque.candidate_frontier.candidates[0].element_tid.block_number,
        opaque.candidate_frontier.candidates[0].element_tid.offset_number,
    );
    let exhausted_score = opaque.candidate_frontier.candidates[0].score;

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        before_valid,
        before_tid,
        before_score,
        partial_valid,
        partial_tid,
        partial_score,
        exhausted_valid,
        exhausted_tid,
        exhausted_score,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_current_result_lifecycle(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (
    HeapTidCoords,
    HeapTidCoords,
    bool,
    f32,
    HeapTidCoords,
    bool,
    f32,
    HeapTidCoords,
    bool,
) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "first tuple production should succeed for lifecycle debug helper"
    );
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let first_tid = (
        opaque.current_result.element_tid.block_number,
        opaque.current_result.element_tid.offset_number,
    );

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "second tuple production should succeed for duplicate-drain lifecycle debug helper"
    );
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let second_tid = (
        opaque.current_result.element_tid.block_number,
        opaque.current_result.element_tid.offset_number,
    );
    let second_score = opaque.current_result.score_valid;
    let second_score_value = opaque.current_result.score;

    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {}

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let exhausted_tid = (
        opaque.current_result.element_tid.block_number,
        opaque.current_result.element_tid.offset_number,
    );
    let exhausted_score = opaque.current_result.score_valid;
    let exhausted_score_value = opaque.current_result.score;

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let rescanned_tid = (
        opaque.current_result.element_tid.block_number,
        opaque.current_result.element_tid.offset_number,
    );
    let rescanned_score = opaque.current_result.score_valid;

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        first_tid,
        second_tid,
        second_score,
        second_score_value,
        exhausted_tid,
        exhausted_score,
        exhausted_score_value,
        rescanned_tid,
        rescanned_score,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_current_result_neighbors(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (HeapTidCoords, usize) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };
    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "neighbor debug helper requires a non-empty scan result"
    );

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let current_result_tid = (
        opaque.current_result.element_tid.block_number,
        opaque.current_result.element_tid.offset_number,
    );
    let (_element, neighbors) = unsafe {
        graph::load_graph_adjacency(
            index_relation,
            opaque.current_result.element_tid,
            opaque.scan_code_len,
        )
    };

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (current_result_tid, neighbors.tids.len())
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_current_result_heap_progress(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (HeapTidCoords, HeapTidCoords, HeapTidCoords, f32, f32) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "heap-progress debug helper requires a first tuple"
    );
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let first_heap_tid = (
        opaque.current_result.heap_tid.block_number,
        opaque.current_result.heap_tid.offset_number,
    );
    let element_tid = (
        opaque.current_result.element_tid.block_number,
        opaque.current_result.element_tid.offset_number,
    );
    let first_score = opaque.current_result.score;

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "heap-progress debug helper requires a duplicate tuple"
    );
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let second_heap_tid = (
        opaque.current_result.heap_tid.block_number,
        opaque.current_result.heap_tid.offset_number,
    );
    let second_score = opaque.current_result.score;

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        element_tid,
        first_heap_tid,
        second_heap_tid,
        first_score,
        second_score,
    )
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
        first_pass.push(pgrx::itemptr::item_pointer_get_both(unsafe { (*scan).xs_heaptid }));
    }

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1) };

    let mut rescanned = Vec::new();
    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {
        rescanned.push(pgrx::itemptr::item_pointer_get_both(unsafe { (*scan).xs_heaptid }));
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
    assert!(found_first, "partial scan should yield at least one heap tid");
    let first_tid = pgrx::itemptr::item_pointer_get_both(unsafe { (*scan).xs_heaptid });

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1) };

    let mut tids = Vec::new();
    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {
        tids.push(pgrx::itemptr::item_pointer_get_both(unsafe { (*scan).xs_heaptid }));
    }

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (first_tid, tids)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_entry_point_neighbor_tids(index_oid: pg_sys::Oid) -> Vec<HeapTidCoords> {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let metadata = unsafe { super::read_metadata_page(index_relation) };
    if metadata.entry_point == page::ItemPointer::INVALID || metadata.dimensions == 0 {
        unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
        return Vec::new();
    }

    let code_len = crate::code_len(metadata.dimensions as usize, metadata.bits);
    let (_element, neighbors) =
        unsafe { graph::load_graph_adjacency(index_relation, metadata.entry_point, code_len) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    neighbors
        .tids
        .into_iter()
        .map(|tid| (tid.block_number, tid.offset_number))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_candidate_frontier_head_reselects_then_clears() {
        let mut opaque = TqScanOpaque::default();
        opaque.candidate_frontier.candidates[0] = ScanCandidate {
            element_tid: page::ItemPointer {
                block_number: 7,
                offset_number: 1,
            },
            score: -2.0,
            score_valid: true,
        };
        opaque.candidate_frontier.candidates[1] = ScanCandidate {
            element_tid: page::ItemPointer {
                block_number: 7,
                offset_number: 2,
            },
            score: 3.5,
            score_valid: true,
        };
        recompute_candidate_frontier_head(&mut opaque);

        assert_eq!(
            opaque.candidate_frontier_head, 0,
            "frontier head should start at the lower-scoring valid slot"
        );

        let consumed = consume_candidate_frontier_head(&mut opaque)
            .expect("frontier head consumption should return the current best slot");
        assert_eq!(
            (consumed.element_tid.block_number, consumed.element_tid.offset_number),
            (7, 1),
            "consumption should return the previously best frontier slot"
        );
        assert_eq!(
            opaque.candidate_frontier_head, 1,
            "consuming the best slot should reselect the remaining valid slot as the new head"
        );
        assert!(
            !opaque.candidate_frontier.candidates[0].score_valid,
            "consuming the head should clear the consumed slot"
        );
        assert_eq!(
            opaque.candidate_frontier.candidates[1].score, 3.5,
            "consuming the head should leave the remaining valid slot unchanged"
        );

        let consumed = consume_candidate_frontier_head(&mut opaque)
            .expect("a remaining valid slot should still be consumable");
        assert_eq!(
            (consumed.element_tid.block_number, consumed.element_tid.offset_number),
            (7, 2),
            "the second consumption should return the reseated head slot"
        );
        assert_eq!(
            opaque.candidate_frontier_head, u8::MAX,
            "consuming the last valid slot should invalidate the frontier head"
        );
        assert!(
            !opaque.candidate_frontier.candidates[0].score_valid
                && !opaque.candidate_frontier.candidates[1].score_valid,
            "consuming both valid slots should leave both frontier slots invalid"
        );
        assert_eq!(
            (
                opaque.candidate_frontier.candidates[0].element_tid.block_number,
                opaque.candidate_frontier.candidates[0].element_tid.offset_number,
                opaque.candidate_frontier.candidates[0].score,
                opaque.candidate_frontier.candidates[1].element_tid.block_number,
                opaque.candidate_frontier.candidates[1].element_tid.offset_number,
                opaque.candidate_frontier.candidates[1].score,
            ),
            (u32::MAX, u16::MAX, 0.0, u32::MAX, u16::MAX, 0.0),
            "consuming both valid slots should leave the fixed frontier fully cleared"
        );
        assert!(
            consume_candidate_frontier_head(&mut opaque).is_none(),
            "consuming an empty frontier should stay a no-op"
        );
    }

    #[test]
    fn select_successor_candidate_skips_invalid_then_falls_through() {
        let skipped = page::ItemPointer::INVALID;
        let first_valid = page::ItemPointer {
            block_number: 8,
            offset_number: 1,
        };
        let second_valid = page::ItemPointer {
            block_number: 8,
            offset_number: 2,
        };
        let mut visited = Vec::new();

        let candidate =
            select_successor_candidate(&[skipped, first_valid, second_valid], |neighbor_tid| {
                visited.push((neighbor_tid.block_number, neighbor_tid.offset_number));
                if neighbor_tid == first_valid {
                    return None;
                }

                Some(ScanCandidate {
                    element_tid: neighbor_tid,
                    score: 2.5,
                    score_valid: true,
                })
            })
            .expect("second valid neighbor should be selected after skipping invalid and empty");

        assert_eq!(
            visited,
            vec![(first_valid.block_number, first_valid.offset_number), (
                second_valid.block_number,
                second_valid.offset_number
            )],
            "selection should skip INVALID neighbors and continue until a concrete candidate exists"
        );
        assert_eq!(
            candidate.element_tid, second_valid,
            "selection should return the first neighbor that produces a concrete candidate"
        );
        assert!(candidate.score_valid, "returned candidate should stay marked valid");
    }
}
