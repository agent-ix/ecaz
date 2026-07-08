//! Read-side helpers for `ec_distann` persisted structures: chain
//! materialization, the sorted vec_id→TID directory, and the FR-080
//! entry-region sample. Consumed by the FR-081 scan slice and TC-037.

use pgrx::pg_sys;

use crate::storage::{
    buffer_guard::{LockedBufferGuard, LockedPageTupleVisit},
    page::{DataPageChain, ItemPointer, FIRST_DATA_BLOCK_NUMBER},
    relation::RelationHandle,
};

use super::{
    ambuild::read_metadata_from_index_handle,
    page::DistannMetadataPage,
    tuple::{DistannDirectoryTuple, DistannHeadSampleTuple, DistannNodeTuple, DISTANN_NODE_TAG},
};

pub(crate) fn materialize_chain_from_index_handle(
    handle: RelationHandle,
) -> Result<(DistannMetadataPage, DataPageChain), String> {
    let metadata = read_metadata_from_index_handle(handle)?;

    let block_count = crate::storage::relation::main_fork_block_count_handle(handle);
    let mut chain = DataPageChain::new(pg_sys::BLCKSZ as usize);
    for block_number in FIRST_DATA_BLOCK_NUMBER..block_count {
        let buffer = LockedBufferGuard::read_main_handle(
            handle,
            block_number,
            pg_sys::ReadBufferMode::RBM_NORMAL,
            pg_sys::BUFFER_LOCK_SHARE as i32,
        )
        .ok_or_else(|| format!("ec_distann could not open data block {block_number}"))?;
        let max_offset = buffer.max_offset_number();
        for offset in 1..=max_offset {
            let tid = ItemPointer {
                block_number,
                offset_number: offset,
            };
            let visit = buffer.visit_tuple_bytes(tid, "ec_distann data block", |tuple_bytes| {
                Ok(tuple_bytes.to_vec())
            })?;
            if let LockedPageTupleVisit::Present(tuple_bytes) = visit {
                chain.insert_raw_tuple(tuple_bytes)?;
            }
        }
    }

    Ok((metadata, chain))
}

/// Fetch one raw tuple's bytes from the live relation (buffer read).
pub(crate) fn read_raw_tuple_bytes_from_relation(
    handle: RelationHandle,
    tid: ItemPointer,
    context: &str,
) -> Result<Vec<u8>, String> {
    let buffer = LockedBufferGuard::read_main_handle(
        handle,
        tid.block_number,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_SHARE as i32,
    )
    .ok_or_else(|| format!("{context}: could not open block {}", tid.block_number))?;
    match buffer.visit_tuple_bytes(tid, context, |raw| Ok(raw.to_vec()))? {
        LockedPageTupleVisit::Present(bytes) => Ok(bytes),
        _ => Err(format!(
            "{context}: no tuple at ({},{})",
            tid.block_number, tid.offset_number
        )),
    }
}

/// Walk the directory chain from the live relation into one ascending
/// vec_id -> TID vector (buffer reads; no chain materialization).
pub(crate) fn read_directory_from_relation(
    handle: RelationHandle,
    head_tid: ItemPointer,
    expected_entries: usize,
) -> Result<Vec<(u64, ItemPointer)>, String> {
    let mut entries = Vec::with_capacity(expected_entries);
    let mut cursor = head_tid;
    while cursor != ItemPointer::INVALID {
        let raw =
            read_raw_tuple_bytes_from_relation(handle, cursor, "ec_distann directory chunk")?;
        let tuple = DistannDirectoryTuple::decode(&raw)?;
        entries.extend_from_slice(&tuple.entries);
        cursor = tuple.next_tid;
        if entries.len() > expected_entries {
            return Err(format!(
                "ec_distann directory chain yielded more than {expected_entries} entries"
            ));
        }
    }
    if entries.len() != expected_entries {
        return Err(format!(
            "ec_distann directory chain yielded {} entries, expected {expected_entries}",
            entries.len()
        ));
    }
    if !entries.windows(2).all(|pair| pair[0].0 < pair[1].0) {
        return Err("ec_distann directory chain is not strictly ascending by vec_id".to_owned());
    }
    Ok(entries)
}

/// Walk the FR-080 head-sample chain from the live relation in persisted
/// (BFS) order.
pub(crate) fn read_head_samples_from_relation(
    handle: RelationHandle,
    head_tid: ItemPointer,
    dimensions: usize,
    cap: usize,
) -> Result<Vec<DistannHeadSampleTuple>, String> {
    let mut samples = Vec::new();
    let mut cursor = head_tid;
    while cursor != ItemPointer::INVALID {
        let raw =
            read_raw_tuple_bytes_from_relation(handle, cursor, "ec_distann head sample")?;
        let tuple = DistannHeadSampleTuple::decode(&raw, dimensions)?;
        cursor = tuple.next_tid;
        samples.push(tuple);
        if samples.len() > cap {
            return Err(format!(
                "ec_distann head-sample chain exceeds the head_index_cap {cap}"
            ));
        }
    }
    if samples.is_empty() {
        return Err("ec_distann head-sample chain is empty (FR-080 strict policy)".to_owned());
    }
    Ok(samples)
}

