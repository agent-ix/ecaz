use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::Instant;

use super::quantizer::{self, IvfPqFastScanModel, IvfQuantizer};
use super::{build_parallel, options, page, training, P_NEW};
use crate::am::common::{
    callback::pg_am_callback, detoast::DetoastedVarlena, training as common_training,
};
use crate::quant::prod::ProdQuantizer;
use crate::storage::{
    buffer_guard::LockedBufferGuard,
    page::{DataPageChain, ItemPointer},
    relation::RelationHandle,
    wal,
};
use pgrx::{pg_sys, PgBox, PgTupleDesc};

const DEFAULT_AUTO_TRAINING_SAMPLE_ROWS: usize = 10_000;
const DEFAULT_KMEANS_ITERATIONS: usize = 8;
const DEFAULT_PQ_FASTSCAN_KMEANS_ITERATIONS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IndexedVectorKind {
    Ecvector,
    Tqvector,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct BuildTuple {
    pub(super) heap_tid: ItemPointer,
    pub(super) dimensions: u16,
    pub(super) gamma: f32,
    pub(super) payload: Vec<u8>,
    pub(super) source_vector: Vec<f32>,
}

static LAST_BUILD_REQUESTED_WORKERS: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_WORKERS_LAUNCHED: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_HEAP_TUPLES: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_INDEX_TUPLES: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_HEAP_INGEST_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_TRAIN_MODEL_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_STAGE_BUILD_PLAN_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_STAGE_PQ_TRAIN_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_STAGE_CENTROIDS_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_STAGE_ASSIGN_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_STAGE_POSTINGS_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_STAGE_DIRECTORY_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_FLUSH_BUILD_PLAN_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_BEGIN_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_DRAIN_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_SORT_PUSH_US: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_WORKER_TUPLE_BUFFER_CAPACITY: AtomicU64 = AtomicU64::new(0);
static LAST_BUILD_PARALLEL_WORKER_TUPLE_BUFFER_STRUCT_BYTES: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct BuildTimingSnapshot {
    pub(crate) requested_workers: u64,
    pub(crate) workers_launched: u64,
    pub(crate) heap_tuples: u64,
    pub(crate) index_tuples: u64,
    pub(crate) heap_ingest_us: u64,
    pub(crate) train_model_us: u64,
    pub(crate) stage_build_plan_us: u64,
    pub(crate) stage_pq_train_us: u64,
    pub(crate) stage_centroids_us: u64,
    pub(crate) stage_assign_us: u64,
    pub(crate) stage_postings_us: u64,
    pub(crate) stage_directory_us: u64,
    pub(crate) flush_build_plan_us: u64,
    pub(crate) parallel_begin_us: u64,
    pub(crate) parallel_drain_us: u64,
    pub(crate) parallel_sort_push_us: u64,
    pub(crate) parallel_worker_tuple_buffer_capacity: u64,
    pub(crate) parallel_worker_tuple_buffer_struct_bytes: u64,
}

pub(super) struct BuildState {
    pub(super) options: options::EcIvfOptions,
    pub(super) indexed_vector_kind: IndexedVectorKind,
    page_size: usize,
    pub(super) scanned_tuples: usize,
    heap_tuples: Vec<BuildTuple>,
    dimensions: Option<u16>,
}

pub(super) struct IvfBuildPlan {
    data_pages: DataPageChain,
    metadata: page::MetadataPage,
    centroid_tids: Vec<ItemPointer>,
    directory_tids: Vec<ItemPointer>,
    posting_tids_by_list: Vec<Vec<ItemPointer>>,
    directory_entries: Vec<page::IvfListDirectoryTuple>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
struct StageBuildTiming {
    pq_train_us: u64,
    centroids_us: u64,
    assign_us: u64,
    postings_us: u64,
    directory_us: u64,
}

impl IvfBuildPlan {
    fn data_page_count(&self) -> usize {
        self.data_pages.pages().len()
    }

    fn centroid_count(&self) -> usize {
        self.centroid_tids.len()
    }

    fn directory_count(&self) -> usize {
        self.directory_tids.len()
    }

    fn posting_count(&self) -> usize {
        self.directory_entries
            .iter()
            .map(|entry| usize::try_from(entry.live_count).unwrap_or(usize::MAX))
            .sum::<usize>()
    }

    fn empty_list_count(&self) -> usize {
        self.directory_entries
            .iter()
            .filter(|entry| entry.live_count == 0)
            .count()
    }

    fn total_live_tuples(&self) -> u64 {
        self.metadata.total_live_tuples
    }
}

pub(super) fn stage_single_tuple_build_plan(
    options: options::EcIvfOptions,
    tuple: BuildTuple,
) -> Result<IvfBuildPlan, String> {
    let mut state = BuildState::new(options, IndexedVectorKind::Ecvector);
    state.try_push(tuple)?;
    let model = state.train_model()?;
    state.stage_build_plan(&model)
}

unsafe extern "C-unwind" fn ec_ivf_build_callback(
    _index: pg_sys::Relation,
    tid: pg_sys::ItemPointer,
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    _tuple_is_alive: bool,
    state: *mut c_void,
) {
    // SAFETY: PostgreSQL calls this during `table_index_build_scan` with live
    // datum/null arrays, heap TID, and the BuildState pointer supplied below.
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = &mut *state.cast::<BuildState>();
            let heap_tid = decode_heap_tid(tid, "ambuild");
            let tuple = build_index_tuple(
                values,
                isnull,
                heap_tid,
                state.indexed_vector_kind,
                state.options.storage_format,
                state.options.effective_quant_bits(),
                "ambuild",
            );
            state.push(tuple);
        })
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    pg_am_callback!({
        // SAFETY: PostgreSQL invokes ambuild with a live IVF index relation.
        let index_relation_handle = NonNull::new(index_relation)
            .unwrap_or_else(|| pgrx::error!("ec_ivf ambuild received null index relation"));
        let options = options::relation_options(index_relation_handle);
        options
            .storage_format
            .validate_v1_supported()
            .unwrap_or_else(|e| pgrx::error!("{e}"));
        options
            .rerank
            .validate_v1_supported()
            .unwrap_or_else(|e| pgrx::error!("{e}"));
        page::initialize_metadata_page(index_relation, page::MetadataPage::empty(options));

        let indexed_vector_kind = resolve_indexed_vector_kind(heap_relation, index_info, "ambuild");
        let mut state = BuildState::new(options, indexed_vector_kind);
        reset_debug_last_build_timing();
        build_parallel::reset_debug_last_parallel_build_workers_launched();
        let parallel_plan = build_parallel::EcIvfParallelBuildPlan::from_index_info(index_info);
        let mut parallel_begin_us = 0_u64;
        let mut parallel_drain_us = 0_u64;
        let mut parallel_sort_push_us = 0_u64;
        let mut parallel_worker_tuple_buffer_capacity = 0_u64;
        let mut parallel_worker_tuple_buffer_struct_bytes = 0_u64;
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
                parallel_worker_tuple_buffer_capacity = result.worker_tuple_buffer_capacity;
                parallel_worker_tuple_buffer_struct_bytes = result.worker_tuple_buffer_struct_bytes;
                result.heap_tuples
            })
            .unwrap_or_else(|| {
                table_index_build_scan(heap_relation, index_relation, index_info, &mut state)
            })
        } else {
            table_index_build_scan(heap_relation, index_relation, index_info, &mut state)
        };
        let heap_ingest_us = elapsed_us(heap_ingest_start);
        let mut train_model_us = 0_u64;
        let mut stage_build_plan_us = 0_u64;
        let mut stage_timing = StageBuildTiming::default();
        let mut flush_build_plan_us = 0_u64;
        let index_tuples = if state.scanned_tuples == 0 {
            0.0
        } else {
            let train_start = Instant::now();
            let model = state
                .train_model()
                .unwrap_or_else(|e| pgrx::error!("ec_ivf centroid training failed: {e}"));
            train_model_us = elapsed_us(train_start);
            let stage_start = Instant::now();
            let (plan, measured_stage_timing) = state
                .stage_build_plan_with_timing(&model)
                .unwrap_or_else(|e| pgrx::error!("ec_ivf populated index staging failed: {e}"));
            stage_timing = measured_stage_timing;
            stage_build_plan_us = elapsed_us(stage_start);
            let flush_start = Instant::now();
            flush_build_plan(index_relation, &plan);
            flush_build_plan_us = elapsed_us(flush_start);
            plan.posting_count() as f64
        };
        if heap_tuples != state.scanned_tuples as f64 {
            pgrx::error!(
                "ec_ivf ambuild scanned {heap_tuples} heap tuples but observed {}",
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
            train_model_us,
            stage_build_plan_us,
            stage_pq_train_us: stage_timing.pq_train_us,
            stage_centroids_us: stage_timing.centroids_us,
            stage_assign_us: stage_timing.assign_us,
            stage_postings_us: stage_timing.postings_us,
            stage_directory_us: stage_timing.directory_us,
            flush_build_plan_us,
            parallel_begin_us,
            parallel_drain_us,
            parallel_sort_push_us,
            parallel_worker_tuple_buffer_capacity,
            parallel_worker_tuple_buffer_struct_bytes,
        });

        crate::fault::maybe_fail_palloc("ec_ivf ambuild result");
        let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
        result.heap_tuples = heap_tuples;
        result.index_tuples = index_tuples;
        result.into_pg()
    })
}

