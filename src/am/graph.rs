use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashSet};
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
        tids: neighbor.tids,
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
    m: usize,
) -> Vec<page::ItemPointer> {
    let (element, neighbors) =
        unsafe { load_graph_adjacency(index_relation, element_tid, code_len) };
    valid_neighbor_tids_for_layer(&neighbors.tids, element.level, m, 0)
}

pub(crate) unsafe fn load_neighbor_tids_for_layer(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    code_len: usize,
    m: usize,
    layer: u8,
) -> Vec<page::ItemPointer> {
    let (element, neighbors) =
        unsafe { load_graph_adjacency(index_relation, element_tid, code_len) };
    valid_neighbor_tids_for_layer(&neighbors.tids, element.level, m, layer)
}

pub(crate) unsafe fn load_layer0_successor_candidates<KeepFn, ScoreFn>(
    index_relation: pg_sys::Relation,
    source_tid: page::ItemPointer,
    code_len: usize,
    m: usize,
    mut keep_neighbor_tid: KeepFn,
    mut score_candidate: ScoreFn,
) -> Vec<search::BeamCandidate<page::ItemPointer>>
where
    KeepFn: FnMut(page::ItemPointer) -> bool,
    ScoreFn: FnMut(&GraphElement) -> Option<f32>,
{
    let neighbor_tids =
        unsafe { load_layer0_neighbor_tids(index_relation, source_tid, code_len, m) };
    layer0_successor_candidates_from_elements(
        source_tid,
        neighbor_tids
            .into_iter()
            .filter(|neighbor_tid| keep_neighbor_tid(*neighbor_tid))
            .map(|neighbor_tid| unsafe {
                load_graph_element(index_relation, neighbor_tid, code_len)
            }),
        |neighbor| score_candidate(neighbor),
    )
}

pub(crate) unsafe fn greedy_descend_from_entry<ScoreFn>(
    index_relation: pg_sys::Relation,
    code_len: usize,
    m: usize,
    entry_candidate: search::BeamCandidate<page::ItemPointer>,
    mut score_candidate: ScoreFn,
) -> search::BeamCandidate<page::ItemPointer>
where
    ScoreFn: FnMut(&GraphElement) -> Option<f32>,
{
    let entry_element =
        unsafe { load_graph_element(index_relation, entry_candidate.node, code_len) };
    greedy_descend_with_successors(
        entry_candidate,
        entry_element.level,
        |source_tid, layer| unsafe {
            let neighbor_tids =
                load_neighbor_tids_for_layer(index_relation, source_tid, code_len, m, layer);
            layer0_successor_candidates_from_elements(
                source_tid,
                neighbor_tids
                    .into_iter()
                    .map(|neighbor_tid| load_graph_element(index_relation, neighbor_tid, code_len)),
                |neighbor| score_candidate(neighbor),
            )
        },
    )
}

pub(crate) unsafe fn run_layer0_beam_search<SeedIter, KeepFn, ScoreFn>(
    index_relation: pg_sys::Relation,
    code_len: usize,
    m: usize,
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
            m,
            &mut keep_neighbor_tid,
            &mut score_candidate,
        )
    })
}

pub(crate) unsafe fn search_layer0_result_candidates<SeedIter, KeepFn, ScoreFn>(
    index_relation: pg_sys::Relation,
    code_len: usize,
    m: usize,
    ef_search: usize,
    seeds: SeedIter,
    mut keep_neighbor_tid: KeepFn,
    mut score_candidate: ScoreFn,
) -> Vec<search::BeamCandidate<page::ItemPointer>>
where
    SeedIter: IntoIterator<Item = search::BeamCandidate<page::ItemPointer>>,
    KeepFn: FnMut(page::ItemPointer) -> bool,
    ScoreFn: FnMut(&GraphElement) -> Option<f32>,
{
    search_layer0_result_candidates_with_successors(ef_search, seeds, |source_tid| unsafe {
        load_layer0_successor_candidates(
            index_relation,
            source_tid,
            code_len,
            m,
            &mut keep_neighbor_tid,
            &mut score_candidate,
        )
    })
}

