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

use std::ffi::{c_void, CStr};
use std::ptr;

use pgrx::{itemptr::item_pointer_get_both, pg_sys, PgBox, PgTupleDesc};

use crate::am::common::training;
use crate::quant::prod::ProdQuantizer;
use crate::storage::page::{DataPageChain, ItemPointer, METADATA_BLOCK_NUMBER};
use crate::storage::wal;
use crate::{DEFAULT_QUANT_BITS, DEFAULT_QUANT_SEED};

use super::build::{build_and_persist_vamana, BuildOutput, BuildParams};
use super::insert;
use super::options::{self, TqDiskannOptions};
use super::page::VamanaMetadataPage;
use super::persist::{stage_grouped_codebook_chain, NodePayload};
use super::{
    warn_on_non_unit_source_vector_sample, ECDISKANN_UNIT_NORM_BUILD_SAMPLE_CAP,
    ECDISKANN_UNIT_NORM_DISTANCE_BIAS,
};

const PQ_FASTSCAN_TARGET_GROUP_SIZE: usize = 16;
const PQ_FASTSCAN_DEFAULT_MAX_TRAIN_SIZE: usize = 1024;
const PQ_FASTSCAN_DEFAULT_KMEANS_ITERS: usize = 8;
const P_NEW: pg_sys::BlockNumber = u32::MAX;

#[derive(Debug)]
struct RawHeapTuple {
    primary_heap_tid: ItemPointer,
    overflow_heap_tids: Vec<ItemPointer>,
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

impl BuildState {
    unsafe fn new(index_relation: pg_sys::Relation) -> Self {
        let options = unsafe { options::relation_options(index_relation) };
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
        if let Some(existing) = self
            .heap_tuples
            .iter_mut()
            .find(|existing| source_vectors_match_exactly(&existing.source_vector, &source_vector))
        {
            existing.overflow_heap_tids.push(heap_tid);
            return;
        }
        self.heap_tuples.push(RawHeapTuple {
            primary_heap_tid: heap_tid,
            overflow_heap_tids: Vec::new(),
            source_vector,
        });
    }
}

fn source_vectors_match_exactly(left: &[f32], right: &[f32]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(lhs, rhs)| lhs.to_bits() == rhs.to_bits())
}

