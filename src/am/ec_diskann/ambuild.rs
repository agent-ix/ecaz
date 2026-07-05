//! pgrx-side ambuild wiring for `ec_diskann` (task 17 Phase 5C-3).
//!
//! Heap scan → per-row grouped-PQ4 encode → [`build_and_persist_vamana`]
//! → stage codebook chain → write data pages + metadata page under a
//! single WAL-wrapped sequence. See `plan/design/diskann-build-algorithm.md`
//! for the full pipeline.
//!
//! V0 scope: indexed column must be `ecvector` (flat f32). The build
//! distance is `1 - ip(source_vector, source_vector)` and ambuild rejects
//! sampled source vectors whose norms drift outside the unit-normalized
//! precondition. That keeps the `<#>` ordering while satisfying Vamana's
//! nonnegative-distance requirement. This yields a higher-quality graph
//! than scoring on quantized codes and matches the intent of ADR-034
//! (IP-first).
//!
//! ADR-046 frozen rule 1 / ADR-047 frozen rule 4: `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD`
//! stays clear on V0 builds. The V0 rerank source is the heap
//! `ecvector` row (ADR-044 default).
//!
//! Build-time exact duplicate vectors are intentionally represented as
//! distinct graph nodes. Runtime insert still supports overflow heap TIDs, but
//! ambuild avoids the prior O(N^2) exact-match scan over all collected rows.

use std::ffi::c_void;
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{Duration, Instant};

use pgrx::{pg_sys, PgBox};
use rayon::prelude::*;

use crate::am::common::{
    callback::pg_am_callback, detoast::DetoastedVarlena,
    parallel_context::index_info_parallel_workers,
};
use crate::storage::buffer_guard::LockedBufferGuard;
use crate::storage::page::{DataPageChain, ItemPointer, METADATA_BLOCK_NUMBER};
use crate::storage::relation::RelationHandle;
use crate::storage::wal;
use crate::DEFAULT_QUANT_SEED;

use super::build::{
    build_and_persist_vamana_with_parallel_config, BuildOutput, BuildParallelConfig,
    BuildParallelStats, BuildParams,
};
use super::options::{self, TqDiskannOptions};
use super::page::VamanaMetadataPage;
use super::persist::{stage_grouped_codebook_chain, NodePayload};
use super::quantizer::DiskannBuildBinding;
use super::{
    warn_on_non_unit_source_vector_sample, ECDISKANN_UNIT_NORM_BUILD_SAMPLE_CAP,
    ECDISKANN_UNIT_NORM_DISTANCE_BIAS,
};

const PQ_FASTSCAN_TARGET_GROUP_SIZE: usize = 16;
const PQ_FASTSCAN_DEFAULT_MAX_TRAIN_SIZE: usize = 1024;
const PQ_FASTSCAN_DEFAULT_KMEANS_ITERS: usize = 8;
const P_NEW: pg_sys::BlockNumber = u32::MAX;

static LAST_BUILD_HEAP_TUPLES: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_SCANNED_TUPLES: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_UNIQUE_TUPLES: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_DATA_PAGES: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_HEAP_SCAN_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_SOURCE_REF_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_TRAINING_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_SIDECAR_SETUP_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PAYLOAD_DERIVATION_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_BUILD_PERSIST_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_CORE_MEDOID_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_CORE_GRAPH_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_CORE_PERSIST_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_REQUESTED_WORKERS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_EFFECTIVE_WORKERS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_REQUESTED_BATCH_SIZE: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_BATCH_SIZE: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_FLUSH_RATE: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_RAYON_SCAFFOLD: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_ALPHA_GROWTH_DISABLED: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_EPOCHS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_PROPOSAL_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_REDUCER_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_SAME_EPOCH_CANDIDATE_READS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_TOTAL_CANDIDATE_READS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_STALE_READ_PPM: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_OVERFLOW_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_CODEBOOK_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_WRITE_PAGES_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_METADATA_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_FLUSH_TOTAL_MS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_TOTAL_MS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct BuildTimingSnapshot {
    pub(crate) heap_tuples: u64,
    pub(crate) scanned_tuples: u64,
    pub(crate) unique_tuples: u64,
    pub(crate) data_pages: u64,
    pub(crate) heap_scan_ms: u64,
    pub(crate) source_ref_ms: u64,
    pub(crate) training_ms: u64,
    pub(crate) sidecar_setup_ms: u64,
    pub(crate) payload_derivation_ms: u64,
    pub(crate) build_persist_ms: u64,
    pub(crate) core_medoid_ms: u64,
    pub(crate) core_graph_ms: u64,
    pub(crate) core_persist_ms: u64,
    pub(crate) parallel_requested_workers: u64,
    pub(crate) parallel_effective_workers: u64,
    pub(crate) parallel_requested_batch_size: u64,
    pub(crate) parallel_batch_size: u64,
    pub(crate) parallel_flush_rate: u64,
    pub(crate) parallel_rayon_scaffold: u64,
    pub(crate) parallel_alpha_growth_disabled: u64,
    pub(crate) parallel_epochs: u64,
    pub(crate) parallel_proposal_ms: u64,
    pub(crate) parallel_reducer_ms: u64,
    pub(crate) parallel_same_epoch_candidate_reads: u64,
    pub(crate) parallel_total_candidate_reads: u64,
    pub(crate) parallel_stale_read_ppm: u64,
    pub(crate) overflow_ms: u64,
    pub(crate) codebook_ms: u64,
    pub(crate) write_pages_ms: u64,
    pub(crate) metadata_ms: u64,
    pub(crate) flush_total_ms: u64,
    pub(crate) total_ms: u64,
}

