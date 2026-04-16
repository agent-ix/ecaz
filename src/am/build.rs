use std::cmp::Ordering;
use std::collections::HashSet;
use std::ffi::c_void;
use std::ptr;

use hnsw_rs::anndists::dist::distances::Distance;
use hnsw_rs::prelude::Hnsw;
use pgrx::{itemptr::item_pointer_get_both, pg_sys, varlena, FromDatum, PgBox};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::quant::{
    grouped_pq::{encode_grouped_pq, nearest_centroid_l2, GROUPED_PQ_CENTROIDS},
    prod::ProdQuantizer,
};

use super::{options, page, shared, source, wal, P_NEW};

const ADR030_EXPERIMENTAL_GROUP_SIZE: usize = 16;
const ADR030_EXPERIMENTAL_MAX_TRAIN_SIZE: usize = 1024;
const ADR030_EXPERIMENTAL_KMEANS_ITERS: usize = 8;
const ADR030_EXPERIMENTAL_BUILD_ENV: &str = "TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD";

#[derive(Debug, Clone)]
pub(super) struct BuildTuple {
    pub(super) heap_tids: Vec<page::ItemPointer>,
    pub(super) dimensions: u16,
    pub(super) bits: u8,
    pub(super) seed: u64,
    pub(super) gamma: f32,
    pub(super) code: Vec<u8>,
    pub(super) source_vector: Option<Vec<f32>>,
    pub(super) source_count: usize,
}

#[derive(Debug)]
pub(super) struct BuildState {
    pub(super) options: options::TqHnswOptions,
    pub(super) page_size: usize,
    pub(super) scanned_tuples: usize,
    pub(super) heap_tuples: Vec<BuildTuple>,
    pub(super) dimensions: Option<u16>,
    pub(super) bits: Option<u8>,
    pub(super) seed: Option<u64>,
}

pub(super) unsafe extern "C-unwind" fn tqhnsw_build_callback(
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
            let heap_tid = shared::decode_heap_tid(tid);
            let tuple = build_heap_tuple(values, isnull, heap_tid);
            state.push(tuple);
        })
    }
}

pub(super) unsafe extern "C-unwind" fn tqhnsw_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut state = BuildState::new(index_relation);

            shared::initialize_metadata_page(index_relation, state.initial_metadata());

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

pub(super) unsafe extern "C-unwind" fn tqhnsw_ambuildempty(index_relation: pg_sys::Relation) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = BuildState::new(index_relation);
            shared::initialize_metadata_page(index_relation, state.initial_metadata());
        })
    }
}

