use std::collections::HashSet;
use std::ptr;
use std::sync::Arc;

use pgrx::{pg_sys, FromDatum, IntoDatum, PgBox};

use crate::quant::prod::{PreparedQuery, ProdQuantizer};

use super::explain::TqExplainCounters;
use super::graph;
use super::page;
use super::search;
use super::stream::{GraphPrefetchState, LinearPrefetchState};

const MAX_BOOTSTRAP_FRONTIER_CANDIDATES: usize = 3;

#[cfg(any(test, feature = "pg_test"))]
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
            opaque.scan_m = metadata.m;
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
            let scan_tuning = super::options::resolve_scan_tuning(
                &super::options::relation_options((*scan).indexRelation),
            );
            opaque.bootstrap_frontier_limit = usize::try_from(scan_tuning.effective_ef_search)
                .expect("ef_search should fit in usize")
                .max(1);
            store_scan_query(opaque, &query);
            opaque.explain_counters.reset();
            store_scan_prepared_query(opaque, &query, &metadata);
            reset_scan_position(opaque);
            reset_linear_prefetch_state(opaque);
            reset_graph_prefetch_state(opaque);
            initialize_scan_entry_candidate(
                (*scan).indexRelation,
                (*scan).heapRelation,
                opaque,
                &metadata,
            );
            let opaque_ptr = opaque as *mut TqScanOpaque;
            if !graph_traversal_cursor(opaque)
                .ensure_prefetched_output((*scan).indexRelation, opaque_ptr)
            {
                enter_linear_fallback_phase(opaque);
                reset_linear_prefetch_state(opaque);
            }
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
                clear_scan_orderby_output(scan);
                return false;
            }

            let opaque = &mut *opaque_ptr;
            if produce_next_scan_heap_tid(scan, (*scan).indexRelation, opaque, opaque.scan_code_len)
            {
                return true;
            }

            clear_scan_orderby_output(scan);
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

            let opaque_ptr = (*scan).opaque;
            if !opaque_ptr.is_null() {
                let opaque = &mut *opaque_ptr.cast::<TqScanOpaque>();
                free_graph_prefetch_state(opaque);
                free_scan_candidate_frontier(opaque);
                free_bootstrap_expansion(opaque);
                free_scan_expanded_set(opaque);
                free_scan_visited_set(opaque);
                free_scan_emitted_set(opaque);
                free_scan_prepared_query(opaque);
                free_scan_query(opaque);
                pg_sys::pfree(opaque_ptr);
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

    let cache_hit = ProdQuantizer::contains_cached(
        metadata.dimensions as usize,
        metadata.bits,
        metadata.seed,
    );
    let quantizer =
        ProdQuantizer::cached(metadata.dimensions as usize, metadata.bits, metadata.seed);
    let prepared = quantizer.prepare_ip_query(query);
    opaque.prepared_query = Box::into_raw(Box::new(prepared));
    opaque.cached_quantizer = Arc::into_raw(quantizer);
    if cache_hit {
        opaque.explain_counters.record_quantizer_cache_hit();
    }
}

fn free_scan_prepared_query(opaque: &mut TqScanOpaque) {
    if !opaque.prepared_query.is_null() {
        drop(unsafe { Box::from_raw(opaque.prepared_query) });
        opaque.prepared_query = ptr::null_mut();
    }
    if !opaque.cached_quantizer.is_null() {
        drop(unsafe { Arc::from_raw(opaque.cached_quantizer) });
        opaque.cached_quantizer = ptr::null();
    }
}

fn reset_scan_position(opaque: &mut TqScanOpaque) {
    opaque.next_block_number = page::FIRST_DATA_BLOCK_NUMBER;
    opaque.next_offset_number = 1;
    opaque.execution_phase = ScanExecutionPhase::GraphTraversal;
    clear_scan_candidate_state(opaque);
    opaque.result_state.clear();
    opaque.fallback_result_state.clear();
    reset_bootstrap_expansion_state(opaque, bootstrap_frontier_limit(opaque));
    reset_scan_expanded_state(opaque);
    reset_scan_visited_state(opaque);
    reset_scan_emitted_state(opaque);
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PendingScanOutput {
    heap_tid: page::ItemPointer,
    score: f32,
}

struct GraphTraversalCursor<'a> {
    result_state: &'a mut ScanResultState,
}

impl<'a> GraphTraversalCursor<'a> {
    fn new(result_state: &'a mut ScanResultState) -> Self {
        Self { result_state }
    }

    fn has_prefetched_output(&self) -> bool {
        self.result_state.pending_count() != 0
    }

    fn prefetch_ready(&mut self) -> bool {
        if self.has_prefetched_output() {
            return true;
        }

        if self.result_state.current().has_element() {
            self.result_state.clear_current();
        }

        false
    }

    fn needs_prefetch_refresh(&self) -> bool {
        self.result_state.pending_count() == 0
    }

    fn take_pending_output(&mut self) -> Option<PendingScanOutput> {
        self.result_state.take_pending_output()
    }

    fn emit_prefetched_output(&mut self, scan: pg_sys::IndexScanDesc) -> bool {
        self.take_pending_output()
            .map(|output| {
                emit_scan_output(scan, output);
                true
            })
            .unwrap_or(false)
    }

    unsafe fn prefetch_next(
        &mut self,
        index_relation: pg_sys::Relation,
        opaque: *mut TqScanOpaque,
    ) -> bool {
        let result_state = self.result_state as *mut ScanResultState;
        unsafe {
            prefetch_next_graph_result_from_frontier(index_relation, &mut *opaque, result_state)
        }
    }

    unsafe fn ensure_prefetched_output(
        &mut self,
        index_relation: pg_sys::Relation,
        opaque: *mut TqScanOpaque,
    ) -> bool {
        let opaque = unsafe { &mut *opaque };
        if !opaque.execution_phase.is_graph_traversal() {
            return false;
        }

        if self.prefetch_ready() {
            return true;
        }

        if !unsafe { self.prefetch_next(index_relation, opaque as *mut TqScanOpaque) } {
            mark_scan_exhausted(opaque);
            return false;
        }

        true
    }
}

fn graph_traversal_cursor(opaque: &mut TqScanOpaque) -> GraphTraversalCursor<'_> {
    GraphTraversalCursor::new(&mut opaque.result_state)
}

struct LinearFallbackCursor<'a> {
    result_state: &'a mut ScanResultState,
}

impl<'a> LinearFallbackCursor<'a> {
    fn new(result_state: &'a mut ScanResultState) -> Self {
        Self { result_state }
    }

    fn materialize(&mut self, selected: SelectedScanResult) {
        self.result_state.materialize(selected);
    }

    fn take_pending_output(&mut self) -> Option<PendingScanOutput> {
        self.result_state.take_pending_output()
    }

    fn emit_pending_output(&mut self, scan: pg_sys::IndexScanDesc) -> bool {
        self.take_pending_output()
            .map(|output| {
                emit_scan_output(scan, output);
                true
            })
            .unwrap_or(false)
    }

    fn advance_after_emit(&mut self) {
        if self.result_state.pending_count() == 0 {
            self.result_state.clear_current();
        }
    }

    fn emit_materialized_output(
        &mut self,
        scan: pg_sys::IndexScanDesc,
        selected: SelectedScanResult,
    ) -> bool {
        self.materialize(selected);
        let emitted = self.emit_pending_output(scan);
        debug_assert!(
            emitted,
            "linear fallback result materialization should seed pending heap tids before returning true"
        );
        if emitted {
            self.advance_after_emit();
        }
        emitted
    }
}

fn linear_fallback_cursor(opaque: &mut TqScanOpaque) -> LinearFallbackCursor<'_> {
    LinearFallbackCursor::new(&mut opaque.fallback_result_state)
}

pub(super) fn active_result_state_ref(opaque: &TqScanOpaque) -> &ScanResultState {
    if opaque.execution_phase == ScanExecutionPhase::LinearFallback {
        &opaque.fallback_result_state
    } else {
        &opaque.result_state
    }
}

unsafe fn produce_next_scan_heap_tid(
    scan: pg_sys::IndexScanDesc,
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    code_len: usize,
) -> bool {
    match opaque.execution_phase {
        ScanExecutionPhase::GraphTraversal => unsafe {
            produce_next_graph_traversal_heap_tid(scan, index_relation, opaque)
        },
        ScanExecutionPhase::LinearFallback => unsafe {
            produce_next_linear_fallback_heap_tid(scan, index_relation, opaque, code_len)
        },
        ScanExecutionPhase::Exhausted => false,
    }
}

fn clear_scan_candidate_state(opaque: &mut TqScanOpaque) {
    visible_frontier_mut(opaque).clear();
}

fn clear_graph_traversal_state(opaque: &mut TqScanOpaque) {
    clear_scan_candidate_state(opaque);
    reset_bootstrap_expansion_state(opaque, bootstrap_frontier_limit(opaque));
    reset_scan_expanded_state(opaque);
}