impl BuildTimingSnapshot {
    fn from_complete_build(
        heap_tuples: f64,
        scanned_tuples: usize,
        unique_tuples: usize,
        heap_scan_elapsed: Duration,
        flush: &BuildFlushTiming,
        total_elapsed: Duration,
    ) -> Self {
        Self {
            heap_tuples: nonnegative_f64_to_u64(heap_tuples),
            scanned_tuples: usize_to_u64(scanned_tuples),
            unique_tuples: usize_to_u64(unique_tuples),
            data_pages: usize_to_u64(flush.data_pages),
            heap_scan_ms: u128_to_u64(elapsed_ms(heap_scan_elapsed)),
            source_ref_ms: u128_to_u64(flush.source_ref_ms),
            training_ms: u128_to_u64(flush.training_ms),
            sidecar_setup_ms: u128_to_u64(flush.sidecar_setup_ms),
            payload_derivation_ms: u128_to_u64(flush.payload_derivation_ms),
            build_persist_ms: u128_to_u64(flush.build_persist_ms),
            core_medoid_ms: u128_to_u64(flush.core_medoid_ms),
            core_graph_ms: u128_to_u64(flush.core_graph_ms),
            core_persist_ms: u128_to_u64(flush.core_persist_ms),
            parallel_requested_workers: u64::from(flush.parallel_stats.requested_workers),
            parallel_effective_workers: u64::from(flush.parallel_stats.effective_workers),
            parallel_requested_batch_size: u64::from(flush.parallel_stats.requested_batch_size),
            parallel_batch_size: u64::from(flush.parallel_stats.batch_size),
            parallel_flush_rate: u64::from(flush.parallel_stats.flush_rate),
            parallel_rayon_scaffold: bool_to_u64(flush.parallel_stats.rayon_scaffold_enabled),
            parallel_alpha_growth_disabled: bool_to_u64(flush.parallel_stats.alpha_growth_disabled),
            parallel_epochs: usize_to_u64(flush.parallel_stats.epochs),
            parallel_proposal_ms: u128_to_u64(flush.parallel_stats.proposal_ms),
            parallel_reducer_ms: u128_to_u64(flush.parallel_stats.reducer_ms),
            parallel_same_epoch_candidate_reads: usize_to_u64(
                flush.parallel_stats.same_epoch_candidate_reads,
            ),
            parallel_total_candidate_reads: usize_to_u64(
                flush.parallel_stats.total_candidate_reads,
            ),
            parallel_stale_read_ppm: stale_read_ppm(
                flush.parallel_stats.same_epoch_candidate_reads,
                flush.parallel_stats.total_candidate_reads,
            ),
            overflow_ms: u128_to_u64(flush.overflow_ms),
            codebook_ms: u128_to_u64(flush.codebook_ms),
            write_pages_ms: u128_to_u64(flush.write_pages_ms),
            metadata_ms: u128_to_u64(flush.metadata_ms),
            flush_total_ms: u128_to_u64(flush.total_ms),
            total_ms: u128_to_u64(elapsed_ms(total_elapsed)),
        }
    }
}

#[derive(Debug)]
struct RawHeapTuple {
    primary_heap_tid: ItemPointer,
    source_vector: Vec<f32>,
}

#[derive(Debug)]
struct BuildState {
    options: TqDiskannOptions,
    page_size: usize,
    dimensions: Option<u16>,
    heap_tuples: Vec<RawHeapTuple>,
    scanned_tuples: usize,
}

#[derive(Debug, Default)]
struct BuildFlushTiming {
    source_ref_ms: u128,
    training_ms: u128,
    sidecar_setup_ms: u128,
    payload_derivation_ms: u128,
    build_persist_ms: u128,
    overflow_ms: u128,
    codebook_ms: u128,
    write_pages_ms: u128,
    metadata_ms: u128,
    total_ms: u128,
    data_pages: usize,
    core_medoid_ms: u128,
    core_graph_ms: u128,
    core_persist_ms: u128,
    parallel_stats: BuildParallelStats,
    build_passes: Vec<BuildPassTimingSummary>,
}

#[derive(Debug, Clone)]
struct BuildPassTimingSummary {
    pass: usize,
    alpha: f32,
    elapsed_ms: u128,
    greedy_search_ms: u128,
    candidate_pool_ms: u128,
    robust_prune_ms: u128,
    backlink_ms: u128,
    greedy_distance_calls: usize,
    candidate_pool_distance_calls: usize,
    robust_prune_distance_calls: usize,
    backlink_distance_calls: usize,
    visited_mean: f64,
    visited_p95: usize,
    candidate_pool_mean: f64,
    candidate_pool_p95: usize,
    reprunes: usize,
}

impl BuildState {
    unsafe fn new(index_relation: pg_sys::Relation) -> Self {
        let options = options::relation_options(index_relation);
        Self {
            options,
            page_size: pg_sys::BLCKSZ as usize,
            dimensions: None,
            heap_tuples: Vec::new(),
            scanned_tuples: 0,
        }
    }

    fn push(&mut self, heap_tid: ItemPointer, source_vector: Vec<f32>) {
        self.scanned_tuples += 1;
        if source_vector.is_empty() {
            pgrx::error!("ec_diskann ambuild received an empty indexed vector");
        }
        let dim = u16::try_from(source_vector.len()).unwrap_or_else(|_| {
            pgrx::error!(
                "ec_diskann indexed vector dimension {} exceeds 65535",
                source_vector.len()
            )
        });
        match self.dimensions {
            None => self.dimensions = Some(dim),
            Some(existing) if existing == dim => {}
            Some(existing) => pgrx::error!(
                "ec_diskann ambuild requires a single dimension; saw {dim} after {existing}"
            ),
        }
        self.heap_tuples.push(RawHeapTuple {
            primary_heap_tid: heap_tid,
            source_vector,
        });
    }
}

pub(super) unsafe extern "C-unwind" fn ec_diskann_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    pg_am_callback!({
        let mut state = BuildState::new(index_relation);
        let index_name = relation_name(index_relation);
        validate_single_ecvector_attribute(heap_relation, index_info);
        reset_debug_last_build_timing();

        initialize_metadata_page(index_relation, empty_metadata(&state));

        let total_started = Instant::now();
        let heap_scan_started = Instant::now();
        let heap_tuples = pg_sys::table_index_build_scan(
            heap_relation,
            index_relation,
            index_info,
            false,
            false,
            Some(ec_diskann_build_callback),
            (&mut state as *mut BuildState).cast(),
            ptr::null_mut(),
        );
        let heap_scan_elapsed = heap_scan_started.elapsed();

        let index_tuples = if state.heap_tuples.is_empty() {
            let total_elapsed = total_started.elapsed();
            record_debug_last_build_timing(BuildTimingSnapshot {
                heap_tuples: nonnegative_f64_to_u64(heap_tuples),
                scanned_tuples: usize_to_u64(state.scanned_tuples),
                heap_scan_ms: u128_to_u64(elapsed_ms(heap_scan_elapsed)),
                total_ms: u128_to_u64(elapsed_ms(total_elapsed)),
                ..BuildTimingSnapshot::default()
            });
            log_ambuild_empty_timing(
                &index_name,
                heap_tuples,
                state.scanned_tuples,
                heap_scan_elapsed,
                total_elapsed,
            );
            0.0
        } else {
            let flush_timing = flush_build_state(
                index_relation,
                &state,
                index_info_parallel_workers(index_info),
            )
            .unwrap_or_else(|e| pgrx::error!("ec_diskann ambuild failed: {e}"));
            let total_elapsed = total_started.elapsed();
            record_debug_last_build_timing(BuildTimingSnapshot::from_complete_build(
                heap_tuples,
                state.scanned_tuples,
                state.heap_tuples.len(),
                heap_scan_elapsed,
                &flush_timing,
                total_elapsed,
            ));
            log_ambuild_timing(
                &index_name,
                heap_tuples,
                state.scanned_tuples,
                state.heap_tuples.len(),
                heap_scan_elapsed,
                &flush_timing,
                total_elapsed,
            );
            state.heap_tuples.len() as f64
        };

        if heap_tuples != state.scanned_tuples as f64 {
            pgrx::error!(
                "ec_diskann ambuild scanned {heap_tuples} heap tuples but observed {}",
                state.scanned_tuples
            );
        }

        crate::fault::maybe_fail_palloc("ec_diskann ambuild result");
        let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
        result.heap_tuples = heap_tuples;
        result.index_tuples = index_tuples;
        result.into_pg()
    })
}

