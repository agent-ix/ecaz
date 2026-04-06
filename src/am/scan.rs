use std::collections::HashSet;
use std::ptr;

use pgrx::{pg_sys, FromDatum, PgBox};

use crate::quant::prod::PreparedQuery;

use super::graph;
use super::page;

const MAX_BOOTSTRAP_FRONTIER_CANDIDATES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BootstrapExpandPolicy {
    ScoreOrder,
}

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

            let metadata = super::shared::read_metadata_page((*scan).indexRelation);
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
            if materialize_next_bootstrap_frontier_result((*scan).indexRelation, opaque) {
                if let Some(heap_tid) = take_pending_scan_heap_tid(opaque) {
                    set_scan_heap_tid(scan, heap_tid);
                    return true;
                }
            }
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
                free_scan_candidate_frontier(&mut *opaque.cast::<TqScanOpaque>());
                free_scan_expanded_set(&mut *opaque.cast::<TqScanOpaque>());
                free_scan_visited_set(&mut *opaque.cast::<TqScanOpaque>());
                free_scan_emitted_set(&mut *opaque.cast::<TqScanOpaque>());
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
    clear_active_scan_candidate(opaque);
    reset_scan_expanded_state(opaque);
    reset_scan_visited_state(opaque);
    reset_scan_emitted_state(opaque);
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

fn clear_active_scan_candidate(opaque: &mut TqScanOpaque) {
    opaque.active_candidate = ScanCandidate::default();
}

fn clear_scan_candidate_state(opaque: &mut TqScanOpaque) {
    if opaque.candidate_frontier.is_null() {
        opaque.candidate_frontier = Box::into_raw(Box::new(Vec::new()));
    } else {
        unsafe { &mut *opaque.candidate_frontier }.clear();
    }
    opaque.candidate_frontier_head = None;
}

fn free_scan_candidate_frontier(opaque: &mut TqScanOpaque) {
    if !opaque.candidate_frontier.is_null() {
        drop(unsafe { Box::from_raw(opaque.candidate_frontier) });
        opaque.candidate_frontier = ptr::null_mut();
    }
    opaque.candidate_frontier_head = None;
}

fn candidate_frontier_ref(opaque: &TqScanOpaque) -> &[ScanCandidate] {
    if opaque.candidate_frontier.is_null() {
        &[]
    } else {
        unsafe { &*opaque.candidate_frontier }
    }
}

fn candidate_frontier_mut(opaque: &mut TqScanOpaque) -> &mut Vec<ScanCandidate> {
    if opaque.candidate_frontier.is_null() {
        opaque.candidate_frontier = Box::into_raw(Box::new(Vec::new()));
    }
    unsafe { &mut *opaque.candidate_frontier }
}