impl BuildState {
    pub(super) fn new(index_relation: pg_sys::Relation) -> Self {
        let options = unsafe { options::relation_options(index_relation) };
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

    pub(super) fn initial_metadata(&self) -> page::MetadataPage {
        page::MetadataPage::current_v1_scalar(page::CurrentFormatMetadata {
            m: u16::try_from(self.options.m).expect("validated m should fit into u16"),
            ef_construction: u16::try_from(self.options.ef_construction)
                .expect("validated ef_construction should fit into u16"),
            entry_point: page::ItemPointer::INVALID,
            dimensions: 0,
            bits: 0,
            max_level: 0,
            seed: 0,
            inserted_since_rebuild: 0,
            persisted_binary_sidecar: false,
        })
    }

    pub(super) fn push(&mut self, tuple: BuildTuple) {
        self.scanned_tuples += tuple.heap_tids.len();
        let binary_word_count =
            persisted_binary_sidecar_word_count(tuple.dimensions, tuple.bits, tuple.seed);

        match (self.dimensions, self.bits, self.seed) {
            (None, None, None) => {
                self.dimensions = Some(tuple.dimensions);
                self.bits = Some(tuple.bits);
                self.seed = Some(tuple.seed);
                if page::raw_tuple_storage_bytes(page::TqElementTuple::encoded_len_with_binary(
                    tuple.code.len(),
                    binary_word_count,
                )) > self.page_size.saturating_sub(page::PAGE_HEADER_BYTES)
                {
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

        if let Some(existing) = self.heap_tuples.iter_mut().find(|existing| {
            existing.gamma.to_bits() == tuple.gamma.to_bits() && existing.code == tuple.code
        }) {
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
                    source::average_source_representatives(
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

fn persisted_binary_sidecar_word_count(dimensions: u16, bits: u8, seed: u64) -> usize {
    let quantizer = ProdQuantizer::cached(dimensions as usize, bits, seed);
    if quantizer.binary_sign_no_qjl_4bit_supported() {
        (dimensions as usize).div_ceil(64)
    } else {
        0
    }
}

pub(super) unsafe fn build_heap_tuple(
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

    let original = datum
        .cast_mut_ptr::<std::ffi::c_void>()
        .cast::<pg_sys::varlena>();
    let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
    let is_copy = !std::ptr::eq(varlena, original);
    let bytes = unsafe { varlena::varlena_to_byte_slice(varlena) }.to_vec();
    if is_copy {
        unsafe { pg_sys::pfree(varlena.cast()) };
    }

    let (dimensions, bits, seed, gamma, code) = crate::unpack(&bytes)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw ambuild found invalid tqvector: {e}"));

    if !gamma.is_finite() {
        pgrx::error!("tqhnsw does not support non-finite gamma values");
    }

    BuildTuple {
        heap_tids: vec![heap_tid],
        dimensions,
        bits,
        seed,
        gamma,
        code: code.to_vec(),
        source_vector: None,
        source_count: 0,
    }
}

pub(super) unsafe fn build_heap_tuple_with_source(
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

    let (dimensions, bits, seed, gamma, code) = crate::unpack(&bytes)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw ambuild found invalid tqvector: {e}"));

    if !gamma.is_finite() {
        pgrx::error!("tqhnsw does not support non-finite gamma values");
    }

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
        gamma,
        code: code.to_vec(),
        source_vector: Some(source_vector),
        source_count: 1,
    }
}

pub(super) unsafe fn tqhnsw_build_scan_with_source(
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
    let source_attribute = unsafe {
        source::resolve_source_attribute(
            heap_relation,
            &source_column,
            "build_source_column",
            source::SourceTypePolicy::RealArrayOnly,
        )
    };

    let slot = unsafe {
        source::allocate_heap_slot(
            heap_relation,
            "tqhnsw ambuild failed to allocate heap scan slot",
        )
    };

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
            unsafe { source::required_slot_datum(slot, index_attnum, "indexed tqvector column") };
        let source_datum = unsafe {
            source::required_slot_datum(slot, source_attribute.attnum, "tqhnsw build_source_column")
        };
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HnswBuildNode {
    pub(super) level: u8,
    pub(super) neighbor_slots: Vec<Option<usize>>,
    pub(super) score_neighbors: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct V2GroupedBuildPayload {
    pub(super) hot: page::TqGroupedHotTuple,
    pub(super) rerank: page::TqRerankTuple,
}

pub(super) fn stage_v2_grouped_build_payload(
    tuple: &BuildTuple,
    level: u8,
    neighbortid: page::ItemPointer,
    reranktid: page::ItemPointer,
    search_code: Vec<u8>,
    persisted_binary_quantizer: &ProdQuantizer,
) -> V2GroupedBuildPayload {
    let binary_words = if persisted_binary_quantizer.binary_sign_no_qjl_4bit_supported() {
        persisted_binary_quantizer.binary_sign_words_from_packed_no_qjl_4bit(&tuple.code)
    } else {
        Vec::new()
    };

    V2GroupedBuildPayload {
        hot: page::TqGroupedHotTuple {
            level,
            deleted: false,
            heaptids: tuple.heap_tids.clone(),
            neighbortid,
            reranktid,
            binary_words,
            search_code,
        },
        rerank: page::TqRerankTuple {
            gamma: tuple.gamma,
            code: tuple.code.clone(),
        },
    }
}

#[derive(Debug, Clone)]
pub(super) struct V2GroupedStagedChain {
    pub(super) data_pages: page::DataPageChain,
    pub(super) hot_tids: Vec<page::ItemPointer>,
    pub(super) rerank_tids: Vec<page::ItemPointer>,
    pub(super) neighbor_tids: Vec<page::ItemPointer>,
}

#[derive(Debug, Clone)]
pub(super) struct V2GroupedBuildPlan {
    pub(super) staged_chain: V2GroupedStagedChain,
    pub(super) entry_point: page::ItemPointer,
    pub(super) max_level: u8,
    pub(super) grouped_model: BuildGroupedPqModel,
}

#[derive(Debug, Clone)]
pub(super) struct BuildFlushOutput {
    pub(super) data_pages: page::DataPageChain,
    pub(super) metadata: page::MetadataPage,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct BuildGroupedPqModel {
    pub(super) codebooks: Vec<Vec<f32>>,
    pub(super) group_count: usize,
    pub(super) group_size: usize,
    pub(super) transform_dim: usize,
    pub(super) signs: Vec<f32>,
}

pub(super) fn stage_v2_grouped_page_chain(
    state: &BuildState,
    graph_nodes: &[HnswBuildNode],
    grouped_search_codes: &[Vec<u8>],
) -> Result<V2GroupedStagedChain, String> {
    if state.heap_tuples.len() != graph_nodes.len()
        || state.heap_tuples.len() != grouped_search_codes.len()
    {
        return Err(format!(
            "staged v2 inputs length mismatch: tuples={} graph_nodes={} grouped_search_codes={}",
            state.heap_tuples.len(),
            graph_nodes.len(),
            grouped_search_codes.len()
        ));
    }

    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions");
    let bits = state.bits.expect("non-empty build should record bits");
    let seed = state.seed.expect("non-empty build should record seed");
    let persisted_binary_quantizer = ProdQuantizer::cached(dimensions as usize, bits, seed);

    let mut data_pages = page::DataPageChain::new(state.page_size);
    let mut hot_tids = Vec::with_capacity(state.heap_tuples.len());
    let mut rerank_tids = Vec::with_capacity(state.heap_tuples.len());
    let mut neighbor_tids = Vec::with_capacity(state.heap_tuples.len());

    for ((tuple, graph_node), grouped_search_code) in state
        .heap_tuples
        .iter()
        .zip(graph_nodes.iter())
        .zip(grouped_search_codes.iter())
    {
        let slot_count = graph_node.neighbor_slots.len();
        let placeholder_neighbor = page::TqNeighborTuple {
            count: slot_count as u16,
            tids: vec![page::ItemPointer::INVALID; slot_count],
        };
        let neighbor_tid = data_pages.insert_neighbor(&placeholder_neighbor)?;
        let rerank_tid = data_pages.insert_rerank(&page::TqRerankTuple {
            gamma: tuple.gamma,
            code: tuple.code.clone(),
        })?;
        let payload = stage_v2_grouped_build_payload(
            tuple,
            graph_node.level,
            neighbor_tid,
            rerank_tid,
            grouped_search_code.clone(),
            &persisted_binary_quantizer,
        );
        let hot_tid = data_pages.insert_grouped_hot(&payload.hot)?;

        hot_tids.push(hot_tid);
        rerank_tids.push(rerank_tid);
        neighbor_tids.push(neighbor_tid);
    }

    for (idx, neighbor_tid) in neighbor_tids.iter().copied().enumerate() {
        let neighbor_refs = graph_nodes[idx]
            .neighbor_slots
            .iter()
            .map(|neighbor_idx| {
                neighbor_idx
                    .map(|ni| hot_tids[ni])
                    .unwrap_or(page::ItemPointer::INVALID)
            })
            .collect::<Vec<_>>();

        data_pages.update_neighbor(
            neighbor_tid,
            &page::TqNeighborTuple {
                count: neighbor_refs.len() as u16,
                tids: neighbor_refs,
            },
        )?;
    }

    Ok(V2GroupedStagedChain {
        data_pages,
        hot_tids,
        rerank_tids,
        neighbor_tids,
    })
}

pub(super) fn stage_v2_grouped_page_chain_from_source(
    state: &BuildState,
    graph_nodes: &[HnswBuildNode],
    group_size: usize,
    train_size: usize,
    kmeans_iters: usize,
) -> Result<(V2GroupedStagedChain, BuildGroupedPqModel), String> {
    let model = train_build_grouped_pq_model(state, group_size, train_size, kmeans_iters)?;
    let grouped_search_codes = state
        .heap_tuples
        .iter()
        .map(|tuple| derive_grouped_search_code_from_source(tuple, &model))
        .collect::<Result<Vec<_>, _>>()?;

    Ok((
        stage_v2_grouped_page_chain(state, graph_nodes, &grouped_search_codes)?,
        model,
    ))
}

pub(super) fn plan_v2_grouped_source_build(
    state: &BuildState,
    group_size: usize,
    train_size: usize,
    kmeans_iters: usize,
) -> Result<V2GroupedBuildPlan, String> {
    let graph_nodes = build_hnsw_graph(state);
    let (staged_chain, grouped_model) = stage_v2_grouped_page_chain_from_source(
        state,
        &graph_nodes,
        group_size,
        train_size,
        kmeans_iters,
    )?;
    let entry_point = choose_entry_point(&staged_chain.hot_tids, &graph_nodes, state)
        .unwrap_or(page::ItemPointer::INVALID);
    let max_level = graph_nodes.iter().map(|node| node.level).max().unwrap_or(0);

    Ok(V2GroupedBuildPlan {
        staged_chain,
        entry_point,
        max_level,
        grouped_model,
    })
}

fn stage_v2_grouped_codebook_tuples(
    data_pages: &mut page::DataPageChain,
    model: &BuildGroupedPqModel,
) -> Result<page::ItemPointer, String> {
    let centroid_count = model.group_size * GROUPED_PQ_CENTROIDS;
    let mut codebook_tids = Vec::with_capacity(model.group_count);

    for (group_index, codebook) in model.codebooks.iter().enumerate() {
        if codebook.len() != centroid_count {
            return Err(format!(
                "grouped codebook {} length mismatch: got {}, expected {}",
                group_index,
                codebook.len(),
                centroid_count
            ));
        }
        codebook_tids.push(
            data_pages.insert_grouped_codebook(&page::TqGroupedCodebookTuple {
                group_index: u16::try_from(group_index).map_err(|_| {
                    format!("grouped codebook index {group_index} does not fit in u16")
                })?,
                nexttid: page::ItemPointer::INVALID,
                centroids: codebook.clone(),
            })?,
        );
    }

    for (group_index, tid) in codebook_tids
        .iter()
        .copied()
        .enumerate()
        .take(codebook_tids.len().saturating_sub(1))
    {
        data_pages.update_grouped_codebook(
            tid,
            &page::TqGroupedCodebookTuple {
                group_index: u16::try_from(group_index).expect("validated group index fits in u16"),
                nexttid: codebook_tids[group_index + 1],
                centroids: model.codebooks[group_index].clone(),
            },
        )?;
    }

    codebook_tids
        .first()
        .copied()
        .ok_or_else(|| "grouped codebook staging requires at least one codebook".to_owned())
}

pub(super) fn train_build_grouped_pq_model(
    state: &BuildState,
    group_size: usize,
    train_size: usize,
    kmeans_iters: usize,
) -> Result<BuildGroupedPqModel, String> {
    let dimensions = usize::from(
        state
            .dimensions
            .expect("non-empty build should record dimensions"),
    );
    let seed = state.seed.expect("non-empty build should record seed");
    let transform_dim = crate::quant::rotation::effective_transform_dim(dimensions);
    if transform_dim % group_size != 0 {
        return Err(format!(
            "transform dim {transform_dim} is not divisible by group_size {group_size}"
        ));
    }

    let signs = crate::quant::rotation::sign_vector(transform_dim, seed);
    let source_vectors = state
        .heap_tuples
        .iter()
        .map(|tuple| {
            tuple
                .source_vector
                .as_ref()
                .ok_or_else(|| "grouped build model requires source vectors".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if source_vectors.len() < 16 {
        return Err(format!(
            "grouped build model needs at least 16 source vectors, got {}",
            source_vectors.len()
        ));
    }

    let transformed = source_vectors
        .iter()
        .map(|vector| crate::quant::rotation::srht_padded(vector, &signs))
        .collect::<Vec<_>>();
    let group_count = transform_dim / group_size;
    let sample_count = train_size.min(transformed.len());
    let sample_indices = sample_indices(
        transformed.len(),
        sample_count,
        seed ^ 0xA5A5_5A5A_DEAD_BEEF,
    );
    let mut codebooks = Vec::with_capacity(group_count);

    for group_index in 0..group_count {
        let mut samples = Vec::with_capacity(sample_count * group_size);
        for &sample_index in &sample_indices {
            let start = group_index * group_size;
            let end = start + group_size;
            samples.extend_from_slice(&transformed[sample_index][start..end]);
        }
        codebooks.push(train_group_codebook(
            &samples,
            group_size,
            kmeans_iters,
            seed ^ (group_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        )?);
    }

    Ok(BuildGroupedPqModel {
        codebooks,
        group_count,
        group_size,
        transform_dim,
        signs,
    })
}

pub(super) fn derive_grouped_search_code_from_source(
    tuple: &BuildTuple,
    model: &BuildGroupedPqModel,
) -> Result<Vec<u8>, String> {
    let source = tuple
        .source_vector
        .as_ref()
        .ok_or_else(|| "grouped search code derivation requires source vector".to_owned())?;
    let rotated = crate::quant::rotation::srht_padded(source, &model.signs);
    Ok(encode_grouped_pq(
        &rotated,
        model.codebooks.iter().map(Vec::as_slice),
        model.group_size,
    ))
}

fn sample_indices(len: usize, sample_count: usize, seed: u64) -> Vec<usize> {
    if sample_count >= len {
        return (0..len).collect();
    }

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut indices = (0..len).collect::<Vec<_>>();
    for i in 0..sample_count {
        let swap_index = rng.gen_range(i..len);
        indices.swap(i, swap_index);
    }
    indices.truncate(sample_count);
    indices
}

fn train_group_codebook(
    samples: &[f32],
    group_size: usize,
    kmeans_iters: usize,
    seed: u64,
) -> Result<Vec<f32>, String> {
    const CENTROIDS: usize = 16;

    let sample_count = samples.len() / group_size;
    if sample_count < CENTROIDS {
        return Err(format!(
            "need at least {CENTROIDS} samples for grouped codebook, got {sample_count}"
        ));
    }

    let init_indices = sample_indices(sample_count, CENTROIDS, seed);
    let mut centroids = vec![0.0_f32; CENTROIDS * group_size];
    for (centroid_index, sample_index) in init_indices.into_iter().enumerate() {
        let sample = sample_slice(samples, sample_index, group_size);
        centroid_slice_mut(&mut centroids, centroid_index, group_size).copy_from_slice(sample);
    }

    let mut assignments = vec![0usize; sample_count];
    let mut sums = vec![0.0_f32; CENTROIDS * group_size];
    let mut counts = [0usize; CENTROIDS];

    for _ in 0..kmeans_iters {
        sums.fill(0.0);
        counts.fill(0);

        for (sample_index, assignment) in assignments.iter_mut().enumerate() {
            let sample = sample_slice(samples, sample_index, group_size);
            let centroid_index = nearest_centroid_l2(sample, &centroids, group_size);
            *assignment = centroid_index;
            counts[centroid_index] += 1;
            let centroid_sum = centroid_slice_mut(&mut sums, centroid_index, group_size);
            for (dst, value) in centroid_sum.iter_mut().zip(sample.iter()) {
                *dst += *value;
            }
        }

        for (centroid_index, &count) in counts.iter().enumerate() {
            if count == 0 {
                let fallback_sample = sample_slice(
                    samples,
                    (seed as usize + centroid_index) % sample_count,
                    group_size,
                );
                centroid_slice_mut(&mut centroids, centroid_index, group_size)
                    .copy_from_slice(fallback_sample);
                continue;
            }

            let inv_count = (count as f32).recip();
            let centroid_sum = centroid_slice(&sums, centroid_index, group_size);
            let centroid = centroid_slice_mut(&mut centroids, centroid_index, group_size);
            for (dst, value) in centroid.iter_mut().zip(centroid_sum.iter()) {
                *dst = *value * inv_count;
            }
        }
    }

    Ok(centroids)
}

fn squared_l2(lhs: &[f32], rhs: &[f32]) -> f32 {
    lhs.iter()
        .zip(rhs.iter())
        .map(|(left, right)| {
            let delta = left - right;
            delta * delta
        })
        .sum()
}

fn sample_slice(samples: &[f32], sample_index: usize, group_size: usize) -> &[f32] {
    let start = sample_index * group_size;
    &samples[start..start + group_size]
}

fn centroid_slice(centroids: &[f32], centroid_index: usize, group_size: usize) -> &[f32] {
    let start = centroid_index * group_size;
    &centroids[start..start + group_size]
}

fn centroid_slice_mut(
    centroids: &mut [f32],
    centroid_index: usize,
    group_size: usize,
) -> &mut [f32] {
    let start = centroid_index * group_size;
    &mut centroids[start..start + group_size]
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

impl Distance<f32> for BuildVectorDistance {
    fn eval(&self, va: &[f32], vb: &[f32]) -> f32 {
        self.score_offset - score_source_inner_product(va, vb)
    }
}

pub(super) unsafe fn flush_build_state(index_relation: pg_sys::Relation, state: &BuildState) {
    let output = if experimental_grouped_v2_build_enabled()
        && state.options.build_source_column.is_some()
    {
        experimental_grouped_v2_flush_output(state)
            .unwrap_or_else(|e| pgrx::error!("tqhnsw experimental ADR-030 v2 build failed: {e}"))
    } else {
        current_format_flush_output(state)
    };
    unsafe { flush_build_output(index_relation, &output) };
}

pub(super) fn grouped_v2_flush_output(
    state: &BuildState,
    plan: &V2GroupedBuildPlan,
    group_size: usize,
) -> Result<BuildFlushOutput, String> {
    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions");
    let bits = state.bits.expect("non-empty build should record bits");
    let seed = state.seed.expect("non-empty build should record seed");
    let transform_dim = crate::quant::rotation::effective_transform_dim(dimensions as usize);
    if transform_dim % group_size != 0 {
        return Err(format!(
            "transform dim {transform_dim} is not divisible by group_size {group_size}"
        ));
    }
    let search_subvector_count = u16::try_from(transform_dim / group_size)
        .map_err(|_| "grouped search subvector count does not fit into u16".to_owned())?;
    let search_subvector_dim = u16::try_from(group_size)
        .map_err(|_| "grouped search subvector dim does not fit into u16".to_owned())?;
    let mut payload_flags =
        page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE | page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD;
    if persisted_binary_sidecar_word_count(dimensions, bits, seed) > 0 {
        payload_flags |= page::PAYLOAD_FLAG_BINARY_SIDECAR;
    }
    let mut data_pages = plan.staged_chain.data_pages.clone();
    let grouped_codebook_head =
        stage_v2_grouped_codebook_tuples(&mut data_pages, &plan.grouped_model)?;

    Ok(BuildFlushOutput {
        data_pages,
        metadata: page::MetadataPage {
            m: u16::try_from(state.options.m).expect("validated m should fit into u16"),
            ef_construction: u16::try_from(state.options.ef_construction)
                .expect("validated ef_construction should fit into u16"),
            entry_point: plan.entry_point,
            dimensions,
            bits,
            max_level: plan.max_level,
            seed,
            inserted_since_rebuild: 0,
            format_version: page::INDEX_FORMAT_V2_GROUPED,
            transform_kind: page::TransformKind::Srht,
            search_codec_kind: page::SearchCodecKind::GroupedPq,
            payload_flags,
            search_bits: 4,
            rerank_codec_kind: page::RerankCodecKind::ScalarQuantized,
            search_subvector_count,
            search_subvector_dim,
            grouped_codebook_head,
        },
    })
}

fn experimental_grouped_v2_build_enabled() -> bool {
    std::env::var_os(ADR030_EXPERIMENTAL_BUILD_ENV).is_some()
}

fn experimental_grouped_v2_flush_output(state: &BuildState) -> Result<BuildFlushOutput, String> {
    let train_size = state
        .heap_tuples
        .len()
        .min(ADR030_EXPERIMENTAL_MAX_TRAIN_SIZE);
    let plan = plan_v2_grouped_source_build(
        state,
        ADR030_EXPERIMENTAL_GROUP_SIZE,
        train_size,
        ADR030_EXPERIMENTAL_KMEANS_ITERS,
    )?;
    grouped_v2_flush_output(state, &plan, ADR030_EXPERIMENTAL_GROUP_SIZE)
}

fn current_format_flush_output(state: &BuildState) -> BuildFlushOutput {
    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions");
    let bits = state.bits.expect("non-empty build should record bits");
    let mut data_pages = page::DataPageChain::new(state.page_size);
    let mut element_tids = Vec::with_capacity(state.heap_tuples.len());
    let mut neighbor_tids = Vec::with_capacity(state.heap_tuples.len());
    let graph_nodes = build_hnsw_graph(state);
    let persisted_binary_quantizer = ProdQuantizer::cached(
        dimensions as usize,
        bits,
        state.seed.expect("non-empty build should record seed"),
    );
    let write_persisted_binary = persisted_binary_quantizer.binary_sign_no_qjl_4bit_supported();

    for (idx, tuple) in state.heap_tuples.iter().enumerate() {
        let slot_count = graph_nodes[idx].neighbor_slots.len();
        let placeholder_neighbor = page::TqNeighborTuple {
            count: slot_count as u16,
            tids: vec![page::ItemPointer::INVALID; slot_count],
        };
        let neighbor_tid = data_pages
            .insert_neighbor(&placeholder_neighbor)
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to stage neighbor tuple: {e}"));

        let element_tid = data_pages
            .insert_element(&page::TqElementTuple {
                level: graph_nodes[idx].level,
                deleted: false,
                heaptids: tuple.heap_tids.clone(),
                gamma: tuple.gamma,
                neighbortid: neighbor_tid,
                code: tuple.code.clone(),
                binary_words: if write_persisted_binary {
                    persisted_binary_quantizer
                        .binary_sign_words_from_packed_no_qjl_4bit(&tuple.code)
                } else {
                    Vec::new()
                },
            })
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to stage element tuple: {e}"));

        element_tids.push(element_tid);
        neighbor_tids.push(neighbor_tid);
    }

    for (idx, neighbor_tid) in neighbor_tids.iter().copied().enumerate() {
        let neighbor_refs = graph_nodes[idx]
            .neighbor_slots
            .iter()
            .map(|neighbor_idx| {
                neighbor_idx
                    .map(|ni| element_tids[ni])
                    .unwrap_or(page::ItemPointer::INVALID)
            })
            .collect::<Vec<_>>();

        data_pages
            .update_neighbor(
                neighbor_tid,
                &page::TqNeighborTuple {
                    count: neighbor_refs.len() as u16,
                    tids: neighbor_refs,
                },
            )
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to update neighbor tuple: {e}"));
    }

    let entry_point = choose_entry_point(&element_tids, &graph_nodes, state)
        .unwrap_or(page::ItemPointer::INVALID);
    let max_level = graph_nodes.iter().map(|node| node.level).max().unwrap_or(0);
    let seed = state.seed.expect("non-empty build should record seed");

    BuildFlushOutput {
        data_pages,
        metadata: page::MetadataPage::current_v1_scalar(page::CurrentFormatMetadata {
            m: u16::try_from(state.options.m).expect("validated m should fit into u16"),
            ef_construction: u16::try_from(state.options.ef_construction)
                .expect("validated ef_construction should fit into u16"),
            entry_point,
            dimensions,
            bits,
            max_level,
            seed,
            inserted_since_rebuild: 0,
            persisted_binary_sidecar: persisted_binary_sidecar_word_count(dimensions, bits, seed)
                > 0,
        }),
    }
}

unsafe fn flush_build_output(index_relation: pg_sys::Relation, output: &BuildFlushOutput) {
    unsafe { write_data_pages(index_relation, &output.data_pages) };
    unsafe { shared::initialize_metadata_page(index_relation, output.metadata.clone()) };
}

pub(super) fn build_hnsw_graph(state: &BuildState) -> Vec<HnswBuildNode> {
    let m = usize::try_from(state.options.m).expect("validated m should be non-negative");
    if state.heap_tuples.len() <= 1 {
        return vec![
            HnswBuildNode {
                level: 0,
                neighbor_slots: empty_neighbor_slots(0, m),
                score_neighbors: Vec::new(),
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
    let max_level_cap = page::max_level_that_fits(
        u16::try_from(state.options.m).expect("validated m should fit into u16"),
        state.page_size,
    );
    let max_layer = usize::from(max_level_cap).saturating_add(1).max(1);
    let hnsw = Hnsw::new_with_seed(
        m,
        state.heap_tuples.len(),
        max_layer,
        usize::try_from(state.options.ef_construction)
            .expect("validated ef_construction should be non-negative"),
        deterministic_hnsw_build_seed(state, 0x5343_414c_4152_5f31),
        BuildCodeDistance::new(dimensions, bits, seed),
    );

    for (origin_id, tuple) in state.heap_tuples.iter().enumerate() {
        hnsw.insert((&tuple.code, origin_id));
    }

    let mut nodes = vec![
        HnswBuildNode {
            level: 0,
            neighbor_slots: empty_neighbor_slots(0, m),
            score_neighbors: Vec::new(),
        };
        state.heap_tuples.len()
    ];
    for point in hnsw.get_point_indexation() {
        let origin_id = point.get_origin_id();
        let level = point.get_point_id().0.min(max_level_cap);
        let neighborhoods = point.get_neighborhood_id();
        let neighbor_slots = pack_point_neighbor_slots(origin_id, level, m, &neighborhoods);
        let score_neighbors = flatten_point_neighbors(origin_id, level, &neighborhoods);
        nodes[origin_id] = HnswBuildNode {
            level,
            neighbor_slots,
            score_neighbors,
        };
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
    let hnsw = Hnsw::new_with_seed(
        m,
        state.heap_tuples.len(),
        max_layer,
        usize::try_from(state.options.ef_construction)
            .expect("validated ef_construction should be non-negative"),
        deterministic_hnsw_build_seed(state, 0x534f_5552_4345_5f31),
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
            neighbor_slots: empty_neighbor_slots(0, m),
            score_neighbors: Vec::new(),
        };
        state.heap_tuples.len()
    ];
    for point in hnsw.get_point_indexation() {
        let origin_id = point.get_origin_id();
        let level = point.get_point_id().0.min(max_level_cap);
        let neighborhoods = point.get_neighborhood_id();
        let neighbor_slots = pack_point_neighbor_slots(origin_id, level, m, &neighborhoods);
        let score_neighbors = flatten_point_neighbors(origin_id, level, &neighborhoods);
        nodes[origin_id] = HnswBuildNode {
            level,
            neighbor_slots,
            score_neighbors,
        };
    }

    nodes
}

fn empty_neighbor_slots(level: u8, m: usize) -> Vec<Option<usize>> {
    vec![None; page::neighbor_slots(level, m as u16)]
}

fn deterministic_hnsw_build_seed(state: &BuildState, domain_tag: u64) -> u64 {
    let base_seed = state.seed.expect("non-empty build should record seed");
    let dimensions = u64::from(
        state
            .dimensions
            .expect("non-empty build should record dimensions"),
    );
    let bits = u64::from(state.bits.expect("non-empty build should record bits"));
    let tuple_count =
        u64::try_from(state.heap_tuples.len()).expect("tuple count should fit into u64");
    let build_hash = base_seed
        ^ domain_tag
        ^ dimensions.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ bits.wrapping_mul(0xA24B_AED4_963E_E407)
        ^ (state.options.m as u64).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ (state.options.ef_construction as u64).wrapping_mul(0xD134_2543_DE82_EF95)
        ^ tuple_count.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    splitmix64(build_hash)
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[cfg(any(test, feature = "pg_test"))]
pub(super) fn build_scored_neighbor_graph(state: &BuildState) -> Vec<Vec<usize>> {
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

fn pack_point_neighbor_slots(
    origin_id: usize,
    level: u8,
    m: usize,
    neighbors_per_layer: &[Vec<hnsw_rs::hnsw::Neighbour>],
) -> Vec<Option<usize>> {
    let mut slots = vec![None; page::neighbor_slots(level, m as u16)];
    fill_point_neighbor_layer_slots(
        &mut slots,
        origin_id,
        0,
        0,
        m.saturating_mul(2),
        neighbors_per_layer,
    );

    for layer in 1..=usize::from(level) {
        let start = m.saturating_mul(2) + ((layer - 1) * m);
        fill_point_neighbor_layer_slots(
            &mut slots,
            origin_id,
            layer,
            start,
            m,
            neighbors_per_layer,
        );
    }

    slots
}

fn fill_point_neighbor_layer_slots(
    slots: &mut [Option<usize>],
    origin_id: usize,
    layer: usize,
    start: usize,
    width: usize,
    neighbors_per_layer: &[Vec<hnsw_rs::hnsw::Neighbour>],
) {
    if width == 0 || start >= slots.len() {
        return;
    }

    let Some(layer_neighbors) = neighbors_per_layer.get(layer) else {
        return;
    };

    let end = start.saturating_add(width).min(slots.len());
    let mut next_slot = start;
    for neighbor in layer_neighbors {
        if neighbor.d_id == origin_id {
            continue;
        }
        if next_slot >= end {
            break;
        }

        slots[next_slot] = Some(neighbor.d_id);
        next_slot += 1;
    }
}

fn score_source_inner_product(left: &[f32], right: &[f32]) -> f32 {
    left.iter().zip(right.iter()).map(|(l, r)| l * r).sum()
}

pub(super) fn choose_entry_point(
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
        .score_neighbors
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
                gamma: 0.0,
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
                gamma: 0.0,
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
                gamma: 0.0,
                code: encoded_code(&[0.98, 0.02, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
        ];
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 1,
                ef_construction: 32,
                ef_search: 40,
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
                gamma: 0.0,
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
                gamma: 0.0,
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
                gamma: 0.0,
                code: encoded_code(&[-1.0, 0.5, 0.0, 1.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
        ];
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 10,
                ef_construction: 90,
                ef_search: 40,
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
        assert!(nodes.iter().any(|node| {
            !node
                .neighbor_slots
                .iter()
                .all(|neighbor_slot| neighbor_slot.is_none())
        }));
    }

    #[test]
    fn hnsw_graph_build_is_deterministic_for_scalar_codes() {
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
                gamma: 0.0,
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
                gamma: 0.0,
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
                gamma: 0.0,
                code: encoded_code(&[-1.0, 0.5, 0.0, 1.0], bits, seed),
                source_vector: None,
                source_count: 0,
            },
        ];
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 10,
                ef_construction: 90,
                ef_search: 40,
                build_source_column: None,
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 3,
            heap_tuples: tuples,
            dimensions: Some(4),
            bits: Some(bits),
            seed: Some(seed),
        };

        let first = build_hnsw_graph(&state);
        let second = build_hnsw_graph(&state);

        assert_eq!(first, second);
    }

    #[test]
    fn hnsw_graph_build_is_deterministic_for_source_vectors() {
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
                gamma: 0.0,
                code: vec![0x12, 0x34],
                source_vector: Some(vec![1.0, 0.0, 0.5, -1.0]),
                source_count: 1,
            },
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 2,
                }],
                dimensions: 4,
                bits,
                seed,
                gamma: 0.0,
                code: vec![0x56, 0x78],
                source_vector: Some(vec![0.0, 1.0, 0.25, -0.5]),
                source_count: 1,
            },
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 0,
                    offset_number: 3,
                }],
                dimensions: 4,
                bits,
                seed,
                gamma: 0.0,
                code: vec![0x9A, 0xBC],
                source_vector: Some(vec![-1.0, 0.5, 0.0, 1.0]),
                source_count: 1,
            },
        ];
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 10,
                ef_construction: 90,
                ef_search: 40,
                build_source_column: Some("source".to_owned()),
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 3,
            heap_tuples: tuples,
            dimensions: Some(4),
            bits: Some(bits),
            seed: Some(seed),
        };

        let first = build_hnsw_graph(&state);
        let second = build_hnsw_graph(&state);

        assert_eq!(first, second);
    }

    #[test]
    fn source_scored_entry_point_prefers_raw_vectors() {
        let seed = 42_u64;
        let bits = 4_u8;
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 2,
                ef_construction: 64,
                ef_search: 40,
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
                    gamma: 0.0,
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
                    gamma: 0.0,
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
                    gamma: 0.0,
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
                neighbor_slots: vec![Some(1)],
                score_neighbors: vec![1],
            },
            HnswBuildNode {
                level: 0,
                neighbor_slots: vec![Some(2)],
                score_neighbors: vec![2],
            },
            HnswBuildNode {
                level: 0,
                neighbor_slots: vec![Some(1)],
                score_neighbors: vec![1],
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
    fn pack_point_neighbor_slots_preserves_layer_boundaries_with_padding() {
        let slots = pack_point_neighbor_slots(
            10,
            2,
            2,
            &[
                vec![
                    hnsw_rs::hnsw::Neighbour::new(11, 0.1, hnsw_rs::hnsw::PointId(0, 11)),
                    hnsw_rs::hnsw::Neighbour::new(12, 0.2, hnsw_rs::hnsw::PointId(0, 12)),
                ],
                vec![hnsw_rs::hnsw::Neighbour::new(
                    13,
                    0.3,
                    hnsw_rs::hnsw::PointId(1, 13),
                )],
                vec![hnsw_rs::hnsw::Neighbour::new(
                    14,
                    0.4,
                    hnsw_rs::hnsw::PointId(2, 14),
                )],
            ],
        );

        assert_eq!(
            slots,
            vec![Some(11), Some(12), None, None, Some(13), None, Some(14), None],
            "persisted neighbor slots should keep fixed 2M / M layer boundaries instead of compacting upper-layer tids into layer-0 space",
        );
    }

    #[test]
    fn average_source_representative_weights_by_duplicate_count() {
        let mut representative = vec![1.0, 0.0];
        source::average_source_representatives(&mut representative, 1, &[0.0, 1.0], 1);
        assert_eq!(representative, vec![0.5, 0.5]);

        source::average_source_representatives(&mut representative, 2, &[1.0, 1.0], 2);
        assert_eq!(representative, vec![0.75, 0.75]);
    }

    #[test]
    fn stage_v2_grouped_build_payload_keeps_hot_and_cold_split() {
        let seed = 42_u64;
        let bits = 4_u8;
        let vector = (0..1536)
            .map(|i| match i % 4 {
                0 => 1.0,
                1 => 0.0,
                2 => 0.5,
                _ => -1.0,
            })
            .collect::<Vec<_>>();
        let tuple = BuildTuple {
            heap_tids: vec![page::ItemPointer {
                block_number: 1,
                offset_number: 7,
            }],
            dimensions: 1536,
            bits,
            seed,
            gamma: 1.25,
            code: encoded_code(&vector, bits, seed),
            source_vector: None,
            source_count: 0,
        };
        let quantizer = ProdQuantizer::cached(1536, bits, seed);
        let payload = stage_v2_grouped_build_payload(
            &tuple,
            3,
            page::ItemPointer {
                block_number: 10,
                offset_number: 2,
            },
            page::ItemPointer {
                block_number: 10,
                offset_number: 3,
            },
            vec![0x12, 0x34],
            &quantizer,
        );

        assert_eq!(payload.hot.level, 3);
        assert_eq!(payload.hot.heaptids, tuple.heap_tids);
        assert_eq!(payload.hot.search_code, vec![0x12, 0x34]);
        assert_eq!(payload.hot.neighbortid.block_number, 10);
        assert_eq!(payload.hot.reranktid.offset_number, 3);
        assert_eq!(
            payload.hot.binary_words,
            quantizer.binary_sign_words_from_packed_no_qjl_4bit(&tuple.code)
        );
        assert_eq!(payload.rerank.gamma.to_bits(), tuple.gamma.to_bits());
        assert_eq!(payload.rerank.code, tuple.code);
    }

    #[test]
    fn stage_v2_grouped_build_payload_skips_binary_sidecar_when_unsupported() {
        let seed = 42_u64;
        let bits = 8_u8;
        let tuple = BuildTuple {
            heap_tids: vec![page::ItemPointer {
                block_number: 1,
                offset_number: 7,
            }],
            dimensions: 8,
            bits,
            seed,
            gamma: 0.5,
            code: encoded_code(&[1.0, 0.0, 0.5, -1.0, 0.25, 0.5, -0.5, 0.75], bits, seed),
            source_vector: None,
            source_count: 0,
        };
        let quantizer = ProdQuantizer::cached(8, bits, seed);
        let payload = stage_v2_grouped_build_payload(
            &tuple,
            1,
            page::ItemPointer::INVALID,
            page::ItemPointer::INVALID,
            vec![0xAB],
            &quantizer,
        );

        assert!(payload.hot.binary_words.is_empty());
        assert_eq!(payload.hot.search_code, vec![0xAB]);
        assert_eq!(payload.rerank.code, tuple.code);
    }

    #[test]
    fn stage_v2_grouped_page_chain_links_hot_neighbor_and_rerank_tuples() {
        let seed = 42_u64;
        let bits = 4_u8;
        let tuples = vec![
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 1,
                    offset_number: 1,
                }],
                dimensions: 1536,
                bits,
                seed,
                gamma: 0.5,
                code: encoded_code(&vec![1.0; 1536], bits, seed),
                source_vector: None,
                source_count: 0,
            },
            BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 1,
                    offset_number: 2,
                }],
                dimensions: 1536,
                bits,
                seed,
                gamma: 0.25,
                code: encoded_code(&vec![0.5; 1536], bits, seed),
                source_vector: None,
                source_count: 0,
            },
        ];
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 2,
                ef_construction: 32,
                ef_search: 40,
                build_source_column: None,
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples.clone(),
            dimensions: Some(1536),
            bits: Some(bits),
            seed: Some(seed),
        };
        let graph_nodes = vec![
            HnswBuildNode {
                level: 1,
                neighbor_slots: vec![Some(1)],
                score_neighbors: vec![1],
            },
            HnswBuildNode {
                level: 0,
                neighbor_slots: vec![Some(0)],
                score_neighbors: vec![0],
            },
        ];
        let grouped_search_codes = vec![vec![0x12, 0x34], vec![0x56, 0x78]];

        let staged =
            stage_v2_grouped_page_chain(&state, &graph_nodes, &grouped_search_codes).unwrap();

        assert_eq!(staged.hot_tids.len(), 2);
        assert_eq!(staged.rerank_tids.len(), 2);
        assert_eq!(staged.neighbor_tids.len(), 2);

        let hot0 = staged
            .data_pages
            .read_grouped_hot(staged.hot_tids[0], 24, 2)
            .unwrap();
        let rerank0 = staged
            .data_pages
            .read_rerank(staged.rerank_tids[0], tuples[0].code.len())
            .unwrap();
        let neighbors0 = staged
            .data_pages
            .read_neighbor(staged.neighbor_tids[0])
            .unwrap();

        assert_eq!(hot0.search_code, grouped_search_codes[0]);
        assert_eq!(hot0.reranktid, staged.rerank_tids[0]);
        assert_eq!(rerank0.code, tuples[0].code);
        assert_eq!(neighbors0.tids[0], staged.hot_tids[1]);
    }

    #[test]
    fn grouped_build_model_trains_and_derives_codes_from_source_vectors() {
        let seed = 42_u64;
        let bits = 4_u8;
        let tuples = (0..16)
            .map(|i| BuildTuple {
                heap_tids: vec![page::ItemPointer {
                    block_number: 1,
                    offset_number: (i + 1) as u16,
                }],
                dimensions: 16,
                bits,
                seed,
                gamma: 0.0,
                code: vec![0xAA; 8],
                source_vector: Some(
                    (0..16)
                        .map(|dim| ((i * 17 + dim) as f32 * 0.07).sin())
                        .collect(),
                ),
                source_count: 1,
            })
            .collect::<Vec<_>>();
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 2,
                ef_construction: 32,
                ef_search: 40,
                build_source_column: Some("source".to_owned()),
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples.clone(),
            dimensions: Some(16),
            bits: Some(bits),
            seed: Some(seed),
        };

        let model = train_build_grouped_pq_model(&state, 4, 16, 3).unwrap();
        assert_eq!(model.group_size, 4);
        assert_eq!(model.group_count, 4);

        let code = derive_grouped_search_code_from_source(&tuples[0], &model).unwrap();
        assert_eq!(code.len(), model.group_count.div_ceil(2));
    }

    #[test]
    fn grouped_build_model_requires_source_vectors() {
        let seed = 42_u64;
        let bits = 4_u8;
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 2,
                ef_construction: 32,
                ef_search: 40,
                build_source_column: None,
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 16,
            heap_tuples: (0..16)
                .map(|i| BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: (i + 1) as u16,
                    }],
                    dimensions: 16,
                    bits,
                    seed,
                    gamma: 0.0,
                    code: vec![0xAA; 8],
                    source_vector: None,
                    source_count: 0,
                })
                .collect(),
            dimensions: Some(16),
            bits: Some(bits),
            seed: Some(seed),
        };

        let error = train_build_grouped_pq_model(&state, 4, 16, 3).unwrap_err();
        assert!(error.contains("source vectors"));
    }

    #[test]
    fn stage_v2_grouped_page_chain_from_source_derives_codes_and_links_pages() {
        let seed = 42_u64;
        let bits = 4_u8;
        let tuples = (0..16)
            .map(|i| {
                let source = (0..16)
                    .map(|dim| ((i * 17 + dim) as f32 * 0.07).sin())
                    .collect::<Vec<_>>();
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: (i + 1) as u16,
                    }],
                    dimensions: 16,
                    bits,
                    seed,
                    gamma: 0.1 * i as f32,
                    code: vec![i as u8; 8],
                    source_vector: Some(source),
                    source_count: 1,
                }
            })
            .collect::<Vec<_>>();
        let graph_nodes = (0..16)
            .map(|i| HnswBuildNode {
                level: (i % 3) as u8,
                neighbor_slots: vec![Some((i + 1) % 16)],
                score_neighbors: vec![(i + 1) % 16],
            })
            .collect::<Vec<_>>();
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 2,
                ef_construction: 32,
                ef_search: 40,
                build_source_column: Some("source".to_owned()),
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples.clone(),
            dimensions: Some(16),
            bits: Some(bits),
            seed: Some(seed),
        };

        let (staged, model) =
            stage_v2_grouped_page_chain_from_source(&state, &graph_nodes, 4, 16, 3).unwrap();

        assert_eq!(staged.hot_tids.len(), 16);
        assert_eq!(model.group_count, 4);
        let first_hot = staged
            .data_pages
            .read_grouped_hot(staged.hot_tids[0], 0, 2)
            .unwrap();
        let first_rerank = staged
            .data_pages
            .read_rerank(staged.rerank_tids[0], tuples[0].code.len())
            .unwrap();
        let first_neighbor = staged
            .data_pages
            .read_neighbor(staged.neighbor_tids[0])
            .unwrap();

        assert_eq!(first_hot.reranktid, staged.rerank_tids[0]);
        assert_eq!(first_rerank.code, tuples[0].code);
        assert_eq!(first_neighbor.tids[0], staged.hot_tids[1]);
        assert_eq!(first_hot.search_code.len(), 2);
        assert!(first_hot.binary_words.is_empty());
    }

    #[test]
    fn plan_v2_grouped_source_build_reports_entry_point_and_levels() {
        let seed = 42_u64;
        let bits = 4_u8;
        let tuples = (0..16)
            .map(|i| {
                let source = (0..16)
                    .map(|dim| ((i * 19 + dim) as f32 * 0.05).cos())
                    .collect::<Vec<_>>();
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: (i + 1) as u16,
                    }],
                    dimensions: 16,
                    bits,
                    seed,
                    gamma: 0.05 * i as f32,
                    code: vec![i as u8; 8],
                    source_vector: Some(source),
                    source_count: 1,
                }
            })
            .collect::<Vec<_>>();
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 2,
                ef_construction: 32,
                ef_search: 40,
                build_source_column: Some("source".to_owned()),
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples,
            dimensions: Some(16),
            bits: Some(bits),
            seed: Some(seed),
        };

        let plan = plan_v2_grouped_source_build(&state, 4, 16, 3).unwrap();

        assert_eq!(plan.staged_chain.hot_tids.len(), 16);
        assert_ne!(plan.entry_point, page::ItemPointer::INVALID);
        assert!(usize::from(plan.max_level) < 16);
        assert_eq!(plan.grouped_model.group_count, 4);
    }

    #[test]
    fn stage_v2_grouped_codebook_tuples_links_groups_in_order() {
        let mut data_pages = page::DataPageChain::new(pg_sys::BLCKSZ as usize);
        let model = BuildGroupedPqModel {
            codebooks: vec![vec![0.1; 64], vec![0.2; 64], vec![0.3; 64], vec![0.4; 64]],
            group_count: 4,
            group_size: 4,
            transform_dim: 16,
            signs: vec![1.0; 16],
        };

        let head_tid = stage_v2_grouped_codebook_tuples(&mut data_pages, &model).unwrap();
        let first = data_pages.read_grouped_codebook(head_tid, 64).unwrap();
        let second = data_pages.read_grouped_codebook(first.nexttid, 64).unwrap();
        let third = data_pages
            .read_grouped_codebook(second.nexttid, 64)
            .unwrap();
        let fourth = data_pages.read_grouped_codebook(third.nexttid, 64).unwrap();

        assert_eq!(first.group_index, 0);
        assert_eq!(second.group_index, 1);
        assert_eq!(third.group_index, 2);
        assert_eq!(fourth.group_index, 3);
        assert_eq!(fourth.nexttid, page::ItemPointer::INVALID);
        assert_eq!(first.centroids, vec![0.1; 64]);
        assert_eq!(fourth.centroids, vec![0.4; 64]);
    }

    #[test]
    fn grouped_v2_flush_output_marks_grouped_metadata_and_pages() {
        let seed = 42_u64;
        let bits = 4_u8;
        let tuples = (0..16)
            .map(|i| {
                let source = (0..16)
                    .map(|dim| ((i * 19 + dim) as f32 * 0.05).cos())
                    .collect::<Vec<_>>();
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: (i + 1) as u16,
                    }],
                    dimensions: 16,
                    bits,
                    seed,
                    gamma: 0.05 * i as f32,
                    code: vec![i as u8; 8],
                    source_vector: Some(source),
                    source_count: 1,
                }
            })
            .collect::<Vec<_>>();
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 2,
                ef_construction: 32,
                ef_search: 40,
                build_source_column: Some("source".to_owned()),
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples,
            dimensions: Some(16),
            bits: Some(bits),
            seed: Some(seed),
        };

        let plan = plan_v2_grouped_source_build(&state, 4, 16, 3).unwrap();
        let output = grouped_v2_flush_output(&state, &plan, 4).unwrap();

        assert_eq!(
            output.metadata.format_version,
            page::INDEX_FORMAT_V2_GROUPED
        );
        assert_eq!(output.metadata.transform_kind, page::TransformKind::Srht);
        assert_eq!(
            output.metadata.search_codec_kind,
            page::SearchCodecKind::GroupedPq
        );
        assert_eq!(
            output.metadata.rerank_codec_kind,
            page::RerankCodecKind::ScalarQuantized
        );
        assert_eq!(output.metadata.search_bits, 4);
        assert_eq!(output.metadata.search_subvector_count, 4);
        assert_eq!(output.metadata.search_subvector_dim, 4);
        assert_ne!(
            output.metadata.grouped_codebook_head,
            page::ItemPointer::INVALID
        );
        assert_eq!(
            output.metadata.payload_flags,
            page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE | page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD
        );

        let tuple_tags = output
            .data_pages
            .pages()
            .iter()
            .flat_map(|page| page.tuples().iter().map(|tuple| tuple[0]))
            .collect::<Vec<_>>();
        assert!(tuple_tags.contains(&page::TQ_GROUPED_HOT_TAG));
        assert!(tuple_tags.contains(&page::TQ_RERANK_TAG));
        assert!(tuple_tags.contains(&page::TQ_NEIGHBOR_TAG));
        assert!(tuple_tags.contains(&page::TQ_GROUPED_CODEBOOK_TAG));
        assert!(!tuple_tags.contains(&page::TQ_ELEMENT_TAG));
    }

    #[test]
    fn experimental_grouped_v2_flush_output_uses_default_v2_parameters() {
        let seed = 42_u64;
        let bits = 4_u8;
        let tuples = (0..16)
            .map(|i| {
                let source = (0..16)
                    .map(|dim| ((i * 19 + dim) as f32 * 0.05).cos())
                    .collect::<Vec<_>>();
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: (i + 1) as u16,
                    }],
                    dimensions: 16,
                    bits,
                    seed,
                    gamma: 0.05 * i as f32,
                    code: vec![i as u8; 8],
                    source_vector: Some(source),
                    source_count: 1,
                }
            })
            .collect::<Vec<_>>();
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 2,
                ef_construction: 32,
                ef_search: 40,
                build_source_column: Some("source".to_owned()),
            },
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples,
            dimensions: Some(16),
            bits: Some(bits),
            seed: Some(seed),
        };

        let output = experimental_grouped_v2_flush_output(&state).unwrap();

        assert_eq!(
            output.metadata.format_version,
            page::INDEX_FORMAT_V2_GROUPED
        );
        assert_eq!(output.metadata.search_subvector_count, 1);
        assert_eq!(
            output.metadata.search_subvector_dim,
            ADR030_EXPERIMENTAL_GROUP_SIZE as u16
        );
        assert_ne!(
            output.metadata.grouped_codebook_head,
            page::ItemPointer::INVALID
        );
    }
}