pub(crate) unsafe fn search_layer_result_candidates<SeedIter, KeepFn, ScoreFn>(
    index_relation: pg_sys::Relation,
    code_len: usize,
    m: usize,
    layer: u8,
    ef_search: usize,
    seeds: SeedIter,
    mut keep_neighbor_tid: KeepFn,
    mut score_candidate: ScoreFn,
) -> Vec<search::BeamCandidate<page::ItemPointer>>
where
    SeedIter: IntoIterator<Item = search::BeamCandidate<page::ItemPointer>>,
    KeepFn: FnMut(page::ItemPointer) -> bool,
    ScoreFn: FnMut(&GraphElement) -> Option<f32>,
{
    search_layer0_result_candidates_with_successors(ef_search, seeds, |source_tid| unsafe {
        let neighbor_tids =
            load_neighbor_tids_for_layer(index_relation, source_tid, code_len, m, layer);
        layer0_successor_candidates_from_elements(
            source_tid,
            neighbor_tids
                .into_iter()
                .filter(|neighbor_tid| keep_neighbor_tid(*neighbor_tid))
                .map(|neighbor_tid| load_graph_element(index_relation, neighbor_tid, code_len)),
            |neighbor| score_candidate(neighbor),
        )
    })
}

pub(crate) unsafe fn search_upper_layer_seed_candidates<ScoreFn>(
    index_relation: pg_sys::Relation,
    code_len: usize,
    m: usize,
    entry_candidate: search::BeamCandidate<page::ItemPointer>,
    entry_level: u8,
    ef_search: usize,
    mut score_candidate: ScoreFn,
) -> Vec<search::BeamCandidate<page::ItemPointer>>
where
    ScoreFn: FnMut(&GraphElement) -> Option<f32>,
{
    if entry_level == 0 {
        return vec![entry_candidate];
    }

    let mut seeds = vec![entry_candidate];
    for layer in (1..=entry_level).rev() {
        seeds = unsafe {
            search_layer_result_candidates(
                index_relation,
                code_len,
                m,
                layer,
                ef_search,
                seeds,
                |_| true,
                |neighbor| score_candidate(neighbor),
            )
        };
        if seeds.is_empty() {
            break;
        }
    }

    seeds
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Layer0VisibleSeedExpansion {
    pub expanded_source_tids: Vec<page::ItemPointer>,
    pub discovered_candidates: Vec<search::BeamCandidate<page::ItemPointer>>,
}

pub(crate) unsafe fn load_layer0_refill_successors<KeepFn, ScoreFn>(
    index_relation: pg_sys::Relation,
    code_len: usize,
    m: usize,
    source_tid: page::ItemPointer,
    max_successor_candidates: usize,
    mut keep_neighbor_tid: KeepFn,
    mut score_candidate: ScoreFn,
) -> Vec<search::BeamCandidate<page::ItemPointer>>
where
    KeepFn: FnMut(page::ItemPointer) -> bool,
    ScoreFn: FnMut(&GraphElement) -> Option<f32>,
{
    if source_tid == page::ItemPointer::INVALID || max_successor_candidates == 0 {
        return Vec::new();
    }

    refill_successors_with_successors(source_tid, max_successor_candidates, |source_tid| unsafe {
        load_layer0_successor_candidates(
            index_relation,
            source_tid,
            code_len,
            m,
            &mut keep_neighbor_tid,
            &mut score_candidate,
        )
    })
}

pub(crate) unsafe fn expand_layer0_visible_seeds<SeedIter, KeepFn, ScoreFn>(
    index_relation: pg_sys::Relation,
    code_len: usize,
    m: usize,
    max_successor_candidates: usize,
    seeds: SeedIter,
    mut keep_neighbor_tid: KeepFn,
    mut score_candidate: ScoreFn,
) -> Layer0VisibleSeedExpansion
where
    SeedIter: IntoIterator<Item = search::BeamCandidate<page::ItemPointer>>,
    KeepFn: FnMut(page::ItemPointer) -> bool,
    ScoreFn: FnMut(&GraphElement) -> Option<f32>,
{
    expand_visible_seeds_with_successors(max_successor_candidates, seeds, |source_tid| unsafe {
        load_layer0_successor_candidates(
            index_relation,
            source_tid,
            code_len,
            m,
            &mut keep_neighbor_tid,
            &mut score_candidate,
        )
    })
}

pub(crate) fn valid_neighbor_tids_for_layer(
    neighbor_tids: &[page::ItemPointer],
    element_level: u8,
    m: usize,
    layer: u8,
) -> Vec<page::ItemPointer> {
    let Some((start, end)) = layer_slot_bounds(element_level, m, layer) else {
        return Vec::new();
    };

    neighbor_tids
        .iter()
        .skip(start)
        .take(end.saturating_sub(start))
        .copied()
        .filter(|tid| *tid != page::ItemPointer::INVALID)
        .collect()
}

pub(crate) fn layer_slot_bounds(element_level: u8, m: usize, layer: u8) -> Option<(usize, usize)> {
    if layer > element_level {
        return None;
    }

    if layer == 0 {
        let end = m.saturating_mul(2);
        return Some((0, end));
    }

    let start = m.saturating_mul(2) + (usize::from(layer).saturating_sub(1) * m);
    Some((start, start.saturating_add(m)))
}

fn greedy_descend_with_successors<NodeId, SuccessorFn>(
    mut current: search::BeamCandidate<NodeId>,
    entry_level: u8,
    mut load_successors: SuccessorFn,
) -> search::BeamCandidate<NodeId>
where
    NodeId: Copy + Eq,
    SuccessorFn: FnMut(NodeId, u8) -> Vec<search::BeamCandidate<NodeId>>,
{
    for layer in (1..=entry_level).rev() {
        loop {
            let next = load_successors(current.node, layer)
                .into_iter()
                .min_by(|left, right| left.score.total_cmp(&right.score));
            let Some(next) = next else {
                break;
            };

            if next.score >= current.score || next.node == current.node {
                break;
            }

            current = search::BeamCandidate::new(next.node, next.score);
        }
    }

    current
}

pub(crate) fn search_layer0_result_candidates_with_successors<NodeId, SeedIter, SuccessorFn>(
    ef_search: usize,
    seeds: SeedIter,
    mut successors: SuccessorFn,
) -> Vec<search::BeamCandidate<NodeId>>
where
    NodeId: Copy + Eq + std::hash::Hash,
    SeedIter: IntoIterator<Item = search::BeamCandidate<NodeId>>,
    SuccessorFn: FnMut(NodeId) -> Vec<search::BeamCandidate<NodeId>>,
{
    if ef_search == 0 {
        return Vec::new();
    }

    let mut visited = HashSet::new();
    let mut candidate_points = BinaryHeap::new();
    let mut result_points = BinaryHeap::new();
    let mut sequence = 0_u64;

    for seed in seeds {
        if !visited.insert(seed.node) {
            continue;
        }

        candidate_points.push(Reverse(LayerSearchCandidate::new(seed, sequence)));
        result_points.push(LayerSearchCandidate::new(seed, sequence));
        sequence += 1;
    }

    while let Some(Reverse(candidate)) = candidate_points.pop() {
        let Some(worst_result) = result_points.peek() else {
            break;
        };

        if result_points.len() >= ef_search
            && candidate.candidate.score > worst_result.candidate.score
        {
            break;
        }

        for neighbor in successors(candidate.candidate.node) {
            if !visited.insert(neighbor.node) {
                continue;
            }

            let should_enqueue = result_points.len() < ef_search
                || result_points
                    .peek()
                    .map(|worst| neighbor.score < worst.candidate.score)
                    .unwrap_or(true);
            if !should_enqueue {
                continue;
            }

            let queued = LayerSearchCandidate::new(neighbor, sequence);
            sequence += 1;
            candidate_points.push(Reverse(queued));
            result_points.push(queued);
            if result_points.len() > ef_search {
                result_points.pop();
            }
        }
    }

    let mut results = result_points
        .into_vec()
        .into_iter()
        .map(|queued| queued.candidate)
        .collect::<Vec<_>>();
    results.sort_by(|left, right| left.score.total_cmp(&right.score));
    results
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LayerSearchCandidate<NodeId> {
    candidate: search::BeamCandidate<NodeId>,
    sequence: u64,
}

impl<NodeId> LayerSearchCandidate<NodeId> {
    fn new(candidate: search::BeamCandidate<NodeId>, sequence: u64) -> Self {
        Self {
            candidate,
            sequence,
        }
    }
}

impl<NodeId: PartialEq> Eq for LayerSearchCandidate<NodeId> {}

impl<NodeId: PartialEq> Ord for LayerSearchCandidate<NodeId> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.candidate
            .score
            .total_cmp(&other.candidate.score)
            .then_with(|| self.sequence.cmp(&other.sequence))
    }
}

