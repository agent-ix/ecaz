use std::ffi::{c_void, CStr};
use std::ptr;

use pgrx::{itemptr::item_pointer_get_both, pg_sys, PgBox, PgTupleDesc};

use super::{options, page, training};
use crate::quant::prod::ProdQuantizer;
use crate::storage::page::ItemPointer;

const DEFAULT_AUTO_TRAINING_SAMPLE_ROWS: usize = 10_000;
const DEFAULT_KMEANS_ITERATIONS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IndexedVectorKind {
    Ecvector,
    Tqvector,
}

#[derive(Debug, Clone, PartialEq)]
struct BuildTuple {
    heap_tid: ItemPointer,
    dimensions: u16,
    gamma: f32,
    payload: Vec<u8>,
    source_vector: Vec<f32>,
}

struct BuildState {
    options: options::EcIvfOptions,
    indexed_vector_kind: IndexedVectorKind,
    page_size: usize,
    scanned_tuples: usize,
    heap_tuples: Vec<BuildTuple>,
    dimensions: Option<u16>,
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
            let heap_tid = decode_heap_tid(tid);
            let tuple = build_heap_tuple(values, isnull, heap_tid, state.indexed_vector_kind);
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
            page::initialize_metadata_page(index_relation, page::MetadataPage::empty(options));

            let indexed_vector_kind = resolve_indexed_vector_kind(heap_relation, index_info);
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
            if state.scanned_tuples != 0 {
                let model = state
                    .train_model()
                    .unwrap_or_else(|e| pgrx::error!("ec_ivf centroid training failed: {e}"));
                let sample_count = state.training_sample_count();
                pgrx::error!(
                    "ec_ivf populated index writes are not implemented yet; collected {} heap tuples, {} training samples, and {} centroids",
                    state.scanned_tuples,
                    sample_count,
                    model.centroid_count()
                );
            }

            let mut result = PgBox::<pg_sys::IndexBuildResult>::alloc0();
            result.heap_tuples = heap_tuples;
            result.index_tuples = 0.0;
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

unsafe fn build_heap_tuple(
    values: *mut pg_sys::Datum,
    isnull: *mut bool,
    heap_tid: ItemPointer,
    indexed_vector_kind: IndexedVectorKind,
) -> BuildTuple {
    if values.is_null() || isnull.is_null() {
        pgrx::error!("ec_ivf ambuild received null tuple value arrays");
    }
    if unsafe { *isnull } {
        pgrx::error!("ec_ivf does not support NULL indexed values");
    }

    let datum = unsafe { *values };
    if datum.is_null() {
        pgrx::error!("ec_ivf ambuild received a null indexed datum");
    }

    let bytes = unsafe { detoasted_varlena_bytes(datum, "indexed vector column") };
    match indexed_vector_kind {
        IndexedVectorKind::Ecvector => build_ecvector_tuple(heap_tid, &bytes),
        IndexedVectorKind::Tqvector => build_tqvector_tuple(heap_tid, &bytes),
    }
}

fn build_ecvector_tuple(heap_tid: ItemPointer, bytes: &[u8]) -> BuildTuple {
    let source_vector = crate::unpack_raw_f32(bytes, "ec_ivf indexed ecvector column")
        .unwrap_or_else(|e| pgrx::error!("ec_ivf ambuild found invalid indexed ecvector: {e}"));
    let (dimensions, gamma, payload) = crate::quantize_embedding_to_code(
        &source_vector,
        crate::DEFAULT_QUANT_BITS,
        crate::DEFAULT_QUANT_SEED,
    )
    .unwrap_or_else(|e| pgrx::error!("ec_ivf ambuild found invalid indexed ecvector: {e}"));

    BuildTuple {
        heap_tid,
        dimensions,
        gamma,
        payload,
        source_vector,
    }
}

fn build_tqvector_tuple(heap_tid: ItemPointer, bytes: &[u8]) -> BuildTuple {
    let (dimensions, bits, seed, gamma, code) = crate::unpack(bytes)
        .unwrap_or_else(|e| pgrx::error!("ec_ivf ambuild found invalid indexed tqvector: {e}"));
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

unsafe fn decode_heap_tid(tid: pg_sys::ItemPointer) -> ItemPointer {
    if tid.is_null() {
        pgrx::error!("ec_ivf ambuild received a null heap tid");
    }
    let (block_number, offset_number) = item_pointer_get_both(unsafe { *tid });
    ItemPointer {
        block_number,
        offset_number,
    }
}

unsafe fn resolve_indexed_vector_kind(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> IndexedVectorKind {
    if index_info.is_null() {
        pgrx::error!("ec_ivf ambuild received a null IndexInfo");
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
}
