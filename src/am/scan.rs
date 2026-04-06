use std::collections::HashSet;
use std::ptr;

use pgrx::{pg_sys, FromDatum, PgBox};

use crate::quant::prod::PreparedQuery;

use super::graph;
use super::page;
use super::search;

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

pub(super) fn candidate_frontier_ref(opaque: &TqScanOpaque) -> &[ScanCandidate] {
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

pub(super) fn candidate_slot(opaque: &TqScanOpaque, index: usize) -> ScanCandidate {
    candidate_frontier_ref(opaque)
        .get(index)
        .copied()
        .unwrap_or_default()
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
        BootstrapExpandPolicy::ScoreOrder => {
            let mut expansion = search::BeamSearch::new(MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
            expansion.seed_many(
                candidate_frontier_ref(opaque)
                    .iter()
                    .copied()
                    .filter(|candidate| {
                        candidate.score_valid
                            && !expanded_contains_source(opaque, candidate.element_tid)
                    })
                    .map(scan_candidate_to_beam_candidate),
            );

            let best = expansion.peek_best()?;
            candidate_frontier_ref(opaque)
                .iter()
                .enumerate()
                .find(|(_, candidate)| {
                    candidate.score_valid
                        && !expanded_contains_source(opaque, candidate.element_tid)
                        && candidate.element_tid == best.node
                })
                .map(|(index, _)| index)
        }
    }
}

fn scan_candidate_to_beam_candidate(
    candidate: ScanCandidate,
) -> search::BeamCandidate<page::ItemPointer> {
    match candidate.source_tid {
        page::ItemPointer::INVALID => search::BeamCandidate::new(candidate.element_tid, candidate.score),
        source_tid => {
            search::BeamCandidate::with_source(candidate.element_tid, candidate.score, source_tid)
        }
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

pub(super) unsafe fn consume_and_refill_bootstrap_frontier(
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

pub(super) fn maybe_consume_bootstrap_frontier_candidate(
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

pub(super) unsafe fn materialize_active_candidate_result(
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
pub(super) unsafe fn read_scan_query(opaque: &TqScanOpaque) -> Vec<f32> {
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
pub(super) struct CurrentScanResult {
    pub(super) element_tid: page::ItemPointer,
    pub(super) heap_tid: page::ItemPointer,
    pub(super) score: f32,
    pub(super) score_valid: bool,
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
pub(super) struct ScanCandidate {
    pub(super) element_tid: page::ItemPointer,
    pub(super) source_tid: page::ItemPointer,
    pub(super) score: f32,
    pub(super) score_valid: bool,
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
pub(super) struct TqScanOpaque {
    pub(super) rescan_called: bool,
    pub(super) query_dimensions: u16,
    pub(super) query_values: *mut f32,
    pub(super) prepared_query: *mut PreparedQuery,
    pub(super) scan_dimensions: u16,
    pub(super) scan_bits: u8,
    pub(super) scan_seed: u64,
    pub(super) scan_code_len: usize,
    pub(super) scan_block_count: u32,
    pub(super) visited_tids: *mut HashSet<page::ItemPointer>,
    pub(super) expanded_source_tids: *mut HashSet<page::ItemPointer>,
    pub(super) emitted_result_tids: *mut HashSet<page::ItemPointer>,
    pub(super) candidate_frontier: *mut Vec<ScanCandidate>,
    pub(super) candidate_frontier_head: Option<usize>,
    pub(super) active_candidate: ScanCandidate,
    pub(super) current_result: CurrentScanResult,
    pub(super) next_block_number: u32,
    pub(super) next_offset_number: u16,
    pub(super) scan_exhausted: bool,
    pub(super) pending_heaptids: [page::ItemPointer; page::HEAPTID_INLINE_CAPACITY],
    pub(super) pending_heaptid_count: u8,
    pub(super) pending_heaptid_index: u8,
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