pub(super) unsafe extern "C-unwind" fn ec_diskann_ambuild(
    heap_relation: pg_sys::Relation,
    index_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) -> *mut pg_sys::IndexBuildResult {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let mut state = BuildState::new(index_relation);
            validate_single_ecvector_attribute(heap_relation, index_info);

            initialize_metadata_page(index_relation, empty_metadata(&state));

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

            let index_tuples = if state.heap_tuples.is_empty() {
                0.0
            } else {
                flush_build_state(index_relation, &state)
                    .unwrap_or_else(|e| pgrx::error!("ec_diskann ambuild failed: {e}"));
                state.heap_tuples.len() as f64
            };

            if heap_tuples != state.scanned_tuples as f64 {
                pgrx::error!(
                    "ec_diskann ambuild scanned {heap_tuples} heap tuples but observed {}",
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

pub(super) unsafe extern "C-unwind" fn ec_diskann_ambuildempty(index_relation: pg_sys::Relation) {
    unsafe {
        pgrx::pgrx_extern_c_guard(|| {
            let state = BuildState::new(index_relation);
            initialize_metadata_page(index_relation, empty_metadata(&state));
        })
    }
}

unsafe extern "C-unwind" fn ec_diskann_build_callback(
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
) -> Result<(), String> {
    let dimensions = state
        .dimensions
        .expect("non-empty build should record dimensions");
    let seed = DEFAULT_QUANT_SEED;
    let group_size = default_group_size(dimensions);
    let train_size = state
        .heap_tuples
        .len()
        .min(PQ_FASTSCAN_DEFAULT_MAX_TRAIN_SIZE);

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

    let model = training::train_grouped_pq4_model(
        &source_refs,
        dimensions as usize,
        seed,
        group_size,
        train_size,
        PQ_FASTSCAN_DEFAULT_KMEANS_ITERS,
    )?;

    let sidecar_word_count =
        training::persisted_binary_sidecar_word_count(dimensions, DEFAULT_QUANT_BITS, seed);
    let has_binary_sidecar = sidecar_word_count > 0;
    let persisted_binary_quantizer = has_binary_sidecar
        .then(|| ProdQuantizer::cached(dimensions as usize, DEFAULT_QUANT_BITS, seed));

    let payloads: Vec<NodePayload> = state
        .heap_tuples
        .iter()
        .map(|t| {
            let search_code = training::derive_grouped_pq4_code(&t.source_vector, &model);
            let binary_words = match &persisted_binary_quantizer {
                Some(q) => {
                    let encoded = q.encode(&t.source_vector);
                    let mut code = encoded.mse_packed;
                    code.extend_from_slice(&encoded.qjl_packed);
                    training::derive_persisted_binary_words(q, &code)
                }
                None => Vec::new(),
            };
            NodePayload {
                primary_heaptid: t.primary_heap_tid,
                binary_words,
                search_code,
            }
        })
        .collect();

    let params = BuildParams {
        graph_degree_r: u16::try_from(state.options.graph_degree)
            .map_err(|_| "graph_degree does not fit in u16".to_owned())?,
        build_list_size_l: u16::try_from(state.options.build_list_size)
            .map_err(|_| "build_list_size does not fit in u16".to_owned())?,
        alpha: state.options.alpha,
        dimensions,
        search_subvector_count: u16::try_from(model.group_count)
            .map_err(|_| "search_subvector_count does not fit in u16".to_owned())?,
        search_subvector_dim: u16::try_from(model.group_size)
            .map_err(|_| "search_subvector_dim does not fit in u16".to_owned())?,
        seed,
        page_size: state.page_size,
        has_binary_sidecar,
    };

    let build_out = build_and_persist_vamana(params, &payloads, |a, b| {
        source_inner_product_distance(source_refs[a as usize], source_refs[b as usize])
    })?;

    let BuildOutput {
        metadata,
        persisted,
    } = build_out;
    let mut chain = persisted.chain;
    let binary_word_count = params.binary_word_count();
    let search_code_len = params.search_code_len();
    for (node_index, tuple) in state.heap_tuples.iter().enumerate() {
        insert::stage_overflow_heap_tids_in_chain(
            &mut chain,
            metadata.graph_degree_r,
            binary_word_count,
            search_code_len,
            persisted.node_to_tid[node_index],
            &tuple.overflow_heap_tids,
        )?;
    }
    let codebook_head = stage_grouped_codebook_chain(&mut chain, &model)?;
    let mut metadata = metadata;
    metadata.grouped_codebook_head = codebook_head;

    unsafe { write_data_pages(index_relation, &chain) };
    unsafe { overwrite_metadata_page(index_relation, &metadata) };
    Ok(())
}

fn source_inner_product_distance(left: &[f32], right: &[f32]) -> f32 {
    debug_assert_eq!(left.len(), right.len());
    let mut ip = 0.0_f32;
    for (l, r) in left.iter().zip(right.iter()) {
        ip += *l * *r;
    }
    let d = ECDISKANN_UNIT_NORM_DISTANCE_BIAS - ip;
    if d < 0.0 {
        0.0
    } else {
        d
    }
}

unsafe fn initialize_metadata_page(index_relation: pg_sys::Relation, metadata: VamanaMetadataPage) {
    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks == 0 {
        P_NEW
    } else {
        METADATA_BLOCK_NUMBER
    };
    let read_mode = if target_block == P_NEW {
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK
    } else {
        pg_sys::ReadBufferMode::RBM_NORMAL
    };
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            target_block,
            read_mode,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("ec_diskann failed to allocate metadata buffer");
    }
    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }
    write_metadata_to_buffer(index_relation, buffer, &metadata);
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

unsafe fn overwrite_metadata_page(index_relation: pg_sys::Relation, metadata: &VamanaMetadataPage) {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        pgrx::error!("ec_diskann failed to open metadata buffer");
    }
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    write_metadata_to_buffer(index_relation, buffer, metadata);
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
}

fn write_metadata_to_buffer(
    index_relation: pg_sys::Relation,
    buffer: pg_sys::Buffer,
    metadata: &VamanaMetadataPage,
) {
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    let metadata_bytes = metadata.encode();
    let special_size = (metadata_bytes.len() + 7) & !7;
    unsafe { pg_sys::PageInit(page_ptr, page_size, special_size) };
    let dst = unsafe { pg_sys::PageGetSpecialPointer(page_ptr) }.cast::<u8>();
    unsafe {
        ptr::copy_nonoverlapping(metadata_bytes.as_ptr(), dst, metadata_bytes.len());
    }
    unsafe { wal_txn.finish() };
}

pub(super) unsafe fn write_data_pages(index_relation: pg_sys::Relation, chain: &DataPageChain) {
    for staged_page in chain.pages() {
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
                "ec_diskann failed to allocate data buffer for block {}",
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
                    "ec_diskann failed to write tuple to block {}",
                    staged_page.block_number()
                );
            }
        }

        unsafe { wal_txn.finish() };
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }
}