unsafe fn prefetch_next_graph_result_from_frontier(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    result_state: *mut ScanResultState,
) -> bool {
    if !opaque.execution_phase.is_graph_traversal()
        || opaque.scan_dimensions == 0
        || unsafe { (&*result_state).pending_count() != 0 }
    {
        return false;
    }

    while let Some(candidate) = consume_candidate_frontier_head(opaque) {
        mark_expanded_source(opaque, candidate.node);
        opaque.explain_counters.record_bootstrap_expansion();
        if unsafe {
            materialize_graph_result_candidate(index_relation, opaque, result_state, candidate)
        }
        .is_some()
        {
            return true;
        }
    }

    false
}

unsafe fn materialize_graph_result_candidate(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    result_state: *mut ScanResultState,
    candidate: search::BeamCandidate<page::ItemPointer>,
) -> Option<()> {
    if emitted_contains_element(opaque, candidate.node) {
        opaque.explain_counters.record_element_skipped();
        return None;
    }

    opaque.explain_counters.record_bootstrap_page_read();
    let element =
        unsafe { graph::load_graph_element(index_relation, candidate.node, opaque.scan_code_len) };
    if element.deleted || element.heaptids.is_empty() {
        opaque.explain_counters.record_element_skipped();
        return None;
    }

    opaque.explain_counters.record_element_scored();
    mark_emitted_element(opaque, candidate.node);
    unsafe { &mut *result_state }.materialize(SelectedScanResult {
        element_tid: candidate.node,
        score: candidate.score,
        heap_tids: element.heaptids,
    });
    Some(())
}

fn enter_linear_fallback_phase(opaque: &mut TqScanOpaque) {
    clear_graph_traversal_state(opaque);
    opaque.fallback_result_state.clear();
    opaque.execution_phase = ScanExecutionPhase::LinearFallback;
}

fn mark_scan_exhausted(opaque: &mut TqScanOpaque) {
    clear_graph_traversal_state(opaque);
    opaque.result_state.clear();
    opaque.fallback_result_state.clear();
    opaque.execution_phase = ScanExecutionPhase::Exhausted;
}

fn reset_bootstrap_expansion_state(opaque: &mut TqScanOpaque, ef_search: usize) {
    let ef_search = ef_search.max(1);
    if opaque.bootstrap_expansion.is_null() {
        opaque.bootstrap_expansion = Box::into_raw(Box::new(search::BeamSearch::new(ef_search)));
    } else {
        *unsafe { &mut *opaque.bootstrap_expansion } = search::BeamSearch::new(ef_search);
    }
}

fn bootstrap_frontier_limit(opaque: &TqScanOpaque) -> usize {
    opaque.bootstrap_frontier_limit.max(1)
}

fn free_scan_candidate_frontier(opaque: &mut TqScanOpaque) {
    if !opaque.candidate_frontier.is_null() {
        drop(unsafe { Box::from_raw(opaque.candidate_frontier) });
        opaque.candidate_frontier = ptr::null_mut();
    }
}

fn free_bootstrap_expansion(opaque: &mut TqScanOpaque) {
    if !opaque.bootstrap_expansion.is_null() {
        drop(unsafe { Box::from_raw(opaque.bootstrap_expansion) });
        opaque.bootstrap_expansion = ptr::null_mut();
    }
}

fn free_graph_prefetch_state(opaque: &mut TqScanOpaque) {
    if !opaque.graph_prefetch_state.is_null() {
        drop(unsafe { Box::from_raw(opaque.graph_prefetch_state) });
        opaque.graph_prefetch_state = ptr::null_mut();
    }
}

fn reset_graph_prefetch_state(opaque: &mut TqScanOpaque) {
    if opaque.graph_prefetch_state.is_null() {
        opaque.graph_prefetch_state = Box::into_raw(Box::new(GraphPrefetchState::new(Vec::new())));
    } else {
        unsafe { &mut *opaque.graph_prefetch_state }.reset(Vec::new());
    }
}

fn reset_linear_prefetch_state(opaque: &mut TqScanOpaque) {
    let first = page::FIRST_DATA_BLOCK_NUMBER;
    let max_block = opaque.scan_block_count.saturating_sub(1).max(first);
    opaque.linear_prefetch_state.reset(first, max_block);
}

type VisibleCandidateFrontierState = search::VisibleFrontier<page::ItemPointer>;

static EMPTY_VISIBLE_FRONTIER_STATE: VisibleCandidateFrontierState =
    VisibleCandidateFrontierState::empty();

#[cfg(any(test, feature = "pg_test"))]
fn visible_frontier_ref(opaque: &TqScanOpaque) -> &VisibleCandidateFrontierState {
    if opaque.candidate_frontier.is_null() {
        &EMPTY_VISIBLE_FRONTIER_STATE
    } else {
        unsafe { &*opaque.candidate_frontier }
    }
}

fn visible_frontier_mut(opaque: &mut TqScanOpaque) -> &mut VisibleCandidateFrontierState {
    if opaque.candidate_frontier.is_null() {
        opaque.candidate_frontier =
            Box::into_raw(Box::new(VisibleCandidateFrontierState::default()));
    }
    unsafe { &mut *opaque.candidate_frontier }
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) fn visible_frontier_candidates(
    opaque: &TqScanOpaque,
) -> Vec<search::BeamCandidate<page::ItemPointer>> {
    visible_frontier_ref(opaque).iter().collect()
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) fn visible_frontier_slot(
    opaque: &TqScanOpaque,
    index: usize,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    visible_frontier_ref(opaque).slot(index)
}

#[cfg(any(test, feature = "pg_test"))]
fn with_visible_frontier_and_bootstrap_expansion<R>(
    opaque: &mut TqScanOpaque,
    f: impl FnOnce(&VisibleCandidateFrontierState, &mut search::BeamSearch<page::ItemPointer>) -> R,
) -> R {
    let visible_frontier = visible_frontier_ref(opaque) as *const VisibleCandidateFrontierState;
    let expansion = bootstrap_expansion_mut(opaque) as *mut search::BeamSearch<page::ItemPointer>;
    // SAFETY: `candidate_frontier` and `bootstrap_expansion` are separate Box-backed heap
    // allocations owned by `TqScanOpaque`, so borrowing the frontier immutably and the
    // scheduler mutably at the same time cannot alias.
    unsafe { f(&*visible_frontier, &mut *expansion) }
}

fn with_visible_frontier_mut_and_bootstrap_expansion<R>(
    opaque: &mut TqScanOpaque,
    f: impl FnOnce(&mut VisibleCandidateFrontierState, &mut search::BeamSearch<page::ItemPointer>) -> R,
) -> R {
    let visible_frontier = visible_frontier_mut(opaque) as *mut VisibleCandidateFrontierState;
    let expansion = bootstrap_expansion_mut(opaque) as *mut search::BeamSearch<page::ItemPointer>;
    // SAFETY: `candidate_frontier` and `bootstrap_expansion` are separate Box-backed heap
    // allocations owned by `TqScanOpaque`, so borrowing the frontier and the scheduler mutably
    // at the same time cannot alias.
    unsafe { f(&mut *visible_frontier, &mut *expansion) }
}

#[cfg(any(test, feature = "pg_test"))]
fn candidate_frontier_head(
    opaque: &mut TqScanOpaque,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    with_visible_frontier_and_bootstrap_expansion(opaque, |visible_frontier, expansion| {
        visible_frontier.best_candidate(expansion)
    })
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) fn current_candidate_frontier_head(
    opaque: &mut TqScanOpaque,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    candidate_frontier_head(opaque)
}

fn bootstrap_expansion_mut(
    opaque: &mut TqScanOpaque,
) -> &mut search::BeamSearch<page::ItemPointer> {
    if opaque.bootstrap_expansion.is_null() {
        reset_bootstrap_expansion_state(opaque, bootstrap_frontier_limit(opaque));
    }
    unsafe { &mut *opaque.bootstrap_expansion }
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
    let entry_candidate = search::BeamCandidate::new(entry.tid, entry_score);
    let bootstrap_limit = bootstrap_frontier_limit(opaque);
    let upper_layer_seeds = unsafe {
        graph::search_upper_layer_seed_candidates(
            index_relation,
            opaque.scan_code_len,
            usize::from(opaque.scan_m),
            entry_candidate,
            entry.level,
            bootstrap_limit,
            |neighbor| {
                Some(score_scan_element_result(
                    opaque,
                    neighbor.gamma,
                    &neighbor.code,
                ))
            },
        )
    };
    let ordered_candidates = unsafe {
        graph::search_layer0_result_candidates(
            index_relation,
            opaque.scan_code_len,
            usize::from(opaque.scan_m),
            bootstrap_limit,
            upper_layer_seeds,
            |neighbor_tid| !visited_contains_element(opaque, neighbor_tid),
            |neighbor| {
                Some(score_scan_element_result(
                    opaque,
                    neighbor.gamma,
                    &neighbor.code,
                ))
            },
        )
    };
    stage_ordered_graph_results(opaque, ordered_candidates);
}

