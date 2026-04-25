use std::ffi::{c_void, CStr};
use std::ptr;

use pgrx::{itemptr::item_pointer_get_both, pg_sys, PgBox, PgTupleDesc};

use super::{options, page, training, P_NEW};
use crate::quant::prod::ProdQuantizer;
use crate::storage::{
    page::{DataPageChain, ItemPointer},
    wal,
};

const DEFAULT_AUTO_TRAINING_SAMPLE_ROWS: usize = 10_000;
const DEFAULT_KMEANS_ITERATIONS: usize = 8;

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

struct BuildState {
    options: options::EcIvfOptions,
    indexed_vector_kind: IndexedVectorKind,
    page_size: usize,
    scanned_tuples: usize,
    heap_tuples: Vec<BuildTuple>,
    dimensions: Option<u16>,
}

struct IvfBuildPlan {
    data_pages: DataPageChain,
    metadata: page::MetadataPage,
    centroid_tids: Vec<ItemPointer>,
    directory_tids: Vec<ItemPointer>,
    posting_tids_by_list: Vec<Vec<ItemPointer>>,
    directory_entries: Vec<page::IvfListDirectoryTuple>,
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
        self.posting_tids_by_list
            .iter()
            .map(Vec::len)
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

unsafe extern "C-unwind" fn ec_ivf_build_callback(
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
            let heap_tid = decode_heap_tid(tid, "ambuild");
            let tuple = build_index_tuple(
                values,
                isnull,
                heap_tid,
                state.indexed_vector_kind,
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
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let options = options::relation_options(index_relation);
            options
                .rerank
                .validate_v1_supported()
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            page::initialize_metadata_page(index_relation, page::MetadataPage::empty(options));

            let indexed_vector_kind =
                resolve_indexed_vector_kind(heap_relation, index_info, "ambuild");
            let mut state = BuildState::new(options, indexed_vector_kind);
            let heap_tuples = pg_sys::table_index_build_scan(
                heap_relation,
                index_relation,
                index_info,
                false,
                false,
                Some(ec_ivf_build_callback),
                (&mut state as *mut BuildState).cast(),
                ptr::null_mut(),
            );
            let index_tuples = if state.scanned_tuples == 0 {
                0.0
            } else {
                let model = state
                    .train_model()
                    .unwrap_or_else(|e| pgrx::error!("ec_ivf centroid training failed: {e}"));
                let plan = state
                    .stage_build_plan(&model)
                    .unwrap_or_else(|e| pgrx::error!("ec_ivf populated index staging failed: {e}"));
                flush_build_plan(index_relation, &plan);
                plan.posting_count() as f64
            };

            let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
            result.heap_tuples = heap_tuples;
            result.index_tuples = index_tuples;
            result.into_pg()
        })
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