pub(crate) fn debug_last_build_timing() -> BuildTimingSnapshot {
    BuildTimingSnapshot {
        requested_workers: LAST_BUILD_REQUESTED_WORKERS.load(AtomicOrdering::Acquire),
        workers_launched: LAST_BUILD_WORKERS_LAUNCHED.load(AtomicOrdering::Acquire),
        heap_tuples: LAST_BUILD_HEAP_TUPLES.load(AtomicOrdering::Acquire),
        index_tuples: LAST_BUILD_INDEX_TUPLES.load(AtomicOrdering::Acquire),
        heap_ingest_us: LAST_BUILD_HEAP_INGEST_US.load(AtomicOrdering::Acquire),
        train_model_us: LAST_BUILD_TRAIN_MODEL_US.load(AtomicOrdering::Acquire),
        stage_build_plan_us: LAST_BUILD_STAGE_BUILD_PLAN_US.load(AtomicOrdering::Acquire),
        stage_pq_train_us: LAST_BUILD_STAGE_PQ_TRAIN_US.load(AtomicOrdering::Acquire),
        stage_centroids_us: LAST_BUILD_STAGE_CENTROIDS_US.load(AtomicOrdering::Acquire),
        stage_assign_us: LAST_BUILD_STAGE_ASSIGN_US.load(AtomicOrdering::Acquire),
        stage_postings_us: LAST_BUILD_STAGE_POSTINGS_US.load(AtomicOrdering::Acquire),
        stage_directory_us: LAST_BUILD_STAGE_DIRECTORY_US.load(AtomicOrdering::Acquire),
        flush_build_plan_us: LAST_BUILD_FLUSH_BUILD_PLAN_US.load(AtomicOrdering::Acquire),
        parallel_begin_us: LAST_BUILD_PARALLEL_BEGIN_US.load(AtomicOrdering::Acquire),
        parallel_drain_us: LAST_BUILD_PARALLEL_DRAIN_US.load(AtomicOrdering::Acquire),
        parallel_sort_push_us: LAST_BUILD_PARALLEL_SORT_PUSH_US.load(AtomicOrdering::Acquire),
        parallel_worker_tuple_buffer_capacity: LAST_BUILD_PARALLEL_WORKER_TUPLE_BUFFER_CAPACITY
            .load(AtomicOrdering::Acquire),
        parallel_worker_tuple_buffer_struct_bytes:
            LAST_BUILD_PARALLEL_WORKER_TUPLE_BUFFER_STRUCT_BYTES.load(AtomicOrdering::Acquire),
    }
}

fn reset_debug_last_build_timing() {
    record_debug_last_build_timing(BuildTimingSnapshot {
        requested_workers: 0,
        workers_launched: 0,
        heap_tuples: 0,
        index_tuples: 0,
        heap_ingest_us: 0,
        train_model_us: 0,
        stage_build_plan_us: 0,
        stage_pq_train_us: 0,
        stage_centroids_us: 0,
        stage_assign_us: 0,
        stage_postings_us: 0,
        stage_directory_us: 0,
        flush_build_plan_us: 0,
        parallel_begin_us: 0,
        parallel_drain_us: 0,
        parallel_sort_push_us: 0,
        parallel_worker_tuple_buffer_capacity: 0,
        parallel_worker_tuple_buffer_struct_bytes: 0,
    });
}