fn stage_ordered_graph_results(
    opaque: &mut TqScanOpaque,
    candidates: impl IntoIterator<Item = search::BeamCandidate<page::ItemPointer>>,
) {
    clear_scan_candidate_state(opaque);
    reset_bootstrap_expansion_state(opaque, bootstrap_frontier_limit(opaque));
    reset_scan_expanded_state(opaque);
    seed_discovered_candidates(opaque, candidates);
}

#[cfg(any(test, feature = "pg_test"))]
fn seed_bootstrap_trace(
    opaque: &mut TqScanOpaque,
    max_candidates: usize,
    trace: search::BeamTrace<page::ItemPointer>,
) {
    reset_bootstrap_expansion_state(opaque, max_candidates);
    reset_scan_expanded_state(opaque);
    let opaque_ptr = opaque as *mut TqScanOpaque;
    with_visible_frontier_mut_and_bootstrap_expansion(
        unsafe { &mut *opaque_ptr },
        |visible_frontier, expansion| {
            visible_frontier.seed_bootstrap_trace(
                expansion,
                trace,
                max_candidates,
                |node| mark_visited_element(unsafe { &mut *opaque_ptr }, node),
                |node| mark_expanded_source(unsafe { &mut *opaque_ptr }, node),
            );
        },
    );
}

fn seed_discovered_candidates(
    opaque: &mut TqScanOpaque,
    candidates: impl IntoIterator<Item = impl Into<search::BeamCandidate<page::ItemPointer>>>,
) {
    let candidates = candidates.into_iter().map(Into::into).collect::<Vec<_>>();
    if candidates.is_empty() {
        return;
    }

    let opaque_ptr = opaque as *mut TqScanOpaque;
    with_visible_frontier_mut_and_bootstrap_expansion(
        unsafe { &mut *opaque_ptr },
        |visible_frontier, expansion| {
            visible_frontier.seed_discovered(expansion, candidates, |node| {
                mark_visited_element(unsafe { &mut *opaque_ptr }, node)
            });
        },
    );
}

#[cfg(any(test, feature = "pg_test"))]
fn seed_existing_frontier_into_expansion(opaque: &mut TqScanOpaque) {
    let candidates = visible_frontier_ref(opaque)
        .iter()
        .filter(|candidate| !expanded_contains_source(opaque, candidate.node))
        .collect::<Vec<_>>();
    bootstrap_expansion_mut(opaque).seed_many(candidates);
}

#[cfg(any(test, feature = "pg_test"))]
fn fill_bootstrap_frontier<F>(
    opaque: &mut TqScanOpaque,
    max_candidates: usize,
    policy: BootstrapExpandPolicy,
    refill: F,
) where
    F: FnMut(page::ItemPointer, &mut TqScanOpaque),
{
    reset_bootstrap_expansion_state(opaque, max_candidates);
    reset_scan_expanded_state(opaque);
    seed_existing_frontier_into_expansion(opaque);
    top_up_bootstrap_frontier(opaque, max_candidates, policy, refill);
}

#[cfg(any(test, feature = "pg_test"))]
fn top_up_bootstrap_frontier<F>(
    opaque: &mut TqScanOpaque,
    max_candidates: usize,
    policy: BootstrapExpandPolicy,
    mut refill: F,
) where
    F: FnMut(page::ItemPointer, &mut TqScanOpaque),
{
    while visible_frontier_ref(opaque).len() < max_candidates {
        let source_tid = match policy {
            BootstrapExpandPolicy::ScoreOrder => bootstrap_expansion_mut(opaque)
                .expand_one(|_| std::iter::empty::<search::BeamCandidate<page::ItemPointer>>())
                .map(|candidate| candidate.node),
        };
        let Some(source_tid) = source_tid else {
            break;
        };

        if expanded_contains_source(opaque, source_tid) {
            continue;
        }
        mark_expanded_source(opaque, source_tid);
        refill(source_tid, opaque);
    }
}