    fn push(&mut self, tuple: BuildTuple) {
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

        let nlists = model.centroid_count();
        let mut data_pages = DataPageChain::new(self.page_size);
        let mut centroid_tids = Vec::with_capacity(nlists);
        for (list_id, centroid) in model.centroids.iter().enumerate() {
            let centroid = page::IvfCentroidTuple {
                list_id: list_id_u32(list_id)?,
                centroid: centroid.clone(),
            };
            centroid_tids.push(data_pages.insert_ivf_centroid(&centroid)?);
        }

        let mut tuple_indices_by_list = vec![Vec::new(); nlists];
        for (tuple_index, tuple) in self.heap_tuples.iter().enumerate() {
            let list_id = training::assign_vector_to_centroid(&tuple.source_vector, model)?;
            tuple_indices_by_list[list_id].push(tuple_index);
        }

        let mut posting_tids_by_list = vec![Vec::new(); nlists];
        for (list_id, tuple_indices) in tuple_indices_by_list.iter().enumerate() {
            for tuple_index in tuple_indices {
                let tuple = &self.heap_tuples[*tuple_index];
                let posting = page::IvfPostingTuple {
                    list_id: list_id_u32(list_id)?,
                    deleted: false,
                    heaptids: vec![tuple.heap_tid],
                    gamma: tuple.gamma,
                    rerank_tid: ItemPointer::INVALID,
                    payload: tuple.payload.clone(),
                };
                posting_tids_by_list[list_id].push(data_pages.insert_ivf_posting(&posting)?);
            }
        }

        let mut directory_entries = Vec::with_capacity(nlists);
        let mut directory_tids = Vec::with_capacity(nlists);
        for (list_id, posting_tids) in posting_tids_by_list.iter().enumerate() {
            let mut directory = page::IvfListDirectoryTuple::empty(list_id_u32(list_id)?);
            if let (Some(head), Some(tail)) = (posting_tids.first(), posting_tids.last()) {
                directory.head_block = page::BlockRef {
                    block_number: head.block_number,
                };
                directory.tail_block = page::BlockRef {
                    block_number: tail.block_number,
                };
                directory.live_count = u64::try_from(posting_tids.len())
                    .map_err(|_| "posting count exceeds u64".to_owned())?;
            }
            directory_tids.push(data_pages.insert_ivf_list_directory(directory)?);
            directory_entries.push(directory);
        }

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
        metadata.total_live_tuples = u64::try_from(self.heap_tuples.len())
            .map_err(|_| "heap tuple count exceeds u64".to_owned())?;

        Ok(IvfBuildPlan {
            data_pages,
            metadata,
            centroid_tids,
            directory_tids,
            posting_tids_by_list,
            directory_entries,
        })
    }
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

unsafe fn flush_build_plan(index_relation: pg_sys::Relation, plan: &IvfBuildPlan) {
    let metadata_nlists = usize::try_from(plan.metadata.nlists).expect("u32 nlists should fit");
    debug_assert_eq!(plan.centroid_count(), metadata_nlists);
    debug_assert_eq!(plan.directory_count(), metadata_nlists);
    debug_assert!(plan.empty_list_count() <= metadata_nlists);
    debug_assert!(plan.data_page_count() > 0);
    debug_assert_eq!(plan.total_live_tuples(), plan.posting_count() as u64);

    unsafe { write_data_pages(index_relation, &plan.data_pages) };
    unsafe { page::initialize_metadata_page(index_relation, plan.metadata) };
}

unsafe fn write_data_pages(index_relation: pg_sys::Relation, data_pages: &DataPageChain) {
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
                "ec_ivf failed to allocate data buffer for block {}",
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
                    "ec_ivf failed to write tuple to block {}",
                    staged_page.block_number()
                );
            }
        }

        unsafe { wal_txn.finish() };
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }
}

fn list_id_u32(list_id: usize) -> Result<u32, String> {
    u32::try_from(list_id).map_err(|_| format!("ec_ivf list id {list_id} exceeds u32"))
}

pub(super) unsafe fn build_index_tuple(
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: ItemPointer,
    indexed_vector_kind: IndexedVectorKind,
    context: &str,
) -> BuildTuple {
    if values.is_null() || isnull.is_null() {
        pgrx::error!("ec_ivf {context} received null tuple value arrays");
    }
    if unsafe { *isnull } {
        pgrx::error!("ec_ivf does not support NULL indexed values");
    }

    let datum = unsafe { *values };
    if datum.is_null() {
        pgrx::error!("ec_ivf {context} received a null indexed datum");
    }

    let bytes = unsafe { detoasted_varlena_bytes(datum, "indexed vector column") };
    match indexed_vector_kind {
        IndexedVectorKind::Ecvector => build_ecvector_tuple(heap_tid, &bytes, context),
        IndexedVectorKind::Tqvector => build_tqvector_tuple(heap_tid, &bytes, context),
    }
}

fn build_ecvector_tuple(heap_tid: ItemPointer, bytes: &[u8], context: &str) -> BuildTuple {
    let source_vector = crate::unpack_raw_f32(bytes, "ec_ivf indexed ecvector column")
        .unwrap_or_else(|e| pgrx::error!("ec_ivf {context} found invalid indexed ecvector: {e}"));
    let (dimensions, gamma, payload) = crate::quantize_embedding_to_code(
        &source_vector,
        crate::DEFAULT_QUANT_BITS,
        crate::DEFAULT_QUANT_SEED,
    )
    .unwrap_or_else(|e| pgrx::error!("ec_ivf {context} found invalid indexed ecvector: {e}"));

    BuildTuple {
        heap_tid,
        dimensions,
        gamma,
        payload,
        source_vector,
    }
}

