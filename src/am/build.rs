use std::cmp::Ordering;
use std::collections::HashSet;
use std::ptr;

use hnsw_rs::anndists::dist::distances::Distance;
use hnsw_rs::prelude::Hnsw;
use pgrx::{itemptr::item_pointer_get_both, pg_sys, varlena, FromDatum, PgBox, PgTupleDesc};

use super::{
    average_source_representatives, initialize_metadata_page, options, page, wal, P_NEW,
    tqhnsw_build_callback,
};

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

pub(super) unsafe extern "C-unwind" fn tqhnsw_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut state = BuildState::new(index_relation);

            initialize_metadata_page(index_relation, state.initial_metadata());

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
            initialize_metadata_page(index_relation, state.initial_metadata());
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
        page::MetadataPage {
            m: u16::try_from(self.options.m).expect("validated m should fit into u16"),
            ef_construction: u16::try_from(self.options.ef_construction)
                .expect("validated ef_construction should fit into u16"),
            entry_point: page::ItemPointer::INVALID,
            dimensions: 0,
            bits: 0,
            max_level: 0,
            seed: 0,
        }
    }

    pub(super) fn push(&mut self, tuple: BuildTuple) {
        self.scanned_tuples += tuple.heap_tids.len();

        match (self.dimensions, self.bits, self.seed) {
            (None, None, None) => {
                self.dimensions = Some(tuple.dimensions);
                self.bits = Some(tuple.bits);
                self.seed = Some(tuple.seed);
                if !page::element_tuple_fits_on_page(tuple.code.len(), self.page_size) {
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

        if let Some(existing) = self
            .heap_tuples
            .iter_mut()
            .find(|existing| {
                existing.gamma.to_bits() == tuple.gamma.to_bits() && existing.code == tuple.code
            })
        {
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
                    average_source_representatives(
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

    let original = datum.cast_mut_ptr::<std::ffi::c_void>().cast::<pg_sys::varlena>();
    let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
    let is_copy = !std::ptr::eq(varlena, original);
    let bytes = unsafe { varlena::varlena_to_byte_slice(varlena) }.to_vec();
    if is_copy {
        unsafe { pg_sys::pfree(varlena.cast()) };
    }

    let (dimensions, bits, seed, gamma, code) = crate::unpack(&bytes)
        .unwrap_or_else(|e| pgrx::error!("tqhnsw ambuild found invalid tqvector: {e}"));

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

unsafe fn build_heap_tuple_with_source(
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
    let source_attnum = unsafe { resolve_source_attnum(heap_relation, &source_column) };
    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let att = tuple_desc
        .get(source_attnum as usize - 1)
        .expect("resolved build source attribute should exist");
    if att.attisdropped {
        pgrx::error!("tqhnsw build_source_column \"{source_column}\" references a dropped column");
    }
    if att.atttypid != pg_sys::FLOAT4ARRAYOID {
        pgrx::error!(
            "tqhnsw build_source_column \"{source_column}\" must be real[], got type oid {}",
            u32::from(att.atttypid)
        );
    }

    let slot = unsafe {
        pg_sys::MakeSingleTupleTableSlot(
            (*heap_relation).rd_att,
            pg_sys::table_slot_callbacks(heap_relation),
        )
    };
    if slot.is_null() {
        pgrx::error!("tqhnsw ambuild failed to allocate heap scan slot");
    }

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
            unsafe { required_slot_datum(slot, index_attnum, "indexed tqvector column") };
        let source_datum =
            unsafe { required_slot_datum(slot, source_attnum, "tqhnsw build_source_column") };
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

unsafe fn resolve_source_attnum(heap_relation: pg_sys::Relation, source_column: &str) -> i32 {
    let source_column = std::ffi::CString::new(source_column).unwrap_or_else(|_| {
        pgrx::error!("tqhnsw build_source_column contains an invalid NUL byte")
    });
    let attnum = unsafe { pg_sys::get_attnum((*heap_relation).rd_id, source_column.as_ptr()) };
    let attnum = i32::from(attnum);
    if attnum <= 0 {
        pgrx::error!(
            "tqhnsw build_source_column \"{}\" does not name a user column on the heap relation",
            source_column.to_string_lossy()
        );
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

unsafe fn required_slot_datum(
    slot: *mut pg_sys::TupleTableSlot,
    attnum: i32,
    label: &str,
) -> pg_sys::Datum {
    if unsafe { (*slot).tts_nvalid } < attnum as i16 {
        unsafe { pg_sys::slot_getsomeattrs_int(slot, attnum) };
    }
    let attr_index = usize::try_from(attnum - 1).expect("attribute number should be positive");
    if unsafe { *(*slot).tts_isnull.add(attr_index) } {
        pgrx::error!("tqhnsw does not support NULL {label}");
    }
    unsafe { *(*slot).tts_values.add(attr_index) }
}

#[derive(Debug, Clone)]
pub(super) struct HnswBuildNode {
    pub(super) level: u8,
    pub(super) neighbors: Vec<usize>,
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
    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions");
    let bits = state.bits.expect("non-empty build should record bits");
    let mut data_pages = page::DataPageChain::new(state.page_size);
    let mut element_tids = Vec::with_capacity(state.heap_tuples.len());
    let graph_nodes = build_hnsw_graph(state);

    for (idx, tuple) in state.heap_tuples.iter().enumerate() {
        let element_tid = data_pages
            .insert_element(&page::TqElementTuple {
                level: graph_nodes[idx].level,
                deleted: false,
                heaptids: tuple.heap_tids.clone(),
                neighbortid: page::ItemPointer::INVALID,
                code: tuple.code.clone(),
            })
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to stage element tuple: {e}"));
        element_tids.push(element_tid);
    }

    for (idx, element_tid) in element_tids.iter().copied().enumerate() {
        let neighbor_refs = graph_nodes[idx]
            .neighbors
            .iter()
            .map(|neighbor_idx| element_tids[*neighbor_idx])
            .collect::<Vec<_>>();

        let neighbor_tid = data_pages
            .insert_neighbor(&page::TqNeighborTuple {
                count: neighbor_refs.len() as u16,
                tids: neighbor_refs,
            })
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to stage neighbor tuple: {e}"));
        let mut element = data_pages
            .read_element(element_tid, state.heap_tuples[idx].code.len())
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to read staged element tuple: {e}"));
        element.neighbortid = neighbor_tid;
        data_pages
            .update_element(element_tid, &element)
            .unwrap_or_else(|e| pgrx::error!("tqhnsw failed to backfill element tuple: {e}"));
    }

    let entry_point = choose_entry_point(&element_tids, &graph_nodes, state)
        .unwrap_or(page::ItemPointer::INVALID);
    let max_level = graph_nodes.iter().map(|node| node.level).max().unwrap_or(0);

    unsafe { write_data_pages(index_relation, &data_pages) };
    unsafe {
        initialize_metadata_page(
            index_relation,
            page::MetadataPage {
                m: u16::try_from(state.options.m).expect("validated m should fit into u16"),
                ef_construction: u16::try_from(state.options.ef_construction)
                    .expect("validated ef_construction should fit into u16"),
                entry_point,
                dimensions,
                bits,
                max_level,
                seed: state.seed.expect("non-empty build should record seed"),
            },
        )
    };
}

pub(super) fn build_hnsw_graph(state: &BuildState) -> Vec<HnswBuildNode> {
    if state.heap_tuples.len() <= 1 {
        return vec![
            HnswBuildNode {
                level: 0,
                neighbors: Vec::new(),
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
    let m = usize::try_from(state.options.m).expect("validated m should be non-negative");
    let max_level_cap = page::max_level_that_fits(
        u16::try_from(state.options.m).expect("validated m should fit into u16"),
        state.page_size,
    );
    let max_layer = usize::from(max_level_cap).saturating_add(1).max(1);
    let hnsw = Hnsw::new(
        m,
        state.heap_tuples.len(),
        max_layer,
        usize::try_from(state.options.ef_construction)
            .expect("validated ef_construction should be non-negative"),
        BuildCodeDistance::new(dimensions, bits, seed),
    );

    for (origin_id, tuple) in state.heap_tuples.iter().enumerate() {
        hnsw.insert((&tuple.code, origin_id));
    }

    let mut nodes = vec![
        HnswBuildNode {
            level: 0,
            neighbors: Vec::new(),
        };
        state.heap_tuples.len()
    ];
    for point in hnsw.get_point_indexation().get_layer_iterator(0) {
        let origin_id = point.get_origin_id();
        let level = point.get_point_id().0.min(max_level_cap);
        let neighbors = flatten_point_neighbors(origin_id, level, &point.get_neighborhood_id());
        nodes[origin_id] = HnswBuildNode { level, neighbors };
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
    let hnsw = Hnsw::new(
        m,
        state.heap_tuples.len(),
        max_layer,
        usize::try_from(state.options.ef_construction)
            .expect("validated ef_construction should be non-negative"),
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
            neighbors: Vec::new(),
        };
        state.heap_tuples.len()
    ];
    for point in hnsw.get_point_indexation().get_layer_iterator(0) {
        let origin_id = point.get_origin_id();
        let level = point.get_point_id().0.min(max_level_cap);
        let neighbors = flatten_point_neighbors(origin_id, level, &point.get_neighborhood_id());
        nodes[origin_id] = HnswBuildNode { level, neighbors };
    }

    nodes
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
        .neighbors
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