fn record_debug_last_build_timing(snapshot: BuildTimingSnapshot) {
    LAST_BUILD_REQUESTED_WORKERS.store(snapshot.requested_workers, AtomicOrdering::Release);
    LAST_BUILD_WORKERS_LAUNCHED.store(snapshot.workers_launched, AtomicOrdering::Release);
    LAST_BUILD_HEAP_TUPLES.store(snapshot.heap_tuples, AtomicOrdering::Release);
    LAST_BUILD_INDEX_TUPLES.store(snapshot.index_tuples, AtomicOrdering::Release);
    LAST_BUILD_HEAP_INGEST_US.store(snapshot.heap_ingest_us, AtomicOrdering::Release);
    LAST_BUILD_TRAIN_MODEL_US.store(snapshot.train_model_us, AtomicOrdering::Release);
    LAST_BUILD_STAGE_BUILD_PLAN_US.store(snapshot.stage_build_plan_us, AtomicOrdering::Release);
    LAST_BUILD_STAGE_PQ_TRAIN_US.store(snapshot.stage_pq_train_us, AtomicOrdering::Release);
    LAST_BUILD_STAGE_CENTROIDS_US.store(snapshot.stage_centroids_us, AtomicOrdering::Release);
    LAST_BUILD_STAGE_ASSIGN_US.store(snapshot.stage_assign_us, AtomicOrdering::Release);
    LAST_BUILD_STAGE_POSTINGS_US.store(snapshot.stage_postings_us, AtomicOrdering::Release);
    LAST_BUILD_STAGE_DIRECTORY_US.store(snapshot.stage_directory_us, AtomicOrdering::Release);
    LAST_BUILD_FLUSH_BUILD_PLAN_US.store(snapshot.flush_build_plan_us, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_BEGIN_US.store(snapshot.parallel_begin_us, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_DRAIN_US.store(snapshot.parallel_drain_us, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_SORT_PUSH_US.store(snapshot.parallel_sort_push_us, AtomicOrdering::Release);
    LAST_BUILD_PARALLEL_WORKER_TUPLE_BUFFER_CAPACITY.store(
        snapshot.parallel_worker_tuple_buffer_capacity,
        AtomicOrdering::Release,
    );
    LAST_BUILD_PARALLEL_WORKER_TUPLE_BUFFER_STRUCT_BYTES.store(
        snapshot.parallel_worker_tuple_buffer_struct_bytes,
        AtomicOrdering::Release,
    );
}

#[cfg(any(test, feature = "pg_test"))]
pub(crate) fn build_tuple_struct_size_for_test() -> usize {
    std::mem::size_of::<BuildTuple>()
}

fn elapsed_us(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
}

fn nonnegative_i32_to_u64(value: i32) -> u64 {
    u64::try_from(value.max(0)).expect("non-negative i32 should fit u64")
}

fn build_state_mut<'a>(state: *mut c_void, context: &str) -> &'a mut BuildState {
    // SAFETY: `table_index_build_scan` invokes the callback with the
    // BuildState pointer supplied by `table_index_build_scan` below.
    unsafe { state.cast::<BuildState>().as_mut() }
        .unwrap_or_else(|| pgrx::error!("ec_ivf {context} received null build state"))
}

fn table_index_build_scan(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    state: &mut BuildState,
) -> f64 {
    // SAFETY: PostgreSQL invokes ambuild with live heap/index relations and
    // IndexInfo; the stack BuildState outlives the synchronous build scan.
    unsafe {
        pg_sys::table_index_build_scan(
            heap_relation,
            index_relation,
            index_info,
            false,
            false,
            Some(ec_ivf_build_callback),
            (state as *mut BuildState).cast(),
            ptr::null_mut(),
        )
    }
}

impl BuildState {
    fn new(options: options::EcIvfOptions, indexed_vector_kind: IndexedVectorKind) -> Self {
        Self {
            options,
            indexed_vector_kind,
            page_size: pg_sys::BLCKSZ as usize,
            scanned_tuples: 0,
            heap_tuples: Vec::new(),
            dimensions: None,
        }
    }

    pub(super) fn push(&mut self, tuple: BuildTuple) {
        self.try_push(tuple)
            .unwrap_or_else(|e| pgrx::error!("ec_ivf ambuild found invalid indexed tuple: {e}"));
    }

    fn try_push(&mut self, tuple: BuildTuple) -> Result<(), String> {
        if tuple.heap_tid == ItemPointer::INVALID {
            return Err("heap tid must be valid".into());
        }
        if !tuple.gamma.is_finite() {
            return Err("posting gamma must be finite".into());
        }
        if tuple.source_vector.len() != usize::from(tuple.dimensions) {
            return Err(format!(
                "source dimensions mismatch: source dim {} vs indexed dim {}",
                tuple.source_vector.len(),
                tuple.dimensions
            ));
        }
        let expected_payload_len = IvfQuantizer::resolve_with_pq_group_size_and_bits(
            self.options.storage_format,
            usize::from(tuple.dimensions),
            self.options.requested_pq_group_size(),
            Some(self.options.effective_quant_bits()),
        )?
        .payload_len();
        let deferred_pq_encode = self.options.storage_format == options::StorageFormat::PqFastScan
            && tuple.payload.is_empty();
        if !deferred_pq_encode && tuple.payload.len() != expected_payload_len {
            return Err(format!(
                "posting payload length mismatch: got {}, expected {expected_payload_len} for dim {}",
                tuple.payload.len(),
                tuple.dimensions
            ));
        }
        training::normalize_vector(&tuple.source_vector, usize::from(tuple.dimensions))?;
        if !page::posting_tuple_fits(tuple.payload.len(), self.page_size) {
            return Err(format!(
                "posting payload for dim {} does not fit on a page",
                tuple.dimensions
            ));
        }

        match self.dimensions {
            None => self.dimensions = Some(tuple.dimensions),
            Some(dimensions) if dimensions == tuple.dimensions => {}
            Some(dimensions) => {
                return Err(format!(
                    "dimension mismatch: saw {} after {}",
                    tuple.dimensions, dimensions
                ));
            }
        }

        self.scanned_tuples += 1;
        self.heap_tuples.push(tuple);
        Ok(())
    }

    fn training_sample_count(&self) -> usize {
        resolve_training_sample_count(self.options.training_sample_rows, self.heap_tuples.len())
    }

    fn training_sample_vectors(&self) -> Vec<&[f32]> {
        let indices = training::deterministic_sample_indices(
            self.heap_tuples.len(),
            self.training_sample_count(),
            self.options.seed as u64,
        );
        indices
            .into_iter()
            .map(|index| self.heap_tuples[index].source_vector.as_slice())
            .collect()
    }

    fn train_model(&self) -> Result<training::SphericalKMeansModel, String> {
        let dimensions = self
            .dimensions
            .ok_or_else(|| "centroid training requires at least one tuple".to_owned())?;
        let nlists = training::resolve_auto_nlists(
            u32::try_from(self.options.nlists)
                .map_err(|_| "validated nlists should be non-negative".to_owned())?,
            self.heap_tuples.len(),
        );
        let sample_vectors = self.training_sample_vectors();
        training::train_spherical_kmeans(
            &sample_vectors,
            usize::from(dimensions),
            nlists,
            self.options.seed as u64,
            DEFAULT_KMEANS_ITERATIONS,
        )
    }

    fn stage_build_plan(
        &self,
        model: &training::SphericalKMeansModel,
    ) -> Result<IvfBuildPlan, String> {
        self.stage_build_plan_with_timing(model)
            .map(|(plan, _timing)| plan)
    }

    fn stage_build_plan_with_timing(
        &self,
        model: &training::SphericalKMeansModel,
    ) -> Result<(IvfBuildPlan, StageBuildTiming), String> {
        let mut timing = StageBuildTiming::default();
        let dimensions = self
            .dimensions
            .ok_or_else(|| "bulk assignment requires at least one tuple".to_owned())?;
        if model.dimensions != usize::from(dimensions) {
            return Err(format!(
                "model dimensions mismatch: got {}, expected {}",
                model.dimensions, dimensions
            ));
        }
        if model.centroid_count() == 0 {
            return Err("bulk assignment requires at least one centroid".into());
        }
        if !page::centroid_tuple_fits(model.dimensions, self.page_size) {
            return Err(format!(
                "centroid tuple for dim {} does not fit on a page",
                model.dimensions
            ));
        }
        if !page::list_directory_tuple_fits(self.page_size) {
            return Err("list directory tuple does not fit on a page".into());
        }
        let pq_train_start = Instant::now();
        let pq_model = self.train_pq_fastscan_model(dimensions)?;
        timing.pq_train_us = elapsed_us(pq_train_start);
        let pq_centroid_count = pq_model
            .as_ref()
            .map(|model| model.group_size * crate::quant::grouped_pq::GROUPED_PQ_CENTROIDS)
            .unwrap_or(0);
        if pq_model.is_some() && !page::pq_codebook_tuple_fits(pq_centroid_count, self.page_size) {
            return Err(format!(
                "pq_fastscan codebook tuple for group centroid count {pq_centroid_count} does not fit on a page"
            ));
        }

        let nlists = model.centroid_count();
        let mut data_pages = DataPageChain::new(self.page_size);
        let mut centroid_tids = Vec::with_capacity(nlists);
        let centroids_start = Instant::now();
        for (list_id, centroid) in model.centroids.iter().enumerate() {
            let centroid = page::IvfCentroidTuple {
                list_id: list_id_u32(list_id)?,
                centroid: centroid.clone(),
            };
            centroid_tids.push(data_pages.insert_ivf_centroid(&centroid)?);
        }

        let mut pq_codebook_head = ItemPointer::INVALID;
        if let Some(pq_model) = &pq_model {
            let mut pq_codebook_tids = Vec::with_capacity(pq_model.group_count);
            for group_index in 0..pq_model.group_count {
                let start = group_index * pq_centroid_count;
                let end = start + pq_centroid_count;
                let tuple = page::IvfPqCodebookTuple {
                    group_index: u16::try_from(group_index)
                        .map_err(|_| "pq_fastscan group index exceeds u16".to_owned())?,
                    next_tid: ItemPointer::INVALID,
                    centroids: pq_model.flat_codebooks[start..end].to_vec(),
                };
                pq_codebook_tids.push(data_pages.insert_ivf_pq_codebook(&tuple)?);
            }
            for group_index in 0..pq_codebook_tids.len() {
                let next_tid = pq_codebook_tids
                    .get(group_index + 1)
                    .copied()
                    .unwrap_or(ItemPointer::INVALID);
                let start = group_index * pq_centroid_count;
                let end = start + pq_centroid_count;
                let tuple = page::IvfPqCodebookTuple {
                    group_index: u16::try_from(group_index)
                        .map_err(|_| "pq_fastscan group index exceeds u16".to_owned())?,
                    next_tid,
                    centroids: pq_model.flat_codebooks[start..end].to_vec(),
                };
                data_pages.update_ivf_pq_codebook(pq_codebook_tids[group_index], &tuple)?;
            }
            pq_codebook_head = pq_codebook_tids
                .first()
                .copied()
                .unwrap_or(ItemPointer::INVALID);
        }
        timing.centroids_us = elapsed_us(centroids_start);

        let source_refs = self
            .heap_tuples
            .iter()
            .map(|tuple| tuple.source_vector.as_slice())
            .collect::<Vec<_>>();
        let assign_start = Instant::now();
        let list_ids = training::assign_vectors_to_centroids(&source_refs, model)?;
        let mut tuple_indices_by_list = vec![Vec::new(); nlists];
        for (tuple_index, list_id) in list_ids.into_iter().enumerate() {
            tuple_indices_by_list[list_id].push(tuple_index);
        }
        timing.assign_us = elapsed_us(assign_start);
        let pq_posting_quantizer = if pq_model.is_some() {
            Some(IvfQuantizer::resolve_with_pq_group_size_and_bits(
                self.options.storage_format,
                usize::from(dimensions),
                self.options.requested_pq_group_size(),
                Some(self.options.effective_quant_bits()),
            )?)
        } else {
            None
        };
        // Task 115: when residual encoding is gated on (RaBitQ only), resolve a
        // residual-mode quantizer once; postings are re-encoded against their
        // assigned centroid inside the per-list loop below.
        let residual_posting_quantizer = if self.options.rabitq_residual {
            Some(IvfQuantizer::resolve_with_pq_group_size_bits_and_residual(
                self.options.storage_format,
                usize::from(dimensions),
                self.options.requested_pq_group_size(),
                Some(self.options.effective_quant_bits()),
                true,
            )?)
        } else {
            None
        };

        // Build the compact rerank sidecar before postings so each posting can
        // persist a direct pointer to its sidecar block. That lets scan skip the
        // O(N) sidecar directory for fresh builds.
        data_pages.start_new_page_if_current_has_tuples();
        let (rerank_sidecar_head, rerank_sidecar_directory_head, rerank_tids_by_heap_tid) =
            build_rerank_sidecar_chain(
                &mut data_pages,
                &self.heap_tuples,
                &self.options,
                dimensions,
            )?;

        let mut posting_tids_by_list = vec![Vec::new(); nlists];
        let mut directory_tail_blocks_by_list = vec![None; nlists];
        let mut live_posting_counts_by_list = vec![0_u64; nlists];
        data_pages.start_new_page_if_current_has_tuples();
        let postings_start = Instant::now();
        let use_dense_posting_blocks = dense_posting_blocks_enabled(self.options);
        let dense_posting_typed_layout = self.options.dense_posting_typed_layout;
        for (list_id, tuple_indices) in tuple_indices_by_list.iter().enumerate() {
            if !tuple_indices.is_empty() {
                data_pages.start_new_page_if_current_has_tuples();
            }
            live_posting_counts_by_list[list_id] = u64::try_from(tuple_indices.len())
                .map_err(|_| "posting count exceeds u64".to_owned())?;
            if use_dense_posting_blocks && !tuple_indices.is_empty() {
                let list_id_u32 = list_id_u32(list_id)?;
                let mut pending = Vec::new();
                let mut pending_payload_len = None;
                let mut pending_posting_limit = usize::MAX;
                let residual_posting =
                    residual_posting_quantizer.map(|q| (q, model.centroids[list_id].as_slice()));
                for tuple_index in tuple_indices {
                    let (heap_tid, gamma, payload) = encode_build_posting(
                        &self.heap_tuples[*tuple_index],
                        &pq_model,
                        pq_posting_quantizer,
                        residual_posting,
                    )?;
                    let payload_len = payload.len();
                    if let Some(existing) = pending_payload_len {
                        if existing != payload_len {
                            return Err(
                                "ec_ivf dense posting block payload length changed within a list"
                                    .to_owned(),
                            );
                        }
                    } else {
                        pending_payload_len = Some(payload_len);
                        pending_posting_limit =
                            dense_posting_group_limit(payload_len, self.page_size)?;
                    }
                    if pending.len() >= pending_posting_limit {
                        insert_dense_posting_group(
                            &mut data_pages,
                            &mut posting_tids_by_list[list_id],
                            list_id_u32,
                            &pending,
                            payload_len,
                            dense_posting_typed_layout,
                        )?;
                        pending.clear();
                    }
                    let rerank_tid = rerank_tids_by_heap_tid
                        .get(&heap_tid)
                        .copied()
                        .unwrap_or(ItemPointer::INVALID);
                    pending.push((heap_tid, gamma, rerank_tid, payload));
                }
                if !pending.is_empty() {
                    let payload_len = pending_payload_len.expect("pending postings have payloads");
                    insert_dense_posting_group(
                        &mut data_pages,
                        &mut posting_tids_by_list[list_id],
                        list_id_u32,
                        &pending,
                        payload_len,
                        dense_posting_typed_layout,
                    )?;
                }
            } else {
                let residual_posting =
                    residual_posting_quantizer.map(|q| (q, model.centroids[list_id].as_slice()));
                for tuple_index in tuple_indices {
                    let (heap_tid, gamma, payload) = encode_build_posting(
                        &self.heap_tuples[*tuple_index],
                        &pq_model,
                        pq_posting_quantizer,
                        residual_posting,
                    )?;
                    posting_tids_by_list[list_id].push(
                        data_pages.insert_ivf_single_heaptid_posting(
                            list_id_u32(list_id)?,
                            heap_tid,
                            gamma,
                            rerank_tids_by_heap_tid
                                .get(&heap_tid)
                                .copied()
                                .unwrap_or(ItemPointer::INVALID),
                            &payload,
                        )?,
                    );
                }
            }
            if !tuple_indices.is_empty() {
                let head = posting_tids_by_list[list_id]
                    .first()
                    .expect("non-empty list should have posting tids")
                    .block_number;
                let tail = directory_tail_blocks_by_list[list_id].unwrap_or_else(|| {
                    posting_tids_by_list[list_id]
                        .last()
                        .expect("non-empty list should have posting tids")
                        .block_number
                });
                let posting_pages = tail - head + 1;
                directory_tail_blocks_by_list[list_id] = Some(tail);
                let slack_pages =
                    posting_slack_pages(posting_pages, self.options.posting_slack_percent)?;
                if slack_pages > 0 {
                    // +1 reserves a separator page after the slack range so the
                    // next list's first insert cannot reuse this list's last
                    // slack page; the directory tail is the last in-range slack
                    // block (slack_tail - 1).
                    let (_, slack_tail) = data_pages
                        .append_empty_pages(slack_pages + 1)
                        .expect("positive slack should append pages");
                    directory_tail_blocks_by_list[list_id] = Some(
                        slack_tail
                            .checked_sub(1)
                            .ok_or_else(|| "posting slack page underflow".to_owned())?,
                    );
                }
            }
        }
        timing.postings_us = elapsed_us(postings_start);

        let mut directory_entries = Vec::with_capacity(nlists);
        let mut directory_tids = Vec::with_capacity(nlists);
        data_pages.start_new_page_if_current_has_tuples();
        let directory_start = Instant::now();
        for (list_id, posting_tids) in posting_tids_by_list.iter().enumerate() {
            let mut directory = page::IvfListDirectoryTuple::empty(list_id_u32(list_id)?);
            if let (Some(head), Some(tail)) = (posting_tids.first(), posting_tids.last()) {
                directory.head_block = page::BlockRef {
                    block_number: head.block_number,
                };
                directory.tail_block = page::BlockRef {
                    block_number: directory_tail_blocks_by_list[list_id]
                        .unwrap_or(tail.block_number),
                };
                directory.live_count = live_posting_counts_by_list[list_id];
            }
            directory_tids.push(data_pages.insert_ivf_list_directory(directory)?);
            directory_entries.push(directory);
        }
        timing.directory_us = elapsed_us(directory_start);

        let mut metadata = page::MetadataPage::empty(self.options);
        metadata.dimensions = dimensions;
        metadata.nlists =
            u32::try_from(nlists).map_err(|_| "centroid count exceeds u32".to_owned())?;
        metadata.training_version = 1;
        metadata.centroid_head = centroid_tids
            .first()
            .copied()
            .unwrap_or(ItemPointer::INVALID);
        metadata.directory_head = directory_tids
            .first()
            .copied()
            .unwrap_or(ItemPointer::INVALID);
        if let Some(pq_model) = pq_model {
            metadata.pq_codebook_head = pq_codebook_head;
            metadata.pq_group_size = u16::try_from(pq_model.group_size)
                .map_err(|_| "pq_fastscan group size exceeds u16".to_owned())?;
        }
        metadata.total_live_tuples = u64::try_from(self.heap_tuples.len())
            .map_err(|_| "heap tuple count exceeds u64".to_owned())?;
        metadata.rerank_sidecar_head = rerank_sidecar_head;
        metadata.rerank_sidecar_directory_head = rerank_sidecar_directory_head;

        Ok((
            IvfBuildPlan {
                data_pages,
                metadata,
                centroid_tids,
                directory_tids,
                posting_tids_by_list,
                directory_entries,
            },
            timing,
        ))
    }

    fn train_pq_fastscan_model(
        &self,
        dimensions: u16,
    ) -> Result<Option<IvfPqFastScanModel>, String> {
        if self.options.storage_format != options::StorageFormat::PqFastScan {
            return Ok(None);
        }
        let group_size = quantizer::resolve_pq_fastscan_group_size(
            usize::from(dimensions),
            self.options.requested_pq_group_size(),
        )?;
        let sample_vectors = self.training_sample_vectors();
        let model = common_training::train_grouped_pq4_model(
            &sample_vectors,
            usize::from(dimensions),
            self.options.seed as u64,
            group_size,
            self.training_sample_count(),
            DEFAULT_PQ_FASTSCAN_KMEANS_ITERATIONS,
        )?;
        Ok(Some(IvfPqFastScanModel {
            group_count: model.group_count,
            group_size: model.group_size,
            signs: model.signs,
            flat_codebooks: model.codebooks.into_iter().flatten().collect(),
        }))
    }
}

fn posting_slack_pages(posting_pages: u32, slack_percent: i32) -> Result<usize, String> {
    if posting_pages == 0 || slack_percent <= 0 {
        return Ok(0);
    }
    let posting_pages = u64::from(posting_pages);
    let slack_percent =
        u64::try_from(slack_percent).map_err(|_| "posting slack percent is negative")?;
    let slack_pages = posting_pages
        .checked_mul(slack_percent)
        .and_then(|value| value.checked_add(99))
        .ok_or_else(|| "posting slack page count overflow".to_owned())?
        / 100;
    usize::try_from(slack_pages).map_err(|_| "posting slack page count exceeds usize".to_owned())
}

fn dense_posting_blocks_enabled(options: options::EcIvfOptions) -> bool {
    options.dense_posting_blocks
        && matches!(
            options.storage_format,
            options::StorageFormat::Auto
                | options::StorageFormat::TurboQuant
                | options::StorageFormat::RaBitQ
                | options::StorageFormat::CoarseRerank
        )
}

fn dense_posting_group_limit(payload_len: usize, page_size: usize) -> Result<usize, String> {
    let fit = page::dense_posting_block_tuple_fits;
    let mut segment_capacity = 1_usize;
    if !fit(1, 1, payload_len, page_size) {
        return Err(format!(
            "ec_ivf dense posting payload length {payload_len} does not fit on a page"
        ));
    }
    while segment_capacity < u16::MAX as usize
        && fit(
            segment_capacity + 1,
            segment_capacity + 1,
            payload_len,
            page_size,
        )
    {
        segment_capacity += 1;
    }
    Ok(segment_capacity.min(u16::MAX as usize).max(1))
}

/// Task 111g: build the compact rerank sidecar chain (tag 0x2A) for
/// `rerank_placement = 'index'`. Returns the head ItemPointer, or `INVALID`
/// when no sidecar is persisted (source placement, or an f32/auto rerank_format
/// that keeps the heap source). Entries are written globally tid-sorted so the
/// scan reads them in heap-TID order; blocks chain via `next_tid`.
fn build_rerank_sidecar_chain(
    data_pages: &mut DataPageChain,
    heap_tuples: &[BuildTuple],
    options: &options::EcIvfOptions,
    dimensions: u16,
) -> Result<(ItemPointer, ItemPointer, HashMap<ItemPointer, ItemPointer>), String> {
    // Returns (sidecar_head, directory_head, heap_tid -> sidecar block tid).
    // ADR-079: the directory chain maps each sidecar block's first heap TID to
    // the block pointer for bounded reads; direct posting pointers are the hot
    // query path when available.
    if options.rerank_placement != options::RerankPlacement::Index {
        return Ok((ItemPointer::INVALID, ItemPointer::INVALID, HashMap::new()));
    }
    let Some(encoder) = super::rerank::RerankSidecarEncoder::resolve(
        options.rerank_format,
        usize::from(dimensions),
    )?
    else {
        // f32 / auto keep the heap source: no sidecar.
        return Ok((ItemPointer::INVALID, ItemPointer::INVALID, HashMap::new()));
    };
    if heap_tuples.is_empty() {
        return Ok((ItemPointer::INVALID, ItemPointer::INVALID, HashMap::new()));
    }
    let payload_len = encoder.payload_len(usize::from(dimensions));
    let rerank_format = options.rerank_format as u8;
    let page_size = data_pages.page_size();

    // Globally tid-sort the rows so the sidecar chain is ascending by heap TID.
    let mut order: Vec<usize> = (0..heap_tuples.len()).collect();
    order.sort_by(|&a, &b| {
        let ta = heap_tuples[a].heap_tid;
        let tb = heap_tuples[b].heap_tid;
        ta.block_number
            .cmp(&tb.block_number)
            .then_with(|| ta.offset_number.cmp(&tb.offset_number))
    });

    let entries_per_block = rerank_sidecar_entries_per_block(payload_len, page_size)?;

    data_pages.start_new_page_if_current_has_tuples();
    let mut block_tids: Vec<ItemPointer> = Vec::new();
    let mut rerank_tids_by_heap_tid: HashMap<ItemPointer, ItemPointer> =
        HashMap::with_capacity(heap_tuples.len());
    // ADR-079: each block's first (smallest) heap TID, parallel to block_tids;
    // becomes the directory key for survivor-directed bounded reads.
    let mut block_first_tids: Vec<ItemPointer> = Vec::new();
    let mut pending_tids: Vec<ItemPointer> = Vec::with_capacity(entries_per_block);
    let mut pending_payloads: Vec<u8> = Vec::with_capacity(entries_per_block * payload_len);

    let flush = |data_pages: &mut DataPageChain,
                 block_tids: &mut Vec<ItemPointer>,
                 rerank_tids_by_heap_tid: &mut HashMap<ItemPointer, ItemPointer>,
                 block_first_tids: &mut Vec<ItemPointer>,
                 pending_tids: &mut Vec<ItemPointer>,
                 pending_payloads: &mut Vec<u8>|
     -> Result<(), String> {
        if pending_tids.is_empty() {
            return Ok(());
        }
        let block = page::IvfRerankSidecarBlockTuple::new(
            rerank_format,
            payload_len,
            std::mem::take(pending_tids),
            std::mem::take(pending_payloads),
        )?;
        let first_tid = block.heap_tids[0];
        let block_tid = data_pages.insert_ivf_rerank_sidecar_block(&block)?;
        for heap_tid in &block.heap_tids {
            rerank_tids_by_heap_tid.insert(*heap_tid, block_tid);
        }
        block_tids.push(block_tid);
        block_first_tids.push(first_tid);
        Ok(())
    };

    for &index in &order {
        let tuple = &heap_tuples[index];
        let payload = encoder.encode(&tuple.source_vector)?;
        if payload.len() != payload_len {
            return Err(format!(
                "ec_ivf rerank sidecar payload length {} does not match expected {payload_len}",
                payload.len()
            ));
        }
        pending_tids.push(tuple.heap_tid);
        pending_payloads.extend_from_slice(&payload);
        if pending_tids.len() >= entries_per_block {
            flush(
                data_pages,
                &mut block_tids,
                &mut rerank_tids_by_heap_tid,
                &mut block_first_tids,
                &mut pending_tids,
                &mut pending_payloads,
            )?;
        }
    }
    flush(
        data_pages,
        &mut block_tids,
        &mut rerank_tids_by_heap_tid,
        &mut block_first_tids,
        &mut pending_tids,
        &mut pending_payloads,
    )?;

    // Link each block to the next via its next_tid field (second pass, like the
    // pq_codebook chain).
    for i in 0..block_tids.len() {
        let next_tid = block_tids
            .get(i + 1)
            .copied()
            .unwrap_or(ItemPointer::INVALID);
        let mut block = data_pages.read_ivf_rerank_sidecar_block_staged(block_tids[i])?;
        block.next_tid = next_tid;
        data_pages.update_ivf_rerank_sidecar_block(block_tids[i], &block)?;
    }

    let sidecar_head = block_tids.first().copied().unwrap_or(ItemPointer::INVALID);

    // ADR-079: build the directory chain. It reuses the 0x2A block format: each
    // entry's "heap_tid" is a sidecar block's first heap TID and its 6-byte
    // "payload" is that block's ItemPointer. Globally tid-sorted (block_first_tids
    // is ascending because the sidecar chain is). Scan binary-searches this small
    // directory to read only the survivor blocks instead of the whole chain.
    let dir_payload_len = crate::storage::page::ITEM_POINTER_BYTES;
    let dir_entries_per_block = rerank_sidecar_entries_per_block(dir_payload_len, page_size)?;
    let mut dir_block_tids: Vec<ItemPointer> = Vec::new();
    let mut start = 0;
    while start < block_first_tids.len() {
        let end = (start + dir_entries_per_block).min(block_first_tids.len());
        let keys = block_first_tids[start..end].to_vec();
        let mut payloads = Vec::with_capacity((end - start) * dir_payload_len);
        for block_tid in &block_tids[start..end] {
            block_tid.encode_into(&mut payloads);
        }
        let dir_block =
            page::IvfRerankSidecarBlockTuple::new(rerank_format, dir_payload_len, keys, payloads)?;
        dir_block_tids.push(data_pages.insert_ivf_rerank_sidecar_block(&dir_block)?);
        start = end;
    }
    for i in 0..dir_block_tids.len() {
        let next_tid = dir_block_tids
            .get(i + 1)
            .copied()
            .unwrap_or(ItemPointer::INVALID);
        let mut block = data_pages.read_ivf_rerank_sidecar_block_staged(dir_block_tids[i])?;
        block.next_tid = next_tid;
        data_pages.update_ivf_rerank_sidecar_block(dir_block_tids[i], &block)?;
    }
    let directory_head = dir_block_tids
        .first()
        .copied()
        .unwrap_or(ItemPointer::INVALID);

    Ok((sidecar_head, directory_head, rerank_tids_by_heap_tid))
}

/// Max sidecar entries that fit in one page-sized 0x2A block, given the compact
/// payload width, capped at the `u16` entry-count field width. Errors if not
/// even one entry fits a page.
fn rerank_sidecar_entries_per_block(payload_len: usize, page_size: usize) -> Result<usize, String> {
    if !page::rerank_sidecar_block_tuple_fits(1, payload_len, page_size) {
        return Err(format!(
            "ec_ivf rerank sidecar payload length {payload_len} does not fit a single entry on a page"
        ));
    }
    // Grow until the next entry would overflow the page (or the u16 cap).
    let mut count = 1;
    while count < u16::MAX as usize
        && page::rerank_sidecar_block_tuple_fits(count + 1, payload_len, page_size)
    {
        count += 1;
    }
    Ok(count)
}

fn insert_dense_posting_group(
    data_pages: &mut DataPageChain,
    posting_tids: &mut Vec<ItemPointer>,
    list_id: u32,
    postings: &[(ItemPointer, f32, ItemPointer, Vec<u8>)],
    payload_len: usize,
    typed_layout: bool,
) -> Result<(), String> {
    let dense = page::IvfDensePostingBlockTuple::from_single_heaptid_postings(
        list_id,
        postings,
        payload_len,
    )?;
    posting_tids.push(if typed_layout {
        data_pages.insert_ivf_dense_posting_aligned_block(&dense)?
    } else {
        data_pages.insert_ivf_dense_posting_block(&dense)?
    });
    Ok(())
}

fn encode_build_posting(
    tuple: &BuildTuple,
    pq_model: &Option<IvfPqFastScanModel>,
    pq_posting_quantizer: Option<IvfQuantizer>,
    residual_posting: Option<(IvfQuantizer, &[f32])>,
) -> Result<(ItemPointer, f32, Vec<u8>), String> {
    let (gamma, payload) = match (pq_model, residual_posting) {
        (Some(pq_model), _) => {
            let (_, gamma, payload) = pq_posting_quantizer
                .expect("pq posting quantizer should exist for pq model")
                .encode_source_with_pq_model(&tuple.source_vector, pq_model)?;
            (gamma, payload)
        }
        // Task 115: residual mode re-encodes the source against the assigned
        // centroid here (the only point in the build where the list's centroid
        // is in hand), mirroring the deferred PQ re-encode path above.
        (None, Some((residual_quantizer, centroid))) => {
            let (_, gamma, payload) =
                residual_quantizer.encode_source_residual(&tuple.source_vector, centroid)?;
            (gamma, payload)
        }
        (None, None) => (tuple.gamma, tuple.payload.clone()),
    };
    Ok((tuple.heap_tid, gamma, payload))
}

fn resolve_training_sample_count(requested_sample_rows: i32, row_count: usize) -> usize {
    if row_count == 0 {
        return 0;
    }
    if requested_sample_rows > 0 {
        return (requested_sample_rows as usize).min(row_count);
    }
    row_count.min(DEFAULT_AUTO_TRAINING_SAMPLE_ROWS)
}

pub(super) unsafe fn flush_build_plan(index_relation: pg_sys::Relation, plan: &IvfBuildPlan) {
    let metadata_nlists = usize::try_from(plan.metadata.nlists).expect("u32 nlists should fit");
    debug_assert_eq!(plan.centroid_count(), metadata_nlists);
    debug_assert_eq!(plan.directory_count(), metadata_nlists);
    debug_assert!(plan.empty_list_count() <= metadata_nlists);
    debug_assert!(plan.data_page_count() > 0);
    debug_assert_eq!(plan.total_live_tuples(), plan.posting_count() as u64);

    let handle = NonNull::new(index_relation)
        .unwrap_or_else(|| pgrx::error!("ec_ivf flush_build_plan received a null index relation"));
    write_data_pages(handle, &plan.data_pages);
    // SAFETY: same live relation; metadata belongs to the staged plan just
    // flushed to disk.
    page::initialize_metadata_page(index_relation, plan.metadata);
}

fn write_data_pages(handle: RelationHandle, data_pages: &DataPageChain) {
    for staged_page in data_pages.pages() {
        write_data_page(handle, staged_page);
    }
}

fn write_data_page(handle: RelationHandle, staged_page: &crate::storage::page::DataPage) {
    let buffer = LockedBufferGuard::read_main_locked_handle(
        handle,
        P_NEW,
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
    )
    .unwrap_or_else(|| {
        pgrx::error!(
            "ec_ivf failed to allocate data buffer for block {}",
            staged_page.block_number()
        )
    });

    let mut wal_txn = wal::WalTxnScope::start_handle(handle);
    {
        let mut page = wal_txn.register_page(&buffer);
        page.init(0);
        for tuple in staged_page.tuples() {
            page.add_item(tuple).unwrap_or_else(|err| {
                pgrx::error!("ec_ivf failed to write tuple to block {}", err.block_number)
            });
        }
    }
    wal_txn.finish();
}

fn list_id_u32(list_id: usize) -> Result<u32, String> {
    u32::try_from(list_id).map_err(|_| format!("ec_ivf list id {list_id} exceeds u32"))
}

pub(super) fn build_index_tuple(
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: ItemPointer,
    indexed_vector_kind: IndexedVectorKind,
    storage_format: options::StorageFormat,
    quant_bits: u8,
    context: &str,
) -> BuildTuple {
    if values.is_null() || isnull.is_null() {
        pgrx::error!("ec_ivf {context} received null tuple value arrays");
    }
    let datum = build_index_tuple_datum(values, isnull, context);
    if datum.is_null() {
        pgrx::error!("ec_ivf {context} received a null indexed datum");
    }

    let bytes = detoasted_varlena_bytes(datum, "indexed vector column");
    match indexed_vector_kind {
        IndexedVectorKind::Ecvector => {
            build_ecvector_tuple(heap_tid, &bytes, storage_format, quant_bits, context)
        }
        IndexedVectorKind::Tqvector => {
            build_tqvector_tuple(heap_tid, &bytes, storage_format, quant_bits, context)
        }
    }
}

fn build_index_tuple_datum(
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    context: &str,
) -> pg_sys::Datum {
    // SAFETY: null was checked by caller and PostgreSQL provides at least the
    // first null flag and datum for this single-column AM callback.
    unsafe {
        if *isnull {
            pgrx::error!("ec_ivf does not support NULL indexed values");
        }
        let datum = *values;
        if datum.is_null() {
            pgrx::error!("ec_ivf {context} received a null indexed datum");
        }
        datum
    }
}

fn build_ecvector_tuple(
    heap_tid: ItemPointer,
    bytes: &[u8],
    storage_format: options::StorageFormat,
    quant_bits: u8,
    context: &str,
) -> BuildTuple {
    let source_vector = crate::unpack_raw_f32(bytes, "ec_ivf indexed ecvector column")
        .unwrap_or_else(|e| pgrx::error!("ec_ivf {context} found invalid indexed ecvector: {e}"));
    if storage_format == options::StorageFormat::PqFastScan {
        let dimensions = u16::try_from(source_vector.len()).unwrap_or_else(|_| {
            pgrx::error!(
                "ec_ivf {context} found invalid indexed ecvector: embedding dimension {} exceeds maximum 65535",
                source_vector.len()
            )
        });
        return BuildTuple {
            heap_tid,
            dimensions,
            gamma: 0.0,
            payload: Vec::new(),
            source_vector,
        };
    }
    let quantizer = IvfQuantizer::resolve_with_pq_group_size_and_bits(
        storage_format,
        source_vector.len(),
        None,
        Some(quant_bits),
    )
    .unwrap_or_else(|e| pgrx::error!("{e}"));
    let (dimensions, gamma, payload) = quantizer
        .encode_source(&source_vector)
        .unwrap_or_else(|e| pgrx::error!("ec_ivf {context} found invalid indexed ecvector: {e}"));

    BuildTuple {
        heap_tid,
        dimensions,
        gamma,
        payload,
        source_vector,
    }
}

fn build_tqvector_tuple(
    heap_tid: ItemPointer,
    bytes: &[u8],
    storage_format: options::StorageFormat,
    _quant_bits: u8,
    context: &str,
) -> BuildTuple {
    let (dimensions, bits, seed, gamma, code) = crate::unpack(bytes)
        .unwrap_or_else(|e| pgrx::error!("ec_ivf {context} found invalid indexed tqvector: {e}"));
    let payload = code.to_vec();

    let quantizer = ProdQuantizer::cached(usize::from(dimensions), bits, seed);
    let mut full_payload = Vec::with_capacity(4 + payload.len());
    full_payload.extend_from_slice(&gamma.to_le_bytes());
    full_payload.extend_from_slice(&payload);
    let source_vector = quantizer.decode_approximate(&full_payload);
    if storage_format == options::StorageFormat::PqFastScan {
        return BuildTuple {
            heap_tid,
            dimensions,
            gamma: 0.0,
            payload: Vec::new(),
            source_vector,
        };
    }

    BuildTuple {
        heap_tid,
        dimensions,
        gamma,
        payload,
        source_vector,
    }
}

fn detoasted_varlena_bytes(datum: pg_sys::Datum, label: &str) -> Vec<u8> {
    // SAFETY: caller passes a non-null varlena datum; DetoastedVarlena owns
    // any detoasted copy for the duration of conversion.
    unsafe { DetoastedVarlena::packed_from_datum(datum) }
        .unwrap_or_else(|| pgrx::error!("ec_ivf could not detoast {label}"))
        .to_vec()
}

pub(super) fn decode_heap_tid(tid: pg_sys::ItemPointer, context: &str) -> ItemPointer {
    if tid.is_null() {
        pgrx::error!("ec_ivf {context} received a null heap tid");
    }
    crate::am::common::pg_ptr::item_pointer(
        std::ptr::NonNull::new(tid).expect("ec_ivf heap tid should be non-null"),
    )
}

pub(super) fn resolve_indexed_vector_kind(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    context: &str,
) -> IndexedVectorKind {
    let index_info = index_info_ref(index_info, context);
    if index_info.ii_NumIndexAttrs != 1 || index_info.ii_NumIndexKeyAttrs != 1 {
        pgrx::error!("ec_ivf currently supports single-column indexes only");
    }
    if !index_info.ii_Expressions.is_null() {
        pgrx::error!("ec_ivf does not support expression indexes yet");
    }
    if !index_info.ii_Predicate.is_null() {
        pgrx::error!("ec_ivf does not support partial indexes yet");
    }

    let attnum = i32::from(index_info.ii_IndexAttrNumbers[0]);
    if attnum <= 0 {
        pgrx::error!("ec_ivf requires a base heap column index key");
    }

    let tuple_desc = unsafe { heap_relation_tuple_desc(heap_relation, context) };
    let att = tuple_desc
        .get(attnum as usize - 1)
        .expect("resolved indexed attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_ivf indexed column references a dropped column");
    }
    resolve_indexed_vector_kind_from_type(att.atttypid)
        .unwrap_or_else(|| pgrx::error!("ec_ivf indexed column must be ecvector or tqvector"))
}

fn index_info_ref<'a>(index_info: *mut pg_sys::IndexInfo, context: &str) -> &'a pg_sys::IndexInfo {
    let Some(index_info) = std::ptr::NonNull::new(index_info) else {
        pgrx::error!("ec_ivf {context} received a null IndexInfo");
    };
    crate::am::common::pg_ptr::index_info(index_info)
}

unsafe fn heap_relation_tuple_desc(
    heap_relation: pg_sys::Relation,
    context: &str,
) -> PgTupleDesc<'static> {
    // SAFETY: heap_relation is live for the AM callback; `from_pg_copy`
    // copies the tuple descriptor pointer contents for safe attribute access.
    unsafe {
        let Some(heap_relation) = heap_relation.as_ref() else {
            pgrx::error!("ec_ivf {context} received a null heap relation");
        };
        PgTupleDesc::from_pg_copy(heap_relation.rd_att)
    }
}

fn resolve_indexed_vector_kind_from_type(type_oid: pg_sys::Oid) -> Option<IndexedVectorKind> {
    let name = crate::storage::type_info::formatted_base_type_name(type_oid)?;
    let type_name = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
    match type_name {
        "ecvector" => Some(IndexedVectorKind::Ecvector),
        "tqvector" => Some(IndexedVectorKind::Tqvector),
        _ => None,
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_ambuildempty(index_relation: pg_sys::Relation) {
    pg_am_callback!({
        // SAFETY: PostgreSQL invokes ambuildempty with a live IVF index relation.
        let index_relation_handle = NonNull::new(index_relation)
            .unwrap_or_else(|| pgrx::error!("ec_ivf ambuildempty received null index relation"));
        let options = options::relation_options(index_relation_handle);
        options
            .storage_format
            .validate_v1_supported()
            .unwrap_or_else(|e| pgrx::error!("{e}"));
        options
            .rerank
            .validate_v1_supported()
            .unwrap_or_else(|e| pgrx::error!("{e}"));
        let metadata = page::MetadataPage::empty(options);
        page::initialize_metadata_page(index_relation, metadata);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(training_sample_rows: i32, nlists: i32) -> options::EcIvfOptions {
        options::EcIvfOptions {
            nlists,
            nprobe: 0,
            rerank_width: 0,
            training_sample_rows,
            seed: 7,
            pq_group_size: 0,
            posting_slack_percent: 0,
            quant_bits: 4,
            coarse_bits: 0,
            dense_posting_blocks: false,
            dense_posting_typed_layout: false,
            rabitq_residual: false,
            storage_format: options::StorageFormat::Auto,
            rerank: options::RerankMode::Auto,
            coarse_format: options::CoarseFormat::Auto,
            rerank_placement: options::RerankPlacement::Auto,
            rerank_format: options::RerankFormat::Auto,
        }
    }

    fn tid(offset_number: u16) -> ItemPointer {
        ItemPointer {
            block_number: 1,
            offset_number,
        }
    }

    fn tuple(offset_number: u16, source_vector: Vec<f32>) -> BuildTuple {
        let (dimensions, gamma, payload) = crate::quantize_embedding_to_code(
            &source_vector,
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        )
        .unwrap();
        BuildTuple {
            heap_tid: tid(offset_number),
            dimensions,
            gamma,
            payload,
            source_vector,
        }
    }

    fn model(centroids: Vec<Vec<f32>>) -> training::SphericalKMeansModel {
        training::SphericalKMeansModel {
            dimensions: centroids.first().map_or(0, Vec::len),
            centroids,
        }
    }

    #[test]
    fn training_sample_count_respects_auto_explicit_and_empty() {
        assert_eq!(resolve_training_sample_count(0, 0), 0);
        assert_eq!(resolve_training_sample_count(0, 12_000), 10_000);
        assert_eq!(resolve_training_sample_count(128, 10), 10);
        assert_eq!(resolve_training_sample_count(3, 10), 3);
    }

    #[test]
    fn build_state_collects_deterministic_training_sample() {
        let mut state = BuildState::new(options(3, 2), IndexedVectorKind::Ecvector);
        for index in 0..6 {
            state
                .try_push(tuple(index + 1, vec![index as f32 + 1.0, 1.0]))
                .unwrap();
        }

        let sample = state.training_sample_vectors();
        let expected_indices = training::deterministic_sample_indices(6, 3, 7);
        let expected = expected_indices
            .into_iter()
            .map(|index| state.heap_tuples[index].source_vector.as_slice())
            .collect::<Vec<_>>();

        assert_eq!(sample, expected);
    }

    #[test]
    fn build_state_rejects_dimension_mismatch() {
        let mut state = BuildState::new(options(0, 2), IndexedVectorKind::Ecvector);
        state.try_push(tuple(1, vec![1.0, 0.0])).unwrap();
        let err = state.try_push(tuple(2, vec![1.0, 0.0, 0.5])).unwrap_err();

        assert!(err.contains("dimension mismatch"));
    }

    #[test]
    fn build_state_rejects_zero_norm_training_source() {
        let mut state = BuildState::new(options(0, 2), IndexedVectorKind::Ecvector);
        let mut tuple = tuple(1, vec![1.0, 0.0]);
        tuple.source_vector = vec![0.0, 0.0];
        let err = state.try_push(tuple).unwrap_err();

        assert!(err.contains("non-zero"));
    }

    #[test]
    fn build_state_trains_model_from_sample() {
        let mut state = BuildState::new(options(3, 2), IndexedVectorKind::Ecvector);
        state.try_push(tuple(1, vec![1.0, 0.0])).unwrap();
        state.try_push(tuple(2, vec![0.9, 0.1])).unwrap();
        state.try_push(tuple(3, vec![-1.0, 0.0])).unwrap();
        state.try_push(tuple(4, vec![-0.9, -0.1])).unwrap();

        let model = state.train_model().unwrap();

        assert_eq!(model.dimensions, 2);
        assert_eq!(model.centroid_count(), 2);
    }

    #[test]
    fn build_state_stages_bulk_assignments_by_list() {
        let mut state = BuildState::new(options(0, 2), IndexedVectorKind::Ecvector);
        state.try_push(tuple(1, vec![1.0, 0.0])).unwrap();
        state.try_push(tuple(2, vec![0.9, 0.1])).unwrap();
        state.try_push(tuple(3, vec![-1.0, 0.0])).unwrap();

        let plan = state
            .stage_build_plan(&model(vec![vec![1.0, 0.0], vec![-1.0, 0.0]]))
            .unwrap();

        assert_eq!(plan.posting_count(), 3);
        assert_eq!(plan.directory_entries[0].live_count, 2);
        assert_eq!(plan.directory_entries[1].live_count, 1);
        assert_eq!(plan.total_live_tuples(), 3);
        assert_eq!(plan.metadata.dimensions, 2);
        assert_eq!(plan.metadata.nlists, 2);

        let payload_len = state.heap_tuples[0].payload.len();
        for (list_id, posting_tids) in plan.posting_tids_by_list.iter().enumerate() {
            for tid in posting_tids {
                let posting = plan.data_pages.read_ivf_posting(*tid, payload_len).unwrap();
                assert_eq!(posting.list_id, list_id as u32);
                assert_eq!(posting.heaptids.len(), 1);
                assert!(!posting.deleted);
            }
        }
    }

    #[test]
    fn build_state_can_stage_dense_posting_blocks_when_gated() {
        let mut opts = options(0, 2);
        opts.dense_posting_blocks = true;
        opts.storage_format = options::StorageFormat::TurboQuant;
        let mut state = BuildState::new(opts, IndexedVectorKind::Ecvector);
        state.try_push(tuple(1, vec![1.0, 0.0])).unwrap();
        state.try_push(tuple(2, vec![0.9, 0.1])).unwrap();
        state.try_push(tuple(3, vec![-1.0, 0.0])).unwrap();

        let plan = state
            .stage_build_plan(&model(vec![vec![1.0, 0.0], vec![-1.0, 0.0]]))
            .unwrap();

        assert_eq!(plan.posting_count(), 3);
        assert_eq!(plan.directory_entries[0].live_count, 2);
        assert_eq!(plan.directory_entries[1].live_count, 1);
        assert_eq!(plan.posting_tids_by_list[0].len(), 1);
        assert_eq!(plan.posting_tids_by_list[1].len(), 1);

        let payload_len = state.heap_tuples[0].payload.len();
        let dense = plan
            .data_pages
            .read_ivf_dense_posting_block(plan.posting_tids_by_list[0][0], payload_len)
            .unwrap();
        assert_eq!(dense.list_id, 0);
        assert_eq!(dense.gammas.len(), 2);
        assert_eq!(dense.heap_tids.len(), 2);
        assert_eq!(dense.payloads.len(), 2 * payload_len);
    }

    #[test]
    fn build_state_segregates_posting_blocks_by_list() {
        let mut state = BuildState::new(options(0, 3), IndexedVectorKind::Ecvector);
        state.try_push(tuple(1, vec![1.0, 0.0])).unwrap();
        state.try_push(tuple(2, vec![0.9, 0.1])).unwrap();
        state.try_push(tuple(3, vec![0.0, 1.0])).unwrap();
        state.try_push(tuple(4, vec![-1.0, 0.0])).unwrap();

        let plan = state
            .stage_build_plan(&model(vec![
                vec![1.0, 0.0],
                vec![0.0, 1.0],
                vec![-1.0, 0.0],
            ]))
            .unwrap();

        let mut posting_blocks = std::collections::BTreeMap::new();
        for (list_id, posting_tids) in plan.posting_tids_by_list.iter().enumerate() {
            for tid in posting_tids {
                assert_ne!(
                    plan.directory_tids[0].block_number, tid.block_number,
                    "directory tuples should not share posting blocks"
                );
                let previous_list = posting_blocks.insert(tid.block_number, list_id);
                assert!(
                    previous_list.is_none() || previous_list == Some(list_id),
                    "posting block {} is shared across lists",
                    tid.block_number
                );
            }
        }
    }

    #[test]
    fn build_state_can_reserve_list_local_posting_slack() {
        let mut options = options(0, 2);
        options.posting_slack_percent = 100;
        let mut state = BuildState::new(options, IndexedVectorKind::Ecvector);
        state.try_push(tuple(1, vec![1.0, 0.0])).unwrap();
        state.try_push(tuple(2, vec![0.9, 0.1])).unwrap();
        state.try_push(tuple(3, vec![-1.0, 0.0])).unwrap();

        let plan = state
            .stage_build_plan(&model(vec![vec![1.0, 0.0], vec![-1.0, 0.0]]))
            .unwrap();

        for list_id in 0..2 {
            let posting_tail = plan.posting_tids_by_list[list_id]
                .last()
                .expect("test list has postings")
                .block_number;
            let directory_tail = plan.directory_entries[list_id].tail_block.block_number;
            assert!(
                directory_tail > posting_tail,
                "list {list_id} should reserve slack after posting tail"
            );
            assert_eq!(
                plan.directory_entries[list_id].live_count,
                2 - list_id as u64
            );
        }
    }

    #[test]
    fn build_state_stages_empty_lists_with_invalid_directory_refs() {
        let mut state = BuildState::new(options(0, 3), IndexedVectorKind::Ecvector);
        state.try_push(tuple(1, vec![1.0, 0.0])).unwrap();
        state.try_push(tuple(2, vec![-1.0, 0.0])).unwrap();

        let plan = state
            .stage_build_plan(&model(vec![
                vec![1.0, 0.0],
                vec![-1.0, 0.0],
                vec![0.0, 1.0],
            ]))
            .unwrap();
        let empty_directory = plan.directory_entries[2];

        assert_eq!(plan.empty_list_count(), 1);
        assert_eq!(empty_directory.live_count, 0);
        assert_eq!(empty_directory.head_block, page::BlockRef::INVALID);
        assert_eq!(empty_directory.tail_block, page::BlockRef::INVALID);
        assert!(plan.posting_tids_by_list[2].is_empty());
    }

    #[test]
    fn build_state_stages_readable_centroid_and_directory_heads() {
        let mut state = BuildState::new(options(0, 2), IndexedVectorKind::Ecvector);
        state.try_push(tuple(1, vec![1.0, 0.0])).unwrap();
        state.try_push(tuple(2, vec![-1.0, 0.0])).unwrap();

        let plan = state
            .stage_build_plan(&model(vec![vec![1.0, 0.0], vec![-1.0, 0.0]]))
            .unwrap();

        assert_ne!(plan.metadata.centroid_head, ItemPointer::INVALID);
        assert_ne!(plan.metadata.directory_head, ItemPointer::INVALID);
        let centroid = plan
            .data_pages
            .read_ivf_centroid(plan.metadata.centroid_head, 2)
            .unwrap();
        let directory = plan
            .data_pages
            .read_ivf_list_directory(plan.metadata.directory_head)
            .unwrap();

        assert_eq!(centroid.list_id, 0);
        assert_eq!(centroid.centroid, vec![1.0, 0.0]);
        assert_eq!(directory.list_id, 0);
        assert_eq!(directory.live_count, 1);
    }
}