fn build_tqvector_tuple(heap_tid: ItemPointer, bytes: &[u8], context: &str) -> BuildTuple {
    let (dimensions, bits, seed, gamma, code) = crate::unpack(bytes)
        .unwrap_or_else(|e| pgrx::error!("ec_ivf {context} found invalid indexed tqvector: {e}"));
    let payload = code.to_vec();

    let quantizer = ProdQuantizer::cached(usize::from(dimensions), bits, seed);
    let mut full_payload = Vec::with_capacity(4 + payload.len());
    full_payload.extend_from_slice(&gamma.to_le_bytes());
    full_payload.extend_from_slice(&payload);
    let source_vector = quantizer.decode_approximate(&full_payload);

    BuildTuple {
        heap_tid,
        dimensions,
        gamma,
        payload,
        source_vector,
    }
}

unsafe fn detoasted_varlena_bytes(datum: pg_sys::Datum, label: &str) -> Vec<u8> {
    let original = datum
        .cast_mut_ptr::<std::ffi::c_void>()
        .cast::<pg_sys::varlena>();
    let varlena = unsafe { pg_sys::pg_detoast_datum_packed(original.cast()) };
    if varlena.is_null() {
        pgrx::error!("ec_ivf could not detoast {label}");
    }
    let owned = !ptr::eq(varlena, original);
    let bytes = unsafe { pgrx::varlena::varlena_to_byte_slice(varlena) }.to_vec();
    if owned {
        unsafe { pg_sys::pfree(varlena.cast()) };
    }
    bytes
}

pub(super) unsafe fn decode_heap_tid(tid: pg_sys::ItemPointer, context: &str) -> ItemPointer {
    if tid.is_null() {
        pgrx::error!("ec_ivf {context} received a null heap tid");
    }
    let (block_number, offset_number) = item_pointer_get_both(unsafe { *tid });
    ItemPointer {
        block_number,
        offset_number,
    }
}

pub(super) unsafe fn resolve_indexed_vector_kind(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
    context: &str,
) -> IndexedVectorKind {
    if index_info.is_null() {
        pgrx::error!("ec_ivf {context} received a null IndexInfo");
    }
    let index_info = unsafe { &*index_info };
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

    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let att = tuple_desc
        .get(attnum as usize - 1)
        .expect("resolved indexed attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_ivf indexed column references a dropped column");
    }
    unsafe { resolve_indexed_vector_kind_from_type(att.atttypid) }
        .unwrap_or_else(|| pgrx::error!("ec_ivf indexed column must be ecvector or tqvector"))
}

unsafe fn resolve_indexed_vector_kind_from_type(
    type_oid: pg_sys::Oid,
) -> Option<IndexedVectorKind> {
    let base_type_oid = unsafe { pg_sys::getBaseType(type_oid) };
    let formatted = unsafe { pg_sys::format_type_be(base_type_oid) };
    if formatted.is_null() {
        return None;
    }
    let name = unsafe { CStr::from_ptr(formatted) }
        .to_string_lossy()
        .into_owned();
    unsafe { pg_sys::pfree(formatted.cast()) };
    let type_name = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
    match type_name {
        "ecvector" => Some(IndexedVectorKind::Ecvector),
        "tqvector" => Some(IndexedVectorKind::Tqvector),
        _ => None,
    }
}

pub(super) unsafe extern "C-unwind" fn ec_ivf_ambuildempty(index_relation: pg_sys::Relation) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let options = options::relation_options(index_relation);
            options
                .rerank
                .validate_v1_supported()
                .unwrap_or_else(|e| pgrx::error!("{e}"));
            let metadata = page::MetadataPage::empty(options);
            page::initialize_metadata_page(index_relation, metadata);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(training_sample_rows: i32, nlists: i32) -> options::EcIvfOptions {
        options::EcIvfOptions {
            nlists,
            nprobe: 0,
            training_sample_rows,
            seed: 7,
            storage_format: options::StorageFormat::Auto,
            rerank: options::RerankMode::Auto,
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