/// Walk the FR-083 D5 interim delta buffer from the live relation, newest
/// first (the chain is prepended at insert). Bounded by
/// `DISTANN_DELTA_BUFFER_CAP`; a longer chain is a structural fault.
pub(crate) fn read_delta_chain(
    handle: RelationHandle,
    head_tid: ItemPointer,
    dimensions: usize,
) -> Result<Vec<super::tuple::DistannDeltaTuple>, String> {
    let mut entries = Vec::new();
    let mut cursor = head_tid;
    while cursor != ItemPointer::INVALID {
        let raw = read_raw_tuple_bytes_from_relation(handle, cursor, "ec_distann delta entry")?;
        let tuple = super::tuple::DistannDeltaTuple::decode(&raw, dimensions)?;
        cursor = tuple.next_tid;
        entries.push(tuple);
        if entries.len() > 1 << 20 {
            return Err("ec_distann delta chain is unbounded (structural fault)".to_owned());
        }
    }
    Ok(entries)
}

/// Walk the persisted GroupedPq codebook chain from the live relation into
/// the flat centroid array (mirrors ec_diskann's chain reader shape).
pub(crate) fn read_grouped_codebooks_from_relation(
    handle: RelationHandle,
    head_tid: ItemPointer,
    group_count: usize,
    centroid_count: usize,
) -> Result<Vec<f32>, String> {
    if head_tid == ItemPointer::INVALID {
        return Err("ec_distann grouped codebook chain head is INVALID".to_owned());
    }
    let mut flat = Vec::with_capacity(group_count * centroid_count);
    let mut cursor = head_tid;
    let mut group_index = 0_usize;
    while cursor != ItemPointer::INVALID {
        let raw =
            read_raw_tuple_bytes_from_relation(handle, cursor, "ec_distann codebook tuple")?;
        let tuple = crate::am::VamanaCodebookTuple::decode(&raw, centroid_count)?;
        if usize::from(tuple.group_index) != group_index {
            return Err(format!(
                "ec_distann codebook chain out of order: got group {}, expected {group_index}",
                tuple.group_index
            ));
        }
        flat.extend_from_slice(&tuple.centroids);
        cursor = tuple.nexttid;
        group_index += 1;
        if group_index > group_count {
            return Err(format!(
                "ec_distann codebook chain exceeds group_count {group_count}"
            ));
        }
    }
    if group_index != group_count {
        return Err(format!(
            "ec_distann codebook chain yielded {group_index} groups, expected {group_count}"
        ));
    }
    Ok(flat)
}

pub(crate) fn read_node(
    chain: &DataPageChain,
    tid: ItemPointer,
    graph_degree_r: u16,
    code_len: usize,
) -> Result<DistannNodeTuple, String> {
    let page = chain
        .get_page(tid.block_number)
        .ok_or_else(|| format!("ec_distann node block {} not materialized", tid.block_number))?;
    let raw = page.raw_tuple(tid)?;
    if raw.first().copied() != Some(DISTANN_NODE_TAG) {
        return Err(format!(
            "ec_distann tuple at ({},{}) is not a graph-node record",
            tid.block_number, tid.offset_number
        ));
    }
    DistannNodeTuple::decode(raw, graph_degree_r, code_len)
}

/// Walk the directory chain into one ascending vec_id→TID vector, verifying
/// global sort order (binary-search precondition).
pub(crate) fn read_directory_chain(
    chain: &DataPageChain,
    head_tid: ItemPointer,
    expected_entries: usize,
) -> Result<Vec<(u64, ItemPointer)>, String> {
    let mut entries = Vec::with_capacity(expected_entries);
    let mut cursor = head_tid;
    while cursor != ItemPointer::INVALID {
        let page = chain.get_page(cursor.block_number).ok_or_else(|| {
            format!(
                "ec_distann directory block {} not materialized",
                cursor.block_number
            )
        })?;
        let tuple = DistannDirectoryTuple::decode(page.raw_tuple(cursor)?)?;
        entries.extend_from_slice(&tuple.entries);
        cursor = tuple.next_tid;
        if entries.len() > expected_entries {
            return Err(format!(
                "ec_distann directory chain yielded more than {expected_entries} entries"
            ));
        }
    }
    if entries.len() != expected_entries {
        return Err(format!(
            "ec_distann directory chain yielded {} entries, expected {expected_entries}",
            entries.len()
        ));
    }
    if !entries.windows(2).all(|pair| pair[0].0 < pair[1].0) {
        return Err("ec_distann directory chain is not strictly ascending by vec_id".to_owned());
    }
    Ok(entries)
}

/// Resolve one vec_id through a materialized directory (binary search).
pub(crate) fn directory_lookup(
    directory: &[(u64, ItemPointer)],
    vec_id: u64,
) -> Option<ItemPointer> {
    directory
        .binary_search_by_key(&vec_id, |(id, _)| *id)
        .ok()
        .map(|index| directory[index].1)
}

/// Walk the FR-080 head-sample chain in persisted (BFS) order.
pub(crate) fn read_head_sample_chain(
    chain: &DataPageChain,
    head_tid: ItemPointer,
    dimensions: usize,
    cap: usize,
) -> Result<Vec<DistannHeadSampleTuple>, String> {
    let mut samples = Vec::new();
    let mut cursor = head_tid;
    while cursor != ItemPointer::INVALID {
        let page = chain.get_page(cursor.block_number).ok_or_else(|| {
            format!(
                "ec_distann head-sample block {} not materialized",
                cursor.block_number
            )
        })?;
        let tuple = DistannHeadSampleTuple::decode(page.raw_tuple(cursor)?, dimensions)?;
        cursor = tuple.next_tid;
        samples.push(tuple);
        if samples.len() > cap {
            return Err(format!(
                "ec_distann head-sample chain exceeds the head_index_cap {cap}"
            ));
        }
    }
    if samples.is_empty() {
        return Err("ec_distann head-sample chain is empty (FR-080 strict policy)".to_owned());
    }
    Ok(samples)
}