fn candidate_slot(opaque: &TqScanOpaque, index: usize) -> ScanCandidate {
    candidate_frontier_ref(opaque)
        .get(index)
        .copied()
        .unwrap_or_default()
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_candidate_frontier_snapshot(opaque: &TqScanOpaque) -> DebugCandidateFrontier {
    [candidate_slot(opaque, 0), candidate_slot(opaque, 1)].map(|candidate| {
        (
            candidate.score_valid,
            (
                candidate.element_tid.block_number,
                candidate.element_tid.offset_number,
            ),
            candidate.score,
        )
    })
}

fn reset_scan_visited_state(opaque: &mut TqScanOpaque) {
    if opaque.visited_tids.is_null() {
        opaque.visited_tids = Box::into_raw(Box::new(HashSet::new()));
    } else {
        unsafe { &mut *opaque.visited_tids }.clear();
    }
}

fn free_scan_visited_set(opaque: &mut TqScanOpaque) {
    if !opaque.visited_tids.is_null() {
        drop(unsafe { Box::from_raw(opaque.visited_tids) });
        opaque.visited_tids = ptr::null_mut();
    }
}

fn mark_visited_element(opaque: &mut TqScanOpaque, element_tid: page::ItemPointer) {
    if opaque.visited_tids.is_null() || element_tid == page::ItemPointer::INVALID {
        return;
    }

    unsafe { &mut *opaque.visited_tids }.insert(element_tid);
}

fn visited_contains_element(opaque: &TqScanOpaque, element_tid: page::ItemPointer) -> bool {
    if opaque.visited_tids.is_null() || element_tid == page::ItemPointer::INVALID {
        return false;
    }

    unsafe { &*opaque.visited_tids }.contains(&element_tid)
}

fn reset_scan_expanded_state(opaque: &mut TqScanOpaque) {
    if opaque.expanded_source_tids.is_null() {
        opaque.expanded_source_tids = Box::into_raw(Box::new(HashSet::new()));
    } else {
        unsafe { &mut *opaque.expanded_source_tids }.clear();
    }
}

fn free_scan_expanded_set(opaque: &mut TqScanOpaque) {
    if !opaque.expanded_source_tids.is_null() {
        drop(unsafe { Box::from_raw(opaque.expanded_source_tids) });
        opaque.expanded_source_tids = ptr::null_mut();
    }
}

fn mark_expanded_source(opaque: &mut TqScanOpaque, source_tid: page::ItemPointer) {
    if opaque.expanded_source_tids.is_null() || source_tid == page::ItemPointer::INVALID {
        return;
    }

    unsafe { &mut *opaque.expanded_source_tids }.insert(source_tid);
}

fn expanded_contains_source(opaque: &TqScanOpaque, source_tid: page::ItemPointer) -> bool {
    if opaque.expanded_source_tids.is_null() || source_tid == page::ItemPointer::INVALID {
        return false;
    }

    unsafe { &*opaque.expanded_source_tids }.contains(&source_tid)
}

fn reset_scan_emitted_state(opaque: &mut TqScanOpaque) {
    if opaque.emitted_result_tids.is_null() {
        opaque.emitted_result_tids = Box::into_raw(Box::new(HashSet::new()));
    } else {
        unsafe { &mut *opaque.emitted_result_tids }.clear();
    }
}

fn free_scan_emitted_set(opaque: &mut TqScanOpaque) {
    if !opaque.emitted_result_tids.is_null() {
        drop(unsafe { Box::from_raw(opaque.emitted_result_tids) });
        opaque.emitted_result_tids = ptr::null_mut();
    }
}

fn mark_emitted_element(opaque: &mut TqScanOpaque, element_tid: page::ItemPointer) {
    if opaque.emitted_result_tids.is_null() || element_tid == page::ItemPointer::INVALID {
        return;
    }

    unsafe { &mut *opaque.emitted_result_tids }.insert(element_tid);
}

fn emitted_contains_element(opaque: &TqScanOpaque, element_tid: page::ItemPointer) -> bool {
    if opaque.emitted_result_tids.is_null() || element_tid == page::ItemPointer::INVALID {
        return false;
    }

    unsafe { &*opaque.emitted_result_tids }.contains(&element_tid)
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

    let entry = unsafe {
        graph::load_graph_element(index_relation, metadata.entry_point, opaque.scan_code_len)
    };
    if entry.deleted || entry.heaptids.is_empty() {
        return;
    }

    let entry_score = score_scan_element_result(opaque, entry.gamma, &entry.code);
    candidate_frontier_mut(opaque).push(ScanCandidate {
        element_tid: entry.tid,
        source_tid: page::ItemPointer::INVALID,
        score: entry_score,
        score_valid: true,
    });
    mark_visited_element(opaque, entry.tid);

    fill_bootstrap_frontier(
        opaque,
        MAX_BOOTSTRAP_FRONTIER_CANDIDATES,
        BootstrapExpandPolicy::ScoreOrder,
        |source_tid, opaque| unsafe {
            refill_candidate_frontier_from_source(index_relation, opaque, source_tid);
        },
    );
}

fn next_bootstrap_expand_index(
    opaque: &TqScanOpaque,
    policy: BootstrapExpandPolicy,
) -> Option<usize> {
    match policy {
        BootstrapExpandPolicy::ScoreOrder => candidate_frontier_ref(opaque)
            .iter()
            .enumerate()
            .filter(|(_, candidate)| {
                candidate.score_valid && !expanded_contains_source(opaque, candidate.element_tid)
            })
            .min_by(|(left_index, left), (right_index, right)| {
                left.score
                    .total_cmp(&right.score)
                    .then(left_index.cmp(right_index))
            })
            .map(|(index, _)| index),
    }
}

fn fill_bootstrap_frontier<F>(
    opaque: &mut TqScanOpaque,
    max_candidates: usize,
    policy: BootstrapExpandPolicy,
    refill: F,
) where
    F: FnMut(page::ItemPointer, &mut TqScanOpaque),
{
    reset_scan_expanded_state(opaque);
    top_up_bootstrap_frontier(opaque, max_candidates, policy, refill);
}

fn top_up_bootstrap_frontier<F>(
    opaque: &mut TqScanOpaque,
    max_candidates: usize,
    policy: BootstrapExpandPolicy,
    mut refill: F,
) where
    F: FnMut(page::ItemPointer, &mut TqScanOpaque),
{
    while candidate_frontier_ref(opaque).len() < max_candidates {
        let expand_index = match next_bootstrap_expand_index(opaque, policy) {
            Some(index) => index,
            None => break,
        };
        let source_tid = match candidate_frontier_ref(opaque).get(expand_index) {
            Some(candidate) => candidate.element_tid,
            None => break,
        };
        mark_expanded_source(opaque, source_tid);
        refill(source_tid, opaque);
    }
}

fn recompute_candidate_frontier_head(opaque: &mut TqScanOpaque) {
    let mut best: Option<(usize, ScanCandidate)> = None;
    for (index, candidate) in candidate_frontier_ref(opaque).iter().copied().enumerate() {
        if !candidate.score_valid {
            continue;
        }

        best = match best {
            None => Some((index, candidate)),
            Some((best_index, best_candidate)) => {
                if candidate.score < best_candidate.score
                    || (candidate.score == best_candidate.score && index < best_index)
                {
                    Some((index, candidate))
                } else {
                    Some((best_index, best_candidate))
                }
            }
        };
    }
    opaque.candidate_frontier_head = best.map(|(index, _)| index);
}

fn consume_candidate_frontier_head(opaque: &mut TqScanOpaque) -> Option<ScanCandidate> {
    let head = opaque.candidate_frontier_head?;

    if head >= candidate_frontier_ref(opaque).len() {
        return None;
    }

    let consumed = candidate_frontier_mut(opaque).remove(head);
    recompute_candidate_frontier_head(opaque);
    Some(consumed)
}

unsafe fn refill_candidate_frontier_from_source(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    source_tid: page::ItemPointer,
) {
    if source_tid == page::ItemPointer::INVALID {
        recompute_candidate_frontier_head(opaque);
        return;
    }

    let max_successor_candidates =
        MAX_BOOTSTRAP_FRONTIER_CANDIDATES.saturating_sub(candidate_frontier_ref(opaque).len());
    if max_successor_candidates == 0 {
        recompute_candidate_frontier_head(opaque);
        return;
    }

    let (_, neighbors) =
        unsafe { graph::load_graph_adjacency(index_relation, source_tid, opaque.scan_code_len) };
    let successor_candidates =
        collect_successor_candidates(&neighbors.tids, max_successor_candidates, |neighbor_tid| {
            if visited_contains_element(opaque, neighbor_tid) {
                return None;
            }

            let neighbor = unsafe {
                graph::load_graph_element(index_relation, neighbor_tid, opaque.scan_code_len)
            };
            if neighbor.deleted
                || neighbor.heaptids.is_empty()
                || visited_contains_element(opaque, neighbor.tid)
            {
                return None;
            }

            Some(ScanCandidate {
                element_tid: neighbor.tid,
                source_tid,
                score: score_scan_element_result(opaque, neighbor.gamma, &neighbor.code),
                score_valid: true,
            })
        });
    if !successor_candidates.is_empty() {
        candidate_frontier_mut(opaque).extend(successor_candidates.iter().copied());
        for candidate in successor_candidates {
            mark_visited_element(opaque, candidate.element_tid);
        }
    }

    recompute_candidate_frontier_head(opaque);
}

unsafe fn consume_and_refill_bootstrap_frontier(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) -> Option<ScanCandidate> {
    let consumed = consume_candidate_frontier_head(opaque)?;
    refill_bootstrap_frontier_after_consume(opaque, consumed, |source_tid, opaque| unsafe {
        refill_candidate_frontier_from_source(index_relation, opaque, source_tid)
    });
    Some(consumed)
}

unsafe fn materialize_scan_candidate_result(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    candidate: ScanCandidate,
) -> bool {
    let element = unsafe {
        graph::load_graph_element(index_relation, candidate.element_tid, opaque.scan_code_len)
    };
    if element.deleted || element.heaptids.is_empty() {
        return false;
    }

    set_current_scan_result(opaque, candidate.element_tid, candidate.score);
    store_pending_scan_heaptids(opaque, &element.heaptids);
    true
}

fn refill_bootstrap_frontier_after_consume<F>(
    opaque: &mut TqScanOpaque,
    consumed: ScanCandidate,
    mut refill: F,
) where
    F: FnMut(page::ItemPointer, &mut TqScanOpaque),
{
    if !expanded_contains_source(opaque, consumed.element_tid) {
        mark_expanded_source(opaque, consumed.element_tid);
        refill(consumed.element_tid, opaque);
    }

    top_up_bootstrap_frontier(
        opaque,
        MAX_BOOTSTRAP_FRONTIER_CANDIDATES,
        BootstrapExpandPolicy::ScoreOrder,
        refill,
    );
}

fn maybe_consume_bootstrap_frontier_candidate(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) {
    if opaque.active_candidate.score_valid
        || opaque.scan_exhausted
        || opaque.pending_heaptid_count != 0
        || opaque.scan_dimensions == 0
    {
        return;
    }

    if let Some(candidate) =
        unsafe { consume_and_refill_bootstrap_frontier(index_relation, opaque) }
    {
        opaque.active_candidate = candidate;
    }
}

unsafe fn materialize_next_bootstrap_frontier_result(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) -> bool {
    if opaque.pending_heaptid_count != 0 || opaque.scan_exhausted || opaque.scan_dimensions == 0 {
        return false;
    }

    let Some(candidate) =
        (unsafe { consume_and_refill_bootstrap_frontier(index_relation, opaque) })
    else {
        return false;
    };
    unsafe { materialize_scan_candidate_result(index_relation, opaque, candidate) }
}

unsafe fn materialize_active_candidate_result(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) -> bool {
    if !opaque.active_candidate.score_valid || opaque.pending_heaptid_count != 0 {
        return false;
    }

    let candidate = opaque.active_candidate;
    clear_active_scan_candidate(opaque);
    unsafe { materialize_scan_candidate_result(index_relation, opaque, candidate) }
}

fn collect_successor_candidates<F>(
    neighbor_tids: &[page::ItemPointer],
    max_candidates: usize,
    mut candidate_for_tid: F,
) -> Vec<ScanCandidate>
where
    F: FnMut(page::ItemPointer) -> Option<ScanCandidate>,
{
    let mut candidates = Vec::new();
    if max_candidates == 0 {
        return candidates;
    }

    for neighbor_tid in neighbor_tids.iter().copied() {
        if neighbor_tid == page::ItemPointer::INVALID {
            continue;
        }

        if let Some(candidate) = candidate_for_tid(neighbor_tid) {
            candidates.push(candidate);
            if candidates.len() >= max_candidates {
                break;
            }
        }
    }

    candidates
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
        clear_active_scan_candidate(opaque);
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
        let line_pointer_count = super::shared::page_line_pointer_count(page_ptr);
        let offset_start = if block_number == opaque.next_block_number {
            opaque.next_offset_number.max(1)
        } else {
            1
        };

        for offset in offset_start..=line_pointer_count {
            let item_id = unsafe { &*super::shared::page_item_id(page_ptr, offset) };
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
            let element_tid = page::ItemPointer {
                block_number,
                offset_number: offset,
            };
            if emitted_contains_element(opaque, element_tid) {
                continue;
            }
            if opaque.active_candidate.score_valid
                && opaque.active_candidate.element_tid == element_tid
            {
                set_current_scan_result(opaque, element_tid, opaque.active_candidate.score);
                clear_active_scan_candidate(opaque);
            } else {
                set_current_scan_result(
                    opaque,
                    element_tid,
                    score_scan_element_result(opaque, element.gamma, &element.code),
                );
            }

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
    clear_active_scan_candidate(opaque);
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

fn set_current_scan_result(opaque: &mut TqScanOpaque, element_tid: page::ItemPointer, score: f32) {
    mark_emitted_element(opaque, element_tid);
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

    let query = unsafe {
        std::slice::from_raw_parts(opaque.query_values, opaque.query_dimensions as usize)
    };
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
    source_tid: page::ItemPointer,
    score: f32,
    score_valid: bool,
}

impl Default for ScanCandidate {
    fn default() -> Self {
        Self {
            element_tid: page::ItemPointer::INVALID,
            source_tid: page::ItemPointer::INVALID,
            score: 0.0,
            score_valid: false,
        }
    }
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
    visited_tids: *mut HashSet<page::ItemPointer>,
    expanded_source_tids: *mut HashSet<page::ItemPointer>,
    emitted_result_tids: *mut HashSet<page::ItemPointer>,
    candidate_frontier: *mut Vec<ScanCandidate>,
    candidate_frontier_head: Option<usize>,
    active_candidate: ScanCandidate,
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
            visited_tids: ptr::null_mut(),
            expanded_source_tids: ptr::null_mut(),
            emitted_result_tids: ptr::null_mut(),
            candidate_frontier: ptr::null_mut(),
            candidate_frontier_head: None,
            active_candidate: ScanCandidate::default(),
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
type DebugCandidateProvenanceSlot = (bool, HeapTidCoords, HeapTidCoords, f32);

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateHead = Option<usize>;

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontierSlots = Vec<DebugCandidateSlot>;

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontierProvenanceSlots = Vec<DebugCandidateProvenanceSlot>;

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontierLifecycle = (
    DebugCandidateHead,
    DebugCandidateFrontier,
    DebugCandidateHead,
    DebugCandidateFrontier,
    DebugCandidateHead,
    DebugCandidateFrontier,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontierConsume = (
    DebugCandidateHead,
    DebugCandidateFrontier,
    DebugCandidateHead,
    DebugCandidateFrontier,
    DebugCandidateHead,
    DebugCandidateFrontier,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontierSlotConsume = (
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
    Vec<HeapTidCoords>,
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
    DebugCandidateFrontierProvenanceSlots,
);

#[cfg(any(test, feature = "pg_test"))]
fn debug_candidate_frontier_slots(opaque: &TqScanOpaque) -> DebugCandidateFrontierSlots {
    candidate_frontier_ref(opaque)
        .iter()
        .map(|candidate| {
            (
                candidate.score_valid,
                (
                    candidate.element_tid.block_number,
                    candidate.element_tid.offset_number,
                ),
                candidate.score,
            )
        })
        .collect::<Vec<_>>()
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_candidate_frontier_provenance_slots(
    opaque: &TqScanOpaque,
) -> DebugCandidateFrontierProvenanceSlots {
    candidate_frontier_ref(opaque)
        .iter()
        .map(|candidate| {
            (
                candidate.score_valid,
                (
                    candidate.element_tid.block_number,
                    candidate.element_tid.offset_number,
                ),
                (
                    candidate.source_tid.block_number,
                    candidate.source_tid.offset_number,
                ),
                candidate.score,
            )
        })
        .collect::<Vec<_>>()
}

#[cfg(any(test, feature = "pg_test"))]
type DebugVisitedSeedsLifecycle = (Vec<HeapTidCoords>, Vec<HeapTidCoords>, Vec<HeapTidCoords>);

#[cfg(any(test, feature = "pg_test"))]
type DebugBootstrapSeedState = (
    DebugCandidateHead,
    DebugCandidateFrontier,
    DebugCandidateFrontierSlots,
    DebugCandidateFrontierProvenanceSlots,
    Vec<HeapTidCoords>,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugBootstrapConsumeState = (
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
    (bool, HeapTidCoords, HeapTidCoords, f32),
    HeapTidCoords,
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugActiveCandidateMaterializationState = (
    (bool, HeapTidCoords, f32),
    bool,
    HeapTidCoords,
    Vec<HeapTidCoords>,
    bool,
);

#[cfg(any(test, feature = "pg_test"))]
fn debug_sorted_visited_tids(opaque: &TqScanOpaque) -> Vec<HeapTidCoords> {
    if opaque.visited_tids.is_null() {
        return Vec::new();
    }

    let mut tids = unsafe { &*opaque.visited_tids }
        .iter()
        .map(|tid| (tid.block_number, tid.offset_number))
        .collect::<Vec<_>>();
    tids.sort_unstable();
    tids
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_sorted_expanded_source_tids(opaque: &TqScanOpaque) -> Vec<HeapTidCoords> {
    if opaque.expanded_source_tids.is_null() {
        return Vec::new();
    }

    let mut tids = unsafe { &*opaque.expanded_source_tids }
        .iter()
        .map(|tid| (tid.block_number, tid.offset_number))
        .collect::<Vec<_>>();
    tids.sort_unstable();
    tids
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
        tids.push(pgrx::itemptr::item_pointer_get_both(unsafe {
            (*scan).xs_heaptid
        }));
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
) -> (
    bool,
    HeapTidCoords,
    bool,
    f32,
    bool,
    HeapTidCoords,
    bool,
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
    let before = candidate_slot(opaque, 0);
    let before_valid = before.score_valid;
    let before_tid = (
        before.element_tid.block_number,
        before.element_tid.offset_number,
    );
    let before_score = before.score;

    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {}

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let after = candidate_slot(opaque, 0);
    let after_valid = after.score_valid;
    let after_tid = (
        after.element_tid.block_number,
        after.element_tid.offset_number,
    );
    let after_score = after.score;

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
) -> (HeapTidCoords, Vec<HeapTidCoords>, bool, HeapTidCoords, f32) {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
    let entry_tid = (
        metadata.entry_point.block_number,
        metadata.entry_point.offset_number,
    );
    let entry_neighbors = unsafe { super::debug_entry_point_neighbor_tids(index_oid) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let successor = candidate_slot(opaque, 1);
    let successor_valid = successor.score_valid;
    let successor_tid = (
        successor.element_tid.block_number,
        successor.element_tid.offset_number,
    );
    let successor_score = successor.score;

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
) -> DebugBootstrapSeedState {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let frontier = debug_candidate_frontier_snapshot(opaque);
    let frontier_slots = debug_candidate_frontier_slots(opaque);
    let frontier_provenance = debug_candidate_frontier_provenance_slots(opaque);
    let expanded_sources = debug_sorted_expanded_source_tids(opaque);
    let head = opaque.candidate_frontier_head;

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        head,
        frontier,
        frontier_slots,
        frontier_provenance,
        expanded_sources,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_consumes_bootstrap_candidate(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugBootstrapConsumeState {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let before_head = opaque.candidate_frontier_head;
    let before_slots = debug_candidate_frontier_slots(opaque);

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "bootstrap-consume helper requires a first tuple"
    );

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let active_candidate = (
        opaque.active_candidate.score_valid,
        (
            opaque.active_candidate.element_tid.block_number,
            opaque.active_candidate.element_tid.offset_number,
        ),
        (
            opaque.active_candidate.source_tid.block_number,
            opaque.active_candidate.source_tid.offset_number,
        ),
        opaque.active_candidate.score,
    );
    let current_result_tid = (
        opaque.current_result.element_tid.block_number,
        opaque.current_result.element_tid.offset_number,
    );
    let after_head = opaque.candidate_frontier_head;
    let after_slots = debug_candidate_frontier_slots(opaque);

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        before_head,
        before_slots,
        active_candidate,
        current_result_tid,
        after_head,
        after_slots,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_materialize_active_candidate_result(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugActiveCandidateMaterializationState {
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &mut *(*scan).opaque.cast::<TqScanOpaque>() };
    maybe_consume_bootstrap_frontier_candidate(index_relation, opaque);
    let active_before = (
        opaque.active_candidate.score_valid,
        (
            opaque.active_candidate.element_tid.block_number,
            opaque.active_candidate.element_tid.offset_number,
        ),
        opaque.active_candidate.score,
    );
    let materialized = unsafe { materialize_active_candidate_result(index_relation, opaque) };
    let current_result_tid = (
        opaque.current_result.element_tid.block_number,
        opaque.current_result.element_tid.offset_number,
    );
    let pending_heap_tids = opaque.pending_heaptids[..opaque.pending_heaptid_count as usize]
        .iter()
        .map(|tid| (tid.block_number, tid.offset_number))
        .collect::<Vec<_>>();
    let active_cleared = !opaque.active_candidate.score_valid;

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        active_before,
        materialized,
        current_result_tid,
        pending_heap_tids,
        active_cleared,
    )
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
    let before_frontier = debug_candidate_frontier_snapshot(opaque);

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "frontier-head lifecycle helper requires a first tuple"
    );
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let partial_head = opaque.candidate_frontier_head;
    let partial_frontier = debug_candidate_frontier_snapshot(opaque);

    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {}

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let exhausted_head = opaque.candidate_frontier_head;
    let exhausted_frontier = debug_candidate_frontier_snapshot(opaque);

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
    let before_frontier = debug_candidate_frontier_snapshot(opaque);

    let first_consumed = unsafe { consume_and_refill_bootstrap_frontier(index_relation, opaque) };
    debug_assert_eq!(first_consumed.is_some(), before_head.is_some());
    let after_first_head = opaque.candidate_frontier_head;
    let after_first_frontier = debug_candidate_frontier_snapshot(opaque);

    unsafe { consume_and_refill_bootstrap_frontier(index_relation, opaque) };
    let after_second_head = opaque.candidate_frontier_head;
    let after_second_frontier = debug_candidate_frontier_snapshot(opaque);

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
pub(crate) unsafe fn debug_consume_candidate_frontier_head_slots(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugCandidateFrontierSlotConsume {
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
    let before_slots = debug_candidate_frontier_slots(opaque);
    let consumed_neighbors = before_head
        .and_then(|index| before_slots.get(index))
        .map(|slot| {
            let consumed_tid = page::ItemPointer {
                block_number: slot.1 .0,
                offset_number: slot.1 .1,
            };
            let (_, neighbors) = unsafe {
                graph::load_graph_adjacency(index_relation, consumed_tid, opaque.scan_code_len)
            };
            neighbors
                .tids
                .into_iter()
                .map(|tid| (tid.block_number, tid.offset_number))
                .filter(|tid| *tid != (u32::MAX, u16::MAX))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    unsafe { consume_and_refill_bootstrap_frontier(index_relation, opaque) };

    let after_head = opaque.candidate_frontier_head;
    let after_slots = debug_candidate_frontier_slots(opaque);
    let after_provenance_slots = debug_candidate_frontier_provenance_slots(opaque);

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        before_head,
        before_slots,
        consumed_neighbors,
        after_head,
        after_slots,
        after_provenance_slots,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_visited_seed_lifecycle(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugVisitedSeedsLifecycle {
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
    let before = debug_sorted_visited_tids(opaque);

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "visited-seed lifecycle helper requires a first tuple"
    );
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let partial = debug_sorted_visited_tids(opaque);

    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {}
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let exhausted = debug_sorted_visited_tids(opaque);

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (before, partial, exhausted)
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
    HeapTidCoords,
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
    let before = candidate_slot(opaque, 0);
    let before_valid = before.score_valid;
    let before_tid = (
        before.element_tid.block_number,
        before.element_tid.offset_number,
    );
    let before_score = before.score;

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "entry-candidate lifecycle helper requires a first tuple"
    );
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let partial = candidate_slot(opaque, 0);
    let partial_valid = partial.score_valid;
    let partial_tid = (
        partial.element_tid.block_number,
        partial.element_tid.offset_number,
    );
    let partial_score = partial.score;
    let partial_result_tid = (
        opaque.current_result.element_tid.block_number,
        opaque.current_result.element_tid.offset_number,
    );

    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {}

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let exhausted = candidate_slot(opaque, 0);
    let exhausted_valid = exhausted.score_valid;
    let exhausted_tid = (
        exhausted.element_tid.block_number,
        exhausted.element_tid.offset_number,
    );
    let exhausted_score = exhausted.score;

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
        partial_result_tid,
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
        first_pass.push(pgrx::itemptr::item_pointer_get_both(unsafe {
            (*scan).xs_heaptid
        }));
    }

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1) };

    let mut rescanned = Vec::new();
    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {
        rescanned.push(pgrx::itemptr::item_pointer_get_both(unsafe {
            (*scan).xs_heaptid
        }));
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
    let first_tid = pgrx::itemptr::item_pointer_get_both(unsafe { (*scan).xs_heaptid });

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1) };

    let mut tids = Vec::new();
    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {
        tids.push(pgrx::itemptr::item_pointer_get_both(unsafe {
            (*scan).xs_heaptid
        }));
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
    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
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
        candidate_frontier_mut(&mut opaque).push(ScanCandidate {
            element_tid: page::ItemPointer {
                block_number: 7,
                offset_number: 1,
            },
            source_tid: page::ItemPointer::INVALID,
            score: -2.0,
            score_valid: true,
        });
        candidate_frontier_mut(&mut opaque).push(ScanCandidate {
            element_tid: page::ItemPointer {
                block_number: 7,
                offset_number: 2,
            },
            source_tid: page::ItemPointer::INVALID,
            score: 3.5,
            score_valid: true,
        });
        recompute_candidate_frontier_head(&mut opaque);

        assert_eq!(
            opaque.candidate_frontier_head,
            Some(0),
            "frontier head should start at the lower-scoring valid slot"
        );

        let consumed = consume_candidate_frontier_head(&mut opaque)
            .expect("frontier head consumption should return the current best slot");
        assert_eq!(
            (
                consumed.element_tid.block_number,
                consumed.element_tid.offset_number
            ),
            (7, 1),
            "consumption should return the previously best frontier slot"
        );
        assert_eq!(
            opaque.candidate_frontier_head,
            Some(0),
            "consuming the best slot should compact the candidate vector and reselect the remaining valid slot"
        );
        assert!(
            candidate_slot(&opaque, 0).score_valid,
            "consuming the head should keep the remaining candidate valid"
        );
        assert_eq!(
            candidate_slot(&opaque, 0).score,
            3.5,
            "consuming the head should preserve the remaining candidate after compaction"
        );

        let consumed = consume_candidate_frontier_head(&mut opaque)
            .expect("a remaining valid slot should still be consumable");
        assert_eq!(
            (
                consumed.element_tid.block_number,
                consumed.element_tid.offset_number
            ),
            (7, 2),
            "the second consumption should return the reseated head slot"
        );
        assert_eq!(
            opaque.candidate_frontier_head, None,
            "consuming the last valid slot should invalidate the frontier head"
        );
        assert!(
            candidate_frontier_ref(&opaque).is_empty(),
            "consuming both valid slots should leave the candidate vector empty"
        );
        assert!(
            consume_candidate_frontier_head(&mut opaque).is_none(),
            "consuming an empty frontier should stay a no-op"
        );
    }

    #[test]
    fn collect_successor_candidates_skips_invalid_and_collects_multiple() {
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

        let candidates = collect_successor_candidates(
            &[skipped, first_valid, second_valid],
            2,
            |neighbor_tid| {
                visited.push((neighbor_tid.block_number, neighbor_tid.offset_number));

                Some(ScanCandidate {
                    element_tid: neighbor_tid,
                    source_tid: page::ItemPointer::INVALID,
                    score: 2.5,
                    score_valid: true,
                })
            },
        );

        assert_eq!(
            visited,
            vec![(first_valid.block_number, first_valid.offset_number), (
                second_valid.block_number,
                second_valid.offset_number
            )],
            "collection should skip INVALID neighbors and continue through live candidates in order"
        );
        assert_eq!(
            candidates
                .into_iter()
                .map(|candidate| candidate.element_tid)
                .collect::<Vec<_>>(),
            vec![first_valid, second_valid],
            "collection should return live candidates in neighbor order up to the requested limit"
        );
    }

    #[test]
    fn fill_bootstrap_frontier_can_expand_beyond_entry_neighbors() {
        let entry_tid = page::ItemPointer {
            block_number: 9,
            offset_number: 1,
        };
        let child_tid = page::ItemPointer {
            block_number: 9,
            offset_number: 2,
        };
        let grandchild_tid = page::ItemPointer {
            block_number: 9,
            offset_number: 3,
        };
        let mut opaque = TqScanOpaque::default();
        candidate_frontier_mut(&mut opaque).push(ScanCandidate {
            element_tid: entry_tid,
            source_tid: page::ItemPointer::INVALID,
            score: -3.0,
            score_valid: true,
        });

        fill_bootstrap_frontier(
            &mut opaque,
            3,
            BootstrapExpandPolicy::ScoreOrder,
            |source_tid, opaque| {
                let frontier = candidate_frontier_mut(opaque);
                match (source_tid.block_number, source_tid.offset_number) {
                    (9, 1)
                        if frontier
                            .iter()
                            .all(|candidate| candidate.element_tid != child_tid) =>
                    {
                        frontier.push(ScanCandidate {
                            element_tid: child_tid,
                            source_tid,
                            score: -2.0,
                            score_valid: true,
                        });
                    }
                    (9, 2)
                        if frontier
                            .iter()
                            .all(|candidate| candidate.element_tid != grandchild_tid) =>
                    {
                        frontier.push(ScanCandidate {
                            element_tid: grandchild_tid,
                            source_tid,
                            score: -1.0,
                            score_valid: true,
                        });
                    }
                    _ => {}
                }
            },
        );

        assert_eq!(
            candidate_frontier_ref(&opaque)
                .iter()
                .map(|candidate| candidate.element_tid)
                .collect::<Vec<_>>(),
            vec![entry_tid, child_tid, grandchild_tid],
            "bootstrap frontier filling should keep expanding from newly seeded candidates until capacity is reached"
        );
        assert_eq!(
            candidate_frontier_ref(&opaque)[0].source_tid,
            page::ItemPointer::INVALID,
            "entry-seeded candidates should not claim a discovery source"
        );
        assert_eq!(
            candidate_frontier_ref(&opaque)[1].source_tid,
            entry_tid,
            "first-hop candidates should record the entry candidate as their source"
        );
        assert_eq!(
            candidate_frontier_ref(&opaque)[2].source_tid,
            child_tid,
            "second-hop candidates should record the candidate they were expanded from"
        );
    }

    #[test]
    fn top_up_bootstrap_frontier_preserves_expanded_state() {
        let entry_tid = page::ItemPointer {
            block_number: 11,
            offset_number: 1,
        };
        let sibling_tid = page::ItemPointer {
            block_number: 11,
            offset_number: 2,
        };
        let grandchild_tid = page::ItemPointer {
            block_number: 11,
            offset_number: 3,
        };
        let mut opaque = TqScanOpaque::default();
        reset_scan_expanded_state(&mut opaque);
        candidate_frontier_mut(&mut opaque).push(ScanCandidate {
            element_tid: entry_tid,
            source_tid: page::ItemPointer::INVALID,
            score: -3.0,
            score_valid: true,
        });
        candidate_frontier_mut(&mut opaque).push(ScanCandidate {
            element_tid: sibling_tid,
            source_tid: entry_tid,
            score: -2.0,
            score_valid: true,
        });
        mark_expanded_source(&mut opaque, entry_tid);

        top_up_bootstrap_frontier(
            &mut opaque,
            3,
            BootstrapExpandPolicy::ScoreOrder,
            |source_tid, opaque| {
                if source_tid == sibling_tid
                    && candidate_frontier_ref(opaque)
                        .iter()
                        .all(|candidate| candidate.element_tid != grandchild_tid)
                {
                    candidate_frontier_mut(opaque).push(ScanCandidate {
                        element_tid: grandchild_tid,
                        source_tid,
                        score: -1.0,
                        score_valid: true,
                    });
                }
            },
        );

        assert_eq!(
            candidate_frontier_ref(&opaque)
                .iter()
                .map(|candidate| candidate.element_tid)
                .collect::<Vec<_>>(),
            vec![entry_tid, sibling_tid, grandchild_tid],
            "top-up should keep expanding from remaining unexpanded candidates without resetting prior expanded-source state"
        );
        assert!(
            expanded_contains_source(&opaque, entry_tid),
            "top-up should preserve previously expanded sources"
        );
        assert!(
            expanded_contains_source(&opaque, sibling_tid),
            "top-up should record the newly expanded candidate source"
        );
    }

    #[test]
    fn refill_after_consume_skips_already_expanded_source() {
        let consumed_tid = page::ItemPointer {
            block_number: 12,
            offset_number: 1,
        };
        let sibling_tid = page::ItemPointer {
            block_number: 12,
            offset_number: 2,
        };
        let grandchild_tid = page::ItemPointer {
            block_number: 12,
            offset_number: 3,
        };
        let mut opaque = TqScanOpaque::default();
        reset_scan_expanded_state(&mut opaque);
        candidate_frontier_mut(&mut opaque).push(ScanCandidate {
            element_tid: sibling_tid,
            source_tid: consumed_tid,
            score: -2.0,
            score_valid: true,
        });
        mark_expanded_source(&mut opaque, consumed_tid);

        let mut refilled_sources = Vec::new();
        refill_bootstrap_frontier_after_consume(
            &mut opaque,
            ScanCandidate {
                element_tid: consumed_tid,
                source_tid: page::ItemPointer::INVALID,
                score: -3.0,
                score_valid: true,
            },
            |source_tid, opaque| {
                refilled_sources.push(source_tid);
                if source_tid == sibling_tid
                    && candidate_frontier_ref(opaque)
                        .iter()
                        .all(|candidate| candidate.element_tid != grandchild_tid)
                {
                    candidate_frontier_mut(opaque).push(ScanCandidate {
                        element_tid: grandchild_tid,
                        source_tid,
                        score: -1.0,
                        score_valid: true,
                    });
                }
            },
        );

        assert!(
            !refilled_sources.contains(&consumed_tid),
            "consume/refill should not reread a source that was already expanded during earlier bootstrap work"
        );
        assert_eq!(
            refilled_sources.first().copied(),
            Some(sibling_tid),
            "consume/refill should continue by expanding another remaining frontier candidate first"
        );
        assert_eq!(
            candidate_frontier_ref(&opaque)
                .iter()
                .map(|candidate| candidate.element_tid)
                .collect::<Vec<_>>(),
            vec![sibling_tid, grandchild_tid],
            "consume/refill should still top up from another remaining unexpanded frontier candidate"
        );
    }

    #[test]
    fn next_bootstrap_expand_index_prefers_lowest_score_under_score_order_policy() {
        let mut opaque = TqScanOpaque::default();
        reset_scan_expanded_state(&mut opaque);
        candidate_frontier_mut(&mut opaque).push(ScanCandidate {
            element_tid: page::ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            source_tid: page::ItemPointer::INVALID,
            score: -3.0,
            score_valid: true,
        });
        candidate_frontier_mut(&mut opaque).push(ScanCandidate {
            element_tid: page::ItemPointer {
                block_number: 10,
                offset_number: 2,
            },
            source_tid: page::ItemPointer {
                block_number: 10,
                offset_number: 1,
            },
            score: -4.0,
            score_valid: true,
        });

        assert_eq!(
            next_bootstrap_expand_index(&opaque, BootstrapExpandPolicy::ScoreOrder),
            Some(1),
            "the explicit score-order policy should expand the lowest-score unexpanded seeded candidate first"
        );

        mark_expanded_source(
            &mut opaque,
            page::ItemPointer {
                block_number: 10,
                offset_number: 2,
            },
        );
        assert_eq!(
            next_bootstrap_expand_index(&opaque, BootstrapExpandPolicy::ScoreOrder),
            Some(0),
            "after the best candidate is marked expanded, the score-order policy should fall back to the next best seeded candidate"
        );
    }
}