impl<NodeId: PartialEq> PartialOrd for LayerSearchCandidate<NodeId> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn refill_successors_with_successors<SuccessorFn>(
    source_tid: page::ItemPointer,
    max_successor_candidates: usize,
    successors: SuccessorFn,
) -> Vec<search::BeamCandidate<page::ItemPointer>>
where
    SuccessorFn: FnMut(page::ItemPointer) -> Vec<search::BeamCandidate<page::ItemPointer>>,
{
    if source_tid == page::ItemPointer::INVALID || max_successor_candidates == 0 {
        return Vec::new();
    }

    run_layer0_beam_search_with_successors(
        1,
        [search::BeamCandidate::new(source_tid, 0.0)],
        successors,
    )
    .frontier
    .into_iter()
    .take(max_successor_candidates)
    .collect()
}

fn expand_visible_seeds_with_successors<SeedIter, SuccessorFn>(
    max_successor_candidates: usize,
    seeds: SeedIter,
    successors: SuccessorFn,
) -> Layer0VisibleSeedExpansion
where
    SeedIter: IntoIterator<Item = search::BeamCandidate<page::ItemPointer>>,
    SuccessorFn: FnMut(page::ItemPointer) -> Vec<search::BeamCandidate<page::ItemPointer>>,
{
    let seeds = seeds.into_iter().collect::<Vec<_>>();
    if max_successor_candidates == 0 || seeds.is_empty() {
        return Layer0VisibleSeedExpansion {
            expanded_source_tids: Vec::new(),
            discovered_candidates: Vec::new(),
        };
    }

    let seed_nodes = seeds
        .iter()
        .map(|candidate| candidate.node)
        .collect::<HashSet<_>>();
    let trace = run_layer0_beam_search_with_successors(
        max_successor_candidates,
        seeds.iter().copied(),
        successors,
    );

    Layer0VisibleSeedExpansion {
        expanded_source_tids: trace
            .expanded
            .into_iter()
            .map(|candidate| candidate.node)
            .filter(|node| seed_nodes.contains(node))
            .collect(),
        discovered_candidates: trace
            .discovered
            .into_iter()
            .filter(|candidate| !seed_nodes.contains(&candidate.node))
            .take(max_successor_candidates)
            .collect(),
    }
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
    fn valid_neighbor_tids_for_layer_skips_invalid() {
        let neighbors = vec![
            page::ItemPointer::INVALID,
            tid(7, 1),
            tid(7, 2),
            page::ItemPointer::INVALID,
            tid(7, 3),
        ];

        assert_eq!(
            valid_neighbor_tids_for_layer(&neighbors, 0, 3, 0),
            vec![tid(7, 1), tid(7, 2), tid(7, 3)],
            "layer-0 neighbor loading should skip INVALID slots while preserving neighbor order",
        );
    }

    #[test]
    fn valid_neighbor_tids_for_layer_limits_to_requested_layer_slice() {
        let neighbors = vec![
            tid(8, 1),
            tid(8, 2),
            page::ItemPointer::INVALID,
            tid(8, 3),
            tid(8, 4),
            tid(8, 5),
            tid(8, 6),
        ];

        assert_eq!(
            valid_neighbor_tids_for_layer(&neighbors, 2, 2, 0),
            vec![tid(8, 1), tid(8, 2), tid(8, 3)],
            "layer-0 neighbor loading should ignore flattened upper-layer neighbors beyond the first 2*M slots",
        );
        assert_eq!(
            valid_neighbor_tids_for_layer(&neighbors, 2, 2, 1),
            vec![tid(8, 4), tid(8, 5)],
            "layer-aware loading should recover the first upper-layer slice independently of layer 0",
        );
        assert_eq!(
            valid_neighbor_tids_for_layer(&neighbors, 2, 2, 2),
            vec![tid(8, 6)],
            "layer-aware loading should recover the second upper-layer slice independently of lower layers",
        );
        assert_eq!(
            valid_neighbor_tids_for_layer(&neighbors, 1, 2, 2),
            Vec::<page::ItemPointer>::new(),
            "requests above the element level should return no neighbors",
        );
    }

    #[test]
    fn greedy_descend_with_successors_walks_down_to_best_upper_layer_local_optimum() {
        let descended = greedy_descend_with_successors(
            search::BeamCandidate::new(1_u64, 0.9),
            2,
            |source, layer| match (source, layer) {
                (1, 2) => vec![
                    search::BeamCandidate::new(2_u64, 0.7),
                    search::BeamCandidate::new(3_u64, 0.8),
                ],
                (2, 2) => vec![search::BeamCandidate::new(4_u64, 0.5)],
                (4, 2) => vec![search::BeamCandidate::new(5_u64, 0.55)],
                (4, 1) => vec![search::BeamCandidate::new(6_u64, 0.3)],
                (6, 1) => vec![search::BeamCandidate::new(7_u64, 0.35)],
                _ => Vec::new(),
            },
        );

        assert_eq!(
            descended,
            search::BeamCandidate::new(6_u64, 0.3),
            "greedy descent should keep taking strictly better neighbors within each upper layer before descending",
        );
    }

    #[test]
    fn search_layer0_result_candidates_with_successors_keeps_best_result_window() {
        let results = search_layer0_result_candidates_with_successors(
            3,
            [search::BeamCandidate::new(1_u64, 0.9)],
            |source| match source {
                1 => vec![
                    search::BeamCandidate::with_source(2_u64, 0.7, 1),
                    search::BeamCandidate::with_source(3_u64, 0.2, 1),
                ],
                2 => vec![search::BeamCandidate::with_source(4_u64, 0.1, 2)],
                3 => vec![search::BeamCandidate::with_source(5_u64, 0.05, 3)],
                _ => Vec::new(),
            },
        );

        assert_eq!(
            results,
            vec![
                search::BeamCandidate::with_source(5_u64, 0.05, 3),
                search::BeamCandidate::with_source(4_u64, 0.1, 2),
                search::BeamCandidate::with_source(3_u64, 0.2, 1),
            ],
            "layer-0 result search should keep the best ef-scored candidates rather than stopping after a fixed number of expansions",
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
            vec![search::BeamCandidate::with_source(
                keep_tid, 0.25, source_tid
            )],
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
                    vec![search::BeamCandidate::with_source(
                        right_best_tid,
                        0.05,
                        right_tid,
                    )]
                } else if source_tid == left_tid {
                    vec![search::BeamCandidate::with_source(
                        left_best_tid,
                        0.2,
                        left_tid,
                    )]
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

    #[test]
    fn refill_successors_with_successors_returns_best_frontier_candidates() {
        let source_tid = tid(2, 1);
        let slow_tid = tid(2, 2);
        let fast_tid = tid(2, 3);
        let skipped_deeper_tid = tid(2, 4);

        let successors = refill_successors_with_successors(source_tid, 2, |source| {
            if source == source_tid {
                vec![
                    search::BeamCandidate::with_source(slow_tid, 0.4, source_tid),
                    search::BeamCandidate::with_source(fast_tid, 0.1, source_tid),
                ]
            } else if source == fast_tid {
                vec![search::BeamCandidate::with_source(
                    skipped_deeper_tid,
                    0.05,
                    fast_tid,
                )]
            } else {
                Vec::new()
            }
        });

        assert_eq!(
            successors,
            vec![
                search::BeamCandidate::with_source(fast_tid, 0.1, source_tid),
                search::BeamCandidate::with_source(slow_tid, 0.4, source_tid),
            ],
            "single-source refill should expose the remaining best-first frontier successors after expanding the consumed source once",
        );
    }

    #[test]
    fn expand_visible_seeds_with_successors_reports_only_seed_sources_and_non_seed_discoveries() {
        let seed_a_tid = tid(3, 1);
        let seed_b_tid = tid(3, 2);
        let child_tid = tid(3, 3);
        let grandchild_tid = tid(3, 4);

        let expansion = expand_visible_seeds_with_successors(
            2,
            [
                search::BeamCandidate::new(seed_a_tid, 0.3),
                search::BeamCandidate::new(seed_b_tid, 0.2),
            ],
            |source| {
                if source == seed_b_tid {
                    vec![search::BeamCandidate::with_source(
                        child_tid, 0.1, seed_b_tid,
                    )]
                } else if source == child_tid {
                    vec![search::BeamCandidate::with_source(
                        grandchild_tid,
                        0.05,
                        child_tid,
                    )]
                } else {
                    Vec::new()
                }
            },
        );

        assert_eq!(
            expansion.expanded_source_tids,
            vec![seed_b_tid],
            "visible-seed expansion should report only the original visible seed nodes it consumed for expansion, leaving deeper discoveries eligible for refill when they surface later",
        );
        assert_eq!(
            expansion.discovered_candidates,
            vec![
                search::BeamCandidate::with_source(child_tid, 0.1, seed_b_tid),
                search::BeamCandidate::with_source(grandchild_tid, 0.05, child_tid),
            ],
            "visible-seed expansion should drop the original seeds and keep only newly discovered candidates in traversal order",
        );
    }
}
