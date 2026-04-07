use std::ptr;

use pgrx::pg_sys;

use super::{page, search};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GraphElement {
    pub tid: page::ItemPointer,
    pub level: u8,
    pub deleted: bool,
    pub heaptids: Vec<page::ItemPointer>,
    pub gamma: f32,
    pub neighbortid: page::ItemPointer,
    pub code: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GraphNeighbors {
    pub tid: page::ItemPointer,
    pub count: usize,
    pub tids: Vec<page::ItemPointer>,
}

pub(crate) unsafe fn load_graph_element(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    code_len: usize,
) -> GraphElement {
    let tuple_bytes = unsafe { read_page_tuple_bytes(index_relation, element_tid, "element") };
    let element = page::TqElementTuple::decode(&tuple_bytes, code_len)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode graph element tuple: {e}"));
    GraphElement {
        tid: element_tid,
        level: element.level,
        deleted: element.deleted,
        heaptids: element.heaptids,
        gamma: element.gamma,
        neighbortid: element.neighbortid,
        code: element.code,
    }
}

pub(crate) unsafe fn load_graph_neighbors(
    index_relation: pg_sys::Relation,
    neighbor_tid: page::ItemPointer,
) -> GraphNeighbors {
    if neighbor_tid == page::ItemPointer::INVALID {
        return GraphNeighbors {
            tid: neighbor_tid,
            count: 0,
            tids: Vec::new(),
        };
    }

    let tuple_bytes = unsafe { read_page_tuple_bytes(index_relation, neighbor_tid, "neighbor") };
    let neighbor = page::TqNeighborTuple::decode(&tuple_bytes)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode graph neighbor tuple: {e}"));
    let count = neighbor.count as usize;
    if count > neighbor.tids.len() {
        pgrx::error!(
            "tqhnsw neighbor tuple count {} exceeds payload tid count {}",
            neighbor.count,
            neighbor.tids.len()
        );
    }
    GraphNeighbors {
        tid: neighbor_tid,
        count,
        tids: neighbor.tids[..count].to_vec(),
    }
}

pub(crate) unsafe fn load_graph_adjacency(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    code_len: usize,
) -> (GraphElement, GraphNeighbors) {
    let element = unsafe { load_graph_element(index_relation, element_tid, code_len) };
    let neighbors = unsafe { load_graph_neighbors(index_relation, element.neighbortid) };
    (element, neighbors)
}

pub(crate) unsafe fn load_layer0_neighbor_tids(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    code_len: usize,
) -> Vec<page::ItemPointer> {
    let (_, neighbors) = unsafe { load_graph_adjacency(index_relation, element_tid, code_len) };
    valid_layer0_neighbor_tids(&neighbors.tids)
}

pub(crate) unsafe fn load_layer0_successor_candidates<KeepFn, ScoreFn>(
    index_relation: pg_sys::Relation,
    source_tid: page::ItemPointer,
    code_len: usize,
    mut keep_neighbor_tid: KeepFn,
    mut score_candidate: ScoreFn,
) -> Vec<search::BeamCandidate<page::ItemPointer>>
where
    KeepFn: FnMut(page::ItemPointer) -> bool,
    ScoreFn: FnMut(&GraphElement) -> Option<f32>,
{
    let neighbor_tids = unsafe { load_layer0_neighbor_tids(index_relation, source_tid, code_len) };
    layer0_successor_candidates_from_elements(
        source_tid,
        neighbor_tids
            .into_iter()
            .filter(|neighbor_tid| keep_neighbor_tid(*neighbor_tid))
            .map(|neighbor_tid| unsafe { load_graph_element(index_relation, neighbor_tid, code_len) }),
        |neighbor| score_candidate(neighbor),
    )
}

pub(crate) unsafe fn run_layer0_beam_search<SeedIter, KeepFn, ScoreFn>(
    index_relation: pg_sys::Relation,
    code_len: usize,
    ef_search: usize,
    seeds: SeedIter,
    mut keep_neighbor_tid: KeepFn,
    mut score_candidate: ScoreFn,
) -> search::BeamTrace<page::ItemPointer>
where
    SeedIter: IntoIterator<Item = search::BeamCandidate<page::ItemPointer>>,
    KeepFn: FnMut(page::ItemPointer) -> bool,
    ScoreFn: FnMut(&GraphElement) -> Option<f32>,
{
    run_layer0_beam_search_with_successors(ef_search, seeds, |source_tid| unsafe {
        load_layer0_successor_candidates(
            index_relation,
            source_tid,
            code_len,
            &mut keep_neighbor_tid,
            &mut score_candidate,
        )
    })
}

fn valid_layer0_neighbor_tids(neighbor_tids: &[page::ItemPointer]) -> Vec<page::ItemPointer> {
    neighbor_tids
        .iter()
        .copied()
        .filter(|tid| *tid != page::ItemPointer::INVALID)
        .collect()
}

fn run_layer0_beam_search_with_successors<SeedIter, SuccessorFn>(
    ef_search: usize,
    seeds: SeedIter,
    mut successors: SuccessorFn,
) -> search::BeamTrace<page::ItemPointer>
where
    SeedIter: IntoIterator<Item = search::BeamCandidate<page::ItemPointer>>,
    SuccessorFn: FnMut(page::ItemPointer) -> Vec<search::BeamCandidate<page::ItemPointer>>,
{
    let mut search = search::BeamSearch::new(ef_search);
    search.seed_many(seeds);
    search.run(|candidate| successors(candidate.node))
}

fn layer0_successor_candidates_from_elements<I, F>(
    source_tid: page::ItemPointer,
    neighbors: I,
    mut score_candidate: F,
) -> Vec<search::BeamCandidate<page::ItemPointer>>
where
    I: IntoIterator<Item = GraphElement>,
    F: FnMut(&GraphElement) -> Option<f32>,
{
    neighbors
        .into_iter()
        .filter_map(|neighbor| {
            if neighbor.deleted || neighbor.heaptids.is_empty() {
                return None;
            }

            let score = score_candidate(&neighbor)?;
            Some(search::BeamCandidate::with_source(
                neighbor.tid,
                score,
                source_tid,
            ))
        })
        .collect()
}

unsafe fn read_page_tuple_bytes(
    index_relation: pg_sys::Relation,
    tuple_tid: page::ItemPointer,
    tuple_kind: &str,
) -> Vec<u8> {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            tuple_tid.block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let page_ptr = unsafe { pg_sys::BufferGetPage(buffer) }.cast::<u8>();
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let line_pointer_count = super::shared::page_line_pointer_count(page_ptr);
    if tuple_tid.offset_number == 0 || tuple_tid.offset_number > line_pointer_count {
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        pgrx::error!(
            "tqhnsw graph read found {tuple_kind} tuple offset {} out of range on block {}",
            tuple_tid.offset_number,
            tuple_tid.block_number
        );
    }

    let item_id = unsafe { &*super::shared::page_item_id(page_ptr, tuple_tid.offset_number) };
    if item_id.lp_flags() == 0 {
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        pgrx::error!("tqhnsw graph read found unused {tuple_kind} tuple slot");
    }

    let tuple_offset = item_id.lp_off() as usize;
    let tuple_len = item_id.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
        pgrx::error!(
            "tqhnsw found invalid {tuple_kind} tuple bounds on block {}",
            tuple_tid.block_number
        );
    }

    let tuple_bytes =
        unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) }.to_vec();
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    tuple_bytes
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

    #[test]
    fn valid_layer0_neighbor_tids_skips_invalid() {
        let neighbors = vec![
            page::ItemPointer::INVALID,
            tid(7, 1),
            tid(7, 2),
            page::ItemPointer::INVALID,
            tid(7, 3),
        ];

        assert_eq!(
            valid_layer0_neighbor_tids(&neighbors),
            vec![tid(7, 1), tid(7, 2), tid(7, 3)],
            "layer-0 neighbor loading should skip INVALID slots while preserving neighbor order",
        );
    }

    fn graph_element(
        tid: page::ItemPointer,
        deleted: bool,
        heaptids: Vec<page::ItemPointer>,
        gamma: f32,
    ) -> GraphElement {
        GraphElement {
            tid,
            level: 0,
            deleted,
            heaptids,
            gamma,
            neighbortid: page::ItemPointer::INVALID,
            code: Vec::new(),
        }
    }

    #[test]
    fn layer0_successor_candidates_from_elements_skips_unselectable_neighbors() {
        let source_tid = tid(5, 1);
        let keep_tid = tid(5, 2);
        let skip_deleted_tid = tid(5, 3);
        let skip_empty_tid = tid(5, 4);

        let candidates = layer0_successor_candidates_from_elements(
            source_tid,
            vec![
                graph_element(keep_tid, false, vec![tid(9, 1)], 0.25),
                graph_element(skip_deleted_tid, true, vec![tid(9, 2)], 0.5),
                graph_element(skip_empty_tid, false, Vec::new(), 0.75),
            ],
            |neighbor| Some(neighbor.gamma),
        );

        assert_eq!(
            candidates,
            vec![search::BeamCandidate::with_source(keep_tid, 0.25, source_tid)],
            "layer-0 successor loading should keep only live neighbors with heap tids",
        );
    }

    #[test]
    fn run_layer0_beam_search_with_successors_expands_best_first() {
        let seed_tid = tid(1, 1);
        let left_tid = tid(1, 2);
        let right_tid = tid(1, 3);
        let left_best_tid = tid(1, 4);
        let right_best_tid = tid(1, 5);

        let trace = run_layer0_beam_search_with_successors(
            4,
            [search::BeamCandidate::new(seed_tid, 0.9)],
            |source_tid| {
                if source_tid == seed_tid {
                    vec![
                        search::BeamCandidate::with_source(left_tid, 0.3, seed_tid),
                        search::BeamCandidate::with_source(right_tid, 0.1, seed_tid),
                    ]
                } else if source_tid == right_tid {
                    vec![search::BeamCandidate::with_source(right_best_tid, 0.05, right_tid)]
                } else if source_tid == left_tid {
                    vec![search::BeamCandidate::with_source(left_best_tid, 0.2, left_tid)]
                } else {
                    Vec::new()
                }
            },
        );

        assert_eq!(
            trace
                .expanded
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![seed_tid, right_tid, right_best_tid, left_tid],
            "layer-0 beam traversal should expand the best discovered successor first",
        );
        assert_eq!(
            trace
                .frontier
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![left_best_tid],
            "remaining frontier should preserve best-first order after the expansion budget",
        );
    }
}
