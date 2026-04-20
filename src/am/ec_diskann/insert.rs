//! Insert-time payload derivation support for `ec_diskann`.
//!
//! Phase 7 needs two insert-side seams before the full graph-mutation
//! path lands:
//!
//! 1. given the built index's metadata + persisted grouped codebooks,
//!    derive the new node's persisted payload (`search_code`,
//!    optional binary sidecar words) from an incoming source vector
//! 2. bootstrap the first live row into an otherwise-empty index
//!
//! This module owns both seams. The general non-empty pgrx callback
//! still lives in `routine.rs`; later slices will move more of that
//! logic here once the page-write / backlink / overflow story is
//! implemented.

use std::{ptr, slice};

use pgrx::pg_sys;

use crate::am::common::training;
use crate::quant::grouped_pq::{encode_grouped_pq, GROUPED_PQ_CENTROIDS};
use crate::quant::prod::ProdQuantizer;
use crate::storage::page::{DataPageChain, ItemPointer};
use crate::storage::wal;
use crate::{DEFAULT_QUANT_BITS, DEFAULT_QUANT_SEED};

use super::{
    ambuild,
    build::{build_and_persist_vamana, BuildOutput, BuildParams},
    options,
    page::{
        VAMANA_METADATA_BYTES, VamanaMetadataPage, PAYLOAD_FLAG_BINARY_SIDECAR,
        VAMANA_SEARCH_CODEC_GROUPED_PQ, VAMANA_TRANSFORM_KIND_SRHT,
    },
    persist::{stage_grouped_codebook_chain, NodePayload},
};
use super::scan_query::{encode_query_srht, read_grouped_codebook_chain};

const EMPTY_INSERT_BOOTSTRAP_KMEANS_ITERS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DerivedInsertPayload {
    pub(super) binary_words: Vec<u64>,
    pub(super) search_code: Vec<u8>,
}

pub(super) fn derive_insert_payload_from_persisted(
    metadata: &VamanaMetadataPage,
    chain: &DataPageChain,
    source_vector: &[f32],
) -> Result<DerivedInsertPayload, String> {
    let dimensions = usize::from(metadata.dimensions);
    if dimensions == 0 {
        return Err("ec_diskann insert payload derivation requires non-zero dimensions".into());
    }
    if source_vector.len() != dimensions {
        return Err(format!(
            "ec_diskann insert payload dimension mismatch: source dim {}, index dim {}",
            source_vector.len(),
            dimensions
        ));
    }
    if metadata.transform_kind != VAMANA_TRANSFORM_KIND_SRHT {
        return Err(format!(
            "ec_diskann insert payload derivation only supports SRHT transform kind {}, got {}",
            VAMANA_TRANSFORM_KIND_SRHT, metadata.transform_kind
        ));
    }
    if metadata.search_codec_kind != VAMANA_SEARCH_CODEC_GROUPED_PQ {
        return Err(format!(
            "ec_diskann insert payload derivation only supports grouped-PQ codec kind {}, got {}",
            VAMANA_SEARCH_CODEC_GROUPED_PQ, metadata.search_codec_kind
        ));
    }

    let group_count = usize::from(metadata.search_subvector_count);
    let group_size = usize::from(metadata.search_subvector_dim);
    if group_count == 0 || group_size == 0 {
        return Err(
            "ec_diskann insert payload derivation requires non-zero grouped search shape".into(),
        );
    }
    if metadata.grouped_codebook_head == ItemPointer::INVALID {
        return Err("ec_diskann insert payload derivation requires persisted grouped codebooks".into());
    }

    let centroid_count = group_size * GROUPED_PQ_CENTROIDS;
    let flat_codebooks = read_grouped_codebook_chain(
        chain,
        metadata.grouped_codebook_head,
        group_count,
        centroid_count,
    )?;

    let rotated = encode_query_srht(source_vector, dimensions, metadata.seed);
    let expected_rotated_len = group_count
        .checked_mul(group_size)
        .ok_or_else(|| "ec_diskann grouped search shape overflows usize".to_owned())?;
    if rotated.len() != expected_rotated_len {
        return Err(format!(
            "ec_diskann insert payload rotated query length mismatch: got {}, expected {} from metadata",
            rotated.len(),
            expected_rotated_len
        ));
    }

    let codebook_chunk_len = GROUPED_PQ_CENTROIDS * group_size;
    let search_code = encode_grouped_pq(
        &rotated,
        flat_codebooks.chunks_exact(codebook_chunk_len),
        group_size,
    );
    let expected_search_code_len = group_count.div_ceil(2);
    if search_code.len() != expected_search_code_len {
        return Err(format!(
            "ec_diskann insert payload search code length mismatch: got {}, expected {}",
            search_code.len(),
            expected_search_code_len
        ));
    }

    let binary_words = if (metadata.payload_flags & PAYLOAD_FLAG_BINARY_SIDECAR) != 0 {
        let quantizer = ProdQuantizer::cached(dimensions, DEFAULT_QUANT_BITS, metadata.seed);
        let encoded = quantizer.encode(source_vector);
        let mut code = encoded.mse_packed;
        code.extend_from_slice(&encoded.qjl_packed);
        training::derive_persisted_binary_words(&quantizer, &code)
    } else {
        Vec::new()
    };

    Ok(DerivedInsertPayload {
        binary_words,
        search_code,
    })
}

