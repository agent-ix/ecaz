use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::Instant;

use pgrx::{itemptr::item_pointer_get_both, pg_sys, PgBox};

use crate::quant::{grouped_pq::GROUPED_PQ_CENTROIDS, prod::ProdQuantizer};

use super::{build_parallel, graph, insert, options, page, search, shared, source, P_NEW};
use crate::am::common::training::{self, GroupedPq4Model};
use crate::storage::wal;

const PQ_FASTSCAN_TARGET_GROUP_SIZE: usize = 16;
const PQ_FASTSCAN_DEFAULT_MAX_TRAIN_SIZE: usize = 1024;
const PQ_FASTSCAN_DEFAULT_KMEANS_ITERS: usize = 8;

static LAST_BUILD_REQUESTED_WORKERS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_WORKERS_LAUNCHED: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_HEAP_TUPLES: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_INDEX_TUPLES: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_HEAP_INGEST_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_BEGIN_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_DRAIN_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_SORT_PUSH_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_FLUSH_TOTAL_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_GRAPH_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_STAGE_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_WRITE_US: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct BuildTimingSnapshot {
    pub(crate) requested_workers: u64,
    pub(crate) workers_launched: u64,
    pub(crate) heap_tuples: u64,
    pub(crate) index_tuples: u64,
    pub(crate) heap_ingest_us: u64,
    pub(crate) parallel_begin_us: u64,
    pub(crate) parallel_drain_us: u64,
    pub(crate) parallel_sort_push_us: u64,
    pub(crate) flush_total_us: u64,
    pub(crate) graph_us: u64,
    pub(crate) stage_us: u64,
    pub(crate) write_us: u64,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct BuildTupleDedupKey {
    gamma_bits: u32,
    code: Vec<u8>,
}

impl BuildTupleDedupKey {
    fn from_tuple(tuple: &BuildTuple) -> Self {
        Self {
            gamma_bits: tuple.gamma.to_bits(),
            code: tuple.code.clone(),
        }
    }
}

#[derive(Debug)]
pub(super) struct BuildState {
    pub(super) options: options::TqHnswOptions,
    pub(super) indexed_vector_kind: source::IndexedVectorKind,
    pub(super) page_size: usize,
    pub(super) scanned_tuples: usize,
    pub(super) heap_tuples: Vec<BuildTuple>,
    pub(super) tuple_index_by_payload: HashMap<BuildTupleDedupKey, usize>,
    pub(super) dimensions: Option<u16>,
    pub(super) bits: Option<u8>,
    pub(super) seed: Option<u64>,
}

pub(super) unsafe extern "C-unwind" fn ec_hnsw_build_callback(
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
            let tuple = build_heap_tuple(values, isnull, heap_tid, state.indexed_vector_kind);
            state.push(tuple);
        })
    }
}

pub(crate) fn debug_last_build_timing() -> BuildTimingSnapshot {
    BuildTimingSnapshot {
        requested_workers: LAST_BUILD_REQUESTED_WORKERS.load(AtomicOrdering::Acquire),
        workers_launched: LAST_BUILD_WORKERS_LAUNCHED.load(AtomicOrdering::Acquire),
        heap_tuples: LAST_BUILD_HEAP_TUPLES.load(AtomicOrdering::Acquire),
        index_tuples: LAST_BUILD_INDEX_TUPLES.load(AtomicOrdering::Acquire),
        heap_ingest_us: LAST_BUILD_HEAP_INGEST_US.load(AtomicOrdering::Acquire),
        parallel_begin_us: LAST_BUILD_PARALLEL_BEGIN_US.load(AtomicOrdering::Acquire),
        parallel_drain_us: LAST_BUILD_PARALLEL_DRAIN_US.load(AtomicOrdering::Acquire),
        parallel_sort_push_us: LAST_BUILD_PARALLEL_SORT_PUSH_US.load(AtomicOrdering::Acquire),
        flush_total_us: LAST_BUILD_FLUSH_TOTAL_US.load(AtomicOrdering::Acquire),
        graph_us: LAST_BUILD_GRAPH_US.load(AtomicOrdering::Acquire),
        stage_us: LAST_BUILD_STAGE_US.load(AtomicOrdering::Acquire),
        write_us: LAST_BUILD_WRITE_US.load(AtomicOrdering::Acquire),
    }
}

fn reset_debug_last_build_timing() {
    record_debug_last_build_timing(BuildTimingSnapshot {
        requested_workers: 0,
        workers_launched: 0,
        heap_tuples: 0,
        index_tuples: 0,
        heap_ingest_us: 0,
        parallel_begin_us: 0,
        parallel_drain_us: 0,
        parallel_sort_push_us: 0,
        flush_total_us: 0,
        graph_us: 0,
        stage_us: 0,
        write_us: 0,
    });
}

fn record_debug_last_build_timing(snapshot: BuildTimingSnapshot) {
    LAST_BUILD_REQUESTED_WORKERS.store(snapshot.requested_workers, AtomicOrdering::Release);
    LAST_BUILD_WORKERS_LAUNCHED.store(snapshot.workers_launched, AtomicOrdering::Release);
    LAST_BUILD_HEAP_TUPLES.store(snapshot.heap_tuples, AtomicOrdering::Release);
    LAST_BUILD_INDEX_TUPLES.store(snapshot.index_tuples, AtomicOrdering::Release);
    LAST_BUILD_HEAP_INGEST_US.store(snapshot.heap_ingest_us, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_BEGIN_US.store(snapshot.parallel_begin_us, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_DRAIN_US.store(snapshot.parallel_drain_us, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_SORT_PUSH_US
        .store(snapshot.parallel_sort_push_us, AtomicOrdering::Release);
    LAST_BUILD_FLUSH_TOTAL_US.store(snapshot.flush_total_us, AtomicOrdering::Release);
    LAST_BUILD_GRAPH_US.store(snapshot.graph_us, AtomicOrdering::Release);
    LAST_BUILD_STAGE_US.store(snapshot.stage_us, AtomicOrdering::Release);
    LAST_BUILD_WRITE_US.store(snapshot.write_us, AtomicOrdering::Release);
}

fn elapsed_us(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
}

fn nonnegative_i32_to_u64(value: i32) -> u64 {
    u64::try_from(value.max(0)).expect("non-negative i32 should fit u64")
}

pub(super) unsafe extern "C-unwind" fn ec_hnsw_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut state = BuildState::new(index_relation);
            reset_debug_last_build_timing();
            build_parallel::reset_debug_last_parallel_build_workers_launched();
            let parallel_plan =
                build_parallel::EcHnswParallelBuildPlan::from_index_info(index_info);
            validate_grouped_rerank_source_column(heap_relation, &state.options);

            shared::initialize_metadata_page(index_relation, state.initial_metadata());

            let mut parallel_begin_us = 0_u64;
            let mut parallel_drain_us = 0_u64;
            let mut parallel_sort_push_us = 0_u64;
            let heap_ingest_start = Instant::now();
            let heap_tuples = if !parallel_plan.uses_serial_build_path() {
                build_parallel::try_parallel_build(
                    heap_relation,
                    index_relation,
                    index_info,
                    &mut state,
                    parallel_plan,
                )
                .map(|result| {
                    parallel_begin_us = result.begin_us;
                    parallel_drain_us = result.drain_us;
                    parallel_sort_push_us = result.sort_push_us;
                    result.heap_tuples
                })
                .unwrap_or_else(|| {
                    if state.options.build_source_column.is_some() {
                        ec_hnsw_build_scan_with_source(heap_relation, index_info, &mut state)
                    } else {
                        pg_sys::table_index_build_scan(
                            heap_relation,
                            index_relation,
                            index_info,
                            false,
                            false,
                            Some(ec_hnsw_build_callback),
                            (&mut state as *mut BuildState).cast(),
                            ptr::null_mut(),
                        )
                    }
                })
            } else if state.options.build_source_column.is_some() {
                ec_hnsw_build_scan_with_source(heap_relation, index_info, &mut state)
            } else {
                pg_sys::table_index_build_scan(
                    heap_relation,
                    index_relation,
                    index_info,
                    false,
                    false,
                    Some(ec_hnsw_build_callback),
                    (&mut state as *mut BuildState).cast(),
                    ptr::null_mut(),
                )
            };
            let heap_ingest_us = elapsed_us(heap_ingest_start);

            let mut flush_timing = BuildFlushTiming::default();
            let flush_start = Instant::now();
            let index_tuples = if state.heap_tuples.is_empty() {
                0.0
            } else {
                flush_build_state_with_timing(index_relation, &state, &mut flush_timing);
                state.heap_tuples.len() as f64
            };
            let flush_total_us = elapsed_us(flush_start);

            if heap_tuples != state.scanned_tuples as f64 {
                pgrx::error!(
                    "ec_hnsw ambuild scanned {heap_tuples} heap tuples but observed {}",
                    state.scanned_tuples
                );
            }

            record_debug_last_build_timing(BuildTimingSnapshot {
                requested_workers: nonnegative_i32_to_u64(parallel_plan.requested_workers),
                workers_launched: nonnegative_i32_to_u64(
                    build_parallel::debug_last_parallel_build_workers_launched(),
                ),
                heap_tuples: heap_tuples.max(0.0) as u64,
                index_tuples: index_tuples.max(0.0) as u64,
                heap_ingest_us,
                parallel_begin_us,
                parallel_drain_us,
                parallel_sort_push_us,
                flush_total_us,
                graph_us: flush_timing.graph_us,
                stage_us: flush_timing.stage_us,
                write_us: flush_timing.write_us,
            });

            let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
            result.heap_tuples = heap_tuples;
            result.index_tuples = index_tuples;
            result.into_pg()
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_hnsw_ambuildempty(index_relation: pg_sys::Relation) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = BuildState::new(index_relation);
            validate_grouped_rerank_source_column_for_empty_build(index_relation, &state.options);
            shared::initialize_metadata_page(index_relation, state.initial_metadata());
        })
    }
}

