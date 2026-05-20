#[cfg(any(test, feature = "pg_test"))]
use std::ptr;
#[cfg(any(test, feature = "pg_test"))]
use std::time::Instant;

#[cfg(any(test, feature = "pg_test"))]
use pgrx::{pg_sys, FromDatum};

#[cfg(any(test, feature = "pg_test"))]
use super::scan::*;
#[cfg(any(test, feature = "pg_test"))]
use super::{graph, page, search};
#[cfg(any(test, feature = "pg_test"))]
use crate::storage::{
    buffer_guard::LockedBufferGuard,
    relation_guard::{HeapRelationGuard, IndexRelationGuard},
    scan_guard::IndexScanGuard,
    slot_guard::TupleTableSlotGuard,
    snapshot_guard::ActiveSnapshotGuard,
};

#[cfg(any(test, feature = "pg_test"))]
pub(crate) type HeapTidCoords = (u32, u16);

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateSlot = (bool, HeapTidCoords, f32);

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateProvenanceSlot = (bool, HeapTidCoords, HeapTidCoords, f32);

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateHead = Option<HeapTidCoords>;

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontierSlots = Vec<DebugCandidateSlot>;

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontierProvenanceSlots = Vec<DebugCandidateProvenanceSlot>;

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontierLifecycle = (
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontierConsume = (
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugCandidateFrontierSlotConsume = (
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
    HeapTidCoords,
    Vec<HeapTidCoords>,
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
    DebugCandidateFrontierProvenanceSlots,
);

#[cfg(any(test, feature = "pg_test"))]
fn debug_candidate_slot(
    candidate: Option<search::BeamCandidate<page::ItemPointer>>,
) -> DebugCandidateSlot {
    match candidate {
        Some(candidate) => (
            true,
            (candidate.node.block_number, candidate.node.offset_number),
            candidate.score,
        ),
        None => (false, (u32::MAX, u16::MAX), 0.0),
    }
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_item_pointer_coords(tid: page::ItemPointer) -> HeapTidCoords {
    (tid.block_number, tid.offset_number)
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_graph_storage(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
) -> graph::GraphStorageDescriptor {
    graph::GraphStorageDescriptor::from_index_relation(index_relation, metadata)
        .unwrap_or_else(|e| pgrx::error!("ec_hnsw debug failed to resolve graph storage: {e}"))
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_graph_tuple_tag(storage: graph::GraphStorageDescriptor) -> u8 {
    match storage {
        graph::GraphStorageDescriptor::TurboQuant { .. } => page::TQ_ELEMENT_TAG,
        graph::GraphStorageDescriptor::TurboQuantHotCold(_) => page::TQ_TURBO_HOT_TAG,
        graph::GraphStorageDescriptor::PqFastScan(_) => page::TQ_GROUPED_HOT_TAG,
    }
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_with_page_line_tuple_bytes<R, F>(
    page_ptr: *mut u8,
    page_size: usize,
    offset: u16,
    visit: F,
) -> Option<R>
where
    F: for<'a> FnOnce(&'a [u8]) -> R,
{
    // SAFETY: Debug callers pass a locked page pointer and offset discovered
    // while scanning relation pages; the shared helper validates the line
    // pointer and tuple bounds before exposing bytes to `visit`.
    unsafe {
        super::shared::with_page_line_tuple_bytes(
            page_ptr,
            page_size,
            0,
            offset,
            "debug scanning page tuples",
            visit,
        )
    }
    .unwrap_or(None)
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_load_neighbor_tids_for_layer(
    index_relation: pg_sys::Relation,
    storage: graph::GraphStorageDescriptor,
    element_tid: page::ItemPointer,
    m: usize,
    layer: u8,
) -> Vec<page::ItemPointer> {
    // SAFETY: Debug callers pass an element TID discovered from the graph
    // relation; the graph loader validates storage-specific tuple contents.
    let (element, neighbors) =
        unsafe { graph::load_exact_graph_adjacency(index_relation, element_tid, storage) };
    graph::valid_neighbor_tids_for_layer(&neighbors.tids, element.level, m, layer)
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_runtime_ordered_head(opaque: &mut TqScanOpaque) -> DebugCandidateHead {
    let current = active_result_state_ref(opaque).current();
    if current.has_element() {
        return Some(debug_item_pointer_coords(current.element_tid()));
    }

    current_candidate_frontier_head(opaque)
        .map(|candidate| debug_item_pointer_coords(candidate.node))
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_runtime_ordered_slots(opaque: &TqScanOpaque) -> DebugCandidateFrontierSlots {
    let mut slots = Vec::new();
    let current = active_result_state_ref(opaque).current();
    if current.has_element() {
        slots.push((
            true,
            debug_item_pointer_coords(current.element_tid()),
            current.score(),
        ));
    }
    slots.extend(debug_candidate_frontier_slots(opaque));
    slots
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_runtime_ordered_provenance_slots(
    opaque: &TqScanOpaque,
) -> DebugCandidateFrontierProvenanceSlots {
    let mut slots = Vec::new();
    let current = active_result_state_ref(opaque).current();
    if current.has_element() {
        slots.push((
            true,
            debug_item_pointer_coords(current.element_tid()),
            (u32::MAX, u16::MAX),
            current.score(),
        ));
    }
    slots.extend(debug_candidate_frontier_provenance_slots(opaque));
    slots
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_scan_query(opaque: &TqScanOpaque) -> Vec<f32> {
    opaque.query_values_or_empty().to_vec()
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_candidate_frontier_slots(opaque: &TqScanOpaque) -> DebugCandidateFrontierSlots {
    visible_frontier_candidates(opaque)
        .into_iter()
        .map(|candidate| {
            (
                true,
                debug_item_pointer_coords(candidate.node),
                candidate.score,
            )
        })
        .collect::<Vec<_>>()
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_candidate_frontier_provenance_slots(
    opaque: &TqScanOpaque,
) -> DebugCandidateFrontierProvenanceSlots {
    visible_frontier_candidates(opaque)
        .into_iter()
        .map(|candidate| {
            (
                true,
                debug_item_pointer_coords(candidate.node),
                candidate
                    .source
                    .map(debug_item_pointer_coords)
                    .unwrap_or((u32::MAX, u16::MAX)),
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
    DebugCandidateFrontierSlots,
    DebugCandidateFrontierSlots,
    DebugCandidateFrontierProvenanceSlots,
    Vec<HeapTidCoords>,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugBootstrapPhaseTransition = (
    bool,
    bool,
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
    bool,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugBootstrapConsumeState = (
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
    HeapTidCoords,
    DebugCandidateHead,
    DebugCandidateFrontierSlots,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugBootstrapCandidateMaterializationState = (
    (bool, HeapTidCoords, f32),
    HeapTidCoords,
    Vec<HeapTidCoords>,
    bool,
);

#[cfg(any(test, feature = "pg_test"))]
fn debug_sorted_visited_tids(opaque: &TqScanOpaque) -> Vec<HeapTidCoords> {
    if opaque.visited_tids.is_null() {
        return Vec::new();
    }

    // SAFETY: `visited_tids` is owned by the live scan opaque while the debug
    // caller is inspecting the scan state, and null was handled above.
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

    // SAFETY: `expanded_source_tids` is owned by the live scan opaque while the
    // debug caller is inspecting the scan state, and null was handled above.
    let mut tids = unsafe { &*opaque.expanded_source_tids }
        .iter()
        .map(|tid| (tid.block_number, tid.offset_number))
        .collect::<Vec<_>>();
    tids.sort_unstable();
    tids
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_sorted_emitted_tids(opaque: &TqScanOpaque) -> Vec<HeapTidCoords> {
    if opaque.emitted_result_tids.is_null() {
        return Vec::new();
    }

    // SAFETY: `emitted_result_tids` is owned by the live scan opaque while the
    // debug caller is inspecting the scan state, and null was handled above.
    let mut tids = unsafe { &*opaque.emitted_result_tids }
        .iter()
        .map(|tid| (tid.block_number, tid.offset_number))
        .collect::<Vec<_>>();
    tids.sort_unstable();
    tids
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_execution_phase_label(phase: ScanExecutionPhase) -> &'static str {
    match phase {
        ScanExecutionPhase::GraphTraversal => "graph_traversal",
        ScanExecutionPhase::LinearFallback => "linear_fallback",
        ScanExecutionPhase::Exhausted => "exhausted",
    }
}

#[cfg(any(test, feature = "pg_test"))]
type DebugScanProfile = (
    i64,
    i64,
    i64,
    String,
    bool,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    bool,
    i32,
    String,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    bool,
    i32,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i32,
    i32,
    i64,
    i32,
    i32,
    i64,
    i32,
    i64,
    i32,
    i32,
    i32,
    i64,
    i32,
    i64,
    i32,
    i32,
    i32,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugScanHeapFetchProfile = (i64, i64, i64, i64, i64, i32, i32, i32);

#[cfg(any(test, feature = "pg_test"))]
type DebugGroupedRerankProfile = (
    i64,
    i64,
    i64,
    i64,
    i32,
    i32,
    i64,
    i32,
    i64,
    i32,
    i64,
    i64,
    i64,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugTurboQuantScanStageProfile = (
    i64,
    i64,
    i32,
    i64,
    i32,
    i32,
    i64,
    i32,
    i64,
    String,
    bool,
    bool,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugGroupedScanComparisonSummary = (i32, i32, i32, i32, f64, f32, f64);

#[cfg(any(test, feature = "pg_test"))]
type DebugGroupedScanComparisonRow = (
    HeapTidCoords,
    i32,
    f32,
    Option<f32>,
    Option<i32>,
    Option<i32>,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugGroupedScanOrderDriftSummary = (
    i32,
    i32,
    i32,
    f64,
    i32,
    f64,
    Option<i32>,
    Option<i32>,
    bool,
    bool,
    bool,
    bool,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugGroupedScanWindowedRow = (
    HeapTidCoords,
    i32,
    i32,
    f32,
    Option<f32>,
    Option<i32>,
    Option<i32>,
    Option<i32>,
);

#[cfg(any(test, feature = "pg_test"))]
type DebugGroupedScanWindowedSummary = (
    i32,
    i32,
    i32,
    i32,
    Option<i32>,
    Option<i32>,
    Option<i32>,
    Option<i32>,
    f64,
    f64,
    i32,
    i32,
    f64,
    f64,
);

#[cfg(any(test, feature = "pg_test"))]
#[derive(Debug, Clone, Copy, PartialEq)]
struct DebugGroupedRankMetrics {
    compared_result_count: i32,
    mean_abs_rank_shift: f64,
    max_abs_rank_shift: i32,
    spearman_rank_correlation: f64,
    exact_best_observed_rank: Option<i32>,
    exact_top4_max_observed_rank: Option<i32>,
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_am_begin_scan(
    index_relation: pg_sys::Relation,
    nkeys: i32,
    norderbys: i32,
) -> pg_sys::IndexScanDesc {
    // SAFETY: Debug callers keep the index relation open while the AM scan
    // descriptor is allocated by the HNSW begin callback.
    unsafe { ec_hnsw_ambeginscan(index_relation, nkeys, norderbys) }
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_am_rescan(
    scan: pg_sys::IndexScanDesc,
    keys: pg_sys::ScanKey,
    nkeys: i32,
    orderbys: pg_sys::ScanKey,
    norderbys: i32,
) {
    // SAFETY: Debug callers pass a live HNSW scan descriptor and initialized
    // key/order-by buffers matching the supplied counts.
    unsafe { ec_hnsw_amrescan(scan, keys, nkeys, orderbys, norderbys) };
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_am_gettuple(scan: pg_sys::IndexScanDesc, direction: pg_sys::ScanDirection::Type) -> bool {
    // SAFETY: Debug callers invoke gettuple only on live HNSW scan descriptors.
    unsafe { ec_hnsw_amgettuple(scan, direction) }
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_am_end_scan(scan: pg_sys::IndexScanDesc) {
    // SAFETY: Debug callers pass a live HNSW scan descriptor whose AM-owned
    // opaque state may be cleaned by the HNSW end callback.
    unsafe { ec_hnsw_amendscan(scan) };
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_index_scan_end(scan: pg_sys::IndexScanDesc) {
    // SAFETY: Debug callers pass a descriptor allocated by PostgreSQL's index
    // scan machinery and release it exactly once after AM cleanup.
    unsafe { pg_sys::IndexScanEnd(scan) };
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_scan_opaque<'a>(scan: pg_sys::IndexScanDesc) -> &'a TqScanOpaque {
    // SAFETY: Debug callers inspect the HNSW opaque while the scan descriptor is
    // live and after begin/rescan initialized the opaque pointer.
    unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() }
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_scan_opaque_mut<'a>(scan: pg_sys::IndexScanDesc) -> &'a mut TqScanOpaque {
    // SAFETY: Debug callers take exclusive mutable access to the scan opaque
    // while the live scan descriptor is not otherwise borrowed.
    unsafe { &mut *(*scan).opaque.cast::<TqScanOpaque>() }
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_scan_has_opaque(scan: pg_sys::IndexScanDesc) -> bool {
    // SAFETY: Debug callers pass a live scan descriptor and only read the
    // descriptor's opaque pointer.
    unsafe { !(*scan).opaque.is_null() }
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_scan_opaque_is_null(scan: pg_sys::IndexScanDesc) -> bool {
    // SAFETY: Debug callers pass a live scan descriptor and only read the
    // descriptor's opaque pointer.
    unsafe { (*scan).opaque.is_null() }
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_scan_heap_tid(scan: pg_sys::IndexScanDesc) -> HeapTidCoords {
    // SAFETY: Debug callers read xs_heaptid immediately after a successful
    // gettuple call on the same live scan descriptor.
    pgrx::itemptr::item_pointer_get_both(unsafe { (*scan).xs_heaptid })
}

#[cfg(any(test, feature = "pg_test"))]
struct DebugHeapBackedScan {
    scan: IndexScanGuard,
    _snapshot: ActiveSnapshotGuard,
    _index_relation: IndexRelationGuard,
    _heap_relation: HeapRelationGuard,
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_begin_heap_backed_scan(index_oid: pg_sys::Oid) -> DebugHeapBackedScan {
    let index_relation =
        IndexRelationGuard::access_share(index_oid, "debug_begin_heap_backed_scan");
    // SAFETY: `index_relation` is an open PostgreSQL index relation guard.
    let heap_oid = unsafe { pg_sys::IndexGetRelation((*index_relation.as_ptr()).rd_id, false) };
    if heap_oid == pg_sys::InvalidOid {
        pgrx::error!("debug scan could not resolve heap relation for index {index_oid}");
    }

    let heap_relation = HeapRelationGuard::try_access_share(heap_oid)
        .unwrap_or_else(|| pgrx::error!("debug scan failed to open heap relation"));
    let snapshot = ActiveSnapshotGuard::latest_after_command_counter()
        .unwrap_or_else(|| pgrx::error!("debug scan could not acquire a fresh latest snapshot"));
    let scan = IndexScanGuard::begin(&heap_relation, &index_relation, &snapshot, 0, 1)
        .unwrap_or_else(|| pgrx::error!("debug scan failed to begin heap-backed index scan"));

    DebugHeapBackedScan {
        scan,
        _snapshot: snapshot,
        _index_relation: index_relation,
        _heap_relation: heap_relation,
    }
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_end_heap_backed_scan(state: DebugHeapBackedScan) {
    drop(state);
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_begin_end_scan(index_oid: pg_sys::Oid) -> (bool, bool) {
    let index_relation = IndexRelationGuard::access_share(index_oid, "debug_begin_end_scan");
    // SAFETY: The index relation guard keeps the relation open for the scan
    // descriptor returned by the AM begin callback.
    let scan = debug_am_begin_scan(index_relation.as_ptr(), 0, 1);
    // SAFETY: `scan` is the descriptor returned by `ec_hnsw_ambeginscan`.
    let has_opaque = debug_scan_has_opaque(scan);

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: `ec_hnsw_amendscan` clears the opaque field on the same live
    // descriptor.
    let cleared_opaque = debug_scan_opaque_is_null(scan);

    // SAFETY: The descriptor was allocated by `IndexScanBegin` through the AM
    // begin path and has had AM cleanup run above.
    debug_index_scan_end(scan);
    (has_opaque, cleared_opaque)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_end_scan_twice(index_oid: pg_sys::Oid) -> (bool, bool, bool) {
    let index_relation = IndexRelationGuard::access_share(index_oid, "debug_end_scan_twice");
    // SAFETY: The index relation guard keeps the relation open for the scan
    // descriptor returned by the AM begin callback.
    let scan = debug_am_begin_scan(index_relation.as_ptr(), 0, 1);
    // SAFETY: `scan` is the descriptor returned by `ec_hnsw_ambeginscan`.
    let has_opaque = debug_scan_has_opaque(scan);

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: The descriptor remains allocated after AM cleanup for this debug
    // idempotence check.
    let cleared_after_first = debug_scan_opaque_is_null(scan);

    // SAFETY: This deliberately repeats AM cleanup on the same descriptor to
    // verify the end callback tolerates an already-cleared opaque pointer.
    debug_am_end_scan(scan);
    // SAFETY: The descriptor remains allocated until `IndexScanEnd` below.
    let cleared_after_second = debug_scan_opaque_is_null(scan);

    // SAFETY: The descriptor was allocated by the AM begin path and is freed
    // exactly once after the idempotence probe.
    debug_index_scan_end(scan);
    (has_opaque, cleared_after_first, cleared_after_second)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_query_dimensions(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (bool, u16, Vec<f32>, u16, u8, usize, u32, bool, usize, usize) {
    let index_relation =
        IndexRelationGuard::access_share(index_oid, "debug_rescan_query_dimensions");
    // SAFETY: The relation guard keeps the index relation open for the AM scan
    // descriptor.
    let scan = debug_am_begin_scan(index_relation.as_ptr(), 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` points to
    // one initialized order-by key for the duration of the rescan call.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: `ec_hnsw_amrescan` initializes the HNSW scan opaque on the live
    // scan descriptor.
    let opaque = debug_scan_opaque(scan);
    let result = (
        opaque.rescan_called,
        opaque.query_dimensions,
        debug_scan_query(opaque),
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

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_overwrites_query_dimensions(
    index_oid: pg_sys::Oid,
    first_query: Vec<f32>,
    second_query: Vec<f32>,
) -> (bool, u16, Vec<f32>, u16, u8, usize, u32, bool, usize, usize) {
    let index_relation =
        IndexRelationGuard::access_share(index_oid, "debug_rescan_overwrites_query_dimensions");
    // SAFETY: The relation guard keeps the index relation open for the AM scan
    // descriptor.
    let scan = debug_am_begin_scan(index_relation.as_ptr(), 0, 1);

    let mut first_orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(first_query)
            .expect("first query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `first_orderby`
    // points to one initialized order-by key for this rescan.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut first_orderby, 1);

    let mut second_orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(second_query)
            .expect("second query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `second_orderby`
    // points to one initialized order-by key for this overwrite rescan.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut second_orderby, 1);

    // SAFETY: The second AM rescan leaves the HNSW scan opaque initialized on
    // the live descriptor for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let result = (
        opaque.rescan_called,
        opaque.query_dimensions,
        debug_scan_query(opaque),
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

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_null_query(index_oid: pg_sys::Oid) {
    let index_relation = IndexRelationGuard::access_share(index_oid, "debug_rescan_null_query");
    // SAFETY: The relation guard keeps the index relation open for the AM scan
    // descriptor.
    let scan = debug_am_begin_scan(index_relation.as_ptr(), 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_flags: pg_sys::SK_ISNULL as i32,
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` points to
    // one null order-by key for the duration of the error-path rescan probe.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_with_index_qual(index_oid: pg_sys::Oid, query: Vec<f32>) {
    let index_relation =
        IndexRelationGuard::access_share(index_oid, "debug_rescan_with_index_qual");
    // SAFETY: The relation guard keeps the index relation open for the AM scan
    // descriptor.
    let scan = debug_am_begin_scan(index_relation.as_ptr(), 1, 1);

    let mut key = pg_sys::ScanKeyData::default();
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, `key` and `orderby` point to initialized
    // one-element buffers matching the supplied counts.
    debug_am_rescan(scan, &mut key, 1, &mut orderby, 1);
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_with_unused_key_buffer(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (bool, u16, Vec<f32>, u16, u8, usize, u32, bool, usize, usize) {
    let index_relation =
        IndexRelationGuard::access_share(index_oid, "debug_rescan_with_unused_key_buffer");
    // SAFETY: The relation guard keeps the index relation open for the AM scan
    // descriptor.
    let scan = debug_am_begin_scan(index_relation.as_ptr(), 0, 1);

    // SAFETY: PostgreSQL allocates zeroed memory in the current memory context;
    // the pointer is freed before the descriptor is ended below.
    let unused_keys = unsafe { pg_sys::palloc0(std::mem::size_of::<pg_sys::ScanKeyData>()) }
        .cast::<pg_sys::ScanKeyData>();
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, the key count is intentionally zero so
    // `unused_keys` must be ignored, and `orderby` is a valid one-key buffer.
    debug_am_rescan(scan, unused_keys, 0, &mut orderby, 1);

    // SAFETY: AM rescan initializes the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let result = (
        opaque.rescan_called,
        opaque.query_dimensions,
        debug_scan_query(opaque),
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

    // SAFETY: `unused_keys` was allocated by `palloc0` above and has not been
    // freed yet.
    unsafe { pg_sys::pfree(unused_keys.cast()) };
    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_with_multiple_orderbys(index_oid: pg_sys::Oid, query: Vec<f32>) {
    let index_relation =
        IndexRelationGuard::access_share(index_oid, "debug_rescan_with_multiple_orderbys");
    // SAFETY: The relation guard keeps the index relation open for the AM scan
    // descriptor.
    let scan = debug_am_begin_scan(index_relation.as_ptr(), 0, 2);

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
    // SAFETY: `scan` is live, there are no index quals, and `orderbys` contains
    // exactly two initialized keys matching the supplied order-by count.
    debug_am_rescan(scan, ptr::null_mut(), 0, orderbys.as_mut_ptr(), 2);
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_without_rescan(index_oid: pg_sys::Oid) {
    let index_relation =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_without_rescan");
    // SAFETY: The relation guard keeps the index relation open for the AM scan
    // descriptor.
    let scan = debug_am_begin_scan(index_relation.as_ptr(), 0, 1);

    // SAFETY: The debug probe deliberately invokes gettuple before rescan on a
    // live HNSW scan descriptor to exercise that error path.
    debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection);
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_after_rescan(index_oid: pg_sys::Oid, query: Vec<f32>) {
    let index_relation = IndexRelationGuard::access_share(index_oid, "debug_gettuple_after_rescan");
    // SAFETY: The relation guard keeps the index relation open for the AM scan
    // descriptor.
    let scan = debug_am_begin_scan(index_relation.as_ptr(), 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` points to
    // one initialized order-by key.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);
    // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may advance the
    // live descriptor.
    debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection);
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_after_rescan_result(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> bool {
    let index_relation =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_after_rescan_result");
    // SAFETY: The relation guard keeps the index relation open for the AM scan
    // descriptor.
    let scan = debug_am_begin_scan(index_relation.as_ptr(), 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` points to
    // one initialized order-by key.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);
    // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may advance the
    // live descriptor.
    let result = debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection);

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    result
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_scan_heap_tids(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> Vec<HeapTidCoords> {
    // SAFETY: The debug helper opens the index, owning heap, and scan snapshot
    // and keeps them alive in `scan_state`.
    let scan_state = unsafe { debug_begin_heap_backed_scan(index_oid) };
    let scan = scan_state.scan.as_ptr();

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan_state` owns a live heap-backed scan, there are no index
    // quals, and `orderby` is a valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    let mut tids = Vec::new();
    // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple calls
    // may advance the live scan descriptor.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {
        // SAFETY: A successful gettuple call populated `xs_heaptid` for this
        // live index scan descriptor.
        let (block_number, offset_number) = debug_scan_heap_tid(scan);
        tids.push((block_number, offset_number));
    }

    // SAFETY: `scan_state` owns the scan and relation guards and is consumed
    // once after iteration completes.
    unsafe { debug_end_heap_backed_scan(scan_state) };
    tids
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_profile_ordered_scan(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugScanProfile {
    // SAFETY: This debug wrapper forwards the caller-provided index oid and
    // query to the bounded profiler without changing ownership.
    unsafe { debug_profile_ordered_scan_with_limit(index_oid, query, None) }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_profile_ordered_scan_with_limit(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    result_limit: Option<usize>,
) -> DebugScanProfile {
    // SAFETY: The debug helper opens the index, owning heap, and scan snapshot
    // and keeps them alive in `scan_state`.
    let scan_state = unsafe { debug_begin_heap_backed_scan(index_oid) };
    let scan = scan_state.scan.as_ptr();

    let total_started = Instant::now();
    let rescan_started = Instant::now();
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan_state` owns a live heap-backed scan, there are no index
    // quals, and `orderby` is a valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);
    let rescan_elapsed_us = i64::try_from(rescan_started.elapsed().as_micros())
        .expect("rescan timing should fit in i64");

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let rescan_counters = opaque.explain_counters;
    let rescan_phase = debug_execution_phase_label(opaque.execution_phase).to_string();
    let rescan_current_result = active_result_state_ref(opaque).current().has_element();
    let rescan_ordered_slots = i32::try_from(debug_runtime_ordered_slots(opaque).len())
        .expect("slot count should fit in i32");
    let rescan_pending_heap_tids = i32::from(active_result_state_ref(opaque).pending_count());
    let rescan_visited_count = i32::try_from(debug_sorted_visited_tids(opaque).len())
        .expect("visited count should fit in i32");
    let rescan_expanded_count = i32::try_from(debug_sorted_expanded_source_tids(opaque).len())
        .expect("expanded count should fit in i32");
    let rescan_emitted_count = i32::try_from(debug_sorted_emitted_tids(opaque).len())
        .expect("emitted count should fit in i32");
    let rescan_debug_profile = opaque.debug_profile;

    let emit_started = Instant::now();
    let mut result_count = 0_i32;
    let result_limit = result_limit.unwrap_or(usize::MAX);
    while usize::try_from(result_count).expect("result count should fit in usize") < result_limit
        // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple
        // calls may advance the live scan descriptor.
        && debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection)
    {
        result_count += 1;
    }
    let emit_elapsed_us =
        i64::try_from(emit_started.elapsed().as_micros()).expect("emit timing should fit in i64");

    // SAFETY: The scan descriptor remains live until `debug_end_heap_backed_scan`
    // consumes `scan_state` below.
    let opaque = debug_scan_opaque(scan);
    let total_counters = opaque.explain_counters;
    let final_phase = debug_execution_phase_label(opaque.execution_phase).to_string();
    let final_ordered_slots = i32::try_from(debug_runtime_ordered_slots(opaque).len())
        .expect("slot count should fit in i32");
    let final_emitted_count = i32::try_from(debug_sorted_emitted_tids(opaque).len())
        .expect("emitted count should fit in i32");

    let total_elapsed_us =
        i64::try_from(total_started.elapsed().as_micros()).expect("total timing should fit in i64");

    // SAFETY: `scan_state` owns the scan and relation guards and is consumed
    // once after profiling completes.
    unsafe { debug_end_heap_backed_scan(scan_state) };

    (
        rescan_elapsed_us,
        emit_elapsed_us,
        total_elapsed_us,
        rescan_phase,
        rescan_current_result,
        rescan_ordered_slots,
        rescan_pending_heap_tids,
        rescan_visited_count,
        rescan_expanded_count,
        rescan_emitted_count,
        i32::try_from(rescan_counters.stats_bootstrap_expansions)
            .expect("counter should fit in i32"),
        i32::try_from(rescan_counters.stats_bootstrap_pages_read)
            .expect("counter should fit in i32"),
        rescan_counters.stats_quantizer_cache_hit,
        result_count,
        final_phase,
        final_ordered_slots,
        i32::try_from(total_counters.stats_bootstrap_expansions)
            .expect("counter should fit in i32"),
        i32::try_from(total_counters.stats_bootstrap_pages_read)
            .expect("counter should fit in i32"),
        i32::try_from(total_counters.stats_linear_pages_read).expect("counter should fit in i32"),
        i32::try_from(total_counters.stats_elements_scored).expect("counter should fit in i32"),
        i32::try_from(total_counters.stats_elements_skipped).expect("counter should fit in i32"),
        i32::try_from(total_counters.stats_heap_tids_returned).expect("counter should fit in i32"),
        total_counters.stats_quantizer_cache_hit,
        final_emitted_count,
        i64::try_from(rescan_debug_profile.amrescan_total_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(rescan_debug_profile.query_decode_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(rescan_debug_profile.scan_setup_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(rescan_debug_profile.store_query_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(rescan_debug_profile.prepare_query_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(rescan_debug_profile.reset_state_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(rescan_debug_profile.initialize_entry_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(rescan_debug_profile.upper_layer_seed_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(rescan_debug_profile.layer0_seed_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(rescan_debug_profile.stage_ordered_results_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(rescan_debug_profile.initial_prefetch_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(rescan_debug_profile.frontier_consume_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(rescan_debug_profile.graph_result_materialize_elapsed_us)
            .expect("timing should fit in i64"),
        i32::try_from(rescan_debug_profile.graph_element_cache_hits)
            .expect("counter should fit in i32"),
        i32::try_from(rescan_debug_profile.graph_element_cache_misses)
            .expect("counter should fit in i32"),
        i64::try_from(rescan_debug_profile.graph_element_load_elapsed_us)
            .expect("timing should fit in i64"),
        i32::try_from(rescan_debug_profile.graph_neighbor_cache_hits)
            .expect("counter should fit in i32"),
        i32::try_from(rescan_debug_profile.graph_neighbor_cache_misses)
            .expect("counter should fit in i32"),
        i64::try_from(rescan_debug_profile.graph_neighbor_load_elapsed_us)
            .expect("timing should fit in i64"),
        i32::try_from(rescan_debug_profile.candidate_score_calls)
            .expect("counter should fit in i32"),
        i64::try_from(rescan_debug_profile.candidate_score_elapsed_us)
            .expect("timing should fit in i64"),
        i32::try_from(rescan_debug_profile.score_cache_hits).expect("counter should fit in i32"),
        i32::try_from(rescan_debug_profile.score_cache_misses).expect("counter should fit in i32"),
        i32::try_from(rescan_debug_profile.grouped_traversal_approx_score_calls)
            .expect("counter should fit in i32"),
        i64::try_from(rescan_debug_profile.grouped_traversal_approx_score_elapsed_us)
            .expect("timing should fit in i64"),
        i32::try_from(rescan_debug_profile.grouped_traversal_exact_score_calls)
            .expect("counter should fit in i32"),
        i64::try_from(rescan_debug_profile.grouped_traversal_exact_score_elapsed_us)
            .expect("timing should fit in i64"),
        i32::try_from(rescan_debug_profile.grouped_traversal_budgeted_expansions)
            .expect("counter should fit in i32"),
        i32::try_from(rescan_debug_profile.grouped_traversal_budgeted_candidates)
            .expect("counter should fit in i32"),
        i32::try_from(rescan_debug_profile.grouped_traversal_budgeted_exact_candidates)
            .expect("counter should fit in i32"),
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_profile_ordered_scan_with_heap_fetch(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    result_limit: usize,
    project_attnum: Option<i32>,
) -> DebugScanHeapFetchProfile {
    // SAFETY: `index_oid` names the index relation supplied by the pg_test
    // caller; PostgreSQL resolves the owning heap oid without taking
    // ownership.
    let heap_oid = unsafe { pg_sys::IndexGetRelation(index_oid, false) };
    if heap_oid == pg_sys::InvalidOid {
        pgrx::error!(
            "debug heap-fetch profile could not resolve heap relation for index {index_oid}"
        );
    }

    let heap_relation = HeapRelationGuard::try_access_share(heap_oid)
        .unwrap_or_else(|| pgrx::error!("debug heap-fetch profile failed to open heap relation"));
    let index_relation =
        IndexRelationGuard::access_share(index_oid, "debug_profile_ordered_scan_with_heap_fetch");
    let snapshot = ActiveSnapshotGuard::latest_after_command_counter().unwrap_or_else(|| {
        pgrx::error!("debug heap-fetch profile could not acquire a fresh latest snapshot")
    });
    let slot_guard = TupleTableSlotGuard::single_for_heap(heap_relation.as_ptr())
        .unwrap_or_else(|| pgrx::error!("debug heap-fetch profile failed to allocate tuple slot"));
    let scan_guard = IndexScanGuard::begin(&heap_relation, &index_relation, &snapshot, 0, 1)
        .unwrap_or_else(|| pgrx::error!("debug heap-fetch profile failed to begin index scan"));
    let scan = scan_guard.as_ptr();
    let slot = slot_guard.as_ptr();

    let total_started = Instant::now();
    let rescan_started = Instant::now();
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is a live heap-backed index scan, there are no index quals,
    // and `orderby` points to one initialized order-by key.
    unsafe { pg_sys::index_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };
    let rescan_elapsed_us = i64::try_from(rescan_started.elapsed().as_micros())
        .expect("rescan timing should fit in i64");

    let emit_started = Instant::now();
    let mut result_count = 0_i32;
    let mut slot_fetch_count = 0_i32;
    let mut projected_count = 0_i32;
    let mut slot_fetch_elapsed_us = 0_i64;
    let mut projection_elapsed_us = 0_i64;
    while usize::try_from(result_count).expect("result count should fit in usize") < result_limit {
        let slot_fetch_started = Instant::now();
        // SAFETY: `scan` and `slot` are live guards for the same heap-backed
        // scan; PostgreSQL fills the slot when a tuple is found.
        let found = unsafe {
            pg_sys::index_getnext_slot(scan, pg_sys::ScanDirection::ForwardScanDirection, slot)
        };
        slot_fetch_elapsed_us += i64::try_from(slot_fetch_started.elapsed().as_micros())
            .expect("slot-fetch timing should fit in i64");
        if !found {
            break;
        }

        result_count += 1;
        slot_fetch_count += 1;
        if let Some(attnum) = project_attnum {
            let projection_started = Instant::now();
            let mut isnull = false;
            // SAFETY: `slot` contains the tuple produced by the successful
            // `index_getnext_slot` call, and `attnum` is supplied by the debug
            // caller for projection timing.
            let _ = unsafe { pg_sys::slot_getattr(slot, attnum, &mut isnull) };
            projection_elapsed_us += i64::try_from(projection_started.elapsed().as_micros())
                .expect("projection timing should fit in i64");
            projected_count += 1;
        }
        // SAFETY: `slot` is the tuple table slot allocated for this scan and may
        // be cleared between successful fetches.
        unsafe {
            pg_sys::ExecClearTuple(slot);
        }
    }
    let emit_elapsed_us =
        i64::try_from(emit_started.elapsed().as_micros()).expect("emit timing should fit in i64");
    let total_elapsed_us =
        i64::try_from(total_started.elapsed().as_micros()).expect("total timing should fit in i64");

    (
        rescan_elapsed_us,
        emit_elapsed_us,
        total_elapsed_us,
        slot_fetch_elapsed_us,
        projection_elapsed_us,
        result_count,
        slot_fetch_count,
        projected_count,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_grouped_rerank_profile(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    limit_count: i32,
) -> DebugGroupedRerankProfile {
    // SAFETY: The debug helper opens the index, owning heap, and scan snapshot
    // and keeps them alive in `scan_state`.
    let scan_state = unsafe { debug_begin_heap_backed_scan(index_oid) };
    let scan = scan_state.scan.as_ptr();
    let result_limit =
        usize::try_from(limit_count).expect("grouped rerank profile limit should fit in usize");

    let total_started = Instant::now();
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan_state` owns a live heap-backed scan, there are no index
    // quals, and `orderby` is a valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);
    let emit_started = Instant::now();
    let mut emitted = 0_i32;
    while usize::try_from(emitted).expect("emitted count should fit in usize") < result_limit
        // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple
        // calls may advance the live scan descriptor.
        && debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection)
    {
        emitted += 1;
    }
    let result_count = emitted;
    let emit_elapsed_us =
        i64::try_from(emit_started.elapsed().as_micros()).expect("emit timing should fit in i64");
    let total_elapsed_us =
        i64::try_from(total_started.elapsed().as_micros()).expect("total timing should fit in i64");

    // SAFETY: The scan descriptor remains live until `debug_end_heap_backed_scan`
    // consumes `scan_state` below.
    let opaque = debug_scan_opaque(scan);
    let debug_profile = opaque.debug_profile;

    // SAFETY: `scan_state` owns the scan and relation guards and is consumed
    // once after profiling completes.
    unsafe { debug_end_heap_backed_scan(scan_state) };

    (
        i64::try_from(debug_profile.amrescan_total_elapsed_us).expect("timing should fit in i64"),
        i64::try_from(debug_profile.graph_result_materialize_elapsed_us)
            .expect("timing should fit in i64"),
        emit_elapsed_us,
        total_elapsed_us,
        result_count,
        i32::try_from(debug_profile.grouped_rerank_quantized_score_calls)
            .expect("counter should fit in i32"),
        i64::try_from(debug_profile.grouped_rerank_quantized_score_elapsed_us)
            .expect("timing should fit in i64"),
        i32::try_from(debug_profile.grouped_rerank_heap_score_calls)
            .expect("counter should fit in i32"),
        i64::try_from(debug_profile.grouped_rerank_heap_score_elapsed_us)
            .expect("timing should fit in i64"),
        i32::try_from(debug_profile.grouped_rerank_heap_rows_fetched)
            .expect("counter should fit in i32"),
        i64::try_from(debug_profile.grouped_rerank_heap_fetch_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(debug_profile.grouped_rerank_heap_decode_elapsed_us)
            .expect("timing should fit in i64"),
        i64::try_from(debug_profile.grouped_rerank_heap_dot_elapsed_us)
            .expect("timing should fit in i64"),
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_turboquant_scan_stage_profile(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugTurboQuantScanStageProfile {
    // SAFETY: The debug helper opens the index, owning heap, and scan snapshot
    // and keeps them alive in `scan_state`.
    let scan_state = unsafe { debug_begin_heap_backed_scan(index_oid) };
    let scan = scan_state.scan.as_ptr();

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan_state` owns a live heap-backed scan, there are no index
    // quals, and `orderby` is a valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    if !matches!(
        opaque.scan_graph_storage,
        graph::GraphStorageDescriptor::TurboQuant { .. }
            | graph::GraphStorageDescriptor::TurboQuantHotCold(_)
    ) {
        // SAFETY: `scan_state` owns the scan and relation guards and is
        // consumed before the debug error aborts this path.
        unsafe { debug_end_heap_backed_scan(scan_state) };
        pgrx::error!("debug turboquant scan stage profile requires a turboquant index");
    }
    if opaque.cached_quantizer.is_null() {
        // SAFETY: `scan_state` owns the scan and relation guards and is
        // consumed before the debug error aborts this path.
        unsafe { debug_end_heap_backed_scan(scan_state) };
        pgrx::error!("debug turboquant scan stage profile requires a prepared quantizer");
    }

    let debug_profile = opaque.debug_profile;
    let rerank_score_calls = debug_profile
        .grouped_rerank_quantized_score_calls
        .saturating_add(debug_profile.grouped_rerank_heap_score_calls);
    let rerank_score_elapsed_us = debug_profile
        .grouped_rerank_quantized_score_elapsed_us
        .saturating_add(debug_profile.grouped_rerank_heap_score_elapsed_us);
    let traversal_residual_elapsed_us = debug_profile
        .amrescan_total_elapsed_us
        .saturating_sub(debug_profile.binary_prefilter_score_elapsed_us)
        .saturating_sub(debug_profile.candidate_score_elapsed_us)
        .saturating_sub(rerank_score_elapsed_us);
    let exact_score_mode = turboquant_exact_score_mode_name(opaque).to_owned();
    let exact_score_uses_lut = turboquant_exact_score_uses_lut(opaque);
    let exact_score_uses_qjl = turboquant_exact_score_uses_qjl(opaque);

    // SAFETY: `scan_state` owns the scan and relation guards and is consumed
    // once after profiling completes.
    unsafe { debug_end_heap_backed_scan(scan_state) };

    (
        i64::try_from(debug_profile.amrescan_total_elapsed_us).expect("timing should fit in i64"),
        i64::try_from(traversal_residual_elapsed_us).expect("timing should fit in i64"),
        i32::try_from(debug_profile.binary_prefilter_score_calls)
            .expect("counter should fit in i32"),
        i64::try_from(debug_profile.binary_prefilter_score_elapsed_us)
            .expect("timing should fit in i64"),
        i32::try_from(debug_profile.binary_prefilter_survivor_candidates)
            .expect("counter should fit in i32"),
        i32::try_from(debug_profile.candidate_score_calls).expect("counter should fit in i32"),
        i64::try_from(debug_profile.candidate_score_elapsed_us).expect("timing should fit in i64"),
        i32::try_from(rerank_score_calls).expect("counter should fit in i32"),
        i64::try_from(rerank_score_elapsed_us).expect("timing should fit in i64"),
        exact_score_mode,
        exact_score_uses_lut,
        exact_score_uses_qjl,
    )
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_collect_element_tids_at_level(
    index_relation: pg_sys::Relation,
    storage: graph::GraphStorageDescriptor,
    target_level: u8,
) -> Vec<page::ItemPointer> {
    // SAFETY: `index_relation` is an open index relation supplied by the debug
    // caller; PostgreSQL returns the current main-fork block count.
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let element_tag = debug_graph_tuple_tag(storage);
    let mut tids = Vec::new();

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        // SAFETY: The block number is within the relation's main-fork block
        // count and is read under a shared buffer lock for inspection.
        let buffer = unsafe {
            LockedBufferGuard::read_main(
                index_relation,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                pg_sys::BUFFER_LOCK_SHARE as i32,
            )
        };
        let buffer = buffer.unwrap_or_else(|| {
            pgrx::error!("ec_hnsw debug failed to open graph block {block_number}")
        });
        let page_ptr = buffer.page().cast::<u8>();
        let page_size = buffer.page_size();
        let line_pointer_count = super::shared::page_line_pointer_count(page_ptr);

        for offset_number in 1..=line_pointer_count {
            // SAFETY: The buffer remains share-locked while the helper validates
            // this line pointer and exposes tuple bytes only to the closure.
            let matches_element_tag = unsafe {
                debug_with_page_line_tuple_bytes(
                    page_ptr,
                    page_size,
                    offset_number,
                    |tuple_bytes| tuple_bytes.first().copied() == Some(element_tag),
                )
            }
            .unwrap_or(false);
            if !matches_element_tag {
                continue;
            }

            // SAFETY: The tuple tag matched the graph element tag on a locked
            // page, and the graph loader validates the storage-specific body.
            let element = unsafe {
                graph::load_exact_graph_element(
                    index_relation,
                    page::ItemPointer {
                        block_number,
                        offset_number,
                    },
                    storage,
                )
            };
            if element.deleted || element.heaptids.is_empty() || element.level != target_level {
                continue;
            }

            tids.push(page::ItemPointer {
                block_number,
                offset_number,
            });
        }
    }

    tids
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_collect_element_tids_at_or_above_level(
    index_relation: pg_sys::Relation,
    storage: graph::GraphStorageDescriptor,
    min_level: u8,
) -> Vec<page::ItemPointer> {
    // SAFETY: `index_relation` is an open index relation supplied by the debug
    // caller; PostgreSQL returns the current main-fork block count.
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let element_tag = debug_graph_tuple_tag(storage);
    let mut tids = Vec::new();

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        // SAFETY: The block number is within the relation's main-fork block
        // count and is read under a shared buffer lock for inspection.
        let buffer = unsafe {
            LockedBufferGuard::read_main(
                index_relation,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                pg_sys::BUFFER_LOCK_SHARE as i32,
            )
        };
        let buffer = buffer.unwrap_or_else(|| {
            pgrx::error!("ec_hnsw debug failed to open graph block {block_number}")
        });
        let page_ptr = buffer.page().cast::<u8>();
        let page_size = buffer.page_size();
        let line_pointer_count = super::shared::page_line_pointer_count(page_ptr);

        for offset_number in 1..=line_pointer_count {
            // SAFETY: The buffer remains share-locked while the helper validates
            // this line pointer and exposes tuple bytes only to the closure.
            let matches_element_tag = unsafe {
                debug_with_page_line_tuple_bytes(
                    page_ptr,
                    page_size,
                    offset_number,
                    |tuple_bytes| tuple_bytes.first().copied() == Some(element_tag),
                )
            }
            .unwrap_or(false);
            if !matches_element_tag {
                continue;
            }

            // SAFETY: The tuple tag matched the graph element tag on a locked
            // page, and the graph loader validates the storage-specific body.
            let element = unsafe {
                graph::load_exact_graph_element(
                    index_relation,
                    page::ItemPointer {
                        block_number,
                        offset_number,
                    },
                    storage,
                )
            };
            if element.deleted || element.heaptids.is_empty() || element.level < min_level {
                continue;
            }

            tids.push(page::ItemPointer {
                block_number,
                offset_number,
            });
        }
    }

    tids
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_collect_element_tid_by_heap_tid(
    index_relation: pg_sys::Relation,
    storage: graph::GraphStorageDescriptor,
) -> std::collections::HashMap<HeapTidCoords, page::ItemPointer> {
    // SAFETY: `index_relation` is an open index relation supplied by the debug
    // caller; PostgreSQL returns the current main-fork block count.
    let block_count = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let element_tag = debug_graph_tuple_tag(storage);
    let mut map = std::collections::HashMap::new();

    for block_number in page::FIRST_DATA_BLOCK_NUMBER..block_count {
        // SAFETY: The block number is within the relation's main-fork block
        // count and is read under a shared buffer lock for inspection.
        let buffer = unsafe {
            LockedBufferGuard::read_main(
                index_relation,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                pg_sys::BUFFER_LOCK_SHARE as i32,
            )
        };
        let buffer = buffer.unwrap_or_else(|| {
            pgrx::error!("ec_hnsw debug failed to open graph block {block_number}")
        });
        let page_ptr = buffer.page().cast::<u8>();
        let page_size = buffer.page_size();
        let line_pointer_count = super::shared::page_line_pointer_count(page_ptr);

        for offset_number in 1..=line_pointer_count {
            // SAFETY: The buffer remains share-locked while the helper validates
            // this line pointer and exposes tuple bytes only to the closure.
            let matches_element_tag = unsafe {
                debug_with_page_line_tuple_bytes(
                    page_ptr,
                    page_size,
                    offset_number,
                    |tuple_bytes| tuple_bytes.first().copied() == Some(element_tag),
                )
            }
            .unwrap_or(false);
            if !matches_element_tag {
                continue;
            }

            // SAFETY: The tuple tag matched the graph element tag on a locked
            // page, and the graph loader validates the storage-specific body.
            let element = unsafe {
                graph::load_exact_graph_element(
                    index_relation,
                    page::ItemPointer {
                        block_number,
                        offset_number,
                    },
                    storage,
                )
            };
            if element.deleted || element.heaptids.is_empty() {
                continue;
            }

            let element_tid = page::ItemPointer {
                block_number,
                offset_number,
            };
            for heap_tid in element.heaptids {
                map.insert((heap_tid.block_number, heap_tid.offset_number), element_tid);
            }
        }
    }

    map
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_top_level_oracle_scan_heap_tids(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    ef_search: usize,
) -> Vec<HeapTidCoords> {
    // SAFETY: This wrapper forwards the caller-provided index oid, query, and
    // bounded search parameters to the k-seed oracle helper.
    unsafe { debug_top_level_oracle_k_seed_scan_heap_tids(index_oid, query, ef_search, 1) }
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_all_top_level_heap_tids(index_oid: pg_sys::Oid) -> Vec<HeapTidCoords> {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_all_top_level_heap_tids");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index relation open while its
    // metadata page is read.
    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
    if metadata.entry_point == page::ItemPointer::INVALID || metadata.dimensions == 0 {
        return Vec::new();
    }

    // SAFETY: Metadata was read from the open index relation and validated
    // enough to resolve the graph storage descriptor for debug inspection.
    let storage = unsafe { debug_graph_storage(index_relation, &metadata) };
    // SAFETY: The relation guard keeps the graph relation open while the helper
    // scans locked pages for top-level element TIDs.
    let mut heap_tids =
        unsafe { debug_collect_element_tids_at_level(index_relation, storage, metadata.max_level) }
            .into_iter()
            .filter_map(|element_tid| {
                // SAFETY: `element_tid` was collected from graph pages matching
                // the storage element tag, and the graph loader validates the
                // tuple body.
                let element = unsafe {
                    graph::load_exact_graph_element(index_relation, element_tid, storage)
                };
                if element.deleted {
                    return None;
                }
                element
                    .heaptids
                    .first()
                    .copied()
                    .map(debug_item_pointer_coords)
            })
            .collect::<Vec<_>>();
    heap_tids.sort_unstable();
    heap_tids.dedup();

    heap_tids
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_top_level_reachable_heap_tids(
    index_oid: pg_sys::Oid,
) -> Vec<HeapTidCoords> {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_top_level_reachable_heap_tids");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index relation open while its
    // metadata page is read.
    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
    if metadata.entry_point == page::ItemPointer::INVALID || metadata.dimensions == 0 {
        return Vec::new();
    }

    // SAFETY: Metadata was read from the open index relation and validated
    // enough to resolve the graph storage descriptor for debug inspection.
    let storage = unsafe { debug_graph_storage(index_relation, &metadata) };
    let m = usize::from(metadata.m);
    let mut queue = std::collections::VecDeque::from([metadata.entry_point]);
    let mut visited = std::collections::HashSet::new();
    let mut heap_tids = Vec::new();

    while let Some(element_tid) = queue.pop_front() {
        if !visited.insert(element_tid) {
            continue;
        }

        let element =
            // SAFETY: `element_tid` is either the metadata entry point or a
            // neighbor returned by graph adjacency loading; the graph loader
            // validates the tuple body before returning it.
            unsafe { graph::load_exact_graph_element(index_relation, element_tid, storage) };
        if element.deleted {
            continue;
        }

        if let Some(heap_tid) = element.heaptids.first().copied() {
            heap_tids.push(debug_item_pointer_coords(heap_tid));
        }

        // SAFETY: The current element was loaded from graph storage and `m`
        // comes from validated metadata; adjacency loading validates the graph
        // tuple before returning layer neighbors.
        for neighbor_tid in unsafe {
            debug_load_neighbor_tids_for_layer(
                index_relation,
                storage,
                element_tid,
                m,
                metadata.max_level,
            )
        } {
            if !visited.contains(&neighbor_tid) {
                queue.push_back(neighbor_tid);
            }
        }
    }

    heap_tids.sort_unstable();
    heap_tids.dedup();
    heap_tids
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_layer0_reachable_live_element_tids(
    index_oid: pg_sys::Oid,
) -> Vec<page::ItemPointer> {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_layer0_reachable_live_element_tids");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index relation open while its
    // metadata page is read.
    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
    if metadata.entry_point == page::ItemPointer::INVALID || metadata.dimensions == 0 {
        return Vec::new();
    }

    // SAFETY: Metadata was read from the open index relation and validated
    // enough to resolve the graph storage descriptor for debug inspection.
    let storage = unsafe { debug_graph_storage(index_relation, &metadata) };
    let m = usize::from(metadata.m);
    let mut queue = std::collections::VecDeque::from([metadata.entry_point]);
    let mut visited = std::collections::HashSet::new();
    let mut reachable = Vec::new();

    while let Some(element_tid) = queue.pop_front() {
        if !visited.insert(element_tid) {
            continue;
        }

        let element =
            // SAFETY: `element_tid` is either the metadata entry point or a
            // neighbor returned by graph adjacency loading; the graph loader
            // validates the tuple body before returning it.
            unsafe { graph::load_exact_graph_element(index_relation, element_tid, storage) };
        if element.deleted || element.heaptids.is_empty() {
            continue;
        }
        reachable.push(element_tid);

        // SAFETY: The current element was loaded from graph storage and `m`
        // comes from validated metadata; adjacency loading validates the graph
        // tuple before returning layer-0 neighbors.
        for neighbor_tid in unsafe {
            debug_load_neighbor_tids_for_layer(index_relation, storage, element_tid, m, 0)
        } {
            if !visited.contains(&neighbor_tid) {
                queue.push_back(neighbor_tid);
            }
        }
    }

    reachable.sort_unstable_by(|left, right| {
        left.block_number
            .cmp(&right.block_number)
            .then_with(|| left.offset_number.cmp(&right.offset_number))
    });
    reachable
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_top_level_oracle_k_seed_heap_tids(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    top_level_seed_count: usize,
) -> Vec<HeapTidCoords> {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_top_level_oracle_k_seed_heap_tids");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index relation open while its
    // metadata page is read.
    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
    if metadata.entry_point == page::ItemPointer::INVALID
        || metadata.dimensions == 0
        || top_level_seed_count == 0
    {
        return Vec::new();
    }

    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used to prepare query state.
    let scan = debug_am_begin_scan(index_relation, 0, 1);
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let storage = opaque.scan_graph_storage;
    // SAFETY: The opaque belongs to the live scan and rescan prepares a cached
    // quantizer for score computation.
    let quantizer = unsafe { &*opaque.cached_quantizer };
    // SAFETY: The opaque belongs to the live scan and rescan prepares query
    // storage for score computation.
    let prepared_query = unsafe { &*opaque.prepared_query };
    // SAFETY: The relation guard keeps the graph relation open while the helper
    // scans locked pages for top-level element TIDs.
    let top_level_tids =
        unsafe { debug_collect_element_tids_at_level(index_relation, storage, metadata.max_level) };

    let mut heap_tids = top_level_tids
        .into_iter()
        .filter_map(|seed_tid| {
            // SAFETY: `seed_tid` was collected from graph pages matching the
            // storage element tag, and the graph loader validates the tuple.
            let element =
                unsafe { graph::load_exact_graph_element(index_relation, seed_tid, storage) };
            if element.deleted || element.heaptids.is_empty() {
                return None;
            }
            Some((
                search::BeamCandidate::new(
                    seed_tid,
                    -quantizer.score_ip_from_parts(prepared_query, element.gamma, &element.code),
                ),
                debug_item_pointer_coords(*element.heaptids.first().expect("heaptids non-empty")),
            ))
        })
        .collect::<Vec<_>>();
    heap_tids.sort_by(|left, right| left.0.score.total_cmp(&right.0.score));
    heap_tids.truncate(top_level_seed_count);
    let heap_tids = heap_tids
        .into_iter()
        .map(|(_, heap_tid)| heap_tid)
        .collect();

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    heap_tids
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_top_level_oracle_k_seed_scan_heap_tids(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    ef_search: usize,
    top_level_seed_count: usize,
) -> Vec<HeapTidCoords> {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_top_level_oracle_k_seed_scan_heap_tids");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index relation open while its
    // metadata page is read.
    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
    if metadata.entry_point == page::ItemPointer::INVALID
        || metadata.dimensions == 0
        || top_level_seed_count == 0
    {
        return Vec::new();
    }

    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used to prepare query state.
    let scan = debug_am_begin_scan(index_relation, 0, 1);
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let storage = opaque.scan_graph_storage;
    // SAFETY: The opaque belongs to the live scan and rescan prepares a cached
    // quantizer for score computation.
    let quantizer = unsafe { &*opaque.cached_quantizer };
    // SAFETY: The opaque belongs to the live scan and rescan prepares query
    // storage for score computation.
    let prepared_query = unsafe { &*opaque.prepared_query };
    // SAFETY: The relation guard keeps the graph relation open while the helper
    // scans locked pages for top-level element TIDs.
    let top_level_tids =
        unsafe { debug_collect_element_tids_at_level(index_relation, storage, metadata.max_level) };

    let mut seeds = top_level_tids
        .into_iter()
        .filter_map(|seed_tid| {
            // SAFETY: `seed_tid` was collected from graph pages matching the
            // storage element tag, and the graph loader validates the tuple.
            let element =
                unsafe { graph::load_exact_graph_element(index_relation, seed_tid, storage) };
            if element.deleted || element.heaptids.is_empty() {
                return None;
            }
            Some(search::BeamCandidate::new(
                seed_tid,
                -quantizer.score_ip_from_parts(prepared_query, element.gamma, &element.code),
            ))
        })
        .collect::<Vec<_>>();
    seeds.sort_by(|left, right| left.score.total_cmp(&right.score));
    seeds.truncate(top_level_seed_count);

    let tids = if seeds.is_empty() {
        Vec::new()
    } else {
        // SAFETY: The seed candidates were loaded from graph storage for this
        // index, and the search helper validates neighbor tuples as it walks
        // layer 0 with the supplied storage descriptor.
        let ordered_candidates = unsafe {
            graph::search_layer0_result_candidates_with_storage(
                index_relation,
                storage,
                usize::from(opaque.scan_m),
                ef_search.max(1),
                seeds,
                |_| true,
                |neighbor| {
                    Some(-quantizer.score_ip_from_parts(
                        prepared_query,
                        neighbor.gamma,
                        &neighbor.code,
                    ))
                },
            )
        };
        let mut emitted_elements = std::collections::HashSet::new();
        let mut heap_tids = Vec::new();
        for candidate in ordered_candidates {
            if !emitted_elements.insert(candidate.node) {
                continue;
            }

            // SAFETY: Search candidates come from graph traversal on this
            // relation/storage pair, and the loader validates the tuple body.
            let element =
                unsafe { graph::load_exact_graph_element(index_relation, candidate.node, storage) };
            if element.deleted || element.heaptids.is_empty() {
                continue;
            }

            heap_tids.extend(element.heaptids.into_iter().map(debug_item_pointer_coords));
        }
        heap_tids
    };

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    tids
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_layer_oracle_k_carrydown_scan_heap_tids(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    ef_search: usize,
    layer: u8,
    seed_count: usize,
) -> Vec<HeapTidCoords> {
    let index_relation_guard = IndexRelationGuard::access_share(
        index_oid,
        "debug_layer_oracle_k_carrydown_scan_heap_tids",
    );
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index relation open while its
    // metadata page is read.
    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
    if metadata.entry_point == page::ItemPointer::INVALID
        || metadata.dimensions == 0
        || seed_count == 0
        || layer == 0
        || layer > metadata.max_level
    {
        return Vec::new();
    }

    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used to prepare query state.
    let scan = debug_am_begin_scan(index_relation, 0, 1);
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let storage = opaque.scan_graph_storage;
    // SAFETY: The opaque belongs to the live scan and rescan prepares a cached
    // quantizer for score computation.
    let quantizer = unsafe { &*opaque.cached_quantizer };
    // SAFETY: The opaque belongs to the live scan and rescan prepares query
    // storage for score computation.
    let prepared_query = unsafe { &*opaque.prepared_query };
    // SAFETY: The relation guard keeps the graph relation open while the helper
    // scans locked pages for candidate element TIDs.
    let layer_tids =
        unsafe { debug_collect_element_tids_at_or_above_level(index_relation, storage, layer) };

    let mut seeds = layer_tids
        .into_iter()
        .filter_map(|seed_tid| {
            // SAFETY: `seed_tid` was collected from graph pages matching the
            // storage element tag, and the graph loader validates the tuple.
            let element =
                unsafe { graph::load_exact_graph_element(index_relation, seed_tid, storage) };
            if element.deleted || element.heaptids.is_empty() {
                return None;
            }
            Some(search::BeamCandidate::new(
                seed_tid,
                -quantizer.score_ip_from_parts(prepared_query, element.gamma, &element.code),
            ))
        })
        .collect::<Vec<_>>();
    seeds.sort_by(|left, right| left.score.total_cmp(&right.score));
    seeds.truncate(seed_count);

    let tids = if seeds.is_empty() {
        Vec::new()
    } else {
        let mut carrydown_seeds = seeds;
        for current_layer in (1..=layer).rev() {
            // SAFETY: The carrydown seeds were loaded from graph storage for
            // this index, and the search helper validates neighbor tuples as it
            // walks the requested upper layer.
            carrydown_seeds = unsafe {
                graph::search_layer_result_candidates_with_storage(
                    index_relation,
                    storage,
                    usize::from(opaque.scan_m),
                    current_layer,
                    ef_search.max(1),
                    carrydown_seeds,
                    |_| true,
                    |neighbor| {
                        Some(-quantizer.score_ip_from_parts(
                            prepared_query,
                            neighbor.gamma,
                            &neighbor.code,
                        ))
                    },
                )
            };
            if carrydown_seeds.is_empty() {
                break;
            }
        }

        // SAFETY: The carrydown seeds came from upper-layer graph traversal on
        // this relation/storage pair, and the layer-0 helper validates neighbor
        // tuples as it walks.
        let ordered_candidates = unsafe {
            graph::search_layer0_result_candidates_with_storage(
                index_relation,
                storage,
                usize::from(opaque.scan_m),
                ef_search.max(1),
                carrydown_seeds,
                |_| true,
                |neighbor| {
                    Some(-quantizer.score_ip_from_parts(
                        prepared_query,
                        neighbor.gamma,
                        &neighbor.code,
                    ))
                },
            )
        };
        let mut emitted_elements = std::collections::HashSet::new();
        let mut heap_tids = Vec::new();
        for candidate in ordered_candidates {
            if !emitted_elements.insert(candidate.node) {
                continue;
            }

            // SAFETY: Search candidates come from graph traversal on this
            // relation/storage pair, and the loader validates the tuple body.
            let element =
                unsafe { graph::load_exact_graph_element(index_relation, candidate.node, storage) };
            if element.deleted || element.heaptids.is_empty() {
                continue;
            }

            heap_tids.extend(element.heaptids.into_iter().map(debug_item_pointer_coords));
        }
        heap_tids
    };

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    tids
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_layer_oracle_k_seed_layer0_neighbor_heap_tids(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    layer: u8,
    seed_count: usize,
) -> Vec<HeapTidCoords> {
    let index_relation_guard = IndexRelationGuard::access_share(
        index_oid,
        "debug_layer_oracle_k_seed_layer0_neighbor_heap_tids",
    );
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index relation open while its
    // metadata page is read.
    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
    if metadata.entry_point == page::ItemPointer::INVALID
        || metadata.dimensions == 0
        || seed_count == 0
        || layer == 0
        || layer > metadata.max_level
    {
        return Vec::new();
    }

    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used to prepare query state.
    let scan = debug_am_begin_scan(index_relation, 0, 1);
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let storage = opaque.scan_graph_storage;
    // SAFETY: The opaque belongs to the live scan and rescan prepares a cached
    // quantizer for score computation.
    let quantizer = unsafe { &*opaque.cached_quantizer };
    // SAFETY: The opaque belongs to the live scan and rescan prepares query
    // storage for score computation.
    let prepared_query = unsafe { &*opaque.prepared_query };
    // SAFETY: The relation guard keeps the graph relation open while the helper
    // scans locked pages for candidate element TIDs.
    let layer_tids =
        unsafe { debug_collect_element_tids_at_or_above_level(index_relation, storage, layer) };

    let mut seeds = layer_tids
        .into_iter()
        .filter_map(|seed_tid| {
            // SAFETY: `seed_tid` was collected from graph pages matching the
            // storage element tag, and the graph loader validates the tuple.
            let element =
                unsafe { graph::load_exact_graph_element(index_relation, seed_tid, storage) };
            if element.deleted || element.heaptids.is_empty() {
                return None;
            }
            Some(search::BeamCandidate::new(
                seed_tid,
                -quantizer.score_ip_from_parts(prepared_query, element.gamma, &element.code),
            ))
        })
        .collect::<Vec<_>>();
    seeds.sort_by(|left, right| left.score.total_cmp(&right.score));
    seeds.truncate(seed_count);

    let mut scored_elements = Vec::new();
    let mut visited_elements = std::collections::HashSet::new();
    for seed in seeds {
        if !visited_elements.insert(seed.node) {
            continue;
        }

        // SAFETY: Seed candidates were loaded from graph pages for this
        // relation/storage pair, and the graph loader validates the tuple.
        let seed_element =
            unsafe { graph::load_exact_graph_element(index_relation, seed.node, storage) };
        if !seed_element.deleted {
            scored_elements.push((seed.score, seed_element.heaptids.clone()));
        }

        // SAFETY: The seed element was loaded from graph storage and `scan_m`
        // comes from the initialized scan opaque; adjacency loading validates
        // the graph tuple before returning layer-0 neighbors.
        for neighbor_tid in unsafe {
            debug_load_neighbor_tids_for_layer(
                index_relation,
                storage,
                seed.node,
                usize::from(opaque.scan_m),
                0,
            )
        } {
            if !visited_elements.insert(neighbor_tid) {
                continue;
            }

            // SAFETY: Neighbor TIDs come from graph adjacency loading for this
            // relation/storage pair, and the loader validates the tuple body.
            let neighbor =
                unsafe { graph::load_exact_graph_element(index_relation, neighbor_tid, storage) };
            if neighbor.deleted || neighbor.heaptids.is_empty() {
                continue;
            }

            let score =
                -quantizer.score_ip_from_parts(prepared_query, neighbor.gamma, &neighbor.code);
            scored_elements.push((score, neighbor.heaptids));
        }
    }

    scored_elements.sort_by(|left, right| left.0.total_cmp(&right.0));
    let mut heap_tids = Vec::new();
    let mut seen_heap_tids = std::collections::HashSet::new();
    for (_score, element_heap_tids) in scored_elements {
        for heap_tid in element_heap_tids {
            let coords = debug_item_pointer_coords(heap_tid);
            if seen_heap_tids.insert(coords) {
                heap_tids.push(coords);
            }
        }
    }

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    heap_tids
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_exact_seed_scan_heap_tids(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    seed_heap_tids: Vec<HeapTidCoords>,
    ef_search: usize,
) -> Vec<HeapTidCoords> {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_exact_seed_scan_heap_tids");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index relation open while its
    // metadata page is read.
    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
    if metadata.entry_point == page::ItemPointer::INVALID
        || metadata.dimensions == 0
        || seed_heap_tids.is_empty()
    {
        return Vec::new();
    }

    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used to prepare query state.
    let scan = debug_am_begin_scan(index_relation, 0, 1);
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let storage = opaque.scan_graph_storage;
    // SAFETY: The opaque belongs to the live scan and rescan prepares a cached
    // quantizer for score computation.
    let quantizer = unsafe { &*opaque.cached_quantizer };
    // SAFETY: The opaque belongs to the live scan and rescan prepares query
    // storage for score computation.
    let prepared_query = unsafe { &*opaque.prepared_query };
    // SAFETY: The relation guard keeps the graph relation open while the helper
    // scans locked graph pages and maps heap TIDs to element TIDs.
    let element_by_heap_tid =
        unsafe { debug_collect_element_tid_by_heap_tid(index_relation, storage) };
    let seed_element_tids = seed_heap_tids
        .into_iter()
        .filter_map(|heap_tid| element_by_heap_tid.get(&heap_tid).copied())
        .collect::<Vec<_>>();

    let tids = if seed_element_tids.is_empty() {
        Vec::new()
    } else {
        let seeds = seed_element_tids
            .into_iter()
            .filter_map(|seed_tid| {
                // SAFETY: Seed element TIDs were resolved from graph pages for
                // this relation/storage pair, and the loader validates tuples.
                let element =
                    unsafe { graph::load_exact_graph_element(index_relation, seed_tid, storage) };
                if element.deleted || element.heaptids.is_empty() {
                    return None;
                }
                Some(search::BeamCandidate::new(
                    seed_tid,
                    -quantizer.score_ip_from_parts(prepared_query, element.gamma, &element.code),
                ))
            })
            .collect::<Vec<_>>();
        // SAFETY: Seed candidates were resolved from graph storage for this
        // index, and the search helper validates neighbor tuples as it walks
        // layer 0 with the supplied storage descriptor.
        let ordered_candidates = unsafe {
            graph::search_layer0_result_candidates_with_storage(
                index_relation,
                storage,
                usize::from(opaque.scan_m),
                ef_search.max(1),
                seeds,
                |_| true,
                |neighbor| {
                    Some(-quantizer.score_ip_from_parts(
                        prepared_query,
                        neighbor.gamma,
                        &neighbor.code,
                    ))
                },
            )
        };
        let mut emitted_elements = std::collections::HashSet::new();
        let mut heap_tids = Vec::new();
        for candidate in ordered_candidates {
            if !emitted_elements.insert(candidate.node) {
                continue;
            }

            // SAFETY: Search candidates come from graph traversal on this
            // relation/storage pair, and the loader validates the tuple body.
            let element =
                unsafe { graph::load_exact_graph_element(index_relation, candidate.node, storage) };
            if element.deleted || element.heaptids.is_empty() {
                continue;
            }

            heap_tids.extend(element.heaptids.into_iter().map(debug_item_pointer_coords));
        }
        heap_tids
    };

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    tids
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_scan_heap_tids_with_scores(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> Vec<(HeapTidCoords, f32)> {
    // SAFETY: The debug helper opens the index, owning heap, and scan snapshot
    // and keeps them alive in `scan_state`.
    let scan_state = unsafe { debug_begin_heap_backed_scan(index_oid) };
    let scan = scan_state.scan.as_ptr();

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan_state` owns a live heap-backed scan, there are no index
    // quals, and `orderby` is a valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    let mut tids = Vec::new();
    // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple calls
    // may advance the live scan descriptor.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {
        // SAFETY: A successful gettuple call populated `xs_heaptid` for this
        // live index scan descriptor.
        let heap_tid = debug_scan_heap_tid(scan);
        let score = debug_scan_orderby_score(scan)
            .expect("graph-first scan should publish an order-by score for emitted tuples");
        tids.push((heap_tid, score));
    }

    // SAFETY: `scan_state` owns the scan and relation guards and is consumed
    // once after iteration completes.
    unsafe { debug_end_heap_backed_scan(scan_state) };
    tids
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_scan_heap_tids_with_score_comparisons(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> Vec<(HeapTidCoords, f32, Option<f32>, Option<i32>)> {
    // SAFETY: The debug helper opens the index, owning heap, and scan snapshot
    // and keeps them alive in `scan_state`.
    let scan_state = unsafe { debug_begin_heap_backed_scan(index_oid) };
    let scan = scan_state.scan.as_ptr();

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan_state` owns a live heap-backed scan, there are no index
    // quals, and `orderby` is a valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    let mut tids = Vec::new();
    // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple calls
    // may advance the live scan descriptor.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {
        // SAFETY: A successful gettuple call populated `xs_heaptid` for this
        // live index scan descriptor.
        let heap_tid = debug_scan_heap_tid(scan);
        let approx_score = debug_current_result_approx_score(scan)
            .or_else(|| debug_scan_orderby_score(scan))
            .expect("graph-first scan should publish an approximate score for emitted tuples");
        let comparison_score = debug_current_result_comparison_score(scan);
        let approx_rank = debug_current_result_approx_rank(scan);
        tids.push((heap_tid, approx_score, comparison_score, approx_rank));
    }

    // SAFETY: `scan_state` owns the scan and relation guards and is consumed
    // once after iteration completes.
    unsafe { debug_end_heap_backed_scan(scan_state) };
    tids
}

#[cfg(any(test, feature = "pg_test"))]
unsafe fn debug_scan_uses_grouped_storage(index_oid: pg_sys::Oid) -> bool {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_scan_uses_grouped_storage");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index relation open while its
    // metadata page is read.
    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
    let grouped_results = matches!(
        // SAFETY: Metadata was read from the open index relation and is used to
        // resolve the graph storage descriptor for debug classification.
        unsafe { graph::GraphStorageDescriptor::from_index_relation(index_relation, &metadata) }
            .unwrap_or_else(|e| {
                pgrx::error!("ec_hnsw debug grouped scan comparison requires valid metadata: {e}")
            }),
        graph::GraphStorageDescriptor::PqFastScan(_)
    );
    grouped_results
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_grouped_window_size(window_size: i32) -> usize {
    if window_size <= 0 {
        pgrx::error!("ec_hnsw debug grouped scan window size must be positive");
    }
    usize::try_from(window_size).expect("grouped debug window size should fit in usize")
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_grouped_rank_metrics<I>(rows: I) -> DebugGroupedRankMetrics
where
    I: IntoIterator<Item = (i32, Option<i32>, Option<i32>)>,
{
    let mut compared_result_count = 0_i32;
    let mut abs_rank_shift_sum = 0.0_f64;
    let mut max_abs_rank_shift = 0_i32;
    let mut d_squared_sum = 0.0_f64;
    let mut exact_best_observed_rank = None;
    let mut exact_top4_max_observed_rank = None;

    for (observed_rank, exact_rank, explicit_rank_shift) in rows {
        let Some(exact_rank) = exact_rank else {
            continue;
        };
        compared_result_count += 1;

        let abs_rank_shift = explicit_rank_shift
            .unwrap_or(observed_rank - exact_rank)
            .abs();
        abs_rank_shift_sum += f64::from(abs_rank_shift);
        max_abs_rank_shift = max_abs_rank_shift.max(abs_rank_shift);

        let d = f64::from(observed_rank - exact_rank);
        d_squared_sum += d * d;

        if exact_rank == 1 {
            exact_best_observed_rank = Some(observed_rank);
        }
        if exact_rank <= 4 {
            exact_top4_max_observed_rank = Some(
                exact_top4_max_observed_rank
                    .map_or(observed_rank, |max_rank: i32| max_rank.max(observed_rank)),
            );
        }
    }

    let mean_abs_rank_shift = if compared_result_count == 0 {
        0.0
    } else {
        abs_rank_shift_sum / f64::from(compared_result_count)
    };
    let spearman_rank_correlation = if compared_result_count < 2 {
        0.0
    } else {
        let n = f64::from(compared_result_count);
        1.0 - (6.0 * d_squared_sum / (n * (n * n - 1.0)))
    };

    DebugGroupedRankMetrics {
        compared_result_count,
        mean_abs_rank_shift,
        max_abs_rank_shift,
        spearman_rank_correlation,
        exact_best_observed_rank,
        exact_top4_max_observed_rank,
    }
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_grouped_scan_windowed_rows_from_comparison_rows(
    rows: &[DebugGroupedScanComparisonRow],
    window_size: usize,
) -> Vec<DebugGroupedScanWindowedRow> {
    let mut ordered_rows = rows.to_vec();
    ordered_rows.sort_by_key(|row| row.1);
    let mut buffered_rows = Vec::with_capacity(window_size.max(1));
    let mut next_idx = 0usize;
    let mut output_rows = Vec::with_capacity(ordered_rows.len());

    // This is a sliding prefix window, so the tail drains from progressively smaller
    // buffers once the approximate-order input is exhausted.
    while output_rows.len() < ordered_rows.len() {
        while buffered_rows.len() < window_size && next_idx < ordered_rows.len() {
            buffered_rows.push(ordered_rows[next_idx]);
            next_idx += 1;
        }
        let Some((selected_idx, _)) =
            buffered_rows
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    let left_score = left.3;
                    let right_score = right.3;
                    // Missing comparison scores stay in approximate order instead of being
                    // dropped from the simulation.
                    let left_exact = left_score.unwrap_or(left.2);
                    let right_exact = right_score.unwrap_or(right.2);
                    left_exact
                        .total_cmp(&right_exact)
                        .then_with(|| left.1.cmp(&right.1))
                })
        else {
            break;
        };

        let (heap_tid, approx_rank, approx_score, comparison_score, exact_rank, exact_rank_shift) =
            buffered_rows.remove(selected_idx);
        let windowed_rank =
            i32::try_from(output_rows.len() + 1).expect("windowed rank should fit in i32");
        let windowed_rank_shift = exact_rank.map(|rank| windowed_rank - rank);
        output_rows.push((
            heap_tid,
            approx_rank,
            windowed_rank,
            approx_score,
            comparison_score,
            exact_rank,
            exact_rank_shift,
            windowed_rank_shift,
        ));
    }

    output_rows
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_grouped_scan_comparison_rows(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> Vec<DebugGroupedScanComparisonRow> {
    // SAFETY: The debug wrapper forwards the caller-provided index oid to the
    // grouped-storage classifier, which opens and reads index metadata.
    let grouped_results = unsafe { debug_scan_uses_grouped_storage(index_oid) };
    // SAFETY: The score-comparison helper owns its scan descriptor and returns
    // materialized debug rows before cleanup.
    let rows = unsafe { debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query) };
    let ordered_rows = if grouped_results {
        let mut ordered_rows = rows
            .into_iter()
            .enumerate()
            .map(
                |(idx, (heap_tid, approx_score, comparison_score, approx_rank))| {
                    (
                        heap_tid,
                        approx_score,
                        comparison_score,
                        approx_rank.unwrap_or_else(|| {
                            i32::try_from(idx + 1).expect("approx rank should fit in i32")
                        }),
                    )
                },
            )
            .collect::<Vec<_>>();
        ordered_rows.sort_by_key(|row| row.3);
        ordered_rows
    } else {
        rows.into_iter()
            .enumerate()
            .map(
                |(idx, (heap_tid, approx_score, comparison_score, _approx_rank))| {
                    (
                        heap_tid,
                        approx_score,
                        comparison_score,
                        i32::try_from(idx + 1).expect("approx rank should fit in i32"),
                    )
                },
            )
            .collect::<Vec<_>>()
    };
    let mut exact_ranks = vec![None; ordered_rows.len()];
    if grouped_results {
        let mut compared_rows = ordered_rows
            .iter()
            .enumerate()
            .filter_map(
                |(idx, (_heap_tid, _approx_score, comparison_score, _approx_rank))| {
                    comparison_score.map(|exact_score| (idx, exact_score))
                },
            )
            .collect::<Vec<_>>();
        compared_rows.sort_by(|(left_idx, left_score), (right_idx, right_score)| {
            let left_approx_rank = ordered_rows[*left_idx].3;
            let right_approx_rank = ordered_rows[*right_idx].3;
            left_score
                .total_cmp(right_score)
                .then_with(|| left_approx_rank.cmp(&right_approx_rank))
        });
        for (rank, (idx, _exact_score)) in compared_rows.into_iter().enumerate() {
            exact_ranks[idx] = Some(i32::try_from(rank + 1).expect("exact rank should fit in i32"));
        }
    }

    ordered_rows
        .into_iter()
        .enumerate()
        .map(
            |(idx, (heap_tid, approx_score, comparison_score, approx_rank))| {
                let exact_rank = exact_ranks[idx];
                let exact_rank_shift = exact_rank.map(|rank| approx_rank - rank);
                (
                    heap_tid,
                    approx_rank,
                    approx_score,
                    grouped_results.then_some(comparison_score).flatten(),
                    exact_rank,
                    exact_rank_shift,
                )
            },
        )
        .collect()
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_grouped_scan_comparison_summary(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugGroupedScanComparisonSummary {
    // SAFETY: The debug wrapper forwards the caller-provided index oid to the
    // grouped-storage classifier, which opens and reads index metadata.
    let grouped_results = unsafe { debug_scan_uses_grouped_storage(index_oid) };
    // SAFETY: The score-comparison helper owns its scan descriptor and returns
    // materialized debug rows before cleanup.
    let rows = unsafe { debug_gettuple_scan_heap_tids_with_score_comparisons(index_oid, query) };
    let emitted_result_count =
        i32::try_from(rows.len()).expect("debug comparison summary count should fit in i32");
    if !grouped_results {
        return (emitted_result_count, 0, 0, 0, 0.0, 0.0, 0.0);
    }

    let grouped_result_count = emitted_result_count;
    let mut compared_result_count = 0_i32;
    let mut missing_comparison_count = 0_i32;
    let mut abs_delta_sum = 0.0_f64;
    let mut signed_delta_sum = 0.0_f64;
    let mut max_abs_score_delta = 0.0_f32;

    for (_heap_tid, approx_score, comparison_score, _approx_rank) in rows {
        match comparison_score {
            Some(exact_score) => {
                compared_result_count += 1;
                let signed_delta = approx_score - exact_score;
                abs_delta_sum += f64::from(signed_delta.abs());
                signed_delta_sum += f64::from(signed_delta);
                max_abs_score_delta = max_abs_score_delta.max(signed_delta.abs());
            }
            None => missing_comparison_count += 1,
        }
    }

    let mean_abs_score_delta = if compared_result_count == 0 {
        0.0
    } else {
        abs_delta_sum / f64::from(compared_result_count)
    };
    let mean_signed_score_delta = if compared_result_count == 0 {
        0.0
    } else {
        signed_delta_sum / f64::from(compared_result_count)
    };

    (
        emitted_result_count,
        grouped_result_count,
        compared_result_count,
        missing_comparison_count,
        mean_abs_score_delta,
        max_abs_score_delta,
        mean_signed_score_delta,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_grouped_scan_order_drift_summary(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugGroupedScanOrderDriftSummary {
    // SAFETY: The debug wrapper forwards the caller-provided index oid to the
    // grouped-storage classifier, which opens and reads index metadata.
    let grouped_results = unsafe { debug_scan_uses_grouped_storage(index_oid) };
    // SAFETY: The comparison-row helper owns its scan descriptor and returns
    // materialized debug rows before cleanup.
    let rows = unsafe { debug_grouped_scan_comparison_rows(index_oid, query) };
    let emitted_result_count =
        i32::try_from(rows.len()).expect("debug order drift summary count should fit in i32");
    if !grouped_results {
        return (
            emitted_result_count,
            0,
            0,
            0.0,
            0,
            0.0,
            None,
            None,
            false,
            false,
            false,
            false,
        );
    }

    let grouped_result_count = emitted_result_count;
    let metrics = debug_grouped_rank_metrics(rows.iter().map(
        |(
            _heap_tid,
            approx_rank,
            _approx_score,
            _comparison_score,
            exact_rank,
            exact_rank_shift,
        )| { (*approx_rank, *exact_rank, *exact_rank_shift) },
    ));
    let window_1_contains_exact_best = metrics
        .exact_best_observed_rank
        .is_some_and(|rank| rank <= 1);
    let window_2_contains_exact_best = metrics
        .exact_best_observed_rank
        .is_some_and(|rank| rank <= 2);
    let window_4_contains_exact_best = metrics
        .exact_best_observed_rank
        .is_some_and(|rank| rank <= 4);
    let window_8_contains_exact_best = metrics
        .exact_best_observed_rank
        .is_some_and(|rank| rank <= 8);

    (
        emitted_result_count,
        grouped_result_count,
        metrics.compared_result_count,
        metrics.mean_abs_rank_shift,
        metrics.max_abs_rank_shift,
        metrics.spearman_rank_correlation,
        metrics.exact_best_observed_rank,
        metrics.exact_top4_max_observed_rank,
        window_1_contains_exact_best,
        window_2_contains_exact_best,
        window_4_contains_exact_best,
        window_8_contains_exact_best,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_grouped_scan_windowed_rows(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    window_size: i32,
) -> Vec<DebugGroupedScanWindowedRow> {
    // SAFETY: The comparison-row helper owns its scan descriptor and returns
    // materialized debug rows before cleanup.
    let rows = unsafe { debug_grouped_scan_comparison_rows(index_oid, query) };
    let window_size = debug_grouped_window_size(window_size);
    debug_grouped_scan_windowed_rows_from_comparison_rows(&rows, window_size)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_grouped_scan_windowed_summary(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
    window_size: i32,
) -> DebugGroupedScanWindowedSummary {
    // SAFETY: The debug wrapper forwards the caller-provided index oid to the
    // grouped-storage classifier, which opens and reads index metadata.
    let grouped_results = unsafe { debug_scan_uses_grouped_storage(index_oid) };
    // SAFETY: The comparison-row helper owns its scan descriptor and returns
    // materialized debug rows before cleanup.
    let rows = unsafe { debug_grouped_scan_comparison_rows(index_oid, query) };
    let window_size = debug_grouped_window_size(window_size);
    let emitted_result_count =
        i32::try_from(rows.len()).expect("debug grouped window summary count should fit in i32");
    if !grouped_results {
        return (
            emitted_result_count,
            0,
            0,
            i32::try_from(window_size).expect("grouped debug window size should fit in i32"),
            None,
            None,
            None,
            None,
            0.0,
            0.0,
            0,
            0,
            0.0,
            0.0,
        );
    }

    let grouped_result_count = emitted_result_count;
    let baseline_metrics = debug_grouped_rank_metrics(rows.iter().map(
        |(
            _heap_tid,
            approx_rank,
            _approx_score,
            _comparison_score,
            exact_rank,
            exact_rank_shift,
        )| { (*approx_rank, *exact_rank, *exact_rank_shift) },
    ));
    let windowed_rows = debug_grouped_scan_windowed_rows_from_comparison_rows(&rows, window_size);
    let windowed_metrics = debug_grouped_rank_metrics(windowed_rows.iter().map(
        |(
            _heap_tid,
            _approx_rank,
            windowed_rank,
            _approx_score,
            _comparison_score,
            exact_rank,
            _exact_rank_shift,
            windowed_rank_shift,
        )| (*windowed_rank, *exact_rank, *windowed_rank_shift),
    ));

    (
        emitted_result_count,
        grouped_result_count,
        baseline_metrics.compared_result_count,
        i32::try_from(window_size).expect("grouped debug window size should fit in i32"),
        baseline_metrics.exact_best_observed_rank,
        windowed_metrics.exact_best_observed_rank,
        baseline_metrics.exact_top4_max_observed_rank,
        windowed_metrics.exact_top4_max_observed_rank,
        baseline_metrics.mean_abs_rank_shift,
        windowed_metrics.mean_abs_rank_shift,
        baseline_metrics.max_abs_rank_shift,
        windowed_metrics.max_abs_rank_shift,
        baseline_metrics.spearman_rank_correlation,
        windowed_metrics.spearman_rank_correlation,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_exhaustion_state(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (Vec<HeapTidCoords>, bool, bool) {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_exhaustion_state");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this exhaustion probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    let mut tids = Vec::new();
    // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple calls
    // may advance the live scan descriptor.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {
        // SAFETY: A successful gettuple call populated `xs_heaptid` for this
        // live index scan descriptor.
        tids.push(pgrx::itemptr::item_pointer_get_both(unsafe {
            (*scan).xs_heaptid
        }));
    }

    // SAFETY: The scan descriptor remains live after exhaustion for this debug
    // probe's first post-exhaustion gettuple call.
    let exhausted_once = debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection);
    // SAFETY: The scan descriptor remains live after the first exhaustion probe
    // for this second idempotence check.
    let exhausted_twice = debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection);

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
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
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_current_result_state");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this current-result probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let before_found = active_result_state_ref(opaque).current().has_element();
    let before_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().element_tid());
    let before_score = active_result_state_ref(opaque).current().score_valid();
    let before_score_value = active_result_state_ref(opaque).current().score();

    // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may advance the
    // live descriptor and update current-result state.
    let found = debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection);
    // SAFETY: The scan descriptor remains live after gettuple and still owns its
    // HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let after_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().element_tid());
    let after_score = active_result_state_ref(opaque).current().score_valid();
    let after_score_value = active_result_state_ref(opaque).current().score();

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
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
pub(crate) unsafe fn debug_gettuple_orderby_score(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (bool, bool, f32) {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_orderby_score");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this order-by score probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may advance the
    // live descriptor and publish order-by score slots.
    let found = debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection);
    // SAFETY: `scan` is a live descriptor; null is checked before reading the
    // order-by null flag.
    let is_null = if unsafe { (*scan).xs_orderbynulls.is_null() } {
        true
    } else {
        // SAFETY: The order-by nulls pointer was checked non-null above.
        unsafe { *(*scan).xs_orderbynulls }
    };
    // SAFETY: `scan` is a live descriptor; null is checked before reading the
    // order-by datum slot.
    let score = if unsafe { (*scan).xs_orderbyvals.is_null() } {
        0.0
    } else {
        // SAFETY: The order-by datum pointer was checked non-null above, and the
        // HNSW AM publishes f32 scores in this debug path.
        f32::from_datum(unsafe { *(*scan).xs_orderbyvals }, is_null)
            .expect("orderby score should decode")
    };

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    (found, is_null, score)
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_scan_orderby_score(scan: pg_sys::IndexScanDesc) -> Option<f32> {
    // SAFETY: Callers pass a live scan descriptor; both order-by pointers are
    // checked before dereference, and the AM publishes f32 order-by datums.
    unsafe {
        if (*scan).xs_orderbyvals.is_null() || (*scan).xs_orderbynulls.is_null() {
            return None;
        }
        if *(*scan).xs_orderbynulls {
            return None;
        }

        f32::from_datum(*(*scan).xs_orderbyvals, false)
    }
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_current_result_comparison_score(scan: pg_sys::IndexScanDesc) -> Option<f32> {
    // SAFETY: Callers pass a live HNSW scan descriptor whose opaque was
    // initialized by AM rescan.
    let opaque = debug_scan_opaque(scan);
    opaque
        .last_emitted_comparison_score_valid
        .then_some(opaque.last_emitted_comparison_score)
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_current_result_approx_score(scan: pg_sys::IndexScanDesc) -> Option<f32> {
    // SAFETY: Callers pass a live HNSW scan descriptor whose opaque was
    // initialized by AM rescan.
    let opaque = debug_scan_opaque(scan);
    opaque
        .last_emitted_approx_score_valid
        .then_some(opaque.last_emitted_approx_score)
}

#[cfg(any(test, feature = "pg_test"))]
fn debug_current_result_approx_rank(scan: pg_sys::IndexScanDesc) -> Option<i32> {
    // SAFETY: Callers pass a live HNSW scan descriptor whose opaque was
    // initialized by AM rescan.
    let opaque = debug_scan_opaque(scan);
    opaque
        .last_emitted_approx_rank_valid
        .then_some(opaque.last_emitted_approx_rank)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_orderby_score_lifecycle(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_orderby_score_lifecycle");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this order-by lifecycle probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    let before = debug_scan_orderby_score(scan);

    // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may advance the
    // live descriptor and publish an order-by score.
    debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection);
    let after_first = debug_scan_orderby_score(scan);

    // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple calls
    // may advance the live descriptor until exhaustion.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {}
    let exhausted = debug_scan_orderby_score(scan);

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` remains live after exhaustion, and `rescan_orderby` is a
    // valid one-key buffer for the lifecycle rescan.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1);
    let rescanned = debug_scan_orderby_score(scan);

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    (before, after_first, exhausted, rescanned)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_entry_candidate_state(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (bool, HeapTidCoords, f32, bool, HeapTidCoords, f32) {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_rescan_entry_candidate_state");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this candidate-state probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let current = active_result_state_ref(opaque).current();
    let (before_valid, before_tid, before_score) = if current.has_element() {
        (
            true,
            debug_item_pointer_coords(current.element_tid()),
            current.score(),
        )
    } else {
        debug_candidate_slot(visible_frontier_slot(opaque, 0))
    };

    // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple calls
    // may advance the live descriptor until exhaustion.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {}

    // SAFETY: The scan descriptor remains live after exhaustion and still owns
    // its HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let (after_valid, after_tid, after_score) =
        debug_candidate_slot(visible_frontier_slot(opaque, 0));

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
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
    HeapTidCoords,
    f32,
) {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_rescan_successor_candidate_state");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this successor-state probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: The relation guard keeps the index relation open while its
    // metadata page is read.
    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
    let entry_tid = (
        metadata.entry_point.block_number,
        metadata.entry_point.offset_number,
    );
    // SAFETY: The debug helper opens the same index and materializes entry
    // point neighbor TIDs before returning.
    let entry_neighbors = unsafe { super::debug_entry_point_neighbor_tids(index_oid) };

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let successor_slot = debug_runtime_ordered_provenance_slots(opaque)
        .get(1)
        .copied()
        .unwrap_or((false, (u32::MAX, u16::MAX), (u32::MAX, u16::MAX), 0.0));

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    (
        entry_tid,
        entry_neighbors,
        successor_slot.0,
        successor_slot.1,
        successor_slot.2,
        successor_slot.3,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_rescan_candidate_frontier(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugBootstrapSeedState {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_rescan_candidate_frontier");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this frontier probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque_mut(scan);
    let frontier_slots = debug_runtime_ordered_slots(opaque);
    let frontier = frontier_slots.clone();
    let frontier_provenance = debug_runtime_ordered_provenance_slots(opaque);
    let expanded_sources = debug_sorted_expanded_source_tids(opaque);
    let head = debug_runtime_ordered_head(opaque);

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
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
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_consumes_bootstrap_candidate");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this bootstrap-consume probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque_mut(scan);
    let before_head = debug_runtime_ordered_head(opaque);
    let before_slots = debug_runtime_ordered_slots(opaque);
    let current_result_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().element_tid());

    assert!(
        // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may
        // advance the live descriptor and consume the bootstrap candidate.
        debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection),
        "bootstrap-consume helper requires a first tuple"
    );

    // SAFETY: The scan descriptor remains live after gettuple and still owns its
    // HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque_mut(scan);
    let after_head = debug_runtime_ordered_head(opaque);
    let after_slots = debug_runtime_ordered_slots(opaque);

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    (
        before_head,
        before_slots,
        current_result_tid,
        after_head,
        after_slots,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_materialize_bootstrap_candidate_result(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugBootstrapCandidateMaterializationState {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_materialize_bootstrap_candidate_result");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this materialization probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque_mut(scan);
    let current = active_result_state_ref(opaque).current();
    let candidate_before = if current.has_element() {
        (
            true,
            debug_item_pointer_coords(current.element_tid()),
            current.score(),
        )
    } else {
        let candidate = current_candidate_frontier_head(opaque);
        (
            candidate.is_some(),
            candidate
                .map(|candidate| debug_item_pointer_coords(candidate.node))
                .unwrap_or((u32::MAX, u16::MAX)),
            candidate.map(|candidate| candidate.score).unwrap_or(0.0),
        )
    };
    let materialized = current.has_element()
        // SAFETY: `opaque` belongs to the live scan and `index_relation` is held
        // open by the guard while prefetch materializes the next graph result.
        || unsafe { prefetch_next_graph_traversal_result(index_relation, opaque) };
    let current_result_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().element_tid());
    let pending_heap_tids = active_result_state_ref(opaque)
        .pending_heap_tids()
        .iter()
        .map(|tid| (tid.block_number, tid.offset_number))
        .collect::<Vec<_>>();

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    (
        candidate_before,
        current_result_tid,
        pending_heap_tids,
        materialized,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_bootstrap_phase_transition(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugBootstrapPhaseTransition {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_bootstrap_phase_transition");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this phase-transition probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque_mut(scan);
    let before_complete = !opaque.execution_phase.is_graph_traversal();

    while opaque.execution_phase.is_graph_traversal()
        // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple
        // calls may advance the live descriptor through graph traversal.
        && debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection)
    {}

    if opaque.execution_phase.is_graph_traversal() {
        // SAFETY: The descriptor remains live and this final gettuple probes the
        // transition out of graph traversal.
        let _ = debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection);
    }

    // SAFETY: The scan descriptor remains live after traversal and still owns
    // its HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque_mut(scan);
    let after_complete = !opaque.execution_phase.is_graph_traversal();
    let after_head = current_candidate_frontier_head(opaque)
        .map(|candidate| debug_item_pointer_coords(candidate.node));
    let after_frontier = debug_candidate_frontier_slots(opaque);

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` remains live after traversal, and `rescan_orderby` is a
    // valid one-key buffer for this reset probe.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1);

    // SAFETY: AM rescan refreshed the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque_mut(scan);
    let rescanned_complete = !opaque.execution_phase.is_graph_traversal();

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    (
        before_complete,
        after_complete,
        after_head,
        after_frontier,
        rescanned_complete,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_candidate_frontier_head_lifecycle(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> DebugCandidateFrontierLifecycle {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_candidate_frontier_head_lifecycle");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this frontier lifecycle probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque_mut(scan);
    let before_head = debug_runtime_ordered_head(opaque);
    let before_frontier = debug_runtime_ordered_slots(opaque);

    assert!(
        // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may
        // advance the live descriptor for this partial lifecycle sample.
        debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection),
        "frontier-head lifecycle helper requires a first tuple"
    );
    // SAFETY: The scan descriptor remains live after gettuple and still owns its
    // HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque_mut(scan);
    let partial_head = debug_runtime_ordered_head(opaque);
    let partial_frontier = debug_runtime_ordered_slots(opaque);

    // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple calls
    // may advance the live descriptor until exhaustion.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {}

    // SAFETY: The scan descriptor remains live after exhaustion and still owns
    // its HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque_mut(scan);
    let exhausted_head = current_candidate_frontier_head(opaque)
        .map(|candidate| debug_item_pointer_coords(candidate.node));
    let exhausted_frontier = debug_candidate_frontier_slots(opaque);

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
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
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_consume_candidate_frontier_head");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this frontier consume probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque_mut(scan);
    let before_head = current_candidate_frontier_head(opaque)
        .map(|candidate| debug_item_pointer_coords(candidate.node));
    let before_frontier = debug_candidate_frontier_slots(opaque);

    // SAFETY: `opaque` belongs to the live scan and `index_relation` is held
    // open by the guard while the bootstrap frontier is consumed/refilled.
    let first_consumed = unsafe { consume_and_refill_bootstrap_frontier(index_relation, opaque) };
    debug_assert_eq!(first_consumed.is_some(), before_head.is_some());
    let after_first_head = current_candidate_frontier_head(opaque)
        .map(|candidate| debug_item_pointer_coords(candidate.node));
    let after_first_frontier = debug_candidate_frontier_slots(opaque);

    // SAFETY: `opaque` still belongs to the live scan and `index_relation` is
    // held open while the second frontier consumption is probed.
    unsafe { consume_and_refill_bootstrap_frontier(index_relation, opaque) };
    let after_second_head = current_candidate_frontier_head(opaque)
        .map(|candidate| debug_item_pointer_coords(candidate.node));
    let after_second_frontier = debug_candidate_frontier_slots(opaque);

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
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
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_consume_candidate_frontier_head_slots");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this frontier slot consume probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque_mut(scan);
    let before_head = current_candidate_frontier_head(opaque)
        .map(|candidate| debug_item_pointer_coords(candidate.node));
    let before_slots = debug_candidate_frontier_slots(opaque);
    // SAFETY: `opaque` belongs to the live scan and `index_relation` is held
    // open by the guard while the bootstrap frontier is consumed/refilled.
    let consumed = unsafe { consume_and_refill_bootstrap_frontier(index_relation, opaque) };
    let consumed_tid = consumed
        .map(|candidate| (candidate.node.block_number, candidate.node.offset_number))
        .unwrap_or((u32::MAX, u16::MAX));
    let consumed_neighbors = consumed
        .map(|candidate| {
            // SAFETY: The consumed candidate came from the scan's graph
            // frontier; adjacency loading validates the graph tuple body.
            let (_, neighbors) = unsafe {
                graph::load_exact_graph_adjacency(
                    index_relation,
                    candidate.node,
                    opaque.scan_graph_storage,
                )
            };
            neighbors
                .tids
                .into_iter()
                .map(|tid| (tid.block_number, tid.offset_number))
                .filter(|tid| *tid != (u32::MAX, u16::MAX))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let after_head = current_candidate_frontier_head(opaque)
        .map(|candidate| debug_item_pointer_coords(candidate.node));
    let after_slots = debug_candidate_frontier_slots(opaque);
    let after_provenance_slots = debug_candidate_frontier_provenance_slots(opaque);

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    (
        before_head,
        before_slots,
        consumed_tid,
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
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_visited_seed_lifecycle");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this visited-set lifecycle probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let before = debug_sorted_visited_tids(opaque);

    assert!(
        // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may
        // advance the live descriptor for this partial lifecycle sample.
        debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection),
        "visited-seed lifecycle helper requires a first tuple"
    );
    // SAFETY: The scan descriptor remains live after gettuple and still owns its
    // HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let partial = debug_sorted_visited_tids(opaque);

    // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple calls
    // may advance the live descriptor until exhaustion.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {}
    // SAFETY: The scan descriptor remains live after exhaustion and still owns
    // its HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let exhausted = debug_sorted_visited_tids(opaque);

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
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
    bool,
    HeapTidCoords,
    f32,
) {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_entry_candidate_lifecycle");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this entry-candidate lifecycle probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let current = active_result_state_ref(opaque).current();
    let (before_valid, before_tid, before_score) = if current.has_element() {
        (
            true,
            debug_item_pointer_coords(current.element_tid()),
            current.score(),
        )
    } else {
        debug_candidate_slot(visible_frontier_slot(opaque, 0))
    };

    assert!(
        // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may
        // advance the live descriptor for this partial lifecycle sample.
        debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection),
        "entry-candidate lifecycle helper requires a first tuple"
    );
    // SAFETY: The scan descriptor remains live after gettuple and still owns its
    // HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let (partial_valid, partial_tid, partial_score) =
        debug_candidate_slot(visible_frontier_slot(opaque, 0));
    let partial_result_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().element_tid());
    let partial_exhausted = opaque.execution_phase.is_exhausted();

    // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple calls
    // may advance the live descriptor until exhaustion.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {}

    // SAFETY: The scan descriptor remains live after exhaustion and still owns
    // its HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let (exhausted_valid, exhausted_tid, exhausted_score) =
        debug_candidate_slot(visible_frontier_slot(opaque, 0));

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    (
        before_valid,
        before_tid,
        before_score,
        partial_valid,
        partial_tid,
        partial_score,
        partial_result_tid,
        partial_exhausted,
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
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_current_result_lifecycle");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this current-result lifecycle probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    assert!(
        // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may
        // advance the live descriptor for the first lifecycle sample.
        debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection),
        "first tuple production should succeed for lifecycle debug helper"
    );
    // SAFETY: The scan descriptor remains live after gettuple and still owns its
    // HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let first_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().element_tid());

    assert!(
        // SAFETY: The live descriptor may be advanced again to sample the second
        // current-result state.
        debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection),
        "second tuple production should succeed for duplicate-drain lifecycle debug helper"
    );
    // SAFETY: The scan descriptor remains live after the second gettuple and
    // still owns its HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let second_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().element_tid());
    let second_score = active_result_state_ref(opaque).current().score_valid();
    let second_score_value = active_result_state_ref(opaque).current().score();

    // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple calls
    // may advance the live descriptor until exhaustion.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {}

    // SAFETY: The scan descriptor remains live after exhaustion and still owns
    // its HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let exhausted_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().element_tid());
    let exhausted_score = active_result_state_ref(opaque).current().score_valid();
    let exhausted_score_value = active_result_state_ref(opaque).current().score();

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` remains live after exhaustion, and `rescan_orderby` is a
    // valid one-key buffer for this lifecycle rescan.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1);

    // SAFETY: AM rescan refreshed the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let rescanned_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().element_tid());
    let rescanned_score = active_result_state_ref(opaque).current().score_valid();

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
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
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_current_result_neighbors");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this neighbor probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);
    // SAFETY: AM rescan initialized the HNSW scan opaque on the live descriptor.
    let opaque = debug_scan_opaque(scan);
    let prefetched_tid = active_result_state_ref(opaque).current().element_tid();
    assert!(
        // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may
        // advance the live descriptor for the neighbor sample.
        debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection),
        "neighbor debug helper requires a non-empty scan result"
    );

    // SAFETY: The scan descriptor remains live after gettuple and still owns its
    // HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let current_result_tid = if active_result_state_ref(opaque).current().has_element() {
        active_result_state_ref(opaque).current().element_tid()
    } else {
        prefetched_tid
    };
    // SAFETY: `current_result_tid` was produced by the live scan, and adjacency
    // loading validates the graph tuple body.
    let (_element, neighbors) = unsafe {
        graph::load_exact_graph_adjacency(
            index_relation,
            current_result_tid,
            opaque.scan_graph_storage,
        )
    };

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    (
        debug_item_pointer_coords(current_result_tid),
        neighbors.tids.len(),
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_current_result_heap_progress(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (
    HeapTidCoords,
    HeapTidCoords,
    HeapTidCoords,
    HeapTidCoords,
    f32,
    f32,
) {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_current_result_heap_progress");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this heap-progress probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    assert!(
        // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may
        // advance the live descriptor for the first heap-progress sample.
        debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection),
        "heap-progress debug helper requires a first tuple"
    );
    // SAFETY: The scan descriptor remains live after gettuple and still owns its
    // HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let first_heap_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().heap_tid());
    let element_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().element_tid());
    let first_score = active_result_state_ref(opaque).current().score();

    assert!(
        // SAFETY: The live descriptor may be advanced again to sample duplicate
        // heap progress for the same graph element.
        debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection),
        "heap-progress debug helper requires a duplicate tuple"
    );
    // SAFETY: The scan descriptor remains live after the second gettuple and
    // still owns its HNSW opaque for debug inspection.
    let opaque = debug_scan_opaque(scan);
    let second_heap_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().heap_tid());
    let second_element_tid =
        debug_item_pointer_coords(active_result_state_ref(opaque).current().element_tid());
    let second_score = active_result_state_ref(opaque).current().score();

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    (
        element_tid,
        first_heap_tid,
        second_element_tid,
        second_heap_tid,
        first_score,
        second_score,
    )
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_backward_after_rescan(index_oid: pg_sys::Oid, query: Vec<f32>) {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_backward_after_rescan");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this backward-direction error probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);
    // SAFETY: The debug probe deliberately invokes backward gettuple on a live
    // HNSW descriptor after rescan to exercise that error path.
    debug_am_gettuple(scan, pg_sys::ScanDirection::BackwardScanDirection);
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_rescan_after_exhaustion(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (Vec<HeapTidCoords>, Vec<HeapTidCoords>) {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_rescan_after_exhaustion");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this exhaustion-rescan probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    let mut first_pass = Vec::new();
    // SAFETY: AM rescan initialized the HNSW opaque, so repeated gettuple calls
    // may advance the live descriptor until exhaustion.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {
        // SAFETY: A successful gettuple call populated `xs_heaptid` for this
        // live index scan descriptor.
        first_pass.push(pgrx::itemptr::item_pointer_get_both(unsafe {
            (*scan).xs_heaptid
        }));
    }

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` remains live after exhaustion, and `rescan_orderby` is a
    // valid one-key buffer for the second pass.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1);

    let mut rescanned = Vec::new();
    // SAFETY: The second AM rescan reinitialized the HNSW opaque, so repeated
    // gettuple calls may advance the live descriptor.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {
        // SAFETY: A successful gettuple call populated `xs_heaptid` for this
        // live index scan descriptor.
        rescanned.push(pgrx::itemptr::item_pointer_get_both(unsafe {
            (*scan).xs_heaptid
        }));
    }

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    (first_pass, rescanned)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_gettuple_rescan_after_partial(
    index_oid: pg_sys::Oid,
    query: Vec<f32>,
) -> (HeapTidCoords, Vec<HeapTidCoords>) {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_gettuple_rescan_after_partial");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index open for the AM scan
    // descriptor used by this partial-rescan probe.
    let scan = debug_am_begin_scan(index_relation, 0, 1);

    let query_datum = pgrx::IntoDatum::into_datum(query).expect("query should convert to datum");
    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` is live, there are no index quals, and `orderby` is a
    // valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut orderby, 1);

    // SAFETY: AM rescan initialized the HNSW opaque, so gettuple may advance the
    // live descriptor for the partial first pass.
    let found_first = debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection);
    assert!(
        found_first,
        "partial scan should yield at least one heap tid"
    );
    // SAFETY: The successful gettuple call populated `xs_heaptid` for this live
    // index scan descriptor.
    let first_tid = debug_scan_heap_tid(scan);

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    // SAFETY: `scan` remains live after the partial first pass, and
    // `rescan_orderby` is a valid one-key buffer.
    debug_am_rescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1);

    let mut tids = Vec::new();
    // SAFETY: The second AM rescan reinitialized the HNSW opaque, so repeated
    // gettuple calls may advance the live descriptor.
    while debug_am_gettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) {
        // SAFETY: A successful gettuple call populated `xs_heaptid` for this
        // live index scan descriptor.
        tids.push(pgrx::itemptr::item_pointer_get_both(unsafe {
            (*scan).xs_heaptid
        }));
    }

    // SAFETY: The scan descriptor is live and belongs to the HNSW AM.
    debug_am_end_scan(scan);
    // SAFETY: AM cleanup has run, and the descriptor is released once here.
    debug_index_scan_end(scan);
    (first_tid, tids)
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) unsafe fn debug_entry_point_neighbor_tids(index_oid: pg_sys::Oid) -> Vec<HeapTidCoords> {
    let index_relation_guard =
        IndexRelationGuard::access_share(index_oid, "debug_entry_point_neighbor_tids");
    let index_relation = index_relation_guard.as_ptr();
    // SAFETY: The relation guard keeps the index relation open while its
    // metadata page is read.
    let metadata = unsafe { super::shared::read_metadata_page(index_relation) };
    if metadata.entry_point == page::ItemPointer::INVALID || metadata.dimensions == 0 {
        return Vec::new();
    }

    // SAFETY: Metadata was read from the open index relation and validated
    // enough to resolve the graph storage descriptor for debug inspection.
    let storage = unsafe { debug_graph_storage(index_relation, &metadata) };
    // SAFETY: The metadata entry point is valid, and adjacency loading validates
    // the graph tuple body before returning neighbors.
    let (_element, neighbors) =
        unsafe { graph::load_exact_graph_adjacency(index_relation, metadata.entry_point, storage) };
    neighbors
        .tids
        .into_iter()
        .filter(|tid| *tid != page::ItemPointer::INVALID)
        .map(|tid| (tid.block_number, tid.offset_number))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comparison_row(
        block_number: u32,
        offset_number: u16,
        approx_rank: i32,
        approx_score: f32,
        comparison_score: Option<f32>,
    ) -> DebugGroupedScanComparisonRow {
        (
            (block_number, offset_number),
            approx_rank,
            approx_score,
            comparison_score,
            None,
            None,
        )
    }

    #[test]
    fn grouped_window_simulation_is_noop_for_window_one() {
        let rows = vec![
            comparison_row(1, 1, 1, -4.0, Some(-3.5)),
            comparison_row(1, 2, 2, -3.0, Some(-2.5)),
            comparison_row(1, 3, 3, -2.0, Some(-1.5)),
        ];

        let observed = debug_grouped_scan_windowed_rows_from_comparison_rows(&rows, 1);

        assert_eq!(observed.len(), rows.len());
        for (
            idx,
            (heap_tid, approx_rank, windowed_rank, approx_score, comparison_score, _, _, _),
        ) in observed.iter().enumerate()
        {
            assert_eq!(*heap_tid, rows[idx].0);
            assert_eq!(*approx_rank, rows[idx].1);
            assert_eq!(*windowed_rank, rows[idx].1);
            assert_eq!(*approx_score, rows[idx].2);
            assert_eq!(*comparison_score, rows[idx].3);
        }
    }

    #[test]
    fn grouped_window_simulation_keeps_approx_order_for_tied_exact_scores() {
        let rows = vec![
            comparison_row(2, 1, 1, -4.0, Some(-2.0)),
            comparison_row(2, 2, 2, -3.0, Some(-2.0)),
            comparison_row(2, 3, 3, -2.0, Some(-1.5)),
        ];

        let observed = debug_grouped_scan_windowed_rows_from_comparison_rows(&rows, 3);
        let observed_approx_ranks = observed
            .iter()
            .map(
                |(
                    _heap_tid,
                    approx_rank,
                    _windowed_rank,
                    _approx_score,
                    _comparison_score,
                    _,
                    _,
                    _,
                )| { *approx_rank },
            )
            .collect::<Vec<_>>();

        assert_eq!(
            observed_approx_ranks,
            vec![1, 2, 3],
            "exact-score ties should preserve the original approximate rank order"
        );
    }

    #[test]
    fn grouped_window_simulation_handles_windows_at_or_beyond_row_count() {
        let rows = vec![
            comparison_row(3, 1, 1, -4.0, Some(-1.0)),
            comparison_row(3, 2, 2, -3.0, Some(-3.0)),
            comparison_row(3, 3, 3, -2.0, Some(-2.0)),
        ];

        let exact_window = debug_grouped_scan_windowed_rows_from_comparison_rows(&rows, rows.len());
        let oversized_window =
            debug_grouped_scan_windowed_rows_from_comparison_rows(&rows, rows.len() + 5);

        assert_eq!(oversized_window, exact_window);
    }
}