pub(super) unsafe extern "C-unwind" fn ec_diskann_ambuildempty(index_relation: pg_sys::Relation) {
    pg_am_callback!({
        let state = BuildState::new(index_relation);
        initialize_metadata_page(index_relation, empty_metadata(&state));
    })
}

unsafe extern "C-unwind" fn ec_diskann_build_callback(
    _index: pg_sys::Relation,
    tid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut c_void,
) {
    pg_am_callback!({
        let state = &mut *state.cast::<BuildState>();
        if values.is_null() || isnull.is_null() {
            pgrx::error!("ec_diskann ambuild received null tuple value arrays");
        }
        if *isnull {
            pgrx::error!("ec_diskann does not support NULL indexed values");
        }
        let datum = *values;
        if datum.is_null() {
            pgrx::error!("ec_diskann ambuild received a null indexed datum");
        }
        let source_vector = ecvector_datum_to_vec(datum);
        let heap_tid = decode_heap_tid(tid);
        state.push(heap_tid, source_vector);
    })
}

fn empty_metadata(state: &BuildState) -> VamanaMetadataPage {
    VamanaMetadataPage::empty(
        state.options.graph_degree as u16,
        state.options.build_list_size as u16,
        state.options.alpha,
        state.dimensions.unwrap_or(0),
        DEFAULT_QUANT_SEED,
    )
}

pub(super) fn default_group_size(dimensions: u16) -> usize {
    let transform_dim = crate::quant::rotation::effective_transform_dim(dimensions as usize);
    transform_dim.min(PQ_FASTSCAN_TARGET_GROUP_SIZE)
}

unsafe fn flush_build_state(
    index_relation: pg_sys::Relation,
    state: &BuildState,
    requested_workers: i32,
) -> Result<BuildFlushTiming, String> {
    let total_started = Instant::now();
    let mut timing = BuildFlushTiming::default();
    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions");
    let seed = DEFAULT_QUANT_SEED;
    let group_size = default_group_size(dimensions);
    let train_size = state
        .heap_tuples
        .len()
        .min(PQ_FASTSCAN_DEFAULT_MAX_TRAIN_SIZE);

    let source_ref_started = Instant::now();
    let source_refs: Vec<&[f32]> = state
        .heap_tuples
        .iter()
        .map(|t| t.source_vector.as_slice())
        .collect();
    warn_on_non_unit_source_vector_sample(
        &source_refs,
        ECDISKANN_UNIT_NORM_BUILD_SAMPLE_CAP,
        "ambuild",
    );
    timing.source_ref_ms = elapsed_ms(source_ref_started.elapsed());

    let codec_started = Instant::now();
    let codec = DiskannBuildBinding::prepare(
        state.options.storage_format,
        &source_refs,
        dimensions as usize,
        seed,
        group_size,
        train_size,
        PQ_FASTSCAN_DEFAULT_KMEANS_ITERS,
    )?;
    timing.training_ms = elapsed_ms(codec_started.elapsed());

    let payload_started = Instant::now();
    let payloads: Vec<NodePayload> = state
        .heap_tuples
        .par_iter()
        .map(|t| {
            let encoded = codec.encode(&t.source_vector);
            NodePayload {
                primary_heaptid: t.primary_heap_tid,
                binary_words: encoded.binary_words,
                search_code: encoded.search_code,
            }
        })
        .collect();
    timing.payload_derivation_ms = elapsed_ms(payload_started.elapsed());

    let params = BuildParams {
        graph_degree_r: u16::try_from(state.options.graph_degree)
            .map_err(|_| "graph_degree does not fit in u16".to_owned())?,
        build_list_size_l: u16::try_from(state.options.build_list_size)
            .map_err(|_| "build_list_size does not fit in u16".to_owned())?,
        alpha: state.options.alpha,
        dimensions,
        search_subvector_count: codec.search_subvector_count(),
        search_subvector_dim: codec.search_subvector_dim(),
        search_codec_kind: codec.search_codec_kind(),
        seed,
        page_size: state.page_size,
        has_binary_sidecar: codec.has_binary_sidecar(),
    };
    let parallel_config = BuildParallelConfig {
        requested_workers: u16::try_from(requested_workers)
            .map_err(|_| "PostgreSQL ii_ParallelWorkers does not fit in u16".to_owned())?,
        batch_size: u32::try_from(state.options.parallel_build_batch_size)
            .map_err(|_| "parallel_build_batch_size does not fit in u32".to_owned())?,
        flush_rate: u32::try_from(state.options.parallel_build_flush_rate)
            .map_err(|_| "parallel_build_flush_rate does not fit in u32".to_owned())?,
    };
    if parallel_config.requested_workers > 0 {
        pgrx::notice!(
            "ec_diskann_parallel_build_policy index={} requested_workers={} requested_batch_size={} small_build_batch_cap=64 small_build_node_cap=10000 alpha_growth_disabled_when_effective_batch_gt_1=true stale_read_metric=same_epoch_candidate_proxy",
            relation_name(index_relation),
            parallel_config.requested_workers,
            parallel_config.batch_size
        );
    }

    let build_persist_started = Instant::now();
    let build_out = build_and_persist_vamana_with_parallel_config(
        params,
        &payloads,
        parallel_config,
        |a, b| source_inner_product_distance(source_refs[a as usize], source_refs[b as usize]),
    )?;
    timing.build_persist_ms = elapsed_ms(build_persist_started.elapsed());

    let BuildOutput {
        metadata,
        persisted,
        build_stats,
        core_timing,
        parallel_stats,
    } = build_out;
    timing.core_medoid_ms = core_timing.medoid_ms;
    timing.core_graph_ms = core_timing.graph_ms;
    timing.core_persist_ms = core_timing.persist_ms;
    timing.parallel_stats = parallel_stats;
    timing.build_passes = build_stats
        .passes
        .iter()
        .enumerate()
        .map(|(pass, stats)| BuildPassTimingSummary {
            pass,
            alpha: stats.alpha,
            elapsed_ms: stats.elapsed_ms,
            greedy_search_ms: stats.greedy_search_ms,
            candidate_pool_ms: stats.candidate_pool_ms,
            robust_prune_ms: stats.robust_prune_ms,
            backlink_ms: stats.backlink_ms,
            greedy_distance_calls: stats.greedy_distance_calls,
            candidate_pool_distance_calls: stats.candidate_pool_distance_calls,
            robust_prune_distance_calls: stats.robust_prune_distance_calls,
            backlink_distance_calls: stats.backlink_distance_calls,
            visited_mean: stats.visited.mean,
            visited_p95: stats.visited.p95,
            candidate_pool_mean: stats.candidate_pool.mean,
            candidate_pool_p95: stats.candidate_pool.p95,
            reprunes: stats.reprunes,
        })
        .collect();
    let mut chain = persisted.chain;
    timing.data_pages = chain.pages().len();
    let mut metadata = metadata;
    if let Some(model) = codec.pq_model() {
        let codebook_started = Instant::now();
        let codebook_head = stage_grouped_codebook_chain(&mut chain, &model)?;
        timing.codebook_ms = elapsed_ms(codebook_started.elapsed());
        metadata.grouped_codebook_head = codebook_head;
    }

    let write_pages_started = Instant::now();
    let pages_handle = NonNull::new(index_relation).unwrap_or_else(|| {
        pgrx::error!("ec_diskann ambuild write_data_pages received a null index relation")
    });
    write_data_pages(pages_handle, &chain);
    timing.write_pages_ms = elapsed_ms(write_pages_started.elapsed());
    let metadata_started = Instant::now();
    // SAFETY: Metadata was produced for this staged chain and is rewritten under
    // the metadata page's exclusive buffer lock.
    overwrite_metadata_page_handle(pages_handle, &metadata);
    timing.metadata_ms = elapsed_ms(metadata_started.elapsed());
    timing.total_ms = elapsed_ms(total_started.elapsed());
    Ok(timing)
}