impl BuildState {
    pub(super) fn new(index_relation: pg_sys::Relation) -> Self {
        let options = unsafe { options::relation_options(index_relation) };
        let heap_oid = unsafe { pg_sys::IndexGetRelation((*index_relation).rd_id, false) };
        let indexed_vector_kind = if heap_oid == pg_sys::InvalidOid {
            source::IndexedVectorKind::Ecvector
        } else {
            let heap_relation = unsafe {
                pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE)
            };
            let indexed_attribute = unsafe {
                source::resolve_indexed_vector_attribute(
                    heap_relation,
                    index_relation,
                    "indexed column",
                )
            };
            unsafe {
                pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE)
            };
            indexed_attribute.kind
        };
        Self {
            options,
            indexed_vector_kind,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 0,
            heap_tuples: Vec::new(),
            tuple_index_by_payload: HashMap::new(),
            dimensions: None,
            bits: None,
            seed: None,
        }
    }

    pub(super) fn initial_metadata(&self) -> page::MetadataPage {
        let m = u16::try_from(self.options.m).expect("validated m should fit into u16");
        let ef_construction = u16::try_from(self.options.ef_construction)
            .expect("validated ef_construction should fit into u16");

        match self.options.storage_format {
            options::StorageFormat::TurboQuant => {
                page::MetadataPage::current_v3_turbo_hot_cold(page::CurrentFormatMetadata {
                    m,
                    ef_construction,
                    entry_point: page::ItemPointer::INVALID,
                    dimensions: 0,
                    bits: 0,
                    max_level: 0,
                    seed: 0,
                    inserted_since_rebuild: 0,
                    persisted_binary_sidecar: false,
                })
            }
            options::StorageFormat::PqFastScan => page::MetadataPage {
                m,
                ef_construction,
                entry_point: page::ItemPointer::INVALID,
                dimensions: 0,
                bits: 0,
                max_level: 0,
                seed: 0,
                inserted_since_rebuild: 0,
                format_version: page::INDEX_FORMAT_V2_GROUPED,
                transform_kind: page::TransformKind::Srht,
                search_codec_kind: page::SearchCodecKind::GroupedPq,
                payload_flags: page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE
                    | page::PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
                search_bits: 4,
                rerank_codec_kind: page::RerankCodecKind::ScalarQuantized,
                search_subvector_count: 0,
                search_subvector_dim: 0,
                grouped_codebook_head: page::ItemPointer::INVALID,
            },
        }
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
                let fits_on_page = match self.options.storage_format {
                    options::StorageFormat::TurboQuant => {
                        page::raw_tuple_storage_bytes(page::TqTurboHotTuple::encoded_len(
                            binary_word_count,
                        )) + page::raw_tuple_storage_bytes(page::TqRerankTuple::encoded_len(
                            tuple.code.len(),
                        )) <= self.page_size.saturating_sub(page::PAGE_HEADER_BYTES)
                    }
                    options::StorageFormat::PqFastScan => {
                        page::raw_tuple_storage_bytes(
                            page::TqElementTuple::encoded_len_with_binary(
                                tuple.code.len(),
                                binary_word_count,
                            ),
                        ) <= self.page_size.saturating_sub(page::PAGE_HEADER_BYTES)
                    }
                };
                if !fits_on_page {
                    pgrx::error!(
                        "ec_hnsw tuple payload for dim {} bits {} does not fit on a page",
                        tuple.dimensions,
                        tuple.bits
                    );
                }
            }
            (Some(dimensions), Some(bits), Some(seed)) => {
                if tuple.dimensions != dimensions || tuple.bits != bits || tuple.seed != seed {
                    pgrx::error!(
                        "ec_hnsw ambuild requires a single quantized index shape; saw ({},{},{}) after ({},{},{})",
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

        let dedup_key = BuildTupleDedupKey::from_tuple(&tuple);
        if let Some(existing_idx) = self.tuple_index_by_payload.get(&dedup_key).copied() {
            let existing = self
                .heap_tuples
                .get_mut(existing_idx)
                .expect("build tuple dedup index should point at an existing tuple");
            if existing.heap_tids.len() + tuple.heap_tids.len() > page::HEAPTID_INLINE_CAPACITY {
                pgrx::error!(
                    "ec_hnsw ambuild supports at most {} duplicate heap tids per encoded vector",
                    page::HEAPTID_INLINE_CAPACITY
                );
            }
            existing.heap_tids.extend(tuple.heap_tids);
            match (&mut existing.source_vector, tuple.source_vector) {
                (Some(existing_source), Some(tuple_source)) => {
                    if existing.source_count == 0 || tuple.source_count == 0 {
                        pgrx::error!(
                            "ec_hnsw build_source_column representatives must have non-zero counts"
                        );
                    }
                    if existing_source.len() != tuple_source.len() {
                        pgrx::error!(
                            "ec_hnsw build_source_column representative length mismatch: {} vs {}",
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

        let tuple_idx = self.heap_tuples.len();
        self.heap_tuples.push(tuple);
        self.tuple_index_by_payload.insert(dedup_key, tuple_idx);
    }
}

fn validate_grouped_rerank_source_column(
    heap_relation: pg_sys::Relation,
    options: &options::TqHnswOptions,
) {
    let Some(source_column) = options.rerank_source_column.as_deref() else {
        return;
    };

    unsafe {
        source::resolve_source_attribute(
            heap_relation,
            source_column,
            "rerank_source_column",
            source::SourceTypePolicy::RerankSource,
        )
    };
}

fn validate_grouped_rerank_source_column_for_empty_build(
    index_relation: pg_sys::Relation,
    options: &options::TqHnswOptions,
) {
    if options.rerank_source_column.is_none() {
        return;
    }

    let heap_oid = unsafe { pg_sys::IndexGetRelation((*index_relation).rd_id, false) };
    if heap_oid == pg_sys::InvalidOid {
        pgrx::error!("ec_hnsw rerank_source_column could not resolve heap relation for validation");
    }
    let heap_relation =
        unsafe { pg_sys::table_open(heap_oid, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
    validate_grouped_rerank_source_column(heap_relation, options);
    unsafe { pg_sys::table_close(heap_relation, pg_sys::AccessShareLock as pg_sys::LOCKMODE) };
}

fn persisted_binary_sidecar_word_count(dimensions: u16, bits: u8, seed: u64) -> usize {
    crate::quant::rabitq::persisted_sidecar_word_count(dimensions, bits, seed)
}

pub(super) unsafe fn build_heap_tuple(
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: page::ItemPointer,
    indexed_vector_kind: source::IndexedVectorKind,
) -> BuildTuple {
    if values.is_null() || isnull.is_null() {
        pgrx::error!("ec_hnsw ambuild received null tuple value arrays");
    }
    if unsafe { *isnull } {
        pgrx::error!("ec_hnsw does not support NULL indexed values");
    }

    let datum = unsafe { *values };
    if datum.is_null() {
        pgrx::error!("ec_hnsw ambuild received a null indexed datum");
    }

    unsafe { build_heap_tuple_from_indexed_datum(datum, heap_tid, indexed_vector_kind, None) }
}

fn build_quantized_build_tuple(
    dimensions: u16,
    gamma: f32,
    code: Vec<u8>,
    heap_tid: page::ItemPointer,
    source_vector: Option<Vec<f32>>,
) -> BuildTuple {
    if !gamma.is_finite() {
        pgrx::error!("ec_hnsw does not support non-finite gamma values");
    }

    if let Some(source_vector) = source_vector {
        if source_vector.is_empty() {
            pgrx::error!("ec_hnsw build_source_column vectors must not be empty");
        }
        if source_vector.len() != dimensions as usize {
            pgrx::error!(
                "ec_hnsw build_source_column dimension mismatch: source dim {} vs indexed ecvector dim {}",
                source_vector.len(),
                dimensions
            );
        }

        return BuildTuple {
            heap_tids: vec![heap_tid],
            dimensions,
            bits: crate::DEFAULT_QUANT_BITS,
            seed: crate::DEFAULT_QUANT_SEED,
            gamma,
            code,
            source_vector: Some(source_vector),
            source_count: 1,
        };
    }

    BuildTuple {
        heap_tids: vec![heap_tid],
        dimensions,
        bits: crate::DEFAULT_QUANT_BITS,
        seed: crate::DEFAULT_QUANT_SEED,
        gamma,
        code,
        source_vector: None,
        source_count: 0,
    }
}

unsafe fn build_heap_tuple_from_indexed_datum(
    vector_datum: pg_sys::Datum,
    heap_tid: page::ItemPointer,
    indexed_vector_kind: source::IndexedVectorKind,
    source_vector: Option<Vec<f32>>,
) -> BuildTuple {
    match indexed_vector_kind {
        source::IndexedVectorKind::Ecvector => {
            let indexed_vector = unsafe {
                source::FlatFloat4SourceRef::from_datum(
                    vector_datum,
                    source::SourceDatumKind::Ecvector,
                    "indexed ecvector column",
                )
            };
            let index_vector = indexed_vector.as_slice().to_vec();
            drop(indexed_vector);
            let (dimensions, gamma, code) = crate::quantize_embedding_to_code(
                &index_vector,
                crate::DEFAULT_QUANT_BITS,
                crate::DEFAULT_QUANT_SEED,
            )
            .unwrap_or_else(|e| {
                pgrx::error!("ec_hnsw ambuild found invalid indexed ecvector: {e}")
            });
            let source_vector = Some(source_vector.unwrap_or(index_vector));
            build_quantized_build_tuple(dimensions, gamma, code, heap_tid, source_vector)
        }
        source::IndexedVectorKind::Tqvector => {
            let original = vector_datum
                .cast_mut_ptr::<std::ffi::c_void>()
                .cast::<pg_sys::varlena>();
            let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
            let is_copy = !std::ptr::eq(varlena, original);
            let bytes = unsafe { pgrx::varlena::varlena_to_byte_slice(varlena) }.to_vec();
            if is_copy {
                unsafe { pg_sys::pfree(varlena.cast()) };
            }

            let (dimensions, bits, seed, gamma, code) = crate::unpack(&bytes).unwrap_or_else(|e| {
                pgrx::error!("ec_hnsw ambuild found invalid indexed tqvector: {e}")
            });
            if bits != crate::DEFAULT_QUANT_BITS || seed != crate::DEFAULT_QUANT_SEED {
                pgrx::error!(
                    "ec_hnsw indexed tqvector must use the canonical quantizer defaults ({},{}), got ({},{})",
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                    bits,
                    seed
                );
            }
            build_quantized_build_tuple(dimensions, gamma, code.to_vec(), heap_tid, source_vector)
        }
    }
}

pub(super) unsafe fn build_heap_tuple_with_source(
    vector_datum: pg_sys::Datum,
    heap_tid: page::ItemPointer,
    source_vector: Vec<f32>,
    indexed_vector_kind: source::IndexedVectorKind,
) -> BuildTuple {
    if vector_datum.is_null() {
        pgrx::error!("ec_hnsw ambuild received a null indexed datum");
    }
    unsafe {
        build_heap_tuple_from_indexed_datum(
            vector_datum,
            heap_tid,
            indexed_vector_kind,
            Some(source_vector),
        )
    }
}

pub(super) unsafe fn ec_hnsw_build_scan_with_source(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    state: &mut BuildState,
) -> f64 {
    let source_column = state
        .options
        .build_source_column
        .clone()
        .expect("source scan should only run when build_source_column is configured");
    let indexed_attribute = unsafe {
        source::resolve_indexed_vector_attribute_from_index_info(
            heap_relation,
            index_info,
            "indexed column",
        )
    };
    let source_attribute = unsafe {
        source::resolve_source_attribute(
            heap_relation,
            &source_column,
            "build_source_column",
            source::SourceTypePolicy::BuildSource,
        )
    };

    let slot = unsafe {
        source::allocate_heap_slot(
            heap_relation,
            "ec_hnsw ambuild failed to allocate heap scan slot",
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
        pgrx::error!("ec_hnsw ambuild failed to begin heap scan");
    }

    let mut scanned_tuples = 0.0_f64;
    while unsafe {
        pg_sys::heap_getnextslot(scan, pg_sys::ScanDirection::ForwardScanDirection, slot)
    } {
        scanned_tuples += 1.0;
        let heap_tid = unsafe { decode_slot_tid(slot) };
        let vector_datum = unsafe {
            source::required_slot_datum(slot, indexed_attribute.attnum, "indexed column")
        };
        let source_datum = unsafe {
            source::required_slot_datum(
                slot,
                source_attribute.attnum,
                "ec_hnsw build_source_column",
            )
        };
        let source_vector = unsafe {
            source::FlatFloat4SourceRef::from_datum(
                source_datum,
                source_attribute.kind,
                "ec_hnsw build_source_column",
            )
        };
        let source_vector = source_vector.as_slice().to_vec();

        let tuple = unsafe {
            build_heap_tuple_with_source(
                vector_datum,
                heap_tid,
                source_vector,
                indexed_attribute.kind,
            )
        };
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

#[derive(Debug, Clone, PartialEq)]
pub(super) struct V3TurboHotBuildPayload {
    pub(super) hot: page::TqTurboHotTuple,
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
    let binary_words = crate::quant::rabitq::derive_persisted_sidecar_words(
        persisted_binary_quantizer,
        &tuple.code,
    );

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

pub(super) fn stage_v3_turbo_hot_build_payload(
    tuple: &BuildTuple,
    level: u8,
    neighbortid: page::ItemPointer,
    reranktid: page::ItemPointer,
    persisted_binary_quantizer: &ProdQuantizer,
) -> V3TurboHotBuildPayload {
    let binary_words = crate::quant::rabitq::derive_persisted_sidecar_words(
        persisted_binary_quantizer,
        &tuple.code,
    );

    V3TurboHotBuildPayload {
        hot: page::TqTurboHotTuple {
            level,
            deleted: false,
            heaptids: tuple.heap_tids.clone(),
            neighbortid,
            reranktid,
            binary_words,
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

pub(super) type BuildGroupedPqModel = GroupedPq4Model;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
struct BuildFlushTiming {
    graph_us: u64,
    stage_us: u64,
    write_us: u64,
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
    let source_vectors = state
        .heap_tuples
        .iter()
        .map(|tuple| {
            tuple
                .source_vector
                .as_ref()
                .ok_or_else(|| "grouped build model requires source vectors".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(Vec::as_slice)
        .collect::<Vec<_>>();

    training::train_grouped_pq4_model(
        &source_vectors,
        dimensions,
        seed,
        group_size,
        train_size,
        kmeans_iters,
    )
}

pub(super) fn derive_grouped_search_code_from_source(
    tuple: &BuildTuple,
    model: &BuildGroupedPqModel,
) -> Result<Vec<u8>, String> {
    let source = tuple
        .source_vector
        .as_ref()
        .ok_or_else(|| "grouped search code derivation requires source vector".to_owned())?;
    Ok(training::derive_grouped_pq4_code(source, model))
}

#[derive(Debug, Clone, Copy)]
enum BuildGraphMetric {
    Code {
        dimensions: usize,
        bits: u8,
        seed: u64,
    },
    Source,
}

impl BuildGraphMetric {
    fn for_state(state: &BuildState) -> Self {
        if state
            .heap_tuples
            .iter()
            .all(|tuple| tuple.source_vector.is_some())
        {
            Self::Source
        } else {
            Self::Code {
                dimensions: state
                    .dimensions
                    .expect("non-empty build should record dimensions")
                    as usize,
                bits: state.bits.expect("non-empty build should record bits"),
                seed: state.seed.expect("non-empty build should record seed"),
            }
        }
    }

    fn score_between(self, state: &BuildState, left_idx: usize, right_idx: usize) -> f32 {
        match self {
            Self::Code {
                dimensions,
                bits,
                seed,
            } => -crate::score_code_inner_product(
                dimensions,
                bits,
                seed,
                &state.heap_tuples[left_idx].code,
                &state.heap_tuples[right_idx].code,
            ),
            Self::Source => -score_source_inner_product(
                state.heap_tuples[left_idx]
                    .source_vector
                    .as_ref()
                    .expect("source graph build requires source vectors"),
                state.heap_tuples[right_idx]
                    .source_vector
                    .as_ref()
                    .expect("source graph build requires source vectors"),
            ),
        }
    }
}

struct NativeBuildQueryScorer<'a> {
    state: &'a BuildState,
    metric: BuildGraphMetric,
    query_idx: usize,
    cache: &'a mut NativeBuildQueryScoreCache,
}

impl<'a> NativeBuildQueryScorer<'a> {
    fn new(
        state: &'a BuildState,
        metric: BuildGraphMetric,
        query_idx: usize,
        cache: &'a mut NativeBuildQueryScoreCache,
    ) -> Self {
        cache.begin_query();
        Self {
            state,
            metric,
            query_idx,
            cache,
        }
    }

    fn score(&mut self, candidate_idx: usize) -> f32 {
        if let Some(score) = self.cache.get(candidate_idx) {
            return score;
        }

        let score = self
            .metric
            .score_between(self.state, self.query_idx, candidate_idx);
        self.cache.insert(candidate_idx, score);
        score
    }
}

struct NativeBuildQueryScoreCache {
    scores: Vec<f32>,
    generations: Vec<u32>,
    current_generation: u32,
}

impl NativeBuildQueryScoreCache {
    fn new(capacity: usize) -> Self {
        Self {
            scores: vec![0.0; capacity],
            generations: vec![0; capacity],
            current_generation: 0,
        }
    }

    fn begin_query(&mut self) {
        self.current_generation = self.current_generation.wrapping_add(1);
        if self.current_generation == 0 {
            self.generations.fill(0);
            self.current_generation = 1;
        }
    }

    fn get(&self, candidate_idx: usize) -> Option<f32> {
        if self.generations[candidate_idx] == self.current_generation {
            Some(self.scores[candidate_idx])
        } else {
            None
        }
    }

    fn insert(&mut self, candidate_idx: usize, score: f32) {
        self.scores[candidate_idx] = score;
        self.generations[candidate_idx] = self.current_generation;
    }
}

struct NativeBuildLayerSearchScratch {
    visited: NativeBuildVisitedSet,
    candidate_points: BinaryHeap<Reverse<NativeBuildLayerSearchCandidate>>,
    result_points: BinaryHeap<NativeBuildLayerSearchCandidate>,
    successors: Vec<search::BeamCandidate<usize>>,
}

impl NativeBuildLayerSearchScratch {
    fn new(node_count: usize, ef_construction: usize, m: usize) -> Self {
        Self {
            visited: NativeBuildVisitedSet::new(node_count),
            candidate_points: BinaryHeap::with_capacity(ef_construction),
            result_points: BinaryHeap::with_capacity(ef_construction.saturating_add(1)),
            successors: Vec::with_capacity(m.saturating_mul(2)),
        }
    }

    fn clear(&mut self) {
        self.visited.begin_search();
        self.candidate_points.clear();
        self.result_points.clear();
        self.successors.clear();
    }
}

struct NativeBuildVisitedSet {
    generations: Vec<u32>,
    current_generation: u32,
}

impl NativeBuildVisitedSet {
    fn new(capacity: usize) -> Self {
        Self {
            generations: vec![0; capacity],
            current_generation: 0,
        }
    }

    fn begin_search(&mut self) {
        self.current_generation = self.current_generation.wrapping_add(1);
        if self.current_generation == 0 {
            self.generations.fill(0);
            self.current_generation = 1;
        }
    }

    fn insert(&mut self, node_idx: usize) -> bool {
        let generation = self
            .generations
            .get_mut(node_idx)
            .expect("native build visited set should cover all graph nodes");
        if *generation == self.current_generation {
            return false;
        }
        *generation = self.current_generation;
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct NativeBuildLayerSearchCandidate {
    candidate: search::BeamCandidate<usize>,
    sequence: u64,
}

impl NativeBuildLayerSearchCandidate {
    fn new(candidate: search::BeamCandidate<usize>, sequence: u64) -> Self {
        Self {
            candidate,
            sequence,
        }
    }
}

impl Eq for NativeBuildLayerSearchCandidate {}

impl Ord for NativeBuildLayerSearchCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.candidate
            .score
            .total_cmp(&other.candidate.score)
            .then_with(|| self.sequence.cmp(&other.sequence))
    }
}

impl PartialOrd for NativeBuildLayerSearchCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct NativeBacklinkTargetScorer<'a> {
    state: &'a BuildState,
    metric: BuildGraphMetric,
    target_idx: usize,
    cache: Vec<(usize, f32)>,
}

impl<'a> NativeBacklinkTargetScorer<'a> {
    fn new(state: &'a BuildState, metric: BuildGraphMetric, cache_capacity: usize) -> Self {
        Self {
            state,
            metric,
            target_idx: 0,
            cache: Vec::with_capacity(cache_capacity),
        }
    }

    fn reset_target(&mut self, target_idx: usize) {
        self.target_idx = target_idx;
        self.cache.clear();
    }

    fn score(&mut self, candidate_idx: usize) -> f32 {
        if let Some((_, score)) = self
            .cache
            .iter()
            .find(|(cached_idx, _)| *cached_idx == candidate_idx)
        {
            return *score;
        }
        let score = self
            .metric
            .score_between(self.state, self.target_idx, candidate_idx);
        self.cache.push((candidate_idx, score));
        score
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeBuildNode {
    level: u8,
    neighbor_slots: Vec<Option<usize>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NativeForwardSelection {
    layer: u8,
    node_idx: usize,
}

pub(super) unsafe fn flush_build_state(index_relation: pg_sys::Relation, state: &BuildState) {
    let mut timing = BuildFlushTiming::default();
    unsafe { flush_build_state_with_timing(index_relation, state, &mut timing) };
}

unsafe fn flush_build_state_with_timing(
    index_relation: pg_sys::Relation,
    state: &BuildState,
    timing: &mut BuildFlushTiming,
) {
    let output = match state.options.storage_format {
        options::StorageFormat::TurboQuant => current_format_flush_output(state, timing),
        options::StorageFormat::PqFastScan => default_pq_fastscan_flush_output(state)
            .unwrap_or_else(|e| pgrx::error!("ec_hnsw pq_fastscan build failed: {e}")),
    };
    let write_start = Instant::now();
    unsafe { flush_build_output(index_relation, &output) };
    timing.write_us = elapsed_us(write_start);
}

pub(super) fn pq_fastscan_flush_output(
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

pub(super) fn default_pq_fastscan_flush_output(
    state: &BuildState,
) -> Result<BuildFlushOutput, String> {
    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions");
    let group_size = default_pq_fastscan_group_size(dimensions);
    let train_size = state
        .heap_tuples
        .len()
        .min(PQ_FASTSCAN_DEFAULT_MAX_TRAIN_SIZE);
    let plan = plan_v2_grouped_source_build(
        state,
        group_size,
        train_size,
        PQ_FASTSCAN_DEFAULT_KMEANS_ITERS,
    )?;
    pq_fastscan_flush_output(state, &plan, group_size)
}

fn default_pq_fastscan_group_size(dimensions: u16) -> usize {
    let transform_dim = crate::quant::rotation::effective_transform_dim(dimensions as usize);
    transform_dim.min(PQ_FASTSCAN_TARGET_GROUP_SIZE)
}

fn current_format_flush_output(
    state: &BuildState,
    timing: &mut BuildFlushTiming,
) -> BuildFlushOutput {
    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions");
    let bits = state.bits.expect("non-empty build should record bits");
    let mut data_pages = page::DataPageChain::new(state.page_size);
    let mut hot_tids = Vec::with_capacity(state.heap_tuples.len());
    let mut neighbor_tids = Vec::with_capacity(state.heap_tuples.len());
    let graph_start = Instant::now();
    let graph_nodes = build_hnsw_graph(state);
    timing.graph_us = elapsed_us(graph_start);
    let stage_start = Instant::now();
    let persisted_binary_quantizer = ProdQuantizer::cached(
        dimensions as usize,
        bits,
        state.seed.expect("non-empty build should record seed"),
    );

    for (idx, tuple) in state.heap_tuples.iter().enumerate() {
        let slot_count = graph_nodes[idx].neighbor_slots.len();
        let placeholder_neighbor = page::TqNeighborTuple {
            count: slot_count as u16,
            tids: vec![page::ItemPointer::INVALID; slot_count],
        };
        let neighbor_tid = data_pages
            .insert_neighbor(&placeholder_neighbor)
            .unwrap_or_else(|e| pgrx::error!("ec_hnsw failed to stage neighbor tuple: {e}"));

        let rerank_tid = data_pages
            .insert_rerank(&page::TqRerankTuple {
                gamma: tuple.gamma,
                code: tuple.code.clone(),
            })
            .unwrap_or_else(|e| pgrx::error!("ec_hnsw failed to stage rerank tuple: {e}"));
        let payload = stage_v3_turbo_hot_build_payload(
            tuple,
            graph_nodes[idx].level,
            neighbor_tid,
            rerank_tid,
            &persisted_binary_quantizer,
        );
        let hot_tid = data_pages
            .insert_turbo_hot(&payload.hot)
            .unwrap_or_else(|e| pgrx::error!("ec_hnsw failed to stage turbo hot tuple: {e}"));

        hot_tids.push(hot_tid);
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

        data_pages
            .update_neighbor(
                neighbor_tid,
                &page::TqNeighborTuple {
                    count: neighbor_refs.len() as u16,
                    tids: neighbor_refs,
                },
            )
            .unwrap_or_else(|e| pgrx::error!("ec_hnsw failed to update neighbor tuple: {e}"));
    }

    let entry_point =
        choose_entry_point(&hot_tids, &graph_nodes, state).unwrap_or(page::ItemPointer::INVALID);
    let max_level = graph_nodes.iter().map(|node| node.level).max().unwrap_or(0);
    let seed = state.seed.expect("non-empty build should record seed");
    timing.stage_us = elapsed_us(stage_start);

    BuildFlushOutput {
        data_pages,
        metadata: page::MetadataPage::current_v3_turbo_hot_cold(page::CurrentFormatMetadata {
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
    build_native_hnsw_graph(state, BuildGraphMetric::for_state(state))
}

fn empty_neighbor_slots(level: u8, m: usize) -> Vec<Option<usize>> {
    vec![None; page::neighbor_slots(level, m as u16)]
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

fn build_native_hnsw_graph(state: &BuildState, metric: BuildGraphMetric) -> Vec<HnswBuildNode> {
    let m = usize::try_from(state.options.m).expect("validated m should be non-negative");
    let m_u16 = u16::try_from(state.options.m).expect("validated m should fit into u16");
    let ef_construction = usize::try_from(state.options.ef_construction)
        .expect("validated ef_construction should be non-negative");
    debug_assert_eq!(
        state.page_size,
        pg_sys::BLCKSZ as usize,
        "native serial build currently assumes BLCKSZ-sized pages when planning graph levels",
    );
    let mut nodes = Vec::with_capacity(state.heap_tuples.len());
    let mut entry_idx = None;
    let mut max_level = 0_u8;
    let mut query_score_cache = NativeBuildQueryScoreCache::new(state.heap_tuples.len());
    let mut layer_search_scratch =
        NativeBuildLayerSearchScratch::new(state.heap_tuples.len(), ef_construction, m);

    for node_idx in 0..state.heap_tuples.len() {
        let level = insert::choose_insert_level_for_page_size(
            m_u16,
            state.seed.expect("non-empty build should record seed"),
            state.heap_tuples[node_idx].heap_tids[0],
            state.heap_tuples[node_idx].code.len(),
            state.page_size,
        );
        let mut node = NativeBuildNode {
            level,
            neighbor_slots: empty_neighbor_slots(level, m),
        };

        if let Some(current_entry_idx) = entry_idx {
            let mut query_scorer =
                NativeBuildQueryScorer::new(state, metric, node_idx, &mut query_score_cache);
            let entry_candidate = search::BeamCandidate::new(
                current_entry_idx,
                query_scorer.score(current_entry_idx),
            );
            let (mut selections, layer0_seeds) = populate_native_upper_layer_forward_slots(
                &mut node.neighbor_slots,
                &nodes,
                &mut query_scorer,
                level,
                entry_candidate,
                max_level,
                m,
                ef_construction,
                &mut layer_search_scratch,
            );
            let layer0_candidates = search_native_layer_result_candidates(
                ef_construction,
                layer0_seeds,
                &nodes,
                &mut query_scorer,
                0,
                m,
                &mut layer_search_scratch,
            );
            write_native_layer_forward_candidates(
                &mut node.neighbor_slots,
                &mut selections,
                0,
                m,
                layer0_candidates,
            );
            add_native_backlinks(&mut nodes, state, metric, node_idx, &selections, m);
        }

        nodes.push(node);
        if entry_idx.is_none() || level > max_level {
            entry_idx = Some(node_idx);
            max_level = level;
        }
    }

    nodes
        .into_iter()
        .enumerate()
        .map(|(node_idx, node)| HnswBuildNode {
            level: node.level,
            score_neighbors: flatten_native_neighbor_slots(
                node_idx,
                node.level,
                m,
                &node.neighbor_slots,
            ),
            neighbor_slots: node.neighbor_slots,
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn populate_native_upper_layer_forward_slots(
    slots: &mut [Option<usize>],
    nodes: &[NativeBuildNode],
    query_scorer: &mut NativeBuildQueryScorer<'_>,
    insert_level: u8,
    entry_candidate: search::BeamCandidate<usize>,
    entry_level: u8,
    m: usize,
    ef_construction: usize,
    scratch: &mut NativeBuildLayerSearchScratch,
) -> (
    Vec<NativeForwardSelection>,
    Vec<search::BeamCandidate<usize>>,
) {
    let mut selections = Vec::new();
    if entry_level == 0 {
        return (selections, vec![entry_candidate]);
    }

    // Native build intentionally uses the same ef_construction-width successor
    // search on upper layers that it uses at layer 0. This is more expensive
    // than classic greedy ef=1 descent, but it preserves the current
    // higher-recall serial build behavior while FR-021 parallelization stays
    // out of scope for this task.
    let mut seeds = vec![entry_candidate];
    for current_layer in (1..=entry_level).rev() {
        seeds = search_native_layer_result_candidates(
            ef_construction,
            seeds,
            nodes,
            query_scorer,
            current_layer,
            m,
            scratch,
        );
        if current_layer <= insert_level {
            write_native_layer_forward_candidates(
                slots,
                &mut selections,
                current_layer,
                m,
                seeds.iter().copied(),
            );
        }
        if seeds.is_empty() {
            break;
        }
    }

    (selections, seeds)
}

fn search_native_layer_result_candidates(
    ef_search: usize,
    seeds: impl IntoIterator<Item = search::BeamCandidate<usize>>,
    nodes: &[NativeBuildNode],
    query_scorer: &mut NativeBuildQueryScorer<'_>,
    layer: u8,
    m: usize,
    scratch: &mut NativeBuildLayerSearchScratch,
) -> Vec<search::BeamCandidate<usize>> {
    if ef_search == 0 {
        return Vec::new();
    }

    scratch.clear();
    let mut sequence = 0_u64;

    for seed in seeds {
        if !scratch.visited.insert(seed.node) {
            continue;
        }

        let queued = NativeBuildLayerSearchCandidate::new(seed, sequence);
        scratch.candidate_points.push(Reverse(queued));
        scratch.result_points.push(queued);
        sequence += 1;
    }

    while let Some(Reverse(candidate)) = scratch.candidate_points.pop() {
        let Some(worst_result) = scratch.result_points.peek() else {
            break;
        };

        if scratch.result_points.len() >= ef_search
            && candidate.candidate.score > worst_result.candidate.score
        {
            break;
        }

        load_native_successor_candidates_into(
            nodes,
            query_scorer,
            candidate.candidate.node,
            layer,
            m,
            &mut scratch.successors,
        );
        for idx in 0..scratch.successors.len() {
            let neighbor = scratch.successors[idx];
            if !scratch.visited.insert(neighbor.node) {
                continue;
            }

            let should_enqueue = scratch.result_points.len() < ef_search
                || scratch
                    .result_points
                    .peek()
                    .map(|worst| neighbor.score < worst.candidate.score)
                    .unwrap_or(true);
            if !should_enqueue {
                continue;
            }

            let queued = NativeBuildLayerSearchCandidate::new(neighbor, sequence);
            sequence += 1;
            scratch.candidate_points.push(Reverse(queued));
            scratch.result_points.push(queued);
            if scratch.result_points.len() > ef_search {
                scratch.result_points.pop();
            }
        }
    }

    let mut results = Vec::with_capacity(scratch.result_points.len());
    while let Some(queued) = scratch.result_points.pop() {
        results.push(queued.candidate);
    }
    results.sort_by(|left, right| left.score.total_cmp(&right.score));
    results
}

fn load_native_successor_candidates_into(
    nodes: &[NativeBuildNode],
    query_scorer: &mut NativeBuildQueryScorer<'_>,
    source_idx: usize,
    layer: u8,
    m: usize,
    out: &mut Vec<search::BeamCandidate<usize>>,
) {
    out.clear();
    let Some((start, end)) = graph::layer_slot_bounds(nodes[source_idx].level, m, layer) else {
        return;
    };
    debug_assert!(
        end <= nodes[source_idx].neighbor_slots.len(),
        "layer slot bounds should stay within the persisted neighbor slot slice",
    );

    out.reserve(end.saturating_sub(start));
    for neighbor_idx in &nodes[source_idx].neighbor_slots[start..end] {
        let Some(neighbor_idx) = *neighbor_idx else {
            continue;
        };
        out.push(search::BeamCandidate::new(
            neighbor_idx,
            query_scorer.score(neighbor_idx),
        ));
    }
}

fn write_native_layer_forward_candidates(
    slots: &mut [Option<usize>],
    selections: &mut Vec<NativeForwardSelection>,
    layer: u8,
    m: usize,
    candidates: impl IntoIterator<Item = search::BeamCandidate<usize>>,
) {
    let Some((start, end)) = insert::selected_forward_slot_bounds(m, slots.len(), layer) else {
        return;
    };

    for (slot, candidate) in slots[start..end]
        .iter_mut()
        .zip(candidates.into_iter().take(end.saturating_sub(start)))
    {
        *slot = Some(candidate.node);
        selections.push(NativeForwardSelection {
            layer,
            node_idx: candidate.node,
        });
    }
}

fn add_native_backlinks(
    nodes: &mut [NativeBuildNode],
    state: &BuildState,
    metric: BuildGraphMetric,
    new_node_idx: usize,
    selections: &[NativeForwardSelection],
    m: usize,
) {
    let mut pending = selections.to_vec();
    pending.sort_unstable_by(|left, right| {
        left.node_idx
            .cmp(&right.node_idx)
            .then_with(|| left.layer.cmp(&right.layer))
    });
    pending.dedup();
    let mut target_scorer = NativeBacklinkTargetScorer::new(
        state,
        metric,
        m.saturating_mul(2).saturating_add(1),
    );
    let mut current_target_idx = None;

    for selection in pending {
        if current_target_idx != Some(selection.node_idx) {
            target_scorer.reset_target(selection.node_idx);
            current_target_idx = Some(selection.node_idx);
        }
        let Some((start, end)) = insert::backlink_slot_bounds(
            m,
            nodes[selection.node_idx].neighbor_slots.len(),
            selection.layer,
        ) else {
            continue;
        };
        let layer_slice = &mut nodes[selection.node_idx].neighbor_slots[start..end];
        if layer_slice.contains(&Some(new_node_idx)) {
            continue;
        }
        if let Some(slot) = layer_slice.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(new_node_idx);
            continue;
        }

        let mut candidates = layer_slice
            .iter()
            .filter_map(|slot| {
                let neighbor_idx = (*slot)?;
                Some(insert::ScoredBacklinkNode {
                    node: neighbor_idx,
                    score: target_scorer.score(neighbor_idx),
                    is_new: false,
                })
            })
            .collect::<Vec<_>>();
        candidates.push(insert::ScoredBacklinkNode {
            node: new_node_idx,
            score: target_scorer.score(new_node_idx),
            is_new: true,
        });
        let replacement =
            insert::select_best_backlink_candidates(candidates, layer_slice.len(), usize::cmp);
        if !replacement.contains(&new_node_idx) {
            continue;
        }
        for (slot, replacement_idx) in layer_slice.iter_mut().zip(replacement.into_iter()) {
            *slot = Some(replacement_idx);
        }
    }
}

fn flatten_native_neighbor_slots(
    origin_id: usize,
    level: u8,
    m: usize,
    slots: &[Option<usize>],
) -> Vec<usize> {
    let mut seen = HashSet::new();
    let mut flattened = Vec::new();

    for layer in 0..=level {
        let Some((start, end)) = graph::layer_slot_bounds(level, m, layer) else {
            continue;
        };
        for neighbor_idx in slots[start.min(slots.len())..end.min(slots.len())]
            .iter()
            .flatten()
        {
            if *neighbor_idx != origin_id && seen.insert(*neighbor_idx) {
                flattened.push(*neighbor_idx);
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

pub(super) unsafe fn write_data_pages(
    index_relation: pg_sys::Relation,
    data_pages: &page::DataPageChain,
) {
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
                "ec_hnsw failed to allocate data buffer for block {}",
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
                    "ec_hnsw failed to write tuple to block {}",
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
    fn build_state_push_indexes_payloads_for_duplicate_coalescing() {
        let mut state = BuildState {
            options: options::TqHnswOptions {
                m: 8,
                ef_construction: 64,
                ef_search: 40,
                build_source_column: Some("source".to_owned()),
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 0,
            heap_tuples: Vec::new(),
            tuple_index_by_payload: HashMap::new(),
            dimensions: None,
            bits: None,
            seed: None,
        };

        state.push(BuildTuple {
            heap_tids: vec![page::ItemPointer {
                block_number: 1,
                offset_number: 1,
            }],
            dimensions: 2,
            bits: 4,
            seed: 42,
            gamma: 0.25,
            code: vec![0xAA, 0xBB],
            source_vector: Some(vec![1.0, 0.0]),
            source_count: 1,
        });
        state.push(BuildTuple {
            heap_tids: vec![page::ItemPointer {
                block_number: 1,
                offset_number: 2,
            }],
            dimensions: 2,
            bits: 4,
            seed: 42,
            gamma: 0.25,
            code: vec![0xAA, 0xBB],
            source_vector: Some(vec![0.0, 1.0]),
            source_count: 1,
        });
        state.push(BuildTuple {
            heap_tids: vec![page::ItemPointer {
                block_number: 1,
                offset_number: 3,
            }],
            dimensions: 2,
            bits: 4,
            seed: 42,
            gamma: 0.25,
            code: vec![0xAA, 0xBC],
            source_vector: Some(vec![0.0, 1.0]),
            source_count: 1,
        });

        assert_eq!(state.scanned_tuples, 3);
        assert_eq!(state.heap_tuples.len(), 2);
        assert_eq!(state.tuple_index_by_payload.len(), 2);
        assert_eq!(
            state.heap_tuples[0].heap_tids,
            vec![
                page::ItemPointer {
                    block_number: 1,
                    offset_number: 1,
                },
                page::ItemPointer {
                    block_number: 1,
                    offset_number: 2,
                },
            ]
        );
        assert_eq!(state.heap_tuples[0].source_vector, Some(vec![0.5, 0.5]));
        assert_eq!(state.heap_tuples[0].source_count, 2);
    }

    #[test]
    fn initial_metadata_uses_grouped_format_for_pq_fastscan_indexes() {
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 8,
                ef_construction: 64,
                ef_search: 40,
                build_source_column: Some("source".to_owned()),
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 0,
            heap_tuples: Vec::new(),
            tuple_index_by_payload: HashMap::new(),
            dimensions: None,
            bits: None,
            seed: None,
        };

        let metadata = state.initial_metadata();
        assert_eq!(metadata.format_version, page::INDEX_FORMAT_V2_GROUPED);
        assert_eq!(metadata.search_codec_kind, page::SearchCodecKind::GroupedPq);
        assert_eq!(
            metadata.rerank_codec_kind,
            page::RerankCodecKind::ScalarQuantized
        );
        assert_eq!(metadata.search_subvector_count, 0);
        assert_eq!(metadata.search_subvector_dim, 0);
        assert_eq!(metadata.grouped_codebook_head, page::ItemPointer::INVALID);
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::TurboQuant,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 3,
            heap_tuples: tuples,
            tuple_index_by_payload: HashMap::new(),
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::TurboQuant,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 3,
            heap_tuples: tuples,
            tuple_index_by_payload: HashMap::new(),
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
    fn hnsw_graph_builds_for_qjl_enabled_scalar_codes() {
        let seed = 42_u64;
        let bits = 4_u8;
        let tuples = (1_i64..=16_i64)
            .map(|id| {
                let vector = (0_i64..16_i64)
                    .map(|dim| (((id * 29 + dim) as f32) * 0.02).sin())
                    .collect::<Vec<_>>();
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 0,
                        offset_number: u16::try_from(id).expect("id should fit in u16"),
                    }],
                    dimensions: 16,
                    bits,
                    seed,
                    gamma: 0.0,
                    code: encoded_code(&vector, bits, seed),
                    source_vector: None,
                    source_count: 0,
                }
            })
            .collect::<Vec<_>>();
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 6,
                ef_construction: 80,
                ef_search: 40,
                build_source_column: None,
                rerank_source_column: None,
                storage_format: options::StorageFormat::TurboQuant,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples,
            tuple_index_by_payload: HashMap::new(),
            dimensions: Some(16),
            bits: Some(bits),
            seed: Some(seed),
        };

        let nodes = build_hnsw_graph(&state);

        assert_eq!(nodes.len(), 16);
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::TurboQuant,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 3,
            heap_tuples: tuples,
            tuple_index_by_payload: HashMap::new(),
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 3,
            heap_tuples: tuples,
            tuple_index_by_payload: HashMap::new(),
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
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
            tuple_index_by_payload: HashMap::new(),
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
    fn native_forward_slot_packing_preserves_layer_boundaries_with_padding() {
        let mut slots = empty_neighbor_slots(2, 2);
        let mut selections = Vec::new();
        write_native_layer_forward_candidates(
            &mut slots,
            &mut selections,
            0,
            2,
            [
                search::BeamCandidate::new(11, 0.1),
                search::BeamCandidate::new(12, 0.2),
            ],
        );
        write_native_layer_forward_candidates(
            &mut slots,
            &mut selections,
            1,
            2,
            [search::BeamCandidate::new(13, 0.3)],
        );
        write_native_layer_forward_candidates(
            &mut slots,
            &mut selections,
            2,
            2,
            [search::BeamCandidate::new(14, 0.4)],
        );

        assert_eq!(
            slots,
            vec![Some(11), Some(12), None, None, Some(13), None, Some(14), None],
            "persisted neighbor slots should keep fixed 2M / M layer boundaries instead of compacting upper-layer tids into layer-0 space",
        );
    }

    #[test]
    fn flatten_native_neighbor_slots_dedups_and_skips_origin() {
        let slots = vec![
            Some(0),
            Some(1),
            None,
            Some(1),
            Some(2),
            Some(0),
            Some(2),
            None,
        ];

        assert_eq!(
            flatten_native_neighbor_slots(0, 2, 2, &slots),
            vec![1, 2],
            "flattening should preserve first-seen layer order while skipping self-links and duplicates",
        );
    }

    #[test]
    fn add_native_backlinks_uses_free_slot_before_rewrite() {
        let seed = 42_u64;
        let bits = 4_u8;
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 2,
                ef_construction: 32,
                ef_search: 40,
                build_source_column: Some("source".to_owned()),
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
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
                    code: vec![0x00],
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
                    code: vec![0x01],
                    source_vector: Some(vec![0.3, 0.0]),
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
                    code: vec![0x02],
                    source_vector: Some(vec![0.9, 0.0]),
                    source_count: 1,
                },
            ],
            tuple_index_by_payload: HashMap::new(),
            dimensions: Some(2),
            bits: Some(bits),
            seed: Some(seed),
        };
        let mut nodes = vec![
            NativeBuildNode {
                level: 0,
                neighbor_slots: vec![Some(1), None, None, None],
            },
            NativeBuildNode {
                level: 0,
                neighbor_slots: vec![Some(0), None, None, None],
            },
            NativeBuildNode {
                level: 0,
                neighbor_slots: vec![Some(0), None, None, None],
            },
        ];

        add_native_backlinks(
            &mut nodes,
            &state,
            BuildGraphMetric::Source,
            2,
            &[NativeForwardSelection {
                layer: 0,
                node_idx: 0,
            }],
            2,
        );

        assert_eq!(
            nodes[0].neighbor_slots,
            vec![Some(1), Some(2), None, None],
            "a target with free layer capacity should admit the new backlink without rewriting the existing slice",
        );
    }

    #[test]
    fn add_native_backlinks_rewrites_full_slice_for_better_candidate() {
        let seed = 42_u64;
        let bits = 4_u8;
        let state = BuildState {
            options: options::TqHnswOptions {
                m: 2,
                ef_construction: 32,
                ef_search: 40,
                build_source_column: Some("source".to_owned()),
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 6,
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
                    code: vec![0x00],
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
                    code: vec![0x01],
                    source_vector: Some(vec![0.1, 0.0]),
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
                    code: vec![0x02],
                    source_vector: Some(vec![0.95, 0.0]),
                    source_count: 1,
                },
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: 4,
                    }],
                    dimensions: 2,
                    bits,
                    seed,
                    gamma: 0.0,
                    code: vec![0x03],
                    source_vector: Some(vec![0.2, 0.0]),
                    source_count: 1,
                },
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: 5,
                    }],
                    dimensions: 2,
                    bits,
                    seed,
                    gamma: 0.0,
                    code: vec![0x04],
                    source_vector: Some(vec![0.3, 0.0]),
                    source_count: 1,
                },
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: 6,
                    }],
                    dimensions: 2,
                    bits,
                    seed,
                    gamma: 0.0,
                    code: vec![0x05],
                    source_vector: Some(vec![0.4, 0.0]),
                    source_count: 1,
                },
            ],
            tuple_index_by_payload: HashMap::new(),
            dimensions: Some(2),
            bits: Some(bits),
            seed: Some(seed),
        };
        let mut nodes = vec![
            NativeBuildNode {
                level: 0,
                neighbor_slots: vec![Some(1), Some(3), Some(4), Some(5)],
            },
            NativeBuildNode {
                level: 0,
                neighbor_slots: vec![Some(0), None, None, None],
            },
            NativeBuildNode {
                level: 0,
                neighbor_slots: vec![Some(0), None, None, None],
            },
            NativeBuildNode {
                level: 0,
                neighbor_slots: vec![Some(0), None, None, None],
            },
            NativeBuildNode {
                level: 0,
                neighbor_slots: vec![Some(0), None, None, None],
            },
            NativeBuildNode {
                level: 0,
                neighbor_slots: vec![Some(0), None, None, None],
            },
        ];

        add_native_backlinks(
            &mut nodes,
            &state,
            BuildGraphMetric::Source,
            2,
            &[NativeForwardSelection {
                layer: 0,
                node_idx: 0,
            }],
            2,
        );

        assert_eq!(
            nodes[0].neighbor_slots,
            vec![Some(2), Some(5), Some(4), Some(3)],
            "a full backlink slice should be rewritten with the best scored M/2M candidates, including the new node when it outranks an existing backlink",
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::TurboQuant,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples.clone(),
            tuple_index_by_payload: HashMap::new(),
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples.clone(),
            tuple_index_by_payload: HashMap::new(),
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::TurboQuant,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
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
            tuple_index_by_payload: HashMap::new(),
            dimensions: Some(16),
            bits: Some(bits),
            seed: Some(seed),
        };

        let error = train_build_grouped_pq_model(&state, 4, 16, 3).unwrap_err();
        assert!(error.contains("source vectors"));
    }

    #[test]
    fn grouped_build_model_supports_low_cardinality_source_sets() {
        let seed = 42_u64;
        let bits = 4_u8;
        let tuples = (0..3)
            .map(|i| {
                let source = (0..16)
                    .map(|dim| ((i * 13 + dim) as f32 * 0.11).sin())
                    .collect::<Vec<_>>();
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: (i + 1) as u16,
                    }],
                    dimensions: 16,
                    bits,
                    seed,
                    gamma: 0.0,
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples.clone(),
            tuple_index_by_payload: HashMap::new(),
            dimensions: Some(16),
            bits: Some(bits),
            seed: Some(seed),
        };

        let model = train_build_grouped_pq_model(&state, 4, 16, 3).unwrap();
        assert_eq!(model.group_size, 4);
        assert_eq!(model.group_count, 4);
        assert_eq!(model.codebooks.len(), 4);
        assert!(model
            .codebooks
            .iter()
            .all(|codebook| codebook.len() == 4 * GROUPED_PQ_CENTROIDS));

        let code = derive_grouped_search_code_from_source(&tuples[0], &model).unwrap();
        assert_eq!(code.len(), model.group_count.div_ceil(2));
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples.clone(),
            tuple_index_by_payload: HashMap::new(),
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples,
            tuple_index_by_payload: HashMap::new(),
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
    fn pq_fastscan_flush_output_marks_grouped_metadata_and_pages() {
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples,
            tuple_index_by_payload: HashMap::new(),
            dimensions: Some(16),
            bits: Some(bits),
            seed: Some(seed),
        };

        let plan = plan_v2_grouped_source_build(&state, 4, 16, 3).unwrap();
        let output = pq_fastscan_flush_output(&state, &plan, 4).unwrap();

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
    fn default_pq_fastscan_flush_output_uses_default_v2_parameters() {
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples,
            tuple_index_by_payload: HashMap::new(),
            dimensions: Some(16),
            bits: Some(bits),
            seed: Some(seed),
        };

        let output = default_pq_fastscan_flush_output(&state).unwrap();

        assert_eq!(
            output.metadata.format_version,
            page::INDEX_FORMAT_V2_GROUPED
        );
        assert_eq!(output.metadata.search_subvector_count, 1);
        assert_eq!(
            output.metadata.search_subvector_dim,
            PQ_FASTSCAN_TARGET_GROUP_SIZE as u16
        );
        assert_ne!(
            output.metadata.grouped_codebook_head,
            page::ItemPointer::INVALID
        );
    }

    #[test]
    fn default_pq_fastscan_flush_output_derives_small_dimension_group_size() {
        let seed = 42_u64;
        let bits = 4_u8;
        let tuples = (0..8)
            .map(|i| {
                let source = (0..8)
                    .map(|dim| ((i * 17 + dim) as f32 * 0.07).sin())
                    .collect::<Vec<_>>();
                BuildTuple {
                    heap_tids: vec![page::ItemPointer {
                        block_number: 1,
                        offset_number: (i + 1) as u16,
                    }],
                    dimensions: 8,
                    bits,
                    seed,
                    gamma: 0.03 * i as f32,
                    code: vec![i as u8; 4],
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
                rerank_source_column: None,
                storage_format: options::StorageFormat::PqFastScan,
            },
            indexed_vector_kind: source::IndexedVectorKind::Ecvector,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: tuples.len(),
            heap_tuples: tuples,
            tuple_index_by_payload: HashMap::new(),
            dimensions: Some(8),
            bits: Some(bits),
            seed: Some(seed),
        };

        let output = default_pq_fastscan_flush_output(&state).unwrap();

        assert_eq!(output.metadata.search_subvector_count, 1);
        assert_eq!(output.metadata.search_subvector_dim, 8);
        assert_ne!(
            output.metadata.grouped_codebook_head,
            page::ItemPointer::INVALID
        );
    }
}
