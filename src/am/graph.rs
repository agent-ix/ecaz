use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::ptr;

use hashbrown::HashSet;
use pgrx::pg_sys;

use super::{page, search};
use crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GroupedGraphLayout {
    pub binary_word_count: usize,
    pub search_code_len: usize,
    pub rerank_code_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GraphStorageDescriptor {
    ScalarV1 { code_len: usize },
    GroupedV2(GroupedGraphLayout),
}

impl GraphStorageDescriptor {
    pub(crate) fn from_metadata(metadata: &page::MetadataPage) -> Result<Self, String> {
        match metadata.graph_storage_format()? {
            page::GraphStorageFormat::ScalarV1 => Ok(Self::ScalarV1 {
                code_len: if metadata.dimensions == 0 {
                    0
                } else {
                    crate::code_len(metadata.dimensions as usize, metadata.bits)
                },
            }),
            page::GraphStorageFormat::GroupedV2 => {
                if metadata.payload_flags & page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE == 0 {
                    return Err(
                        "grouped-v2 metadata must advertise grouped search-code payloads"
                            .to_owned(),
                    );
                }
                if metadata.payload_flags & page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD == 0 {
                    return Err(
                        "grouped-v2 metadata must advertise cold rerank payloads".to_owned()
                    );
                }
                if metadata.search_codec_kind != page::SearchCodecKind::GroupedPq {
                    return Err(format!(
                        "unsupported grouped-v2 search codec: {:?}",
                        metadata.search_codec_kind
                    ));
                }
                if metadata.search_bits != 4 {
                    return Err(format!(
                        "unsupported grouped-v2 search bits: {}",
                        metadata.search_bits
                    ));
                }
                if metadata.search_subvector_count == 0 || metadata.search_subvector_dim == 0 {
                    return Err(
                        "grouped-v2 metadata must record non-zero grouped search shape".to_owned(),
                    );
                }
                if metadata.rerank_codec_kind != page::RerankCodecKind::ScalarQuantized {
                    return Err(format!(
                        "unsupported grouped-v2 rerank codec: {:?}",
                        metadata.rerank_codec_kind
                    ));
                }
                if metadata.grouped_codebook_head == page::ItemPointer::INVALID {
                    return Err(
                        "grouped-v2 metadata must advertise a persisted grouped codebook chain"
                            .to_owned(),
                    );
                }
                let binary_word_count =
                    if metadata.payload_flags & page::PAYLOAD_FLAG_BINARY_SIDECAR != 0
                        && crate::quant::prod::ProdQuantizer::cached(
                            metadata.dimensions as usize,
                            metadata.bits,
                            metadata.seed,
                        )
                        .binary_sign_no_qjl_4bit_supported()
                    {
                        (metadata.dimensions as usize).div_ceil(64)
                    } else {
                        0
                    };
                Ok(Self::GroupedV2(GroupedGraphLayout {
                    binary_word_count,
                    search_code_len: usize::from(metadata.search_subvector_count).div_ceil(2),
                    rerank_code_len: crate::code_len(metadata.dimensions as usize, metadata.bits),
                }))
            }
        }
    }
}

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

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GroupedGraphElement {
    pub tid: page::ItemPointer,
    pub level: u8,
    pub deleted: bool,
    pub heaptids: Vec<page::ItemPointer>,
    pub neighbortid: page::ItemPointer,
    pub reranktid: page::ItemPointer,
    pub binary_words: Vec<u64>,
    pub search_code: Vec<u8>,
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GroupedRerankPayload {
    pub tid: page::ItemPointer,
    pub gamma: f32,
    pub code: Vec<u8>,
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GroupedCodebookModel {
    pub head_tid: page::ItemPointer,
    pub group_count: usize,
    pub group_size: usize,
    pub flat_codebooks: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GraphNeighbors {
    pub tid: page::ItemPointer,
    pub count: usize,
    pub tids: Vec<page::ItemPointer>,
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
#[derive(Debug, Clone, Copy)]
pub(crate) enum GraphTupleRef<'a> {
    Scalar(page::TqElementTupleRef<'a>),
    GroupedHot(page::TqGroupedHotTupleRef<'a>),
}

impl<'a> GraphTupleRef<'a> {
    pub(crate) fn level(self) -> u8 {
        match self {
            Self::Scalar(tuple) => tuple.level,
            Self::GroupedHot(tuple) => tuple.level,
        }
    }

    pub(crate) fn deleted(self) -> bool {
        match self {
            Self::Scalar(tuple) => tuple.deleted,
            Self::GroupedHot(tuple) => tuple.deleted,
        }
    }

    pub(crate) fn heaptid_count(self) -> usize {
        match self {
            Self::Scalar(tuple) => tuple.heaptid_count(),
            Self::GroupedHot(tuple) => tuple.heaptid_count(),
        }
    }

    pub(crate) fn collect_heaptids(self) -> Vec<page::ItemPointer> {
        match self {
            Self::Scalar(tuple) => tuple.collect_heaptids(),
            Self::GroupedHot(tuple) => tuple.collect_heaptids(),
        }
    }

    pub(crate) fn neighbortid(self) -> page::ItemPointer {
        match self {
            Self::Scalar(tuple) => tuple.neighbortid,
            Self::GroupedHot(tuple) => tuple.neighbortid,
        }
    }

    pub(crate) fn reranktid(self) -> Option<page::ItemPointer> {
        match self {
            Self::Scalar(_) => None,
            Self::GroupedHot(tuple) => Some(tuple.reranktid),
        }
    }

    pub(crate) fn binary_word_count(self) -> usize {
        match self {
            Self::Scalar(tuple) => tuple.binary_word_count(),
            Self::GroupedHot(tuple) => tuple.binary_word_count(),
        }
    }

    pub(crate) fn collect_binary_words(self) -> Vec<u64> {
        match self {
            Self::Scalar(tuple) => tuple.collect_binary_words(),
            Self::GroupedHot(tuple) => tuple.collect_binary_words(),
        }
    }

    pub(crate) fn exact_payload(self) -> Option<(f32, &'a [u8])> {
        match self {
            Self::Scalar(tuple) => Some((tuple.gamma, tuple.code)),
            Self::GroupedHot(_) => None,
        }
    }

    pub(crate) fn grouped_search_code(self) -> Option<&'a [u8]> {
        match self {
            Self::Scalar(_) => None,
            Self::GroupedHot(tuple) => Some(tuple.search_code),
        }
    }
}

pub(crate) unsafe fn load_graph_element(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    code_len: usize,
) -> GraphElement {
    let element = unsafe {
        read_page_tuple(index_relation, element_tid, "element", |tuple_bytes| {
            page::TqElementTuple::decode(tuple_bytes, code_len)
        })
    }
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

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
pub(crate) unsafe fn load_grouped_graph_element(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    layout: GroupedGraphLayout,
) -> GroupedGraphElement {
    let element = unsafe {
        read_page_tuple(index_relation, element_tid, "grouped hot", |tuple_bytes| {
            page::TqGroupedHotTuple::decode(
                tuple_bytes,
                layout.binary_word_count,
                layout.search_code_len,
            )
        })
    }
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode grouped graph tuple: {e}"));
    GroupedGraphElement {
        tid: element_tid,
        level: element.level,
        deleted: element.deleted,
        heaptids: element.heaptids,
        neighbortid: element.neighbortid,
        reranktid: element.reranktid,
        binary_words: element.binary_words,
        search_code: element.search_code,
    }
}

pub(crate) unsafe fn with_graph_element_tuple<R, F>(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    code_len: usize,
    f: F,
) -> R
where
    F: FnOnce(page::TqElementTupleRef<'_>) -> R,
{
    unsafe {
        read_page_tuple(index_relation, element_tid, "element", |tuple_bytes| {
            Ok(f(page::TqElementTupleRef::decode(tuple_bytes, code_len)?))
        })
    }
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode graph element tuple: {e}"))
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
pub(crate) unsafe fn with_grouped_graph_tuple<R, F>(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    layout: GroupedGraphLayout,
    f: F,
) -> R
where
    F: FnOnce(page::TqGroupedHotTupleRef<'_>) -> R,
{
    unsafe {
        read_page_tuple(index_relation, element_tid, "grouped hot", |tuple_bytes| {
            Ok(f(page::TqGroupedHotTupleRef::decode(
                tuple_bytes,
                layout.binary_word_count,
                layout.search_code_len,
            )?))
        })
    }
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode grouped graph tuple: {e}"))
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
pub(crate) unsafe fn with_graph_storage_tuple<R, F>(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    storage: GraphStorageDescriptor,
    f: F,
) -> R
where
    F: FnOnce(GraphTupleRef<'_>) -> R,
{
    match storage {
        GraphStorageDescriptor::ScalarV1 { code_len } => unsafe {
            with_graph_element_tuple(index_relation, element_tid, code_len, |tuple| {
                f(GraphTupleRef::Scalar(tuple))
            })
        },
        GraphStorageDescriptor::GroupedV2(layout) => unsafe {
            with_grouped_graph_tuple(index_relation, element_tid, layout, |tuple| {
                f(GraphTupleRef::GroupedHot(tuple))
            })
        },
    }
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
pub(crate) unsafe fn with_grouped_rerank_tuple<R, F>(
    index_relation: pg_sys::Relation,
    rerank_tid: page::ItemPointer,
    layout: GroupedGraphLayout,
    f: F,
) -> R
where
    F: FnOnce(page::TqRerankTupleRef<'_>) -> R,
{
    unsafe {
        read_page_tuple(index_relation, rerank_tid, "rerank", |tuple_bytes| {
            Ok(f(page::TqRerankTupleRef::decode(
                tuple_bytes,
                layout.rerank_code_len,
            )?))
        })
    }
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode grouped rerank tuple: {e}"))
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
pub(crate) unsafe fn load_grouped_rerank_payload(
    index_relation: pg_sys::Relation,
    rerank_tid: page::ItemPointer,
    layout: GroupedGraphLayout,
) -> GroupedRerankPayload {
    let rerank = unsafe {
        read_page_tuple(index_relation, rerank_tid, "rerank", |tuple_bytes| {
            page::TqRerankTuple::decode(tuple_bytes, layout.rerank_code_len)
        })
    }
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode grouped rerank tuple: {e}"));
    GroupedRerankPayload {
        tid: rerank_tid,
        gamma: rerank.gamma,
        code: rerank.code,
    }
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
pub(crate) unsafe fn with_grouped_codebook_tuple<R, F>(
    index_relation: pg_sys::Relation,
    codebook_tid: page::ItemPointer,
    centroid_count: usize,
    f: F,
) -> R
where
    F: FnOnce(page::TqGroupedCodebookTupleRef<'_>) -> R,
{
    unsafe {
        read_page_tuple(
            index_relation,
            codebook_tid,
            "grouped codebook",
            |tuple_bytes| {
                Ok(f(page::TqGroupedCodebookTupleRef::decode(
                    tuple_bytes,
                    centroid_count,
                )?))
            },
        )
    }
    .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to decode grouped codebook tuple: {e}"))
}

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
pub(crate) unsafe fn load_grouped_codebook_model(
    index_relation: pg_sys::Relation,
    metadata: &page::MetadataPage,
) -> GroupedCodebookModel {
    let group_count = usize::from(metadata.search_subvector_count);
    let group_size = usize::from(metadata.search_subvector_dim);
    if group_count == 0 || group_size == 0 {
        pgrx::error!("tqhnsw grouped codebook load requires non-zero grouped search shape");
    }
    if metadata.grouped_codebook_head == page::ItemPointer::INVALID {
        pgrx::error!("tqhnsw grouped-v2 metadata is missing a grouped codebook head pointer");
    }

    let centroid_count = group_size * GROUPED_PQ_CENTROIDS;
    let mut next_tid = metadata.grouped_codebook_head;
    let mut flat_codebooks = Vec::with_capacity(group_count * centroid_count);

    for expected_group_index in 0..group_count {
        if next_tid == page::ItemPointer::INVALID {
            pgrx::error!(
                "tqhnsw grouped codebook chain ended early at group {} of {}",
                expected_group_index,
                group_count
            );
        }
        let codebook = unsafe {
            with_grouped_codebook_tuple(index_relation, next_tid, centroid_count, |tuple| {
                page::TqGroupedCodebookTuple {
                    group_index: tuple.group_index,
                    nexttid: tuple.nexttid,
                    centroids: tuple.collect_centroids(),
                }
            })
        };
        if usize::from(codebook.group_index) != expected_group_index {
            pgrx::error!(
                "tqhnsw grouped codebook order mismatch: got group {}, expected {}",
                codebook.group_index,
                expected_group_index
            );
        }
        flat_codebooks.extend(codebook.centroids);
        next_tid = codebook.nexttid;
    }

    if next_tid != page::ItemPointer::INVALID {
        pgrx::error!(
            "tqhnsw grouped codebook chain contains trailing tuples beyond metadata shape"
        );
    }

    GroupedCodebookModel {
        head_tid: metadata.grouped_codebook_head,
        group_count,
        group_size,
        flat_codebooks,
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

    let neighbor = unsafe {
        read_page_tuple(
            index_relation,
            neighbor_tid,
            "neighbor",
            page::TqNeighborTuple::decode,
        )
    }
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

#[cfg_attr(not(any(test, feature = "pg_test")), allow(dead_code))]
pub(crate) unsafe fn load_grouped_graph_adjacency(
    index_relation: pg_sys::Relation,
    element_tid: page::ItemPointer,
    layout: GroupedGraphLayout,
) -> (GroupedGraphElement, GraphNeighbors) {
    let element = unsafe { load_grouped_graph_element(index_relation, element_tid, layout) };
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
    unsafe {
        load_successor_candidates_for_layer(
            index_relation,
            source_tid,
            code_len,
            m,
            0,
            &mut keep_neighbor_tid,
            &mut score_candidate,
        )
    }
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
            load_successor_candidates_for_layer(
                index_relation,
                source_tid,
                code_len,
                m,
                layer,
                |_| true,
                &mut score_candidate,
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
        load_successor_candidates_for_layer(
            index_relation,
            source_tid,
            code_len,
            m,
            layer,
            &mut keep_neighbor_tid,
            &mut score_candidate,
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
    let mut tids = Vec::with_capacity(layer_neighbor_slot_capacity(
        neighbor_tids.len(),
        element_level,
        m,
        layer,
    ));
    for_each_valid_neighbor_tid_for_layer(neighbor_tids, element_level, m, layer, |tid| {
        tids.push(tid);
    });
    tids
}

pub(crate) fn for_each_valid_neighbor_tid_for_layer<F>(
    neighbor_tids: &[page::ItemPointer],
    element_level: u8,
    m: usize,
    layer: u8,
    mut visit: F,
) where
    F: FnMut(page::ItemPointer),
{
    let Some((start, end)) = layer_slot_bounds(element_level, m, layer) else {
        return;
    };

    for &tid in neighbor_tids
        .iter()
        .skip(start)
        .take(end.saturating_sub(start))
    {
        if tid != page::ItemPointer::INVALID {
            visit(tid);
        }
    }
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

fn layer_neighbor_slot_capacity(
    neighbor_tid_count: usize,
    element_level: u8,
    m: usize,
    layer: u8,
) -> usize {
    let Some((start, end)) = layer_slot_bounds(element_level, m, layer) else {
        return 0;
    };
    let bounded_start = start.min(neighbor_tid_count);
    let bounded_end = end.min(neighbor_tid_count);
    bounded_end.saturating_sub(bounded_start)
}

pub(crate) fn greedy_descend_with_successors<NodeId, SuccessorFn>(
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

unsafe fn load_successor_candidates_for_layer<KeepFn, ScoreFn>(
    index_relation: pg_sys::Relation,
    source_tid: page::ItemPointer,
    code_len: usize,
    m: usize,
    layer: u8,
    mut keep_neighbor_tid: KeepFn,
    mut score_candidate: ScoreFn,
) -> Vec<search::BeamCandidate<page::ItemPointer>>
where
    KeepFn: FnMut(page::ItemPointer) -> bool,
    ScoreFn: FnMut(&GraphElement) -> Option<f32>,
{
    let (element, neighbors) =
        unsafe { load_graph_adjacency(index_relation, source_tid, code_len) };
    let mut candidates = Vec::with_capacity(layer_neighbor_slot_capacity(
        neighbors.tids.len(),
        element.level,
        m,
        layer,
    ));

    for_each_valid_neighbor_tid_for_layer(
        &neighbors.tids,
        element.level,
        m,
        layer,
        |neighbor_tid| {
            if keep_neighbor_tid(neighbor_tid) {
                let neighbor =
                    unsafe { load_graph_element(index_relation, neighbor_tid, code_len) };
                if !neighbor.deleted && !neighbor.heaptids.is_empty() {
                    if let Some(score) = score_candidate(&neighbor) {
                        candidates.push(search::BeamCandidate::with_source(
                            neighbor.tid,
                            score,
                            source_tid,
                        ));
                    }
                }
            }
        },
    );

    candidates
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

unsafe fn read_page_tuple<T, DecodeFn>(
    index_relation: pg_sys::Relation,
    tuple_tid: page::ItemPointer,
    tuple_kind: &str,
    decode: DecodeFn,
) -> Result<T, String>
where
    DecodeFn: FnOnce(&[u8]) -> Result<T, String>,
{
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

    let tuple_bytes = unsafe { std::slice::from_raw_parts(page_ptr.add(tuple_offset), tuple_len) };
    let decoded = decode(tuple_bytes);
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    decoded
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
    fn graph_storage_descriptor_uses_scalar_code_len_for_v1_metadata() {
        let metadata = page::MetadataPage::current_v1_scalar(page::CurrentFormatMetadata {
            m: 8,
            ef_construction: 40,
            entry_point: page::ItemPointer::INVALID,
            dimensions: 16,
            bits: 4,
            max_level: 0,
            seed: 42,
            inserted_since_rebuild: 0,
            persisted_binary_sidecar: false,
        });

        assert_eq!(
            GraphStorageDescriptor::from_metadata(&metadata).unwrap(),
            GraphStorageDescriptor::ScalarV1 {
                code_len: crate::code_len(16, 4)
            }
        );
    }

    #[test]
    fn graph_storage_descriptor_uses_grouped_lengths_for_v2_metadata() {
        let metadata = page::MetadataPage {
            m: 8,
            ef_construction: 40,
            entry_point: page::ItemPointer::INVALID,
            dimensions: 96,
            bits: 4,
            max_level: 0,
            seed: 42,
            inserted_since_rebuild: 0,
            format_version: page::INDEX_FORMAT_V2_GROUPED,
            transform_kind: page::TransformKind::Srht,
            search_codec_kind: page::SearchCodecKind::GroupedPq,
            payload_flags: page::PAYLOAD_FLAG_BINARY_SIDECAR
                | page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE
                | page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
            search_bits: 4,
            rerank_codec_kind: page::RerankCodecKind::ScalarQuantized,
            search_subvector_count: 6,
            search_subvector_dim: 16,
            grouped_codebook_head: tid(2, 1),
        };

        assert_eq!(
            GraphStorageDescriptor::from_metadata(&metadata).unwrap(),
            GraphStorageDescriptor::GroupedV2(GroupedGraphLayout {
                binary_word_count: 0,
                search_code_len: 3,
                rerank_code_len: crate::code_len(96, 4),
            })
        );
    }

    #[test]
    fn graph_storage_descriptor_rejects_grouped_v2_missing_grouped_payload_flag() {
        let metadata = page::MetadataPage {
            m: 8,
            ef_construction: 40,
            entry_point: page::ItemPointer::INVALID,
            dimensions: 96,
            bits: 4,
            max_level: 0,
            seed: 42,
            inserted_since_rebuild: 0,
            format_version: page::INDEX_FORMAT_V2_GROUPED,
            transform_kind: page::TransformKind::Srht,
            search_codec_kind: page::SearchCodecKind::GroupedPq,
            payload_flags: page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
            search_bits: 4,
            rerank_codec_kind: page::RerankCodecKind::ScalarQuantized,
            search_subvector_count: 6,
            search_subvector_dim: 16,
            grouped_codebook_head: tid(2, 1),
        };

        assert_eq!(
            GraphStorageDescriptor::from_metadata(&metadata),
            Err("grouped-v2 metadata must advertise grouped search-code payloads".to_owned())
        );
    }

    #[test]
    fn graph_storage_descriptor_rejects_grouped_v2_missing_cold_rerank_flag() {
        let metadata = page::MetadataPage {
            m: 8,
            ef_construction: 40,
            entry_point: page::ItemPointer::INVALID,
            dimensions: 96,
            bits: 4,
            max_level: 0,
            seed: 42,
            inserted_since_rebuild: 0,
            format_version: page::INDEX_FORMAT_V2_GROUPED,
            transform_kind: page::TransformKind::Srht,
            search_codec_kind: page::SearchCodecKind::GroupedPq,
            payload_flags: page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE,
            search_bits: 4,
            rerank_codec_kind: page::RerankCodecKind::ScalarQuantized,
            search_subvector_count: 6,
            search_subvector_dim: 16,
            grouped_codebook_head: tid(2, 1),
        };

        assert_eq!(
            GraphStorageDescriptor::from_metadata(&metadata),
            Err("grouped-v2 metadata must advertise cold rerank payloads".to_owned())
        );
    }

    #[test]
    fn graph_storage_descriptor_rejects_grouped_v2_missing_codebook_head() {
        let metadata = page::MetadataPage {
            m: 8,
            ef_construction: 40,
            entry_point: page::ItemPointer::INVALID,
            dimensions: 96,
            bits: 4,
            max_level: 0,
            seed: 42,
            inserted_since_rebuild: 0,
            format_version: page::INDEX_FORMAT_V2_GROUPED,
            transform_kind: page::TransformKind::Srht,
            search_codec_kind: page::SearchCodecKind::GroupedPq,
            payload_flags: page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE
                | page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
            search_bits: 4,
            rerank_codec_kind: page::RerankCodecKind::ScalarQuantized,
            search_subvector_count: 6,
            search_subvector_dim: 16,
            grouped_codebook_head: page::ItemPointer::INVALID,
        };

        assert_eq!(
            GraphStorageDescriptor::from_metadata(&metadata),
            Err("grouped-v2 metadata must advertise a persisted grouped codebook chain".to_owned())
        );
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