pub(super) unsafe fn decode_heap_tid(tid: pg_sys::ItemPointer) -> ItemPointer {
    if tid.is_null() {
        pgrx::error!("ec_diskann ambuild received a null heap tid");
    }
    let (block_number, offset_number) = item_pointer_get_both(unsafe { *tid });
    ItemPointer {
        block_number,
        offset_number,
    }
}

unsafe fn validate_single_ecvector_attribute(
    heap_relation: pg_sys::Relation,
    index_info: *mut pg_sys::IndexInfo,
) {
    if index_info.is_null() {
        pgrx::error!("ec_diskann ambuild received a null IndexInfo");
    }
    let info = unsafe { &*index_info };
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

    let tuple_desc = unsafe { PgTupleDesc::from_pg_copy((*heap_relation).rd_att) };
    let att = tuple_desc
        .get(attnum as usize - 1)
        .expect("indexed attribute should exist");
    if att.attisdropped {
        pgrx::error!("ec_diskann indexed column references a dropped column");
    }
    let base_type_oid = unsafe { pg_sys::getBaseType(att.atttypid) };
    let formatted = unsafe { pg_sys::format_type_be(base_type_oid) };
    if formatted.is_null() {
        pgrx::error!("ec_diskann indexed column has no resolvable type name");
    }
    let name = unsafe { CStr::from_ptr(formatted) }
        .to_string_lossy()
        .into_owned();
    unsafe { pg_sys::pfree(formatted.cast()) };
    let type_name = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
    if type_name != "ecvector" {
        pgrx::error!("ec_diskann indexed column must be ecvector, got {type_name}");
    }
}

pub(super) unsafe fn ecvector_datum_to_vec(datum: pg_sys::Datum) -> Vec<f32> {
    let original = datum
        .cast_mut_ptr::<std::ffi::c_void>()
        .cast::<pg_sys::varlena>();
    let varlena = unsafe { pg_sys::pg_detoast_datum(original.cast()) };
    if varlena.is_null() {
        pgrx::error!("ec_diskann could not detoast indexed ecvector");
    }
    let owned = !ptr::eq(varlena, original);
    let bytes = unsafe { pgrx::varlena::varlena_to_byte_slice(varlena) };
    if bytes.len() % std::mem::size_of::<f32>() != 0 {
        if owned {
            unsafe { pg_sys::pfree(varlena.cast()) };
        }
        pgrx::error!("ec_diskann indexed ecvector payload length must be a multiple of 4 bytes");
    }
    let (prefix, body, suffix) = unsafe { bytes.align_to::<f32>() };
    if !prefix.is_empty() || !suffix.is_empty() {
        if owned {
            unsafe { pg_sys::pfree(varlena.cast()) };
        }
        pgrx::error!("ec_diskann indexed ecvector payload is not aligned for float4 access");
    }
    let vec = body.to_vec();
    if owned {
        unsafe { pg_sys::pfree(varlena.cast()) };
    }
    vec
}

#[cfg(test)]
mod tests {
    use super::source_inner_product_distance;

    #[test]
    fn source_inner_product_distance_keeps_positive_ip_pairs_distinct() {
        let identical = source_inner_product_distance(&[1.0, 0.0], &[1.0, 0.0]);
        let merely_similar = source_inner_product_distance(&[1.0, 0.0], &[0.8, 0.6]);
        let orthogonal = source_inner_product_distance(&[1.0, 0.0], &[0.0, 1.0]);

        assert_eq!(identical, 0.0);
        assert!(merely_similar > identical);
        assert!(orthogonal > merely_similar);
    }
}
