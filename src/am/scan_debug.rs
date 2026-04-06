#[cfg(any(test, feature = "pg_test"))]
use std::ptr;

#[cfg(any(test, feature = "pg_test"))]
use pgrx::pg_sys;

#[cfg(any(test, feature = "pg_test"))]
use super::scan::*;
#[cfg(any(test, feature = "pg_test"))]
use super::{graph, page, search};


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
    let before_found = opaque.current_result.has_element();
    let before_tid = debug_item_pointer_coords(opaque.current_result.element_tid());
    let before_score = opaque.current_result.score_valid();
    let before_score_value = opaque.current_result.score();

    let found = unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) };
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let after_tid = debug_item_pointer_coords(opaque.current_result.element_tid());
    let after_score = opaque.current_result.score_valid();
    let after_score_value = opaque.current_result.score();

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
    let (before_valid, before_tid, before_score) =
        debug_candidate_slot(visible_frontier_slot(opaque, 0));

    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {}

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let (after_valid, after_tid, after_score) =
        debug_candidate_slot(visible_frontier_slot(opaque, 0));

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
    let (successor_valid, successor_tid, successor_score) =
        debug_candidate_slot(visible_frontier_slot(opaque, 1));

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

    let opaque = unsafe { &mut *(*scan).opaque.cast::<TqScanOpaque>() };
    let frontier_slots = debug_candidate_frontier_slots(opaque);
    let frontier = frontier_slots.clone();
    let frontier_provenance = debug_candidate_frontier_provenance_slots(opaque);
    let expanded_sources = debug_sorted_expanded_source_tids(opaque);
    let head = current_candidate_frontier_head_tid(opaque)
        .map(|tid| (tid.block_number, tid.offset_number));

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

    let opaque = unsafe { &mut *(*scan).opaque.cast::<TqScanOpaque>() };
    let before_head = current_candidate_frontier_head_tid(opaque).map(debug_item_pointer_coords);
    let before_slots = debug_candidate_frontier_slots(opaque);

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "bootstrap-consume helper requires a first tuple"
    );

    let opaque = unsafe { &mut *(*scan).opaque.cast::<TqScanOpaque>() };
    let current_result_tid = debug_item_pointer_coords(opaque.current_result.element_tid());
    let after_head = current_candidate_frontier_head_tid(opaque).map(debug_item_pointer_coords);
    let after_slots = debug_candidate_frontier_slots(opaque);

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
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
    let index_relation =
        unsafe { pg_sys::index_open(index_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    let scan = unsafe { tqhnsw_ambeginscan(index_relation, 0, 1) };

    let mut orderby = pg_sys::ScanKeyData {
        sk_argument: pgrx::IntoDatum::into_datum(query).expect("query should convert to datum"),
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut orderby, 1) };

    let opaque = unsafe { &mut *(*scan).opaque.cast::<TqScanOpaque>() };
    let candidate = unsafe { consume_and_refill_bootstrap_frontier(index_relation, opaque) };
    let candidate_before = (
        candidate.is_some(),
        candidate
            .map(|candidate| debug_item_pointer_coords(candidate.node))
            .unwrap_or((u32::MAX, u16::MAX)),
        candidate.map(|candidate| candidate.score).unwrap_or(0.0),
    );
    let materialized = candidate.is_some_and(|candidate| unsafe {
        materialize_scan_candidate_result(index_relation, opaque, candidate)
    });
    let current_result_tid = debug_item_pointer_coords(opaque.current_result.element_tid());
    let pending_heap_tids = opaque.pending_heaptids[..opaque.pending_heaptid_count as usize]
        .iter()
        .map(|tid| (tid.block_number, tid.offset_number))
        .collect::<Vec<_>>();

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    (
        candidate_before,
        current_result_tid,
        pending_heap_tids,
        materialized,
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

    let opaque = unsafe { &mut *(*scan).opaque.cast::<TqScanOpaque>() };
    let before_head = current_candidate_frontier_head_tid(opaque)
        .map(|tid| (tid.block_number, tid.offset_number));
    let before_frontier = debug_candidate_frontier_slots(opaque);

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "frontier-head lifecycle helper requires a first tuple"
    );
    let opaque = unsafe { &mut *(*scan).opaque.cast::<TqScanOpaque>() };
    let partial_head = current_candidate_frontier_head_tid(opaque)
        .map(|tid| (tid.block_number, tid.offset_number));
    let partial_frontier = debug_candidate_frontier_slots(opaque);

    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {}

    let opaque = unsafe { &mut *(*scan).opaque.cast::<TqScanOpaque>() };
    let exhausted_head = current_candidate_frontier_head_tid(opaque)
        .map(|tid| (tid.block_number, tid.offset_number));
    let exhausted_frontier = debug_candidate_frontier_slots(opaque);

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
    let before_head = current_candidate_frontier_head_tid(opaque)
        .map(|tid| (tid.block_number, tid.offset_number));
    let before_frontier = debug_candidate_frontier_slots(opaque);

    let first_consumed = unsafe { consume_and_refill_bootstrap_frontier(index_relation, opaque) };
    debug_assert_eq!(first_consumed.is_some(), before_head.is_some());
    let after_first_head = current_candidate_frontier_head_tid(opaque)
        .map(|tid| (tid.block_number, tid.offset_number));
    let after_first_frontier = debug_candidate_frontier_slots(opaque);

    unsafe { consume_and_refill_bootstrap_frontier(index_relation, opaque) };
    let after_second_head = current_candidate_frontier_head_tid(opaque)
        .map(|tid| (tid.block_number, tid.offset_number));
    let after_second_frontier = debug_candidate_frontier_slots(opaque);

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
    let before_head = current_candidate_frontier_head_tid(opaque)
        .map(|tid| (tid.block_number, tid.offset_number));
    let before_slots = debug_candidate_frontier_slots(opaque);
    let consumed = unsafe { consume_and_refill_bootstrap_frontier(index_relation, opaque) };
    let consumed_tid = consumed
        .map(|candidate| {
            (
                candidate.node.block_number,
                candidate.node.offset_number,
            )
        })
        .unwrap_or((u32::MAX, u16::MAX));
    let consumed_neighbors = consumed
        .map(|candidate| {
            let (_, neighbors) = unsafe {
                graph::load_graph_adjacency(
                    index_relation,
                    candidate.node,
                    opaque.scan_code_len,
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

    let after_head = current_candidate_frontier_head_tid(opaque)
        .map(|tid| (tid.block_number, tid.offset_number));
    let after_slots = debug_candidate_frontier_slots(opaque);
    let after_provenance_slots = debug_candidate_frontier_provenance_slots(opaque);

    unsafe { tqhnsw_amendscan(scan) };
    unsafe { pg_sys::IndexScanEnd(scan) };
    unsafe { pg_sys::index_close(index_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
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
    let (before_valid, before_tid, before_score) =
        debug_candidate_slot(visible_frontier_slot(opaque, 0));

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "entry-candidate lifecycle helper requires a first tuple"
    );
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let (partial_valid, partial_tid, partial_score) =
        debug_candidate_slot(visible_frontier_slot(opaque, 0));
    let partial_result_tid = debug_item_pointer_coords(opaque.current_result.element_tid());

    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {}

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let (exhausted_valid, exhausted_tid, exhausted_score) =
        debug_candidate_slot(visible_frontier_slot(opaque, 0));

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
    let first_tid = debug_item_pointer_coords(opaque.current_result.element_tid());

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "second tuple production should succeed for duplicate-drain lifecycle debug helper"
    );
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let second_tid = debug_item_pointer_coords(opaque.current_result.element_tid());
    let second_score = opaque.current_result.score_valid();
    let second_score_value = opaque.current_result.score();

    while unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) } {}

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let exhausted_tid = debug_item_pointer_coords(opaque.current_result.element_tid());
    let exhausted_score = opaque.current_result.score_valid();
    let exhausted_score_value = opaque.current_result.score();

    let mut rescan_orderby = pg_sys::ScanKeyData {
        sk_argument: query_datum,
        ..Default::default()
    };
    unsafe { tqhnsw_amrescan(scan, ptr::null_mut(), 0, &mut rescan_orderby, 1) };

    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let rescanned_tid = debug_item_pointer_coords(opaque.current_result.element_tid());
    let rescanned_score = opaque.current_result.score_valid();

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
    let current_result_tid = debug_item_pointer_coords(opaque.current_result.element_tid());
    let (_element, neighbors) = unsafe {
        graph::load_graph_adjacency(
            index_relation,
            opaque.current_result.element_tid(),
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
    let first_heap_tid = debug_item_pointer_coords(opaque.current_result.heap_tid());
    let element_tid = debug_item_pointer_coords(opaque.current_result.element_tid());
    let first_score = opaque.current_result.score();

    assert!(
        unsafe { tqhnsw_amgettuple(scan, pg_sys::ScanDirection::ForwardScanDirection) },
        "heap-progress debug helper requires a duplicate tuple"
    );
    let opaque = unsafe { &*(*scan).opaque.cast::<TqScanOpaque>() };
    let second_heap_tid = debug_item_pointer_coords(opaque.current_result.heap_tid());
    let second_score = opaque.current_result.score();

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