#[derive(Debug, Clone)]
pub(super) struct EmptyInsertBootstrapOutput {
    pub(super) metadata: VamanaMetadataPage,
    pub(super) chain: DataPageChain,
}

pub(super) unsafe fn read_metadata_page(
    index_relation: pg_sys::Relation,
) -> Result<VamanaMetadataPage, String> {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            crate::storage::page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        return Err("ec_diskann failed to open metadata buffer".into());
    }
    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_SHARE as i32) };
    let page = unsafe { pg_sys::BufferGetPage(buffer) };
    let special = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    let metadata_bytes = unsafe { slice::from_raw_parts(special, VAMANA_METADATA_BYTES) };
    let metadata = VamanaMetadataPage::decode(metadata_bytes);
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    metadata
}

pub(super) unsafe fn with_locked_metadata_page<T>(
    index_relation: pg_sys::Relation,
    f: impl FnOnce(&mut VamanaMetadataPage) -> Result<T, String>,
) -> Result<T, String> {
    let buffer = unsafe {
        pg_sys::ReadBufferExtended(
            index_relation,
            pg_sys::ForkNumber::MAIN_FORKNUM,
            crate::storage::page::METADATA_BLOCK_NUMBER,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            ptr::null_mut(),
        )
    };
    if !unsafe { pg_sys::BufferIsValid(buffer) } {
        return Err("ec_diskann failed to open metadata buffer".into());
    }

    unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    let page = unsafe { pg_sys::BufferGetPage(buffer) };
    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let special = unsafe { pg_sys::PageGetSpecialPointer(page) }.cast::<u8>();
    let metadata_bytes = unsafe { slice::from_raw_parts(special, VAMANA_METADATA_BYTES) };
    let mut metadata = VamanaMetadataPage::decode(metadata_bytes)?;
    let result = f(&mut metadata)?;

    let encoded = metadata.encode();
    let special_size = (encoded.len() + 7) & !7;
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let writable_page =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    unsafe { pg_sys::PageInit(writable_page, page_size, special_size) };
    let dst = unsafe { pg_sys::PageGetSpecialPointer(writable_page) }.cast::<u8>();
    unsafe { ptr::copy_nonoverlapping(encoded.as_ptr(), dst, encoded.len()) };
    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    Ok(result)
}