fn consume_candidate_frontier_head(
    opaque: &mut TqScanOpaque,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    with_visible_frontier_mut_and_bootstrap_expansion(opaque, |visible_frontier, expansion| {
        visible_frontier.consume_best(expansion)
    })
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn refill_candidate_frontier_from_source_into(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    visible_frontier: &mut VisibleCandidateFrontierState,
    expansion: &mut search::BeamSearch<page::ItemPointer>,
    source_tid: page::ItemPointer,
) {
    let opaque_ptr = opaque as *mut TqScanOpaque;
    visible_frontier.refill_from_source(
        expansion,
        bootstrap_frontier_limit(unsafe { &*opaque_ptr }),
        source_tid,
        |source_tid, max_successor_candidates| unsafe {
            graph::load_layer0_refill_successors(
                index_relation,
                (&*opaque_ptr).scan_code_len,
                usize::from((&*opaque_ptr).scan_m),
                source_tid,
                max_successor_candidates,
                |neighbor_tid| !visited_contains_element(&*opaque_ptr, neighbor_tid),
                |neighbor| {
                    Some(score_scan_element_result(
                        &*opaque_ptr,
                        neighbor.gamma,
                        &neighbor.code,
                    ))
                },
            )
        },
        |node| mark_visited_element(unsafe { &mut *opaque_ptr }, node),
    );
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn top_up_bootstrap_frontier_from_visible_seeds_into(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    visible_frontier: &mut VisibleCandidateFrontierState,
    expansion: &mut search::BeamSearch<page::ItemPointer>,
) {
    let opaque_ptr = opaque as *mut TqScanOpaque;
    visible_frontier.top_up_from_visible_seeds(
        expansion,
        bootstrap_frontier_limit(unsafe { &*opaque_ptr }),
        |node| expanded_contains_source(unsafe { &*opaque_ptr }, node),
        |seed_candidates, max_successor_candidates| {
            let expansion_trace = unsafe {
                graph::expand_layer0_visible_seeds(
                    index_relation,
                    (&*opaque_ptr).scan_code_len,
                    usize::from((&*opaque_ptr).scan_m),
                    max_successor_candidates,
                    seed_candidates.iter().copied(),
                    |neighbor_tid| !visited_contains_element(&*opaque_ptr, neighbor_tid),
                    |neighbor| {
                        Some(score_scan_element_result(
                            &*opaque_ptr,
                            neighbor.gamma,
                            &neighbor.code,
                        ))
                    },
                )
            };
            (
                expansion_trace.expanded_source_tids,
                expansion_trace.discovered_candidates,
            )
        },
        |node| mark_expanded_source(unsafe { &mut *opaque_ptr }, node),
        |node| mark_visited_element(unsafe { &mut *opaque_ptr }, node),
    );
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn refill_bootstrap_frontier_after_success(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    consumed: search::BeamCandidate<page::ItemPointer>,
) {
    let opaque_ptr = opaque as *mut TqScanOpaque;
    with_visible_frontier_mut_and_bootstrap_expansion(
        unsafe { &mut *opaque_ptr },
        |visible_frontier, expansion| unsafe {
            visible_frontier.advance_after_consume(
                expansion,
                consumed,
                |node| expanded_contains_source(&*opaque_ptr, node),
                |node| mark_expanded_source(&mut *opaque_ptr, node),
                |source_tid, visible_frontier, expansion| {
                    refill_candidate_frontier_from_source_into(
                        index_relation,
                        &mut *opaque_ptr,
                        visible_frontier,
                        expansion,
                        source_tid,
                    );
                },
                |visible_frontier, expansion| {
                    top_up_bootstrap_frontier_from_visible_seeds_into(
                        index_relation,
                        &mut *opaque_ptr,
                        visible_frontier,
                        expansion,
                    );
                },
            );
        },
    );
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) unsafe fn consume_and_refill_bootstrap_frontier(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) -> Option<search::BeamCandidate<page::ItemPointer>> {
    let consumed = consume_candidate_frontier_head(opaque)?;
    unsafe { refill_bootstrap_frontier_after_success(index_relation, opaque, consumed) };
    Some(consumed)
}

#[cfg(any(test, feature = "pg_test"))]
fn seed_scan_result_state(opaque: &mut TqScanOpaque, selected: SelectedScanResult) {
    mark_emitted_element(opaque, selected.element_tid);
    opaque.result_state.materialize(selected);
}

#[cfg(any(test, feature = "pg_test"))]
fn refill_bootstrap_frontier_after_consume<F>(
    opaque: &mut TqScanOpaque,
    consumed: search::BeamCandidate<page::ItemPointer>,
    mut refill: F,
) where
    F: FnMut(page::ItemPointer, &mut TqScanOpaque),
{
    if !expanded_contains_source(opaque, consumed.node) {
        mark_expanded_source(opaque, consumed.node);
        refill(consumed.node, opaque);
    }

    top_up_bootstrap_frontier(
        opaque,
        bootstrap_frontier_limit(opaque),
        BootstrapExpandPolicy::ScoreOrder,
        refill,
    );
}

#[cfg(test)]
fn select_next_bootstrap_candidate<CandidateFn, SelectFn>(
    mut next_candidate: CandidateFn,
    mut select: SelectFn,
) -> Option<SelectedScanResult>
where
    CandidateFn: FnMut() -> Option<search::BeamCandidate<page::ItemPointer>>,
    SelectFn: FnMut(search::BeamCandidate<page::ItemPointer>) -> Option<SelectedScanResult>,
{
    while let Some(candidate) = next_candidate() {
        if let Some(selected) = select(candidate) {
            return Some(selected);
        }
    }

    None
}

#[cfg(test)]
fn select_next_bootstrap_candidate_with_refill<CandidateFn, SelectFn, RefillFn>(
    mut next_candidate: CandidateFn,
    mut select: SelectFn,
    mut refill_after_success: RefillFn,
) -> Option<SelectedScanResult>
where
    CandidateFn: FnMut() -> Option<search::BeamCandidate<page::ItemPointer>>,
    SelectFn: FnMut(search::BeamCandidate<page::ItemPointer>) -> Option<SelectedScanResult>,
    RefillFn: FnMut(search::BeamCandidate<page::ItemPointer>),
{
    while let Some(candidate) = next_candidate() {
        if let Some(selected) = select(candidate) {
            refill_after_success(candidate);
            return Some(selected);
        }
    }

    None
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) unsafe fn prefetch_next_graph_traversal_result(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) -> bool {
    if !opaque.execution_phase.is_graph_traversal() || opaque.scan_dimensions == 0 {
        return false;
    }

    let opaque_ptr = opaque as *mut TqScanOpaque;
    unsafe { graph_traversal_cursor(opaque).prefetch_next(index_relation, opaque_ptr) }
}

unsafe fn produce_next_graph_traversal_heap_tid(
    scan: pg_sys::IndexScanDesc,
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
) -> bool {
    if !opaque.execution_phase.is_graph_traversal()
        || !graph_traversal_cursor(opaque).has_prefetched_output()
    {
        debug_assert!(
            opaque.execution_phase.is_exhausted(),
            "graph traversal tuple production should only run with prefetched output or an exhausted graph phase"
        );
        return false;
    }

    let emitted = graph_traversal_cursor(opaque).emit_prefetched_output(scan);
    debug_assert!(
        emitted,
        "graph traversal should materialize pending output before returning true from graph-phase tuple production"
    );
    if emitted {
        opaque.explain_counters.record_heap_tid_returned();
    }
    if emitted && graph_traversal_cursor(opaque).needs_prefetch_refresh() {
        let opaque_ptr = opaque as *mut TqScanOpaque;
        unsafe {
            graph_traversal_cursor(opaque).ensure_prefetched_output(index_relation, opaque_ptr);
        }
    }
    emitted
}

unsafe fn produce_next_linear_fallback_heap_tid(
    scan: pg_sys::IndexScanDesc,
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    code_len: usize,
) -> bool {
    if linear_fallback_cursor(opaque).emit_pending_output(scan) {
        linear_fallback_cursor(opaque).advance_after_emit();
        opaque.explain_counters.record_heap_tid_returned();
        return true;
    }

    let Some(selected) =
        (unsafe { select_next_linear_scan_result(index_relation, opaque, code_len) })
    else {
        return false;
    };

    mark_emitted_element(opaque, selected.element_tid);
    let emitted = linear_fallback_cursor(opaque).emit_materialized_output(scan, selected);
    if emitted {
        opaque.explain_counters.record_heap_tid_returned();
    }
    emitted
}

unsafe fn select_next_linear_scan_result(
    index_relation: pg_sys::Relation,
    opaque: &mut TqScanOpaque,
    code_len: usize,
) -> Option<SelectedScanResult> {
    if opaque.scan_block_count <= page::FIRST_DATA_BLOCK_NUMBER {
        mark_scan_exhausted(opaque);
        return None;
    }

    let max_block = opaque.scan_block_count.saturating_sub(1);
    opaque
        .linear_prefetch_state
        .reset(opaque.next_block_number, max_block);
    while let Some(block_number) = opaque.linear_prefetch_state.next_block() {
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
        opaque.explain_counters.record_linear_page_read();
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
                opaque.explain_counters.record_element_skipped();
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
                opaque.explain_counters.record_element_skipped();
                continue;
            }

            let element = page::TqElementTuple::decode(tuple_bytes, code_len).unwrap_or_else(|e| {
                pgrx::error!("tqhnsw failed to decode scan element tuple: {e}")
            });
            if element.deleted || element.heaptids.is_empty() {
                opaque.explain_counters.record_element_skipped();
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
                opaque.explain_counters.record_element_skipped();
                continue;
            }
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            opaque.explain_counters.record_element_scored();
            let score = score_scan_element_result(opaque, element.gamma, &element.code);
            return Some(SelectedScanResult {
                element_tid,
                score,
                heap_tids: element.heaptids,
            });
        }

        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        opaque.next_block_number = block_number + 1;
        opaque.next_offset_number = 1;
    }

    mark_scan_exhausted(opaque);
    None
}

#[cfg(test)]
fn collect_successor_candidates<F>(
    neighbor_tids: &[page::ItemPointer],
    max_candidates: usize,
    mut candidate_for_tid: F,
) -> Vec<search::BeamCandidate<page::ItemPointer>>
where
    F: FnMut(page::ItemPointer) -> Option<search::BeamCandidate<page::ItemPointer>>,
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

unsafe fn score_scan_element_result(opaque: &TqScanOpaque, gamma: f32, code_bytes: &[u8]) -> f32 {
    if opaque.prepared_query.is_null() {
        pgrx::error!("tqhnsw scan scoring requires a prepared query");
    }
    if opaque.cached_quantizer.is_null() {
        pgrx::error!("tqhnsw scan scoring requires a cached quantizer");
    }

    let quantizer = unsafe { &*opaque.cached_quantizer };
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

fn emit_scan_output(scan: pg_sys::IndexScanDesc, output: PendingScanOutput) {
    set_scan_heap_tid(scan, output.heap_tid);
    set_scan_orderby_score(scan, output.score);
}

fn set_scan_orderby_score(scan: pg_sys::IndexScanDesc, score: f32) {
    unsafe {
        if (*scan).xs_orderbyvals.is_null() {
            (*scan).xs_orderbyvals =
                pg_sys::palloc0(std::mem::size_of::<pg_sys::Datum>()).cast::<pg_sys::Datum>();
        }
        if (*scan).xs_orderbynulls.is_null() {
            (*scan).xs_orderbynulls = pg_sys::palloc0(std::mem::size_of::<bool>()).cast::<bool>();
        }

        *(*scan).xs_orderbyvals = score.into_datum().expect("score should convert to datum");
        *(*scan).xs_orderbynulls = false;
    }
}

fn clear_scan_orderby_output(scan: pg_sys::IndexScanDesc) {
    unsafe {
        if !(*scan).xs_orderbynulls.is_null() {
            *(*scan).xs_orderbynulls = true;
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(super) struct CurrentScanResult {
    element_tid: page::ItemPointer,
    heap_tid: page::ItemPointer,
    score: f32,
    score_valid: bool,
}

impl CurrentScanResult {
    pub(super) fn has_element(&self) -> bool {
        self.element_tid != page::ItemPointer::INVALID
    }

    pub(super) fn element_tid(&self) -> page::ItemPointer {
        self.element_tid
    }

    pub(super) fn heap_tid(&self) -> page::ItemPointer {
        self.heap_tid
    }

    pub(super) fn score(&self) -> f32 {
        self.score
    }

    pub(super) fn score_valid(&self) -> bool {
        self.score_valid
    }
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

#[derive(Debug)]
struct SelectedScanResult {
    element_tid: page::ItemPointer,
    score: f32,
    heap_tids: Vec<page::ItemPointer>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(super) struct ScanResultState {
    current: CurrentScanResult,
    pending_heaptids: [page::ItemPointer; page::HEAPTID_INLINE_CAPACITY],
    pending_heaptid_count: u8,
    pending_heaptid_index: u8,
}

impl ScanResultState {
    fn clear_pending(&mut self) {
        self.pending_heaptids.fill(page::ItemPointer::INVALID);
        self.pending_heaptid_count = 0;
        self.pending_heaptid_index = 0;
    }

    fn store_pending(&mut self, heaptids: &[page::ItemPointer]) {
        debug_assert!(heaptids.len() <= page::HEAPTID_INLINE_CAPACITY);

        self.clear_pending();
        self.pending_heaptid_count =
            u8::try_from(heaptids.len()).expect("heap tid count should fit in u8");

        for (index, tid) in heaptids.iter().copied().enumerate() {
            self.pending_heaptids[index] = tid;
        }
    }

    fn take_pending(&mut self) -> Option<page::ItemPointer> {
        if self.pending_heaptid_index >= self.pending_heaptid_count {
            return None;
        }

        let tid = self.pending_heaptids[self.pending_heaptid_index as usize];
        self.pending_heaptid_index += 1;
        if self.pending_heaptid_index >= self.pending_heaptid_count {
            self.clear_pending();
        }
        self.update_current_heap_tid(tid);
        Some(tid)
    }

    fn take_pending_output(&mut self) -> Option<PendingScanOutput> {
        let heap_tid = self.take_pending()?;
        Some(PendingScanOutput {
            heap_tid,
            score: self.current.score(),
        })
    }

    pub(super) fn clear(&mut self) {
        self.clear_pending();
        self.current = CurrentScanResult::default();
    }

    fn clear_current(&mut self) {
        self.current = CurrentScanResult::default();
    }

    fn materialize(&mut self, selected: SelectedScanResult) {
        self.set_current(selected.element_tid, selected.score);
        self.store_pending(&selected.heap_tids);
    }

    fn set_current(&mut self, element_tid: page::ItemPointer, score: f32) {
        self.current = CurrentScanResult {
            element_tid,
            heap_tid: page::ItemPointer::INVALID,
            score,
            score_valid: true,
        };
    }

    fn update_current_heap_tid(&mut self, heap_tid: page::ItemPointer) {
        if self.current.element_tid != page::ItemPointer::INVALID {
            self.current.heap_tid = heap_tid;
        }
    }

    pub(super) fn current(&self) -> CurrentScanResult {
        self.current
    }

    pub(super) fn pending_count(&self) -> u8 {
        self.pending_heaptid_count
    }

    pub(super) fn pending_index(&self) -> u8 {
        self.pending_heaptid_index
    }

    pub(super) fn pending_heap_tids(&self) -> &[page::ItemPointer] {
        &self.pending_heaptids[..self.pending_heaptid_count as usize]
    }
}

impl Default for ScanResultState {
    fn default() -> Self {
        Self {
            current: CurrentScanResult::default(),
            pending_heaptids: [page::ItemPointer::INVALID; page::HEAPTID_INLINE_CAPACITY],
            pending_heaptid_count: 0,
            pending_heaptid_index: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum ScanExecutionPhase {
    #[default]
    GraphTraversal,
    LinearFallback,
    Exhausted,
}

impl ScanExecutionPhase {
    pub(super) fn is_graph_traversal(self) -> bool {
        matches!(self, Self::GraphTraversal)
    }

    pub(super) fn is_exhausted(self) -> bool {
        matches!(self, Self::Exhausted)
    }
}

#[repr(C)]
#[derive(Debug)]
pub(super) struct TqScanOpaque {
    pub(super) rescan_called: bool,
    pub(super) query_dimensions: u16,
    pub(super) query_values: *mut f32,
    pub(super) prepared_query: *mut PreparedQuery,
    pub(super) cached_quantizer: *const ProdQuantizer,
    pub(super) scan_dimensions: u16,
    pub(super) scan_m: u16,
    pub(super) scan_bits: u8,
    pub(super) scan_seed: u64,
    pub(super) scan_code_len: usize,
    pub(super) scan_block_count: u32,
    pub(super) bootstrap_frontier_limit: usize,
    pub(super) visited_tids: *mut HashSet<page::ItemPointer>,
    pub(super) expanded_source_tids: *mut HashSet<page::ItemPointer>,
    pub(super) emitted_result_tids: *mut HashSet<page::ItemPointer>,
    pub(super) candidate_frontier: *mut VisibleCandidateFrontierState,
    pub(super) bootstrap_expansion: *mut search::BeamSearch<page::ItemPointer>,
    pub(super) result_state: ScanResultState,
    pub(super) fallback_result_state: ScanResultState,
    pub(super) next_block_number: u32,
    pub(super) next_offset_number: u16,
    pub(super) execution_phase: ScanExecutionPhase,
    pub(super) graph_prefetch_state: *mut GraphPrefetchState,
    pub(super) linear_prefetch_state: LinearPrefetchState,
    pub(super) explain_counters: TqExplainCounters,
}

impl Default for TqScanOpaque {
    fn default() -> Self {
        Self {
            rescan_called: false,
            query_dimensions: 0,
            query_values: ptr::null_mut(),
            prepared_query: ptr::null_mut(),
            cached_quantizer: ptr::null(),
            scan_dimensions: 0,
            scan_m: 0,
            scan_bits: 0,
            scan_seed: 0,
            scan_code_len: 0,
            scan_block_count: 0,
            bootstrap_frontier_limit: MAX_BOOTSTRAP_FRONTIER_CANDIDATES,
            visited_tids: ptr::null_mut(),
            expanded_source_tids: ptr::null_mut(),
            emitted_result_tids: ptr::null_mut(),
            candidate_frontier: ptr::null_mut(),
            bootstrap_expansion: ptr::null_mut(),
            result_state: ScanResultState::default(),
            fallback_result_state: ScanResultState::default(),
            next_block_number: page::FIRST_DATA_BLOCK_NUMBER,
            next_offset_number: 1,
            execution_phase: ScanExecutionPhase::GraphTraversal,
            graph_prefetch_state: ptr::null_mut(),
            linear_prefetch_state: LinearPrefetchState::new(
                page::FIRST_DATA_BLOCK_NUMBER,
                page::FIRST_DATA_BLOCK_NUMBER,
            ),
            explain_counters: TqExplainCounters::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tid(block_number: u32, offset_number: u16) -> page::ItemPointer {
        page::ItemPointer {
            block_number,
            offset_number,
        }
    }

    fn beam_candidate(
        block_number: u32,
        offset_number: u16,
        score: f32,
    ) -> search::BeamCandidate<page::ItemPointer> {
        search::BeamCandidate::new(tid(block_number, offset_number), score)
    }

    fn sourced_beam_candidate(
        block_number: u32,
        offset_number: u16,
        source_tid: page::ItemPointer,
        score: f32,
    ) -> search::BeamCandidate<page::ItemPointer> {
        search::BeamCandidate::with_source(tid(block_number, offset_number), score, source_tid)
    }

    #[test]
    fn select_next_bootstrap_candidate_skips_unselectable_candidates() {
        let mut queued = vec![beam_candidate(21, 1, -3.0), beam_candidate(21, 2, -2.0)].into_iter();
        let mut attempted = Vec::new();

        let selected = select_next_bootstrap_candidate(
            || queued.next(),
            |candidate| {
                attempted.push((candidate.node.block_number, candidate.node.offset_number));
                (candidate.node.offset_number == 2).then(|| SelectedScanResult {
                    element_tid: candidate.node,
                    score: candidate.score,
                    heap_tids: vec![tid(41, 1)],
                })
            },
        );

        assert!(
            selected.is_some(),
            "bootstrap selection should keep trying later candidates after one fails"
        );
        assert_eq!(
            attempted,
            vec![(21, 1), (21, 2)],
            "candidate selection should proceed in consumption order until one succeeds"
        );
    }

    #[test]
    fn select_next_bootstrap_candidate_returns_none_when_frontier_never_selects() {
        let mut queued = vec![beam_candidate(22, 1, -3.0), beam_candidate(22, 2, -2.0)].into_iter();
        let mut attempts = 0;

        let selected = select_next_bootstrap_candidate(
            || queued.next(),
            |_candidate| {
                attempts += 1;
                None
            },
        );

        assert!(
            selected.is_none(),
            "bootstrap selection should return none only after every candidate fails"
        );
        assert_eq!(
            attempts, 2,
            "bootstrap selection should exhaust the queued frontier before giving up"
        );
    }

    #[test]
    fn select_next_bootstrap_candidate_refills_only_after_successful_adjudication() {
        let candidate_a = beam_candidate(23, 1, -3.0);
        let candidate_b = beam_candidate(23, 2, -2.0);
        let mut queued = vec![candidate_a, candidate_b].into_iter();
        let mut attempted = Vec::new();
        let mut refilled_after = Vec::new();

        let selected = select_next_bootstrap_candidate_with_refill(
            || queued.next(),
            |candidate| {
                attempted.push(candidate.node);
                (candidate == candidate_b).then(|| SelectedScanResult {
                    element_tid: candidate.node,
                    score: candidate.score,
                    heap_tids: vec![tid(42, 1)],
                })
            },
            |candidate| refilled_after.push(candidate.node),
        );

        assert!(
            selected.is_some(),
            "bootstrap selection should still succeed once a later visible candidate selects"
        );
        assert_eq!(
            attempted,
            vec![candidate_a.node, candidate_b.node],
            "bootstrap candidates should be adjudicated in consume order before any refill path runs"
        );
        assert_eq!(
            refilled_after,
            vec![candidate_b.node],
            "bootstrap refill should only run for the candidate that actually materialized"
        );
    }

    #[test]
    fn enter_linear_fallback_phase_clears_frontier_scheduler_and_expanded_state() {
        let mut opaque = TqScanOpaque::default();
        visible_frontier_mut(&mut opaque).push(beam_candidate(24, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(24, 2, -2.0));
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        reset_scan_expanded_state(&mut opaque);
        seed_existing_frontier_into_expansion(&mut opaque);
        mark_expanded_source(&mut opaque, tid(24, 1));

        enter_linear_fallback_phase(&mut opaque);

        assert!(
            opaque.execution_phase == ScanExecutionPhase::LinearFallback,
            "entering linear fallback should transition the scan into its explicit fallback phase"
        );
        assert!(
            visible_frontier_candidates(&opaque).is_empty(),
            "entering linear fallback should clear any leftover visible frontier candidates"
        );
        assert!(
            bootstrap_expansion_mut(&mut opaque).peek_best().is_none(),
            "entering linear fallback should clear the scan-owned scheduler too"
        );
        assert!(
            !expanded_contains_source(&opaque, tid(24, 1)),
            "entering linear fallback should reset expanded-source bookkeeping for the next rescan"
        );
    }

    #[test]
    fn mark_scan_exhausted_clears_result_state() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(25, 1), -3.0);
        opaque.result_state.store_pending(&[tid(30, 1), tid(30, 2)]);

        mark_scan_exhausted(&mut opaque);

        assert!(
            opaque.execution_phase == ScanExecutionPhase::Exhausted,
            "exhausting the scan should transition into the explicit exhausted phase"
        );
        assert!(
            !opaque.result_state.current().has_element(),
            "exhausting the scan should clear the current result slot"
        );
        assert_eq!(
            opaque.result_state.pending_count(),
            0,
            "exhausting the scan should also clear pending duplicate-drain state"
        );
    }

    #[test]
    fn reset_scan_position_restores_bootstrap_execution_phase() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };

        reset_scan_position(&mut opaque);

        assert!(
            opaque.execution_phase == ScanExecutionPhase::GraphTraversal,
            "amrescan/reset should allow graph traversal to run again after prior fallback-phase scans"
        );
        assert!(
            candidate_frontier_head(&mut opaque).is_none(),
            "amrescan/reset should clear prior graph traversal frontier state before rebuilding it"
        );
    }

    #[test]
    fn unseeded_scans_enter_linear_fallback_explicitly() {
        let mut opaque = TqScanOpaque::default();

        enter_linear_fallback_phase(&mut opaque);

        assert_eq!(
            opaque.execution_phase,
            ScanExecutionPhase::LinearFallback,
            "unseeded scans should enter the explicit linear fallback phase"
        );
    }

    #[test]
    fn scan_result_state_take_pending_advances_current_result_progress() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(25, 1), -3.0);
        opaque.result_state.store_pending(&[tid(30, 1), tid(30, 2)]);

        let first = opaque.result_state.take_pending();
        let second = opaque.result_state.take_pending();
        let exhausted = opaque.result_state.take_pending();

        assert_eq!(
            first,
            Some(tid(30, 1)),
            "pending result drain should return the first queued heap tid first"
        );
        assert_eq!(
            second,
            Some(tid(30, 2)),
            "pending result drain should continue through later heap tids in order"
        );
        assert_eq!(
            exhausted, None,
            "pending result drain should stop once the queued heap tids are exhausted"
        );
        assert_eq!(
            opaque.result_state.current().heap_tid(),
            tid(30, 2),
            "draining pending heap tids should keep the current-result heap tid aligned with the last emitted duplicate"
        );
        assert_eq!(
            opaque.result_state.pending_count(),
            0,
            "draining all queued heap tids should reset the pending count"
        );
        assert_eq!(
            opaque.result_state.pending_index(),
            0,
            "draining all queued heap tids should reset the pending index too"
        );
    }

    #[test]
    fn scan_result_state_take_pending_output_preserves_score_and_heap_progress() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(26, 1), -4.0);
        opaque.result_state.store_pending(&[tid(31, 1), tid(31, 2)]);

        let first = opaque.result_state.take_pending_output();
        let second = opaque.result_state.take_pending_output();
        let exhausted = opaque.result_state.take_pending_output();

        assert_eq!(
            first,
            Some(PendingScanOutput {
                heap_tid: tid(31, 1),
                score: -4.0,
            }),
            "pending output should expose the first heap tid together with the current result score"
        );
        assert_eq!(
            second,
            Some(PendingScanOutput {
                heap_tid: tid(31, 2),
                score: -4.0,
            }),
            "pending output should preserve score while draining later heap tids from the same result"
        );
        assert_eq!(
            exhausted, None,
            "pending output should report exhaustion once the duplicate drain is complete"
        );
    }

    #[test]
    fn linear_fallback_cursor_advance_after_emit_keeps_current_result_until_last_duplicate() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };
        opaque.fallback_result_state.set_current(tid(26, 1), -4.0);
        opaque
            .fallback_result_state
            .store_pending(&[tid(31, 1), tid(31, 2)]);

        let first = linear_fallback_cursor(&mut opaque).take_pending_output();
        linear_fallback_cursor(&mut opaque).advance_after_emit();

        assert_eq!(
            first,
            Some(PendingScanOutput {
                heap_tid: tid(31, 1),
                score: -4.0,
            }),
            "linear fallback duplicate drain should still emit the first queued heap tid"
        );
        assert!(
            opaque.fallback_result_state.current().has_element(),
            "linear fallback should keep the current result populated while duplicate drain still remains"
        );
        assert_eq!(
            opaque.fallback_result_state.current().heap_tid(),
            tid(31, 1),
            "linear fallback should keep heap progress aligned with the last emitted duplicate"
        );
    }

    #[test]
    fn linear_fallback_cursor_advance_after_emit_clears_current_result_after_last_duplicate() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };
        opaque.fallback_result_state.set_current(tid(27, 1), -5.0);
        opaque.fallback_result_state.store_pending(&[tid(32, 1)]);

        let emitted = linear_fallback_cursor(&mut opaque).take_pending_output();
        linear_fallback_cursor(&mut opaque).advance_after_emit();

        assert_eq!(
            emitted,
            Some(PendingScanOutput {
                heap_tid: tid(32, 1),
                score: -5.0,
            }),
            "linear fallback should still emit the final queued heap tid before teardown"
        );
        assert!(
            !opaque.fallback_result_state.current().has_element(),
            "linear fallback should clear stale current-result state after the last duplicate drains"
        );
        assert_eq!(
            opaque.fallback_result_state.pending_count(),
            0,
            "linear fallback teardown should only happen once duplicate drain is exhausted"
        );
    }

    #[test]
    fn graph_traversal_prefetch_ready_clears_stale_current_without_pending_output() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::GraphTraversal,
            ..TqScanOpaque::default()
        };
        opaque.result_state.set_current(tid(28, 1), -6.0);

        let ready = graph_traversal_cursor(&mut opaque).prefetch_ready();

        assert!(
            !ready,
            "graph traversal should request a fresh materialization when only stale current-result state remains"
        );
        assert!(
            !opaque.result_state.current().has_element(),
            "graph traversal should clear stale current-result state before trying to prefill a fresh ordered result"
        );
        assert_eq!(
            opaque.result_state.pending_count(),
            0,
            "graph traversal stale-current cleanup should not invent pending duplicate-drain state"
        );
    }

    #[test]
    fn graph_traversal_cursor_has_prefetched_output_requires_pending_duplicate_drain() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::GraphTraversal,
            ..TqScanOpaque::default()
        };
        opaque.result_state.set_current(tid(29, 1), -7.0);

        assert!(
            !graph_traversal_cursor(&mut opaque).has_prefetched_output(),
            "graph traversal should only report prefetched output when duplicate drain is actually queued"
        );

        opaque.result_state.store_pending(&[tid(33, 1)]);

        assert!(
            graph_traversal_cursor(&mut opaque).has_prefetched_output(),
            "graph traversal should report prefetched output once a current result has pending heap tids ready to emit"
        );
    }

    #[test]
    fn graph_traversal_cursor_take_pending_output_drains_prefetched_heap_tid() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(34, 1), -8.0);
        opaque.result_state.store_pending(&[tid(35, 1)]);

        let emitted = graph_traversal_cursor(&mut opaque).take_pending_output();

        assert!(
            emitted.is_some(),
            "graph cursor should surface pending output when prefetched duplicate drain is queued"
        );
        assert_eq!(
            opaque.result_state.current().heap_tid(),
            tid(35, 1),
            "graph cursor pending-output drain should keep current-result heap progress aligned with the drained heap tid"
        );
        assert_eq!(
            opaque.result_state.pending_count(),
            0,
            "graph cursor pending-output drain should consume the prefetched heap tid from pending state"
        );
    }

    #[test]
    fn linear_fallback_cursor_uses_fallback_storage_in_linear_phase() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };
        opaque.result_state.set_current(tid(36, 1), -9.0);

        linear_fallback_cursor(&mut opaque).materialize(SelectedScanResult {
            element_tid: tid(37, 1),
            score: -10.0,
            heap_tids: vec![tid(38, 1)],
        });

        assert_eq!(
            opaque.fallback_result_state.current().element_tid(),
            tid(37, 1),
            "linear fallback should read and write through its dedicated fallback result-state storage"
        );
        assert_eq!(
            opaque.result_state.current().element_tid(),
            tid(36, 1),
            "linear fallback cursor should not backfill graph cursor result-state storage"
        );
    }

    #[test]
    fn linear_fallback_cursor_materialize_uses_fallback_storage() {
        let mut opaque = TqScanOpaque {
            execution_phase: ScanExecutionPhase::LinearFallback,
            ..TqScanOpaque::default()
        };

        linear_fallback_cursor(&mut opaque).materialize(SelectedScanResult {
            element_tid: tid(38, 1),
            score: -11.0,
            heap_tids: vec![tid(39, 1)],
        });

        assert_eq!(
            opaque.fallback_result_state.current().element_tid(),
            tid(38, 1),
            "linear fallback materialization should populate fallback-only result-state storage"
        );
        assert_eq!(
            opaque.result_state.current().element_tid(),
            page::ItemPointer::INVALID,
            "linear fallback materialization should not backfill graph cursor result-state storage"
        );
    }

    #[test]
    fn scan_result_state_clear_clears_pending_heap_tid_drain() {
        let mut opaque = TqScanOpaque::default();
        opaque.result_state.set_current(tid(26, 1), -4.0);
        opaque.result_state.store_pending(&[tid(31, 1), tid(31, 2)]);

        opaque.result_state.clear();

        assert!(
            !opaque.result_state.current().has_element(),
            "clearing scan result state should also clear the current result slot"
        );
        assert_eq!(
            opaque.result_state.pending_count(),
            0,
            "clearing scan result state should clear any pending duplicate drain state"
        );
        assert_eq!(
            opaque.result_state.pending_index(),
            0,
            "clearing scan result state should reset duplicate drain progress"
        );
        assert_eq!(
            opaque
                .result_state
                .pending_heap_tids()
                .first()
                .copied()
                .unwrap_or(page::ItemPointer::INVALID),
            page::ItemPointer::INVALID,
            "clearing scan result state should wipe the pending heap-tid buffer too"
        );
        assert!(
            opaque.result_state.pending_heap_tids().is_empty(),
            "clearing scan result state should expose no pending heap tids after reset"
        );
    }

    #[test]
    fn seed_scan_result_state_seeds_current_result_and_pending_drain() {
        let mut opaque = TqScanOpaque::default();

        seed_scan_result_state(
            &mut opaque,
            SelectedScanResult {
                element_tid: tid(26, 1),
                score: -4.5,
                heap_tids: vec![tid(31, 1), tid(31, 2)],
            },
        );

        assert_eq!(
            opaque.result_state.current().element_tid(),
            tid(26, 1),
            "shared result materialization should record the element tid on current-result state"
        );
        assert_eq!(
            opaque.result_state.current().score(),
            -4.5,
            "shared result materialization should preserve the supplied score"
        );
        assert_eq!(
            opaque.result_state.pending_count(),
            2,
            "shared result materialization should seed pending duplicate drain"
        );
        assert_eq!(
            opaque.result_state.pending_heap_tids()[0],
            tid(31, 1),
            "shared result materialization should preserve heap-tid order for later drain"
        );
        assert_eq!(
            opaque.result_state.pending_heap_tids()[1],
            tid(31, 2),
            "shared result materialization should retain all supplied heap tids"
        );
    }

    #[test]
    fn prepared_query_cache_lifetime_tracks_scan_state() {
        let metadata = page::MetadataPage {
            m: 8,
            ef_construction: 32,
            entry_point: page::ItemPointer::INVALID,
            dimensions: 4,
            bits: 4,
            max_level: 0,
            seed: 42,
        };
        let query = [1.0_f32, 2.0, 3.0, 4.0];
        let mut opaque = TqScanOpaque::default();

        store_scan_prepared_query(&mut opaque, &query, &metadata);

        assert!(
            !opaque.prepared_query.is_null(),
            "storing a prepared query should retain the prepared-query payload"
        );
        assert!(
            !opaque.cached_quantizer.is_null(),
            "storing a prepared query should retain the quantizer used to score future elements"
        );

        free_scan_prepared_query(&mut opaque);

        assert!(
            opaque.prepared_query.is_null(),
            "freeing scan prepared-query state should release the prepared query payload"
        );
        assert!(
            opaque.cached_quantizer.is_null(),
            "freeing scan prepared-query state should release the cached quantizer too"
        );
    }

    #[test]
    fn consume_candidate_frontier_head_reselects_then_clears() {
        let mut opaque = TqScanOpaque::default();
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        visible_frontier_mut(&mut opaque).push(beam_candidate(7, 1, -2.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(7, 2, 3.5));
        assert_eq!(
            candidate_frontier_head(&mut opaque)
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((7, 1)),
            "frontier head should start at the lower-scoring valid candidate"
        );

        let consumed = consume_candidate_frontier_head(&mut opaque)
            .expect("frontier head consumption should return the current best slot");
        assert_eq!(
            (consumed.node.block_number, consumed.node.offset_number),
            (7, 1),
            "consumption should return the previously best frontier slot"
        );
        assert_eq!(
            candidate_frontier_head(&mut opaque)
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((7, 2)),
            "consuming the best slot should reselect the remaining valid candidate"
        );
        assert!(
            visible_frontier_slot(&opaque, 0).is_some(),
            "consuming the head should keep the remaining candidate valid"
        );
        assert_eq!(
            visible_frontier_slot(&opaque, 0)
                .map(|candidate| candidate.score)
                .unwrap_or(0.0),
            3.5,
            "consuming the head should preserve the remaining candidate after compaction"
        );

        let consumed = consume_candidate_frontier_head(&mut opaque)
            .expect("a remaining valid slot should still be consumable");
        assert_eq!(
            (consumed.node.block_number, consumed.node.offset_number),
            (7, 2),
            "the second consumption should return the reseated head slot"
        );
        assert_eq!(
            candidate_frontier_head(&mut opaque).map(|candidate| candidate.node),
            None,
            "consuming the last valid slot should invalidate the frontier head"
        );
        assert!(
            visible_frontier_candidates(&opaque).is_empty(),
            "consuming both valid slots should leave the candidate vector empty"
        );
        assert!(
            consume_candidate_frontier_head(&mut opaque).is_none(),
            "consuming an empty frontier should stay a no-op"
        );
    }

    #[test]
    fn consuming_frontier_head_forgets_it_from_bootstrap_scheduler() {
        let mut opaque = TqScanOpaque::default();
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        visible_frontier_mut(&mut opaque).push(beam_candidate(13, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(13, 2, -1.0));
        seed_existing_frontier_into_expansion(&mut opaque);

        let consumed = consume_candidate_frontier_head(&mut opaque)
            .expect("frontier head consumption should succeed");
        assert_eq!(
            (consumed.node.block_number, consumed.node.offset_number),
            (13, 1),
            "the lower-score candidate should be consumed first"
        );
        assert_eq!(
            bootstrap_expansion_mut(&mut opaque)
                .peek_best()
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((13, 2)),
            "consuming a frontier head should immediately forget it from the scan-owned scheduler"
        );
    }

    #[test]
    fn current_candidate_frontier_head_tid_prefers_scheduler_best_node() {
        let mut opaque = TqScanOpaque::default();
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        visible_frontier_mut(&mut opaque).push(beam_candidate(14, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(14, 2, -1.0));

        bootstrap_expansion_mut(&mut opaque).seed(search::BeamCandidate::new(
            page::ItemPointer {
                block_number: 14,
                offset_number: 2,
            },
            -1.0,
        ));
        assert_eq!(
            candidate_frontier_head(&mut opaque)
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((14, 2)),
            "frontier-head derivation should prefer the scan-owned scheduler's current best queued node"
        );
    }

    #[test]
    fn current_candidate_frontier_head_tid_falls_back_after_scheduler_drains() {
        let mut opaque = TqScanOpaque::default();
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        visible_frontier_mut(&mut opaque).push(beam_candidate(17, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(17, 2, -1.0));
        seed_existing_frontier_into_expansion(&mut opaque);

        bootstrap_expansion_mut(&mut opaque)
            .expand_one(|_| std::iter::empty::<search::BeamCandidate<page::ItemPointer>>());
        bootstrap_expansion_mut(&mut opaque)
            .expand_one(|_| std::iter::empty::<search::BeamCandidate<page::ItemPointer>>());

        assert!(
            bootstrap_expansion_mut(&mut opaque).peek_best().is_none(),
            "expanding both seeded sources should drain the scheduler while leaving the visible frontier intact"
        );
        assert_eq!(
            candidate_frontier_head(&mut opaque)
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((17, 1)),
            "frontier-head derivation must still fall back to the visible frontier once the scheduler has no queued expansion sources"
        );
    }

    #[test]
    fn consume_candidate_frontier_head_prefers_scheduler_best_node() {
        let mut opaque = TqScanOpaque::default();
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        visible_frontier_mut(&mut opaque).push(beam_candidate(15, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(beam_candidate(15, 2, -1.0));

        bootstrap_expansion_mut(&mut opaque).seed(search::BeamCandidate::new(
            page::ItemPointer {
                block_number: 15,
                offset_number: 2,
            },
            -1.0,
        ));

        let consumed = consume_candidate_frontier_head(&mut opaque)
            .expect("frontier consumption should prefer the scheduler's best queued node");
        assert_eq!(
            (consumed.node.block_number, consumed.node.offset_number),
            (15, 2),
            "scheduler-owned best-node selection should override Vec score order during consumption"
        );
        assert_eq!(
            visible_frontier_slot(&opaque, 0).map(|candidate| candidate.node),
            Some(page::ItemPointer {
                block_number: 15,
                offset_number: 1,
            }),
            "consumption should remove the scheduler-selected visible candidate from the compacted frontier"
        );
    }

    #[test]
    fn current_candidate_frontier_head_tid_drops_stale_scheduler_nodes() {
        let mut opaque = TqScanOpaque::default();
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        visible_frontier_mut(&mut opaque).push(beam_candidate(16, 1, -2.0));

        bootstrap_expansion_mut(&mut opaque).seed(search::BeamCandidate::new(
            page::ItemPointer {
                block_number: 16,
                offset_number: 9,
            },
            -3.0,
        ));
        bootstrap_expansion_mut(&mut opaque).seed(search::BeamCandidate::new(
            page::ItemPointer {
                block_number: 16,
                offset_number: 1,
            },
            -2.0,
        ));

        assert_eq!(
            candidate_frontier_head(&mut opaque)
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((16, 1)),
            "stale scheduler nodes should be dropped until the best queued visible frontier node can be mapped"
        );
        assert_eq!(
            bootstrap_expansion_mut(&mut opaque)
                .peek_best()
                .map(|candidate| (candidate.node.block_number, candidate.node.offset_number)),
            Some((16, 1)),
            "recompute should purge unmappable scheduler nodes instead of leaving them at the head forever"
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

                Some(search::BeamCandidate::new(neighbor_tid, 2.5))
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
                .map(|candidate| candidate.node)
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
        visible_frontier_mut(&mut opaque).push(beam_candidate(9, 1, -3.0));

        fill_bootstrap_frontier(
            &mut opaque,
            3,
            BootstrapExpandPolicy::ScoreOrder,
            |source_tid, opaque| match (source_tid.block_number, source_tid.offset_number) {
                (9, 1) => {
                    seed_discovered_candidates(
                        opaque,
                        [sourced_beam_candidate(9, 2, source_tid, -2.0)],
                    );
                }
                (9, 2) => {
                    seed_discovered_candidates(
                        opaque,
                        [sourced_beam_candidate(9, 3, source_tid, -1.0)],
                    );
                }
                _ => {}
            },
        );

        assert_eq!(
            visible_frontier_candidates(&opaque)
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![entry_tid, child_tid, grandchild_tid],
            "bootstrap frontier filling should keep expanding from newly seeded candidates until capacity is reached"
        );
        assert_eq!(
            visible_frontier_candidates(&opaque)[0].source,
            None,
            "entry-seeded candidates should not claim a discovery source"
        );
        assert_eq!(
            visible_frontier_candidates(&opaque)[1].source,
            Some(entry_tid),
            "first-hop candidates should record the entry candidate as their source"
        );
        assert_eq!(
            visible_frontier_candidates(&opaque)[2].source,
            Some(child_tid),
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
        visible_frontier_mut(&mut opaque).push(beam_candidate(11, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(sourced_beam_candidate(11, 2, entry_tid, -2.0));
        mark_expanded_source(&mut opaque, entry_tid);
        reset_bootstrap_expansion_state(&mut opaque, 3);
        seed_existing_frontier_into_expansion(&mut opaque);

        top_up_bootstrap_frontier(
            &mut opaque,
            3,
            BootstrapExpandPolicy::ScoreOrder,
            |source_tid, opaque| {
                if source_tid == sibling_tid {
                    seed_discovered_candidates(
                        opaque,
                        [sourced_beam_candidate(11, 3, source_tid, -1.0)],
                    );
                }
            },
        );

        assert_eq!(
            visible_frontier_candidates(&opaque)
                .iter()
                .map(|candidate| candidate.node)
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
    fn top_up_bootstrap_frontier_requires_seeded_scheduler() {
        let entry_tid = page::ItemPointer {
            block_number: 12,
            offset_number: 1,
        };
        let mut opaque = TqScanOpaque::default();
        visible_frontier_mut(&mut opaque).push(beam_candidate(12, 1, -3.0));
        reset_bootstrap_expansion_state(&mut opaque, 3);

        top_up_bootstrap_frontier(
            &mut opaque,
            3,
            BootstrapExpandPolicy::ScoreOrder,
            |_, opaque| {
                seed_discovered_candidates(
                    opaque,
                    [sourced_beam_candidate(12, 2, entry_tid, -2.0)],
                );
            },
        );

        assert_eq!(
            visible_frontier_candidates(&opaque)
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![entry_tid],
            "top-up should not silently rebuild beam state from the visible frontier when the scheduler is empty"
        );
        assert!(
            !expanded_contains_source(&opaque, entry_tid),
            "without a seeded scheduler, top-up should not mark any source as expanded"
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
        visible_frontier_mut(&mut opaque).push(sourced_beam_candidate(12, 2, consumed_tid, -2.0));
        mark_expanded_source(&mut opaque, consumed_tid);
        reset_bootstrap_expansion_state(&mut opaque, MAX_BOOTSTRAP_FRONTIER_CANDIDATES);
        seed_existing_frontier_into_expansion(&mut opaque);

        let mut refilled_sources = Vec::new();
        refill_bootstrap_frontier_after_consume(
            &mut opaque,
            search::BeamCandidate::new(consumed_tid, -3.0),
            |source_tid, opaque| {
                refilled_sources.push(source_tid);
                if source_tid == sibling_tid {
                    seed_discovered_candidates(
                        opaque,
                        [sourced_beam_candidate(12, 3, source_tid, -1.0)],
                    );
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
            visible_frontier_candidates(&opaque)
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![sibling_tid, grandchild_tid],
            "consume/refill should still top up from another remaining unexpanded frontier candidate"
        );
    }

    #[test]
    fn score_order_policy_prefers_lowest_score_unexpanded_frontier_candidate() {
        let mut opaque = TqScanOpaque::default();
        reset_scan_expanded_state(&mut opaque);
        visible_frontier_mut(&mut opaque).push(beam_candidate(10, 1, -3.0));
        visible_frontier_mut(&mut opaque).push(sourced_beam_candidate(10, 2, tid(10, 1), -4.0));

        assert_eq!(
            visible_frontier_ref(&opaque)
                .iter()
                .filter(|candidate| !expanded_contains_source(&opaque, candidate.node))
                .min_by(|left, right| left.score.total_cmp(&right.score))
                .map(|candidate| candidate.node),
            Some(page::ItemPointer {
                block_number: 10,
                offset_number: 2,
            }),
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
            visible_frontier_ref(&opaque)
                .iter()
                .filter(|candidate| !expanded_contains_source(&opaque, candidate.node))
                .min_by(|left, right| left.score.total_cmp(&right.score))
                .map(|candidate| candidate.node),
            Some(page::ItemPointer {
                block_number: 10,
                offset_number: 1,
            }),
            "after the best candidate is marked expanded, the score-order policy should fall back to the next best seeded candidate"
        );
    }

    #[test]
    fn seed_bootstrap_trace_marks_only_seed_entry_as_expanded() {
        let entry_tid = tid(15, 1);
        let sibling_tid = tid(15, 2);
        let grandchild_tid = tid(15, 3);
        let mut opaque = TqScanOpaque::default();

        seed_bootstrap_trace(
            &mut opaque,
            3,
            search::BeamTrace {
                discovered: vec![
                    beam_candidate(15, 1, -3.0),
                    sourced_beam_candidate(15, 2, entry_tid, -2.0),
                    sourced_beam_candidate(15, 3, sibling_tid, -1.0),
                ],
                expanded: vec![
                    beam_candidate(15, 1, -3.0),
                    sourced_beam_candidate(15, 2, entry_tid, -2.0),
                ],
                frontier: vec![sourced_beam_candidate(15, 3, sibling_tid, -1.0)],
            },
        );

        assert!(
            expanded_contains_source(&opaque, entry_tid),
            "trace seeding should keep the entry candidate marked expanded"
        );
        assert!(
            !expanded_contains_source(&opaque, sibling_tid),
            "trace seeding should not pre-mark later discovered candidates as expanded"
        );
        assert!(
            !expanded_contains_source(&opaque, grandchild_tid),
            "trace seeding should leave deeper discovered candidates available for later refill"
        );
    }
}