fn elapsed_ms(duration: Duration) -> u128 {
    duration.as_millis()
}

fn bool_to_u64(value: bool) -> u64 {
    if value {
        1
    } else {
        0
    }
}

fn stale_read_ppm(same_epoch_candidate_reads: usize, total_candidate_reads: usize) -> u64 {
    if total_candidate_reads == 0 {
        0
    } else {
        let ppm =
            (same_epoch_candidate_reads as u128 * 1_000_000u128) / total_candidate_reads as u128;
        u128_to_u64(ppm)
    }
}

pub(crate) fn debug_last_build_timing() -> BuildTimingSnapshot {
    BuildTimingSnapshot {
        heap_tuples: LAST_BUILD_HEAP_TUPLES.load(AtomicOrdering::Acquire),
        scanned_tuples: LAST_BUILD_SCANNED_TUPLES.load(AtomicOrdering::Acquire),
        unique_tuples: LAST_BUILD_UNIQUE_TUPLES.load(AtomicOrdering::Acquire),
        data_pages: LAST_BUILD_DATA_PAGES.load(AtomicOrdering::Acquire),
        heap_scan_ms: LAST_BUILD_HEAP_SCAN_MS.load(AtomicOrdering::Acquire),
        source_ref_ms: LAST_BUILD_SOURCE_REF_MS.load(AtomicOrdering::Acquire),
        training_ms: LAST_BUILD_TRAINING_MS.load(AtomicOrdering::Acquire),
        sidecar_setup_ms: LAST_BUILD_SIDECAR_SETUP_MS.load(AtomicOrdering::Acquire),
        payload_derivation_ms: LAST_BUILD_PAYLOAD_DERIVATION_MS.load(AtomicOrdering::Acquire),
        build_persist_ms: LAST_BUILD_BUILD_PERSIST_MS.load(AtomicOrdering::Acquire),
        core_medoid_ms: LAST_BUILD_CORE_MEDOID_MS.load(AtomicOrdering::Acquire),
        core_graph_ms: LAST_BUILD_CORE_GRAPH_MS.load(AtomicOrdering::Acquire),
        core_persist_ms: LAST_BUILD_CORE_PERSIST_MS.load(AtomicOrdering::Acquire),
        parallel_requested_workers: LAST_BUILD_PARALLEL_REQUESTED_WORKERS
            .load(AtomicOrdering::Acquire),
        parallel_effective_workers: LAST_BUILD_PARALLEL_EFFECTIVE_WORKERS
            .load(AtomicOrdering::Acquire),
        parallel_requested_batch_size: LAST_BUILD_PARALLEL_REQUESTED_BATCH_SIZE
            .load(AtomicOrdering::Acquire),
        parallel_batch_size: LAST_BUILD_PARALLEL_BATCH_SIZE.load(AtomicOrdering::Acquire),
        parallel_flush_rate: LAST_BUILD_PARALLEL_FLUSH_RATE.load(AtomicOrdering::Acquire),
        parallel_rayon_scaffold: LAST_BUILD_PARALLEL_RAYON_SCAFFOLD.load(AtomicOrdering::Acquire),
        parallel_alpha_growth_disabled: LAST_BUILD_PARALLEL_ALPHA_GROWTH_DISABLED
            .load(AtomicOrdering::Acquire),
        parallel_epochs: LAST_BUILD_PARALLEL_EPOCHS.load(AtomicOrdering::Acquire),
        parallel_proposal_ms: LAST_BUILD_PARALLEL_PROPOSAL_MS.load(AtomicOrdering::Acquire),
        parallel_reducer_ms: LAST_BUILD_PARALLEL_REDUCER_MS.load(AtomicOrdering::Acquire),
        parallel_same_epoch_candidate_reads: LAST_BUILD_PARALLEL_SAME_EPOCH_CANDIDATE_READS
            .load(AtomicOrdering::Acquire),
        parallel_total_candidate_reads: LAST_BUILD_PARALLEL_TOTAL_CANDIDATE_READS
            .load(AtomicOrdering::Acquire),
        parallel_stale_read_ppm: LAST_BUILD_PARALLEL_STALE_READ_PPM.load(AtomicOrdering::Acquire),
        overflow_ms: LAST_BUILD_OVERFLOW_MS.load(AtomicOrdering::Acquire),
        codebook_ms: LAST_BUILD_CODEBOOK_MS.load(AtomicOrdering::Acquire),
        write_pages_ms: LAST_BUILD_WRITE_PAGES_MS.load(AtomicOrdering::Acquire),
        metadata_ms: LAST_BUILD_METADATA_MS.load(AtomicOrdering::Acquire),
        flush_total_ms: LAST_BUILD_FLUSH_TOTAL_MS.load(AtomicOrdering::Acquire),
        total_ms: LAST_BUILD_TOTAL_MS.load(AtomicOrdering::Acquire),
    }
}