pub(super) unsafe fn bootstrap_empty_insert_output(
    index_relation: pg_sys::Relation,
    heap_tid: ItemPointer,
    source_vector: &[f32],
) -> Result<EmptyInsertBootstrapOutput, String> {
    if source_vector.is_empty() {
        return Err("ec_diskann empty-index bootstrap requires a non-empty source vector".into());
    }

    let dimensions = u16::try_from(source_vector.len())
        .map_err(|_| format!("ec_diskann insert source dimension {} exceeds u16", source_vector.len()))?;
    let seed = DEFAULT_QUANT_SEED;
    let group_size = ambuild::default_group_size(dimensions);
    let source_refs = vec![source_vector];
    let model = training::train_grouped_pq4_model(
        &source_refs,
        source_vector.len(),
        seed,
        group_size,
        1,
        EMPTY_INSERT_BOOTSTRAP_KMEANS_ITERS,
    )?;

    let sidecar_word_count =
        training::persisted_binary_sidecar_word_count(dimensions, DEFAULT_QUANT_BITS, seed);
    let has_binary_sidecar = sidecar_word_count > 0;
    let binary_words = if has_binary_sidecar {
        let quantizer = ProdQuantizer::cached(source_vector.len(), DEFAULT_QUANT_BITS, seed);
        let encoded = quantizer.encode(source_vector);
        let mut code = encoded.mse_packed;
        code.extend_from_slice(&encoded.qjl_packed);
        training::derive_persisted_binary_words(&quantizer, &code)
    } else {
        Vec::new()
    };

    let payloads = vec![NodePayload {
        primary_heaptid: heap_tid,
        binary_words,
        search_code: training::derive_grouped_pq4_code(source_vector, &model),
    }];

    let relopts = unsafe { options::relation_options(index_relation) };
    let params = BuildParams {
        graph_degree_r: u16::try_from(relopts.graph_degree)
            .map_err(|_| "graph_degree does not fit in u16".to_owned())?,
        build_list_size_l: u16::try_from(relopts.build_list_size)
            .map_err(|_| "build_list_size does not fit in u16".to_owned())?,
        alpha: relopts.alpha,
        dimensions,
        search_subvector_count: u16::try_from(model.group_count)
            .map_err(|_| "search_subvector_count does not fit in u16".to_owned())?,
        search_subvector_dim: u16::try_from(model.group_size)
            .map_err(|_| "search_subvector_dim does not fit in u16".to_owned())?,
        seed,
        page_size: pg_sys::BLCKSZ as usize,
        has_binary_sidecar,
    };

    let BuildOutput { mut metadata, persisted } =
        build_and_persist_vamana(params, &payloads, |_, _| 0.0)?;
    let mut chain = persisted.chain;
    let codebook_head = stage_grouped_codebook_chain(&mut chain, &model)?;
    metadata.grouped_codebook_head = codebook_head;
    metadata.inserted_since_rebuild = 1;

    Ok(EmptyInsertBootstrapOutput { metadata, chain })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::common::training::{self, train_grouped_pq4_model};
    use crate::am::ec_diskann::page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE;
    use crate::am::ec_diskann::persist::stage_grouped_codebook_chain;
    use crate::storage::page::DEFAULT_PAGE_SIZE;

    fn training_vectors() -> Vec<Vec<f32>> {
        vec![
            vec![1.0, 0.0, 0.5, -1.0, 0.25, -0.5, 0.75, -0.25],
            vec![0.9, 0.1, 0.45, -0.95, 0.2, -0.45, 0.7, -0.2],
            vec![0.0, 1.0, 0.25, -0.5, -0.1, 0.3, 0.2, -0.7],
            vec![-1.0, 0.5, 0.0, 1.0, -0.2, 0.4, -0.6, 0.8],
            vec![0.3, -0.7, 0.8, -0.1, 0.9, -0.4, 0.6, -0.2],
            vec![-0.4, 0.6, -0.9, 0.2, -0.8, 0.5, -0.3, 0.1],
        ]
    }

    fn staged_metadata(
        with_binary_sidecar: bool,
    ) -> (VamanaMetadataPage, DataPageChain, Vec<Vec<f32>>, training::GroupedPq4Model) {
        let vectors = training_vectors();
        let refs: Vec<&[f32]> = vectors.iter().map(Vec::as_slice).collect();
        let dimensions = vectors[0].len();
        let seed = 42_u64;
        let group_size = 4_usize;
        let model =
            train_grouped_pq4_model(&refs, dimensions, seed, group_size, refs.len(), 6).expect("train");

        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let codebook_head = stage_grouped_codebook_chain(&mut chain, &model).expect("stage codebooks");

        let mut metadata = VamanaMetadataPage::empty(32, 100, 1.2, dimensions as u16, seed);
        metadata.search_subvector_count = model.group_count as u16;
        metadata.search_subvector_dim = model.group_size as u16;
        metadata.grouped_codebook_head = codebook_head;
        metadata.payload_flags = PAYLOAD_FLAG_GROUPED_SEARCH_CODE;
        if with_binary_sidecar {
            metadata.payload_flags |= PAYLOAD_FLAG_BINARY_SIDECAR;
        }

        (metadata, chain, vectors, model)
    }

    // IN-001: derive_insert_payload_from_persisted matches the build-side
    // grouped-PQ search code and persisted binary sidecar derivation.
    #[test]
    fn in_001_payload_matches_training_model_with_binary_sidecar() {
        let (metadata, chain, vectors, model) = staged_metadata(true);
        let source = &vectors[0];

        let observed =
            derive_insert_payload_from_persisted(&metadata, &chain, source).expect("derive");

        let expected_search_code = training::derive_grouped_pq4_code(source, &model);
        let quantizer =
            ProdQuantizer::cached(source.len(), DEFAULT_QUANT_BITS, metadata.seed);
        let encoded = quantizer.encode(source);
        let mut code = encoded.mse_packed;
        code.extend_from_slice(&encoded.qjl_packed);
        let expected_binary_words = training::derive_persisted_binary_words(&quantizer, &code);

        assert_eq!(observed.search_code, expected_search_code);
        assert_eq!(observed.binary_words, expected_binary_words);
    }

    // IN-002: the helper honors metadata with no binary sidecar bit.
    #[test]
    fn in_002_payload_omits_binary_words_without_sidecar_flag() {
        let (metadata, chain, vectors, model) = staged_metadata(false);
        let source = &vectors[1];

        let observed =
            derive_insert_payload_from_persisted(&metadata, &chain, source).expect("derive");

        assert_eq!(observed.search_code, training::derive_grouped_pq4_code(source, &model));
        assert!(
            observed.binary_words.is_empty(),
            "payload should omit binary sidecar words when the flag is clear"
        );
    }

    // IN-003: grouped codebooks are mandatory.
    #[test]
    fn in_003_missing_codebook_head_errors() {
        let (mut metadata, chain, vectors, _) = staged_metadata(true);
        metadata.grouped_codebook_head = ItemPointer::INVALID;
        let err = derive_insert_payload_from_persisted(&metadata, &chain, &vectors[0])
            .expect_err("missing codebooks should fail");
        assert!(err.contains("persisted grouped codebooks"), "got: {err}");
    }

    // IN-004: source dimension must match metadata.
    #[test]
    fn in_004_dimension_mismatch_errors() {
        let (metadata, chain, _, _) = staged_metadata(true);
        let err = derive_insert_payload_from_persisted(&metadata, &chain, &[1.0, 2.0, 3.0])
            .expect_err("dim mismatch should fail");
        assert!(err.contains("dimension mismatch"), "got: {err}");
    }

    // IN-005: unsupported metadata transform / codec is rejected up front.
    #[test]
    fn in_005_transform_and_codec_are_validated() {
        let (mut metadata, chain, vectors, _) = staged_metadata(true);
        metadata.transform_kind = 99;
        let err = derive_insert_payload_from_persisted(&metadata, &chain, &vectors[0])
            .expect_err("bad transform should fail");
        assert!(err.contains("transform kind"), "got: {err}");

        let (mut metadata, chain, vectors, _) = staged_metadata(true);
        metadata.search_codec_kind = 99;
        let err = derive_insert_payload_from_persisted(&metadata, &chain, &vectors[0])
            .expect_err("bad codec should fail");
        assert!(err.contains("codec kind"), "got: {err}");
    }
}
