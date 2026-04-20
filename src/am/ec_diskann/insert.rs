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
use crate::storage::page::{
    element_or_neighbor_tuple_fits, raw_tuple_storage_bytes, DataPageChain, ItemPointer,
    FIRST_DATA_BLOCK_NUMBER,
};
use crate::storage::wal;
use crate::{DEFAULT_QUANT_BITS, DEFAULT_QUANT_SEED};

use super::scan_query::{encode_query_srht, read_grouped_codebook_chain};
use super::{
    ambuild,
    build::{build_and_persist_vamana, BuildOutput, BuildParams},
    options,
    page::{
        VamanaMetadataPage, PAYLOAD_FLAG_BINARY_SIDECAR, VAMANA_METADATA_BYTES,
        VAMANA_SEARCH_CODEC_GROUPED_PQ, VAMANA_TRANSFORM_KIND_SRHT,
    },
    persist::{stage_grouped_codebook_chain, NodePayload},
    reader::PersistedGraphReader,
    scan_state,
    tuple::VamanaNodeTuple,
    vamana::{robust_prune, Candidate},
};

const EMPTY_INSERT_BOOTSTRAP_KMEANS_ITERS: usize = 8;
const P_NEW: pg_sys::BlockNumber = u32::MAX;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DerivedInsertPayload {
    pub(super) binary_words: Vec<u64>,
    pub(super) search_code: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ForwardNeighborCandidate {
    pub(super) tid: ItemPointer,
    pub(super) source_vector: Vec<f32>,
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
        return Err(
            "ec_diskann insert payload derivation requires persisted grouped codebooks".into(),
        );
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

pub(super) fn duplicate_candidate_tids_by_payload(
    reader: &PersistedGraphReader<'_>,
    payload: &DerivedInsertPayload,
) -> Result<Vec<ItemPointer>, String> {
    let mut matches = Vec::new();
    for tid in reader.iter_node_tids() {
        let tid = tid?;
        let tuple = reader.read_node(tid)?;
        if tuple.deleted || tuple.primary_heaptid == ItemPointer::INVALID {
            continue;
        }
        if tuple.binary_words == payload.binary_words && tuple.search_code == payload.search_code {
            matches.push(tid);
        }
    }
    Ok(matches)
}

pub(super) fn select_insert_forward_neighbors(
    source_vector: &[f32],
    candidates: &[ForwardNeighborCandidate],
    alpha: f32,
    max_degree: usize,
) -> Result<Vec<ItemPointer>, String> {
    if source_vector.is_empty() {
        return Err("ec_diskann insert planning requires a non-empty source vector".into());
    }
    if !(alpha.is_finite() && alpha >= 1.0) {
        return Err(format!(
            "ec_diskann insert planning alpha must be finite and >= 1.0, got {alpha}"
        ));
    }
    if max_degree == 0 {
        return Err("ec_diskann insert planning max_degree must be > 0".into());
    }
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let source_distances = candidates
        .iter()
        .map(|candidate| source_inner_product_distance(source_vector, &candidate.source_vector))
        .collect::<Result<Vec<_>, _>>()?;
    let mut pairwise_distances = vec![vec![0.0_f32; candidates.len()]; candidates.len()];
    for left in 0..candidates.len() {
        for right in (left + 1)..candidates.len() {
            let distance = source_inner_product_distance(
                &candidates[left].source_vector,
                &candidates[right].source_vector,
            )?;
            pairwise_distances[left][right] = distance;
            pairwise_distances[right][left] = distance;
        }
    }

    let initial = source_distances
        .into_iter()
        .enumerate()
        .map(|(idx, distance)| Candidate {
            node: idx as u32,
            distance,
        })
        .collect::<Vec<_>>();
    let kept = robust_prune(u32::MAX, initial, alpha, max_degree, |left, right| {
        pairwise_distances[left as usize][right as usize]
    });
    Ok(kept
        .into_iter()
        .map(|idx| candidates[idx as usize].tid)
        .collect())
}

pub(super) fn insert_backlink_if_free(
    tuple: &mut VamanaNodeTuple,
    backlink_tid: ItemPointer,
) -> bool {
    if backlink_tid == ItemPointer::INVALID {
        return false;
    }
    if tuple.neighbors.contains(&backlink_tid) {
        return false;
    }

    let Some((slot_idx, slot)) = tuple
        .neighbors
        .iter_mut()
        .enumerate()
        .find(|(_, tid)| **tid == ItemPointer::INVALID)
    else {
        return false;
    };
    *slot = backlink_tid;

    let neighbor_count = usize::from(tuple.neighbor_count);
    if slot_idx >= neighbor_count {
        tuple.neighbor_count = u16::try_from(slot_idx + 1).expect("neighbor count fits in u16");
    }
    true
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

    let dimensions = u16::try_from(source_vector.len()).map_err(|_| {
        format!(
            "ec_diskann insert source dimension {} exceeds u16",
            source_vector.len()
        )
    })?;
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

    let BuildOutput {
        mut metadata,
        persisted,
    } = build_and_persist_vamana(params, &payloads, |_, _| 0.0)?;
    let mut chain = persisted.chain;
    let codebook_head = stage_grouped_codebook_chain(&mut chain, &model)?;
    metadata.grouped_codebook_head = codebook_head;
    metadata.inserted_since_rebuild = 1;

    Ok(EmptyInsertBootstrapOutput { metadata, chain })
}

pub(super) unsafe fn append_live_node(
    index_relation: pg_sys::Relation,
    metadata: &VamanaMetadataPage,
    heap_tid: ItemPointer,
    payload: &DerivedInsertPayload,
    forward_neighbors: &[ItemPointer],
) -> Result<ItemPointer, String> {
    if heap_tid == ItemPointer::INVALID {
        return Err("ec_diskann append requires a valid heap tid".into());
    }
    if forward_neighbors.len() > metadata.graph_degree_r as usize {
        return Err(format!(
            "ec_diskann append forward-neighbor count {} exceeds graph degree {}",
            forward_neighbors.len(),
            metadata.graph_degree_r
        ));
    }

    let mut tuple = VamanaNodeTuple::placeholder(
        metadata.graph_degree_r,
        payload.binary_words.len(),
        payload.search_code.len(),
    );
    tuple.primary_heaptid = heap_tid;
    tuple.binary_words = payload.binary_words.clone();
    tuple.search_code = payload.search_code.clone();
    tuple.neighbor_count = u16::try_from(forward_neighbors.len())
        .map_err(|_| "forward neighbor count does not fit in u16".to_owned())?;
    for (slot, neighbor) in forward_neighbors.iter().copied().enumerate() {
        tuple.neighbors[slot] = neighbor;
    }

    let encoded = tuple.encode(
        metadata.graph_degree_r,
        payload.binary_words.len(),
        payload.search_code.len(),
    )?;
    if !element_or_neighbor_tuple_fits(encoded.len(), pg_sys::BLCKSZ as usize) {
        return Err(format!(
            "ec_diskann append node payload {} exceeds page capacity {}",
            encoded.len(),
            pg_sys::BLCKSZ as usize
        ));
    }

    let existing_blocks = unsafe {
        pg_sys::RelationGetNumberOfBlocksInFork(index_relation, pg_sys::ForkNumber::MAIN_FORKNUM)
    };
    let target_block = if existing_blocks > FIRST_DATA_BLOCK_NUMBER {
        existing_blocks - 1
    } else {
        P_NEW
    };
    unsafe {
        append_live_node_payload(
            index_relation,
            &encoded,
            raw_tuple_storage_bytes(encoded.len()),
            target_block,
        )
    }
}

unsafe fn append_live_node_payload(
    index_relation: pg_sys::Relation,
    encoded: &[u8],
    required_bytes: usize,
    target_block: pg_sys::BlockNumber,
) -> Result<ItemPointer, String> {
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
        return Err("ec_diskann failed to allocate append buffer".into());
    }

    if target_block != P_NEW {
        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
    }

    let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
    let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
    let page_ptr =
        unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
    if target_block == P_NEW {
        unsafe { pg_sys::PageInit(page_ptr, page_size, 0) };
    } else {
        let free_space = unsafe { pg_sys::PageGetFreeSpace(page_ptr) as usize };
        if free_space < required_bytes {
            std::mem::drop(wal_txn);
            unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
            return unsafe {
                append_live_node_payload(index_relation, encoded, required_bytes, P_NEW)
            };
        }
    }

    let block_number = unsafe { pg_sys::BufferGetBlockNumber(buffer) };
    let offset_number = unsafe {
        pg_sys::PageAddItemExtended(
            page_ptr,
            encoded.as_ptr().cast_mut().cast(),
            encoded.len(),
            pg_sys::InvalidOffsetNumber,
            0,
        )
    };
    if offset_number == pg_sys::InvalidOffsetNumber {
        return Err("ec_diskann failed to append live node tuple".into());
    }

    unsafe { wal_txn.finish() };
    unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    Ok(ItemPointer {
        block_number,
        offset_number,
    })
}

pub(super) unsafe fn add_backlinks_if_free(
    index_relation: pg_sys::Relation,
    metadata: &VamanaMetadataPage,
    backlink_targets: &[ItemPointer],
    new_tid: ItemPointer,
) -> Result<usize, String> {
    if new_tid == ItemPointer::INVALID {
        return Err("ec_diskann backlink write requires a valid new node tid".into());
    }
    if backlink_targets.is_empty() {
        return Ok(0);
    }

    let mut targets = backlink_targets.to_vec();
    sort_and_dedup_backlink_targets(&mut targets);
    let binary_word_count = scan_state::metadata_binary_word_count(metadata);
    let search_code_len = scan_state::metadata_search_code_len(metadata);
    let mut changed = 0usize;
    let mut start = 0usize;

    while start < targets.len() {
        let block_number = targets[start].block_number;
        let buffer = unsafe {
            pg_sys::ReadBufferExtended(
                index_relation,
                pg_sys::ForkNumber::MAIN_FORKNUM,
                block_number,
                pg_sys::ReadBufferMode::RBM_NORMAL,
                ptr::null_mut(),
            )
        };
        if !unsafe { pg_sys::BufferIsValid(buffer) } {
            return Err(format!(
                "ec_diskann backlink write could not open target block {block_number}"
            ));
        }

        unsafe { pg_sys::LockBuffer(buffer, pg_sys::BUFFER_LOCK_EXCLUSIVE as i32) };
        let page_size = unsafe { pg_sys::BufferGetPageSize(buffer) as usize };
        let mut wal_txn = unsafe { wal::GenericXLogTxn::start(index_relation) };
        let writable_page =
            unsafe { wal_txn.register_buffer(buffer, pg_sys::GENERIC_XLOG_FULL_IMAGE as i32) };
        let mut page_changed = false;
        let page_result = (|| -> Result<usize, String> {
            let mut page_changes = 0usize;
            while start < targets.len() && targets[start].block_number == block_number {
                let target_tid = targets[start];
                start += 1;

                let (tuple_ptr, tuple_len) =
                    unsafe { page_tuple_location(writable_page, page_size, target_tid)? };
                let tuple_bytes =
                    unsafe { slice::from_raw_parts(tuple_ptr.cast_const(), tuple_len) };
                let mut tuple = VamanaNodeTuple::decode(
                    tuple_bytes,
                    metadata.graph_degree_r,
                    binary_word_count,
                    search_code_len,
                )?;
                if !tuple.is_live() {
                    continue;
                }
                if !insert_backlink_if_free(&mut tuple, new_tid) {
                    continue;
                }

                let encoded =
                    tuple.encode(metadata.graph_degree_r, binary_word_count, search_code_len)?;
                if encoded.len() != tuple_len {
                    return Err(format!(
                        "ec_diskann backlink target tuple size changed from {} to {} at ({},{})",
                        tuple_len,
                        encoded.len(),
                        target_tid.block_number,
                        target_tid.offset_number
                    ));
                }
                unsafe { ptr::copy_nonoverlapping(encoded.as_ptr(), tuple_ptr, encoded.len()) };
                page_changed = true;
                page_changes += 1;
            }
            Ok(page_changes)
        })();

        match page_result {
            Ok(page_changes) => {
                if page_changed {
                    unsafe { wal_txn.finish() };
                    changed += page_changes;
                } else {
                    std::mem::drop(wal_txn);
                }
            }
            Err(error) => {
                std::mem::drop(wal_txn);
                unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
                return Err(error);
            }
        }
        unsafe { pg_sys::UnlockReleaseBuffer(buffer) };
    }

    Ok(changed)
}

pub(super) unsafe fn increment_inserted_since_rebuild(
    index_relation: pg_sys::Relation,
) -> Result<u64, String> {
    unsafe {
        with_locked_metadata_page(index_relation, |metadata| {
            metadata.inserted_since_rebuild = metadata
                .inserted_since_rebuild
                .checked_add(1)
                .ok_or_else(|| "ec_diskann inserted_since_rebuild overflowed u64".to_owned())?;
            Ok(metadata.inserted_since_rebuild)
        })
    }
}

fn sort_and_dedup_backlink_targets(targets: &mut Vec<ItemPointer>) {
    targets.sort_unstable_by(|left, right| {
        left.block_number
            .cmp(&right.block_number)
            .then_with(|| left.offset_number.cmp(&right.offset_number))
    });
    targets.dedup();
}

unsafe fn page_tuple_location(
    page: pg_sys::Page,
    page_size: usize,
    tid: ItemPointer,
) -> Result<(*mut u8, usize), String> {
    let max_offset = unsafe { pg_sys::PageGetMaxOffsetNumber(page) };
    if tid.offset_number == pg_sys::InvalidOffsetNumber || tid.offset_number > max_offset {
        return Err(format!(
            "ec_diskann backlink target ({},{}) has invalid offset {} (max {})",
            tid.block_number, tid.offset_number, tid.offset_number, max_offset
        ));
    }

    let item_id = unsafe { pg_sys::PageGetItemId(page, tid.offset_number) };
    if item_id.is_null() {
        return Err(format!(
            "ec_diskann backlink target ({},{}) returned a null item id",
            tid.block_number, tid.offset_number
        ));
    }
    let item_id_ref = unsafe { &*item_id };
    if item_id_ref.lp_flags() == 0 {
        return Err(format!(
            "ec_diskann backlink target ({},{}) points at an unused slot",
            tid.block_number, tid.offset_number
        ));
    }

    let tuple_offset = item_id_ref.lp_off() as usize;
    let tuple_len = item_id_ref.lp_len() as usize;
    if tuple_offset + tuple_len > page_size {
        return Err(format!(
            "ec_diskann backlink target ({},{}) has invalid tuple bounds",
            tid.block_number, tid.offset_number
        ));
    }

    let tuple_ptr = unsafe { pg_sys::PageGetItem(page, item_id) }.cast::<u8>();
    if tuple_ptr.is_null() {
        return Err(format!(
            "ec_diskann backlink target ({},{}) returned a null tuple pointer",
            tid.block_number, tid.offset_number
        ));
    }
    Ok((tuple_ptr, tuple_len))
}

fn source_inner_product_distance(left: &[f32], right: &[f32]) -> Result<f32, String> {
    if left.len() != right.len() {
        return Err(format!(
            "ec_diskann exact distance dimension mismatch: left dim {}, right dim {}",
            left.len(),
            right.len()
        ));
    }
    let ip = left
        .iter()
        .zip(right.iter())
        .map(|(lhs, rhs)| lhs * rhs)
        .sum::<f32>();
    Ok((-ip).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::common::training::{self, train_grouped_pq4_model};
    use crate::am::ec_diskann::page::PAYLOAD_FLAG_GROUPED_SEARCH_CODE;
    use crate::am::ec_diskann::persist::stage_grouped_codebook_chain;
    use crate::am::ec_diskann::tuple::VamanaNodeTuple;
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
    ) -> (
        VamanaMetadataPage,
        DataPageChain,
        Vec<Vec<f32>>,
        training::GroupedPq4Model,
    ) {
        let vectors = training_vectors();
        let refs: Vec<&[f32]> = vectors.iter().map(Vec::as_slice).collect();
        let dimensions = vectors[0].len();
        let seed = 42_u64;
        let group_size = 4_usize;
        let model = train_grouped_pq4_model(&refs, dimensions, seed, group_size, refs.len(), 6)
            .expect("train");

        let mut chain = DataPageChain::new(DEFAULT_PAGE_SIZE);
        let codebook_head =
            stage_grouped_codebook_chain(&mut chain, &model).expect("stage codebooks");

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
        let quantizer = ProdQuantizer::cached(source.len(), DEFAULT_QUANT_BITS, metadata.seed);
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

        assert_eq!(
            observed.search_code,
            training::derive_grouped_pq4_code(source, &model)
        );
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

    #[test]
    fn in_006_duplicate_lookup_finds_first_live_match() {
        let (mut metadata, chain, vectors, model) = staged_metadata(true);
        let source = &vectors[0];
        let payload =
            derive_insert_payload_from_persisted(&metadata, &chain, source).expect("derive");
        let mut node_chain = DataPageChain::new(DEFAULT_PAGE_SIZE);

        let mut first = VamanaNodeTuple::placeholder(
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );
        first.binary_words = payload.binary_words.clone();
        first.search_code = payload.search_code.clone();
        first.primary_heaptid = ItemPointer {
            block_number: 500,
            offset_number: 1,
        };
        node_chain
            .insert_raw_tuple(
                first
                    .encode(
                        metadata.graph_degree_r,
                        payload.binary_words.len(),
                        payload.search_code.len(),
                    )
                    .expect("first tuple should encode"),
            )
            .expect("first tuple");

        let mut other = VamanaNodeTuple::placeholder(
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );
        other.binary_words = payload.binary_words.clone();
        other.search_code = payload.search_code.clone();
        other.primary_heaptid = ItemPointer {
            block_number: 501,
            offset_number: 1,
        };
        node_chain
            .insert_raw_tuple(
                other
                    .encode(
                        metadata.graph_degree_r,
                        payload.binary_words.len(),
                        payload.search_code.len(),
                    )
                    .expect("second tuple should encode"),
            )
            .expect("second tuple");
        metadata.grouped_codebook_head =
            stage_grouped_codebook_chain(&mut node_chain, &model).expect("stage codebooks");

        let reader = PersistedGraphReader::new(
            &node_chain,
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );

        let matches = duplicate_candidate_tids_by_payload(&reader, &payload).expect("lookup");

        assert_eq!(
            matches.len(),
            2,
            "both live payload matches should be returned"
        );
        assert_eq!(matches[0].block_number, 1);
        assert_eq!(matches[0].offset_number, 1);
    }

    #[test]
    fn in_007_duplicate_lookup_skips_deleted_and_stripped_tuples() {
        let (mut metadata, chain, vectors, model) = staged_metadata(true);
        let source = &vectors[0];
        let payload =
            derive_insert_payload_from_persisted(&metadata, &chain, source).expect("derive");
        let mut node_chain = DataPageChain::new(DEFAULT_PAGE_SIZE);

        let mut deleted = VamanaNodeTuple::placeholder(
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );
        deleted.binary_words = payload.binary_words.clone();
        deleted.search_code = payload.search_code.clone();
        deleted.primary_heaptid = ItemPointer {
            block_number: 500,
            offset_number: 1,
        };
        deleted.deleted = true;
        node_chain
            .insert_raw_tuple(
                deleted
                    .encode(
                        metadata.graph_degree_r,
                        payload.binary_words.len(),
                        payload.search_code.len(),
                    )
                    .expect("deleted tuple should encode"),
            )
            .expect("deleted tuple");

        let mut stripped = VamanaNodeTuple::placeholder(
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );
        stripped.binary_words = payload.binary_words.clone();
        stripped.search_code = payload.search_code.clone();
        stripped.primary_heaptid = ItemPointer::INVALID;
        stripped.has_overflow_heaptids = true;
        node_chain
            .insert_raw_tuple(
                stripped
                    .encode(
                        metadata.graph_degree_r,
                        payload.binary_words.len(),
                        payload.search_code.len(),
                    )
                    .expect("stripped tuple should encode"),
            )
            .expect("stripped tuple");
        metadata.grouped_codebook_head =
            stage_grouped_codebook_chain(&mut node_chain, &model).expect("stage codebooks");

        let reader = PersistedGraphReader::new(
            &node_chain,
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );

        let tid = duplicate_candidate_tids_by_payload(&reader, &payload).expect("lookup");
        assert!(
            tid.is_empty(),
            "deleted or stripped tuples must not be eligible duplicate targets"
        );
    }

    #[test]
    fn in_008_forward_neighbor_selection_prunes_on_exact_vectors() {
        let source = vec![1.0_f32, 0.0];
        let candidates = vec![
            ForwardNeighborCandidate {
                tid: ItemPointer {
                    block_number: 1,
                    offset_number: 1,
                },
                source_vector: vec![0.0, 1.0],
            },
            ForwardNeighborCandidate {
                tid: ItemPointer {
                    block_number: 1,
                    offset_number: 2,
                },
                source_vector: vec![0.0, -1.0],
            },
            ForwardNeighborCandidate {
                tid: ItemPointer {
                    block_number: 1,
                    offset_number: 3,
                },
                source_vector: vec![-1.0, 0.0],
            },
        ];

        let selected =
            select_insert_forward_neighbors(&source, &candidates, 1.2, 2).expect("select");

        assert_eq!(
            selected,
            vec![candidates[0].tid, candidates[1].tid],
            "the exact-vector alpha prune should retain the two orthogonal neighbors"
        );
    }

    #[test]
    fn in_009_forward_neighbor_selection_rejects_dimension_mismatch() {
        let err = select_insert_forward_neighbors(
            &[1.0, 0.0],
            &[ForwardNeighborCandidate {
                tid: ItemPointer {
                    block_number: 1,
                    offset_number: 1,
                },
                source_vector: vec![1.0, 0.0, -1.0],
            }],
            1.2,
            4,
        )
        .expect_err("dimension mismatch should fail");
        assert!(err.contains("dimension mismatch"), "got: {err}");
    }

    #[test]
    fn in_010_duplicate_lookup_finds_match_after_codebook_tail() {
        let (metadata, chain, vectors, model) = staged_metadata(true);
        let source = &vectors[0];
        let payload =
            derive_insert_payload_from_persisted(&metadata, &chain, source).expect("derive");
        let mut node_chain = DataPageChain::new(DEFAULT_PAGE_SIZE);

        let mut first = VamanaNodeTuple::placeholder(
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );
        first.binary_words = payload.binary_words.clone();
        first.search_code = payload.search_code.clone();
        first.primary_heaptid = ItemPointer {
            block_number: 500,
            offset_number: 1,
        };
        node_chain
            .insert_raw_tuple(
                first
                    .encode(
                        metadata.graph_degree_r,
                        payload.binary_words.len(),
                        payload.search_code.len(),
                    )
                    .expect("first tuple should encode"),
            )
            .expect("first tuple");

        stage_grouped_codebook_chain(&mut node_chain, &model).expect("stage codebooks");

        let mut appended = VamanaNodeTuple::placeholder(
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );
        appended.binary_words = payload.binary_words.clone();
        appended.search_code = payload.search_code.clone();
        appended.primary_heaptid = ItemPointer {
            block_number: 501,
            offset_number: 1,
        };
        node_chain
            .insert_raw_tuple(
                appended
                    .encode(
                        metadata.graph_degree_r,
                        payload.binary_words.len(),
                        payload.search_code.len(),
                    )
                    .expect("appended tuple should encode"),
            )
            .expect("appended tuple");

        let reader = PersistedGraphReader::new(
            &node_chain,
            metadata.graph_degree_r,
            payload.binary_words.len(),
            payload.search_code.len(),
        );

        let matches = duplicate_candidate_tids_by_payload(&reader, &payload).expect("lookup");
        assert_eq!(
            matches,
            vec![
                ItemPointer {
                    block_number: 1,
                    offset_number: 1,
                },
                ItemPointer {
                    block_number: 1,
                    offset_number: 4,
                },
            ]
        );
    }

    #[test]
    fn in_011_insert_backlink_if_free_uses_first_open_slot() {
        let backlink_tid = ItemPointer {
            block_number: 9,
            offset_number: 4,
        };
        let mut tuple = VamanaNodeTuple::placeholder(4, 0, 0);
        tuple.neighbor_count = 1;
        tuple.neighbors[0] = ItemPointer {
            block_number: 3,
            offset_number: 1,
        };

        let changed = insert_backlink_if_free(&mut tuple, backlink_tid);

        assert!(
            changed,
            "a tuple with free neighbor capacity should admit a backlink"
        );
        assert_eq!(tuple.neighbor_count, 2);
        assert_eq!(tuple.neighbors[1], backlink_tid);
    }

    #[test]
    fn in_012_insert_backlink_if_free_rejects_duplicate_and_full_tuples() {
        let backlink_tid = ItemPointer {
            block_number: 9,
            offset_number: 4,
        };
        let mut duplicate = VamanaNodeTuple::placeholder(2, 0, 0);
        duplicate.neighbor_count = 2;
        duplicate.neighbors[0] = backlink_tid;
        duplicate.neighbors[1] = ItemPointer {
            block_number: 5,
            offset_number: 2,
        };
        assert!(
            !insert_backlink_if_free(&mut duplicate, backlink_tid),
            "duplicate backlinks must not rewrite the tuple"
        );

        let mut full = VamanaNodeTuple::placeholder(2, 0, 0);
        full.neighbor_count = 2;
        full.neighbors[0] = ItemPointer {
            block_number: 7,
            offset_number: 1,
        };
        full.neighbors[1] = ItemPointer {
            block_number: 7,
            offset_number: 2,
        };
        assert!(
            !insert_backlink_if_free(&mut full, backlink_tid),
            "full tuples must stay unchanged in the free-capacity slice"
        );
    }
}