fn reset_debug_last_build_timing() {
    record_debug_last_build_timing(BuildTimingSnapshot::default());
}

fn record_debug_last_build_timing(snapshot: BuildTimingSnapshot) {
    LAST_BUILD_HEAP_TUPLES.store(snapshot.heap_tuples, AtomicOrdering::Release);
    LAST_BUILD_SCANNED_TUPLES.store(snapshot.scanned_tuples, AtomicOrdering::Release);
    LAST_BUILD_UNIQUE_TUPLES.store(snapshot.unique_tuples, AtomicOrdering::Release);
    LAST_BUILD_DATA_PAGES.store(snapshot.data_pages, AtomicOrdering::Release);
    LAST_BUILD_HEAP_SCAN_MS.store(snapshot.heap_scan_ms, AtomicOrdering::Release);
    LAST_BUILD_SOURCE_REF_MS.store(snapshot.source_ref_ms, AtomicOrdering::Release);
    LAST_BUILD_TRAINING_MS.store(snapshot.training_ms, AtomicOrdering::Release);
    LAST_BUILD_SIDECAR_SETUP_MS.store(snapshot.sidecar_setup_ms, AtomicOrdering::Release);
    LAST_BUILD_PAYLOAD_DERIVATION_MS.store(snapshot.payload_derivation_ms, AtomicOrdering::Release);
    LAST_BUILD_BUILD_PERSIST_MS.store(snapshot.build_persist_ms, AtomicOrdering::Release);
    LAST_BUILD_CORE_MEDOID_MS.store(snapshot.core_medoid_ms, AtomicOrdering::Release);
    LAST_BUILD_CORE_GRAPH_MS.store(snapshot.core_graph_ms, AtomicOrdering::Release);
    LAST_BUILD_CORE_PERSIST_MS.store(snapshot.core_persist_ms, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_REQUESTED_WORKERS
        .store(snapshot.parallel_requested_workers, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_EFFECTIVE_WORKERS
        .store(snapshot.parallel_effective_workers, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_REQUESTED_BATCH_SIZE.store(
        snapshot.parallel_requested_batch_size,
        AtomicOrdering::Release,
    );
    LAST_BUILD_PARALLEL_BATCH_SIZE.store(snapshot.parallel_batch_size, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_FLUSH_RATE.store(snapshot.parallel_flush_rate, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_RAYON_SCAFFOLD
        .store(snapshot.parallel_rayon_scaffold, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_ALPHA_GROWTH_DISABLED.store(
        snapshot.parallel_alpha_growth_disabled,
        AtomicOrdering::Release,
    );
    LAST_BUILD_PARALLEL_EPOCHS.store(snapshot.parallel_epochs, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_PROPOSAL_MS.store(snapshot.parallel_proposal_ms, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_REDUCER_MS.store(snapshot.parallel_reducer_ms, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_SAME_EPOCH_CANDIDATE_READS.store(
        snapshot.parallel_same_epoch_candidate_reads,
        AtomicOrdering::Release,
    );
    LAST_BUILD_PARALLEL_TOTAL_CANDIDATE_READS.store(
        snapshot.parallel_total_candidate_reads,
        AtomicOrdering::Release,
    );
    LAST_BUILD_PARALLEL_STALE_READ_PPM
        .store(snapshot.parallel_stale_read_ppm, AtomicOrdering::Release);
    LAST_BUILD_OVERFLOW_MS.store(snapshot.overflow_ms, AtomicOrdering::Release);
    LAST_BUILD_CODEBOOK_MS.store(snapshot.codebook_ms, AtomicOrdering::Release);
    LAST_BUILD_WRITE_PAGES_MS.store(snapshot.write_pages_ms, AtomicOrdering::Release);
    LAST_BUILD_METADATA_MS.store(snapshot.metadata_ms, AtomicOrdering::Release);
    LAST_BUILD_FLUSH_TOTAL_MS.store(snapshot.flush_total_ms, AtomicOrdering::Release);
    LAST_BUILD_TOTAL_MS.store(snapshot.total_ms, AtomicOrdering::Release);
}

fn u128_to_u64(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn nonnegative_f64_to_u64(value: f64) -> u64 {
    if value.is_finite() && value > 0.0 {
        value as u64
    } else {
        0
    }
}

fn relation_name(relation: pg_sys::Relation) -> String {
    let relation = NonNull::new(relation)
        .unwrap_or_else(|| pgrx::error!("ec_diskann build needs a valid relation"));
    crate::storage::relation::relation_name_handle(relation)
}

fn log_ambuild_empty_timing(
    index_name: &str,
    heap_tuples: f64,
    scanned_tuples: usize,
    heap_scan_elapsed: Duration,
    total_elapsed: Duration,
) {
    pgrx::notice!(
        "ec_diskann_ambuild_timing index={} phase=empty heap_tuples={} scanned_tuples={} heap_scan_ms={} total_ms={}",
        index_name,
        heap_tuples,
        scanned_tuples,
        elapsed_ms(heap_scan_elapsed),
        elapsed_ms(total_elapsed)
    );
}

fn log_ambuild_timing(
    index_name: &str,
    heap_tuples: f64,
    scanned_tuples: usize,
    unique_tuples: usize,
    heap_scan_elapsed: Duration,
    flush: &BuildFlushTiming,
    total_elapsed: Duration,
) {
    for pass in &flush.build_passes {
        pgrx::notice!(
            "ec_diskann_vamana_pass_timing index={} pass={} alpha={} elapsed_ms={} greedy_search_ms={} candidate_pool_ms={} robust_prune_ms={} backlink_ms={} greedy_distance_calls={} candidate_pool_distance_calls={} robust_prune_distance_calls={} backlink_distance_calls={} visited_mean={:.2} visited_p95={} candidate_pool_mean={:.2} candidate_pool_p95={} reprunes={}",
            index_name,
            pass.pass,
            pass.alpha,
            pass.elapsed_ms,
            pass.greedy_search_ms,
            pass.candidate_pool_ms,
            pass.robust_prune_ms,
            pass.backlink_ms,
            pass.greedy_distance_calls,
            pass.candidate_pool_distance_calls,
            pass.robust_prune_distance_calls,
            pass.backlink_distance_calls,
            pass.visited_mean,
            pass.visited_p95,
            pass.candidate_pool_mean,
            pass.candidate_pool_p95,
            pass.reprunes
        );
    }
    pgrx::notice!(
        "ec_diskann_ambuild_timing index={} phase=complete heap_tuples={} scanned_tuples={} unique_tuples={} data_pages={} heap_scan_ms={} source_ref_ms={} training_ms={} sidecar_setup_ms={} payload_derivation_ms={} build_persist_ms={} core_medoid_ms={} core_graph_ms={} core_persist_ms={} parallel_requested_workers={} parallel_effective_workers={} parallel_requested_batch_size={} parallel_batch_size={} parallel_flush_rate={} parallel_rayon_scaffold={} parallel_alpha_growth_disabled={} parallel_epochs={} parallel_proposal_ms={} parallel_reducer_ms={} parallel_same_epoch_candidate_reads={} parallel_total_candidate_reads={} parallel_stale_read_ppm={} overflow_ms={} codebook_ms={} write_pages_ms={} metadata_ms={} flush_total_ms={} total_ms={}",
        index_name,
        heap_tuples,
        scanned_tuples,
        unique_tuples,
        flush.data_pages,
        elapsed_ms(heap_scan_elapsed),
        flush.source_ref_ms,
        flush.training_ms,
        flush.sidecar_setup_ms,
        flush.payload_derivation_ms,
        flush.build_persist_ms,
        flush.core_medoid_ms,
        flush.core_graph_ms,
        flush.core_persist_ms,
        flush.parallel_stats.requested_workers,
        flush.parallel_stats.effective_workers,
        flush.parallel_stats.requested_batch_size,
        flush.parallel_stats.batch_size,
        flush.parallel_stats.flush_rate,
        flush.parallel_stats.rayon_scaffold_enabled,
        flush.parallel_stats.alpha_growth_disabled,
        flush.parallel_stats.epochs,
        flush.parallel_stats.proposal_ms,
        flush.parallel_stats.reducer_ms,
        flush.parallel_stats.same_epoch_candidate_reads,
        flush.parallel_stats.total_candidate_reads,
        stale_read_ppm(
            flush.parallel_stats.same_epoch_candidate_reads,
            flush.parallel_stats.total_candidate_reads
        ),
        flush.overflow_ms,
        flush.codebook_ms,
        flush.write_pages_ms,
        flush.metadata_ms,
        flush.total_ms,
        elapsed_ms(total_elapsed)
    );
}

fn source_inner_product_distance(left: &[f32], right: &[f32]) -> f32 {
    debug_assert_eq!(left.len(), right.len());
    let ip = source_inner_product(left, right);
    let d = ECDISKANN_UNIT_NORM_DISTANCE_BIAS - ip;
    if d < 0.0 {
        0.0
    } else {
        d
    }
}

#[cfg(target_arch = "x86_64")]
pub(super) fn source_inner_product(left: &[f32], right: &[f32]) -> f32 {
    if std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma") {
        // SAFETY: Runtime feature detection guarantees the target features
        // required by the AVX2/FMA kernel before entering the target-feature fn.
        unsafe { source_inner_product_avx2_fma(left, right) }
    } else {
        source_inner_product_scalar(left, right)
    }
}

#[cfg(target_arch = "aarch64")]
pub(super) fn source_inner_product(left: &[f32], right: &[f32]) -> f32 {
    if std::arch::is_aarch64_feature_detected!("neon") {
        // SAFETY: Runtime feature detection guarantees NEON support before
        // entering the target-feature fn.
        unsafe { source_inner_product_neon(left, right) }
    } else {
        source_inner_product_scalar(left, right)
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub(super) fn source_inner_product(left: &[f32], right: &[f32]) -> f32 {
    source_inner_product_scalar(left, right)
}

fn source_inner_product_scalar(left: &[f32], right: &[f32]) -> f32 {
    let mut ip = 0.0_f32;
    for (l, r) in left.iter().zip(right.iter()) {
        ip += *l * *r;
    }
    ip
}

#[cfg(any(test, feature = "bench"))]
pub(super) fn source_inner_product_scalar_reference(left: &[f32], right: &[f32]) -> f32 {
    source_inner_product_scalar(left, right)
}

#[cfg(all(any(test, feature = "bench"), target_arch = "x86_64"))]
pub(super) fn source_inner_product_avx2_fma_for_test(left: &[f32], right: &[f32]) -> Option<f32> {
    if !std::arch::is_x86_feature_detected!("avx2") || !std::arch::is_x86_feature_detected!("fma") {
        return None;
    }
    // SAFETY: This test/bench helper performs the same runtime feature check
    // as production dispatch before calling the target-feature kernel.
    Some(unsafe { source_inner_product_avx2_fma(left, right) })
}

#[cfg(all(any(test, feature = "bench"), target_arch = "aarch64"))]
pub(super) fn source_inner_product_neon_for_test(left: &[f32], right: &[f32]) -> Option<f32> {
    if !std::arch::is_aarch64_feature_detected!("neon") {
        return None;
    }
    // SAFETY: This test/bench helper performs the same runtime feature check
    // as production dispatch before calling the target-feature kernel.
    Some(unsafe { source_inner_product_neon(left, right) })
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn source_inner_product_avx2_fma(left: &[f32], right: &[f32]) -> f32 {
    use std::arch::x86_64::{
        _mm256_add_ps, _mm256_fmadd_ps, _mm256_loadu_ps, _mm256_setzero_ps, _mm256_storeu_ps,
    };

    let len = left.len().min(right.len());
    let mut i = 0usize;
    let mut acc0 = _mm256_setzero_ps();
    let mut acc1 = _mm256_setzero_ps();
    let mut acc2 = _mm256_setzero_ps();
    let mut acc3 = _mm256_setzero_ps();

    while i + 32 <= len {
        // SAFETY: The loop condition proves `i + 32 <= len`, where `len` is
        // the minimum slice length. Each unaligned 8-lane load stays within
        // both input slices.
        let (l0, r0, l1, r1, l2, r2, l3, r3) = unsafe {
            (
                _mm256_loadu_ps(left.as_ptr().add(i)),
                _mm256_loadu_ps(right.as_ptr().add(i)),
                _mm256_loadu_ps(left.as_ptr().add(i + 8)),
                _mm256_loadu_ps(right.as_ptr().add(i + 8)),
                _mm256_loadu_ps(left.as_ptr().add(i + 16)),
                _mm256_loadu_ps(right.as_ptr().add(i + 16)),
                _mm256_loadu_ps(left.as_ptr().add(i + 24)),
                _mm256_loadu_ps(right.as_ptr().add(i + 24)),
            )
        };

        acc0 = _mm256_fmadd_ps(l0, r0, acc0);
        acc1 = _mm256_fmadd_ps(l1, r1, acc1);
        acc2 = _mm256_fmadd_ps(l2, r2, acc2);
        acc3 = _mm256_fmadd_ps(l3, r3, acc3);
        i += 32;
    }

    let total = _mm256_add_ps(_mm256_add_ps(acc0, acc1), _mm256_add_ps(acc2, acc3));
    let mut lanes = [0.0_f32; 8];
    // SAFETY: `lanes` has exactly eight contiguous `f32` slots, matching the
    // width of a single AVX register store.
    unsafe { _mm256_storeu_ps(lanes.as_mut_ptr(), total) };
    let mut ip = lanes.iter().sum::<f32>();

    while i < len {
        ip += left[i] * right[i];
        i += 1;
    }

    ip
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn source_inner_product_neon(left: &[f32], right: &[f32]) -> f32 {
    use std::arch::aarch64::{
        float32x4_t, vaddq_f32, vdupq_n_f32, vfmaq_f32, vld1q_f32, vst1q_f32,
    };

    let len = left.len().min(right.len());
    let mut acc0: float32x4_t = vdupq_n_f32(0.0);
    let mut acc1: float32x4_t = vdupq_n_f32(0.0);
    let mut acc2: float32x4_t = vdupq_n_f32(0.0);
    let mut acc3: float32x4_t = vdupq_n_f32(0.0);
    let mut i = 0_usize;

    while i + 16 <= len {
        // SAFETY: The loop condition proves `i + 16 <= len`, where `len` is
        // the minimum slice length. Each 4-lane load remains within both
        // input slices.
        let (l0, r0, l1, r1, l2, r2, l3, r3) = unsafe {
            (
                vld1q_f32(left.as_ptr().add(i)),
                vld1q_f32(right.as_ptr().add(i)),
                vld1q_f32(left.as_ptr().add(i + 4)),
                vld1q_f32(right.as_ptr().add(i + 4)),
                vld1q_f32(left.as_ptr().add(i + 8)),
                vld1q_f32(right.as_ptr().add(i + 8)),
                vld1q_f32(left.as_ptr().add(i + 12)),
                vld1q_f32(right.as_ptr().add(i + 12)),
            )
        };
        acc0 = vfmaq_f32(acc0, l0, r0);
        acc1 = vfmaq_f32(acc1, l1, r1);
        acc2 = vfmaq_f32(acc2, l2, r2);
        acc3 = vfmaq_f32(acc3, l3, r3);
        i += 16;
    }

    while i + 4 <= len {
        // SAFETY: The loop condition proves the next 4-lane chunk lies within
        // both slices because `len` is their minimum length.
        let (l, r) = unsafe {
            (
                vld1q_f32(left.as_ptr().add(i)),
                vld1q_f32(right.as_ptr().add(i)),
            )
        };
        acc0 = vfmaq_f32(acc0, l, r);
        i += 4;
    }

    let acc = vaddq_f32(vaddq_f32(acc0, acc1), vaddq_f32(acc2, acc3));
    let mut lanes = [0.0_f32; 4];
    // SAFETY: `lanes` has exactly four contiguous `f32` slots, matching the
    // width of a single NEON register store.
    unsafe { vst1q_f32(lanes.as_mut_ptr(), acc) };
    let mut ip = lanes.iter().sum::<f32>();

    while i < len {
        ip += left[i] * right[i];
        i += 1;
    }

    ip
}

unsafe fn initialize_metadata_page(index_relation: pg_sys::Relation, metadata: VamanaMetadataPage) {
    let handle = NonNull::new(index_relation).unwrap_or_else(|| {
        pgrx::error!("ec_diskann metadata initialization needs a valid index relation")
    });
    initialize_metadata_page_handle(handle, metadata);
}

fn initialize_metadata_page_handle(handle: RelationHandle, metadata: VamanaMetadataPage) {
    let existing_blocks = crate::storage::relation::main_fork_block_count_handle(handle);
    let target_block = if existing_blocks == 0 {
        P_NEW
    } else {
        METADATA_BLOCK_NUMBER
    };
    let buffer = if target_block == P_NEW {
        LockedBufferGuard::read_main_locked_handle(
            handle,
            target_block,
            pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
        )
    } else {
        LockedBufferGuard::read_main_handle(
            handle,
            target_block,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
        )
    }
    .unwrap_or_else(|| pgrx::error!("ec_diskann failed to allocate metadata buffer"));
    write_metadata_to_buffer(handle, &buffer, &metadata);
}

unsafe fn overwrite_metadata_page(index_relation: pg_sys::Relation, metadata: &VamanaMetadataPage) {
    let handle = NonNull::new(index_relation).unwrap_or_else(|| {
        pgrx::error!("ec_diskann overwrite_metadata_page needs a valid index relation")
    });
    overwrite_metadata_page_handle(handle, metadata);
}

fn overwrite_metadata_page_handle(handle: RelationHandle, metadata: &VamanaMetadataPage) {
    let buffer = LockedBufferGuard::read_main_handle(
        handle,
        METADATA_BLOCK_NUMBER,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
    )
    .unwrap_or_else(|| pgrx::error!("ec_diskann failed to open metadata buffer"));
    write_metadata_to_buffer(handle, &buffer, metadata);
}

fn write_metadata_to_buffer(
    handle: RelationHandle,
    buffer: &LockedBufferGuard,
    metadata: &VamanaMetadataPage,
) {
    let mut wal_txn = wal::WalTxnScope::start_handle(handle);
    {
        let mut page = wal_txn.register_page(buffer);
        let metadata_bytes = metadata.encode();
        let special_size = (metadata_bytes.len() + 7) & !7;
        page.init(special_size);
        // SAFETY: `page` is the WAL-registered metadata page; the special area
        // was just sized from the encoded metadata before this owned-memory
        // copy.
        unsafe {
            let dst = pg_sys::PageGetSpecialPointer(page.page_ptr()).cast::<u8>();
            ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), dst, metadata_bytes.len());
        }
    }
    wal_txn.finish();
}

pub(super) fn write_data_pages(handle: RelationHandle, chain: &DataPageChain) {
    for staged_page in chain.pages() {
        let buffer = LockedBufferGuard::read_main_locked_handle(
            handle,
            P_NEW,
            pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
        )
        .unwrap_or_else(|| {
            pgrx::error!(
                "ec_diskann failed to allocate data buffer for block {}",
                staged_page.block_number()
            )
        });
        let mut wal_txn = wal::WalTxnScope::start_handle(handle);
        {
            let mut page = wal_txn.register_page(&buffer);
            page.init(0);
            for tuple in staged_page.tuples() {
                page.add_item(tuple).unwrap_or_else(|err| {
                    pgrx::error!(
                        "ec_diskann failed to write tuple to block {}",
                        err.block_number
                    )
                });
            }
        }
        wal_txn.finish();
    }
}

pub(super) unsafe fn decode_heap_tid(tid: pg_sys::ItemPointer) -> ItemPointer {
    if tid.is_null() {
        pgrx::error!("ec_diskann ambuild received a null heap tid");
    }
    crate::am::common::pg_ptr::item_pointer(
        std::ptr::NonNull::new(tid).expect("ec_diskann heap tid should be non-null"),
    )
}

unsafe fn validate_single_ecvector_attribute(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) {
    if index_info.is_null() {
        pgrx::error!("ec_diskann ambuild received a null IndexInfo");
    }
    let info = crate::am::common::pg_ptr::index_info(
        std::ptr::NonNull::new(index_info).expect("ec_diskann IndexInfo should be non-null"),
    );
    if info.ii_NumIndexAttrs != 1 || info.ii_NumIndexKeyAttrs != 1 {
        pgrx::error!("ec_diskann currently supports single-column indexes only");
    }
    if !info.ii_Expressions.is_null() {
        pgrx::error!("ec_diskann does not support expression indexes yet");
    }
    if !info.ii_Predicate.is_null() {
        pgrx::error!("ec_diskann does not support partial indexes yet");
    }
    let attnum = i32::from(info.ii_IndexAttrNumbers[0]);
    if attnum <= 0 {
        pgrx::error!("ec_diskann ambuild requires a base heap column index key");
    }

    let heap_relation = NonNull::new(heap_relation)
        .unwrap_or_else(|| pgrx::error!("ec_diskann ambuild needs a valid heap relation"));
    let tuple_desc = crate::storage::relation::relation_tuple_desc_copy_handle(heap_relation);
    let att = tuple_desc
        .get(attnum as usize - 1)
        .expect("indexed attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_diskann indexed column references a dropped column");
    }
    let name = crate::storage::type_info::formatted_base_type_name(att.atttypid)
        .unwrap_or_else(|| pgrx::error!("ec_diskann indexed column has no resolvable type name"));
    let type_name = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
    if type_name != "ecvector" {
        pgrx::error!("ec_diskann indexed column must be ecvector, got {type_name}");
    }
}

pub(super) unsafe fn with_ecvector_datum_slice<T>(
    datum: pg_sys::Datum,
    f: impl for<'a> FnOnce(&'a [f32]) -> T,
) -> T {
    // SAFETY: The datum is expected to be a non-null ecvector varlena selected
    // by ambuild type validation or caller metadata checks; `align_to` only
    // validates exact f32 alignment and rejects any non-empty prefix/suffix
    // before exposing the body to the visitor.
    let detoasted = DetoastedVarlena::plain_from_datum(datum)
        .unwrap_or_else(|| pgrx::error!("ec_diskann could not detoast indexed ecvector"));
    let bytes = detoasted.as_bytes();
    if bytes.len() % std::mem::size_of::<f32>() != 0 {
        pgrx::error!("ec_diskann indexed ecvector payload length must be a multiple of 4 bytes");
    }
    let (prefix, body, suffix) = bytes.align_to::<f32>();
    if !prefix.is_empty() || !suffix.is_empty() {
        pgrx::error!("ec_diskann indexed ecvector payload is not aligned for float4 access");
    }
    f(body)
}

pub(super) unsafe fn ecvector_datum_to_vec(datum: pg_sys::Datum) -> Vec<f32> {
    with_ecvector_datum_slice(datum, |body| body.to_vec())
}

#[cfg(test)]
mod tests {
    use super::{source_inner_product, source_inner_product_distance, source_inner_product_scalar};

    #[test]
    fn source_inner_product_distance_keeps_positive_ip_pairs_distinct() {
        let identical = source_inner_product_distance(&[1.0, 0.0], &[1.0, 0.0]);
        let merely_similar = source_inner_product_distance(&[1.0, 0.0], &[0.8, 0.6]);
        let orthogonal = source_inner_product_distance(&[1.0, 0.0], &[0.0, 1.0]);

        assert_eq!(identical, 0.0);
        assert!(merely_similar > identical);
        assert!(orthogonal > merely_similar);
    }

    #[test]
    fn source_inner_product_dispatch_matches_scalar() {
        let left = (0..1536)
            .map(|i| ((i as f32) * 0.013).sin())
            .collect::<Vec<_>>();
        let right = (0..1536)
            .map(|i| ((i as f32) * 0.017).cos())
            .collect::<Vec<_>>();

        let scalar = source_inner_product_scalar(&left, &right);
        let dispatched = source_inner_product(&left, &right);
        assert!(
            (scalar - dispatched).abs() <= 0.0001,
            "scalar={scalar} dispatched={dispatched}"
        );
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn source_inner_product_neon_matches_scalar_at_loop_boundaries() {
        if !std::arch::is_aarch64_feature_detected!("neon") {
            return;
        }

        // Lengths chosen so the kernel exercises the 16-lane main loop, the
        // 4-lane tail, and the scalar remainder in turn.
        for len in [1usize, 3, 4, 7, 16, 17, 19, 64, 1536, 1539] {
            let left: Vec<f32> = (0..len).map(|i| ((i as f32) * 0.013).sin()).collect();
            let right: Vec<f32> = (0..len).map(|i| ((i as f32) * 0.017).cos()).collect();
            let scalar = super::source_inner_product_scalar(&left, &right);
            // SAFETY: The test performs NEON runtime feature detection before
            // directly exercising the target-feature kernel at loop boundaries.
            let neon = unsafe { super::source_inner_product_neon(&left, &right) };
            assert!(
                (scalar - neon).abs() <= 0.0001,
                "len={len} scalar={scalar} neon={neon}"
            );
        }
    }
}
