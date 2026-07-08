//! FR-083 M5 incremental insert — graph-mutation planning.
//!
//! The M3 delta buffer (D5) makes an inserted row queryable immediately by
//! exact-scanning a bounded side buffer; M5 folds inserts into the persisted
//! Vamana graph so they gain graph connectivity (found by traversal, not just
//! the exact-scan tail) and the buffer stays bounded. This module carries the
//! pure planning half — forward-neighbor selection and backlink planning — that
//! the on-disk mutation slices (node append, in-place backlink amendment,
//! directory maintenance) build on. Pure functions here are unit-testable
//! without a live relation.

use std::ptr::NonNull;

use pgrx::{pg_extern, pg_sys};

use crate::am::ec_diskann::write_data_pages;
use crate::am::{robust_prune, Candidate};
use crate::storage::buffer_guard::LockedBufferGuard;
use crate::storage::page::{DataPageChain, ItemPointer};
use crate::storage::relation::{main_fork_block_count_handle, RelationHandle};
use crate::storage::relation_guard::IndexRelationGuard;
use crate::storage::wal;

use super::ambuild::{
    overwrite_metadata_page_handle, read_metadata_from_index_handle, stage_directory_chain,
};
use super::options::NeighborCodeFormat;
use super::quantizer::{metadata_code_len, DistannCodecBinding};
use super::reader::{
    directory_lookup, read_delta_chain, read_directory_from_relation,
    read_head_samples_from_relation, read_raw_tuple_bytes_from_relation,
};
use super::tuple::{
    distann_node_neighbor_codes_offset, distann_node_neighbor_vec_ids_offset, DistannNodeTuple,
    DISTANN_NODE_NEIGHBOR_COUNT_OFFSET, DISTANN_NODE_TAG,
};

/// A candidate forward neighbor for an inserted node: its stable `vec_id`
/// (distann adjacency references neighbors by vec_id, never by record TID) and
/// its full-precision co-placed source vector for exact-distance pruning.
#[derive(Debug, Clone)]
pub(super) struct DistannForwardCandidate {
    pub vec_id: u64,
    pub source_vector: Vec<f32>,
}

/// Exact `-inner_product` distance (the ec_distann rerank metric; smaller =
/// closer), matching the build's `dist` closure so incrementally-inserted
/// edges are selected on the same metric as the batch build.
fn exact_distance(left: &[f32], right: &[f32]) -> f32 {
    -crate::am::ec_diskann::source_inner_product(left, right)
}

/// Select an inserted node's forward edges: `robust_prune` the candidate set
/// (its FR-081 search frontier) to `max_degree` on exact distance, α-diversified
/// exactly like the batch build. Returns the kept neighbors' vec_ids. Pure.
pub(super) fn select_insert_forward_neighbors(
    source_vector: &[f32],
    candidates: &[DistannForwardCandidate],
    alpha: f32,
    max_degree: usize,
) -> Result<Vec<u64>, String> {
    if source_vector.is_empty() {
        return Err("ec_distann insert planning requires a non-empty source vector".to_owned());
    }
    if !(alpha.is_finite() && alpha >= 1.0) {
        return Err(format!(
            "ec_distann insert planning alpha must be finite and >= 1.0, got {alpha}"
        ));
    }
    if max_degree == 0 {
        return Err("ec_distann insert planning max_degree must be > 0".to_owned());
    }
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    for candidate in candidates {
        if candidate.source_vector.len() != source_vector.len() {
            return Err(format!(
                "ec_distann insert planning dimension mismatch: source dim {}, candidate dim {}",
                source_vector.len(),
                candidate.source_vector.len()
            ));
        }
    }

    // Distances from the inserted node to each candidate seed the prune order.
    let initial = candidates
        .iter()
        .enumerate()
        .map(|(idx, candidate)| Candidate {
            node: idx as u32,
            distance: exact_distance(source_vector, &candidate.source_vector),
        })
        .collect::<Vec<_>>();

    // robust_prune's diversity test needs pairwise candidate distances.
    let n = candidates.len();
    let mut pairwise = vec![vec![0.0_f32; n]; n];
    for left in 0..n {
        for right in (left + 1)..n {
            let distance =
                exact_distance(&candidates[left].source_vector, &candidates[right].source_vector);
            pairwise[left][right] = distance;
            pairwise[right][left] = distance;
        }
    }

    let kept = robust_prune(u32::MAX, initial, alpha, max_degree, |left, right| {
        pairwise[left as usize][right as usize]
    });
    Ok(kept
        .into_iter()
        .map(|idx| candidates[idx as usize].vec_id)
        .collect())
}

/// Rank candidate neighbors for an inserted vector by exact distance and keep
/// the closest `limit`. The candidates come from the FR-080 head-sample region
/// (full-precision vectors already persisted for graph seeding); for an index
/// whose node count is within the head-index cap this is every node, so the
/// selected forward edges are exact. Larger indexes seed from the representative
/// head region (a greedy walk is the scaling follow-up). Pure.
pub(super) fn rank_insert_candidates(
    new_vector: &[f32],
    samples: &[(u64, Vec<f32>)],
    limit: usize,
) -> Result<Vec<DistannForwardCandidate>, String> {
    if new_vector.is_empty() {
        return Err("ec_distann insert candidate ranking needs a non-empty vector".to_owned());
    }
    let mut scored: Vec<(f32, &(u64, Vec<f32>))> = Vec::with_capacity(samples.len());
    for sample in samples {
        if sample.1.len() != new_vector.len() {
            return Err(format!(
                "ec_distann insert candidate dim {} != inserted dim {}",
                sample.1.len(),
                new_vector.len()
            ));
        }
        scored.push((exact_distance(new_vector, &sample.1), sample));
    }
    // A candidate that is the inserted vector itself (re-insert) is dropped: a
    // node is never its own neighbor.
    scored.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
    Ok(scored
        .into_iter()
        .take(limit)
        .map(|(_, s)| DistannForwardCandidate {
            vec_id: s.0,
            source_vector: s.1.clone(),
        })
        .collect())
}

/// A neighbor that a newly-inserted node points to, and which must gain a
/// backlink to the new node. Carries the neighbor's own source vector plus its
/// current out-edges (vec_id + source vector each) so a full backlink can be
/// re-pruned on the same exact metric when the neighbor is already at capacity.
#[derive(Debug, Clone)]
pub(super) struct DistannBacklinkTarget {
    pub vec_id: u64,
    pub source_vector: Vec<f32>,
    pub current_neighbors: Vec<DistannForwardCandidate>,
}

/// Plan a neighbor's adjacency after an inserted node points to it (FR-083
/// back-edge amendment). If the neighbor has a free slot, the new node is simply
/// appended (cheap, edge-preserving). If it is already at `max_degree`, the
/// union of its current edges plus the new node is `robust_prune`d back to
/// `max_degree` on exact distance — so a full neighbor keeps its most
/// α-diverse edges rather than rejecting the backlink outright. Returns the
/// amended out-edge vec_id list. Pure; the on-disk slice just writes this.
pub(super) fn plan_insert_backlink(
    target: &DistannBacklinkTarget,
    new_vec_id: u64,
    new_source_vector: &[f32],
    alpha: f32,
    max_degree: usize,
) -> Result<Vec<u64>, String> {
    if max_degree == 0 {
        return Err("ec_distann backlink planning max_degree must be > 0".to_owned());
    }
    // Already linked (idempotent — a re-inserted/duplicate edge is a no-op).
    if target.current_neighbors.iter().any(|n| n.vec_id == new_vec_id) {
        return Ok(target.current_neighbors.iter().map(|n| n.vec_id).collect());
    }
    // Free slot: append, preserving existing edges.
    if target.current_neighbors.len() < max_degree {
        let mut kept: Vec<u64> = target.current_neighbors.iter().map(|n| n.vec_id).collect();
        kept.push(new_vec_id);
        return Ok(kept);
    }
    // Full: re-prune the union (current edges + the new node) from the
    // neighbor's own vantage point.
    let mut union = target.current_neighbors.clone();
    union.push(DistannForwardCandidate {
        vec_id: new_vec_id,
        source_vector: new_source_vector.to_vec(),
    });
    select_insert_forward_neighbors(&target.source_vector, &union, alpha, max_degree)
}

/// A forward neighbor for an inserted node's record: its vec_id plus its
/// coarse `search_code` (embedded into the new node's record per FR-076, so a
/// scan can code-score neighbors without a second record read).
#[derive(Debug, Clone)]
pub(super) struct DistannInsertNeighbor {
    pub vec_id: u64,
    pub code: Vec<u8>,
}

/// Assemble the FR-076 node record for an inserted node — the incremental
/// analog of the batch build's per-node staging (same layout: fixed
/// `graph_degree_r` slots, zero-padded past `neighbor_count`, neighbor codes
/// embedded). Pure; the on-disk append slice encodes + writes this.
pub(super) fn build_insert_node_tuple(
    vec_id: u64,
    heap_tid: ItemPointer,
    search_code: Vec<u8>,
    forward: &[DistannInsertNeighbor],
    graph_degree_r: u16,
    code_len: usize,
) -> Result<DistannNodeTuple, String> {
    let r = graph_degree_r as usize;
    if forward.len() > r {
        return Err(format!(
            "ec_distann insert node has {} neighbors, exceeding graph_degree {r}",
            forward.len()
        ));
    }
    if search_code.len() != code_len {
        return Err(format!(
            "ec_distann insert node search_code len {} != code_len {code_len}",
            search_code.len()
        ));
    }
    let mut neighbor_vec_ids = vec![0_u64; r];
    let mut neighbor_codes = vec![0_u8; r * code_len];
    for (slot, neighbor) in forward.iter().enumerate() {
        if neighbor.code.len() != code_len {
            return Err(format!(
                "ec_distann insert neighbor code len {} != code_len {code_len}",
                neighbor.code.len()
            ));
        }
        neighbor_vec_ids[slot] = neighbor.vec_id;
        neighbor_codes[slot * code_len..(slot + 1) * code_len].copy_from_slice(&neighbor.code);
    }
    Ok(DistannNodeTuple {
        tombstoned: false,
        vec_id,
        heap_tid,
        neighbor_count: u16::try_from(forward.len())
            .map_err(|_| "ec_distann insert neighbor count exceeds u16".to_owned())?,
        search_code,
        neighbor_vec_ids,
        neighbor_codes,
    })
}

/// FR-083 M5 incremental graph insert: fold one record into the persisted
/// Vamana graph so it gains graph connectivity (found by traversal, not just
/// the M3 delta-buffer exact-scan tail). Applies the tested pure pipeline
/// (rank → select → assemble) then mutates on disk: append the node record,
/// append-if-free backlinks on its forward neighbors, and rebuild the sorted
/// directory. Candidates come from the FR-080 head-sample region (no heap
/// reads). WAL-logged throughout. See `reviews/task-167/002-*` for the design.
///
/// # Safety
/// `index_relation` is a live index relation opened for write.
pub(super) unsafe fn graph_insert_record(
    index_relation: pg_sys::Relation,
    new_vec_id: u64,
    new_heap_tid: ItemPointer,
    new_source_vector: &[f32],
) -> Result<(), String> {
    let handle = NonNull::new(index_relation)
        .ok_or_else(|| "ec_distann graph insert needs a valid index relation".to_owned())?;
    let mut metadata = read_metadata_from_index_handle(handle)?;
    let dimensions = usize::from(metadata.dimensions);
    if new_source_vector.len() != dimensions {
        return Err(format!(
            "ec_distann graph insert dimension mismatch: index {dimensions}, inserted {}",
            new_source_vector.len()
        ));
    }
    if metadata.node_count == 0 || metadata.directory_head == ItemPointer::INVALID {
        return Err("ec_distann graph insert into an empty graph is unsupported".to_owned());
    }
    let code_len = metadata_code_len(&metadata)?;
    let graph_degree_r = metadata.graph_degree_r;

    // 1. Reconstruct the codec (seeded formats only; GroupedPq rehydration is a
    //    follow-up) and encode the new node's coarse search code.
    let format = NeighborCodeFormat::from_metadata_kind(metadata.neighbor_codec_kind)?;
    if matches!(format, NeighborCodeFormat::GroupedPq) {
        return Err(
            "ec_distann graph insert for grouped_pq codec is a follow-up (codebook rehydration); \
             the delta buffer still serves inserts"
                .to_owned(),
        );
    }
    let binding = DistannCodecBinding::prepare(format, &[], dimensions, metadata.seed)?;
    let new_code = binding.encode(new_source_vector);

    // 2. Candidate search over the head-sample region → 3. forward selection.
    let samples = read_head_samples_from_relation(
        handle,
        metadata.head_sample_head,
        dimensions,
        metadata.head_index_cap as usize,
    )?;
    let sample_pairs: Vec<(u64, Vec<f32>)> = samples
        .into_iter()
        .filter(|s| s.vec_id != new_vec_id)
        .map(|s| (s.vec_id, s.vector))
        .collect();
    let candidates = rank_insert_candidates(
        new_source_vector,
        &sample_pairs,
        usize::from(metadata.build_list_size_l),
    )?;
    let forward_ids = select_insert_forward_neighbors(
        new_source_vector,
        &candidates,
        metadata.alpha,
        usize::from(graph_degree_r),
    )?;

    // 4. Read each forward neighbor's record for its embedded search code.
    let directory =
        read_directory_from_relation(handle, metadata.directory_head, metadata.node_count as usize)?;
    let mut forward = Vec::with_capacity(forward_ids.len());
    let mut forward_tids = Vec::with_capacity(forward_ids.len());
    for &fid in &forward_ids {
        let tid = directory_lookup(&directory, fid).ok_or_else(|| {
            format!("ec_distann graph insert: forward neighbor {fid:#018x} not in directory")
        })?;
        let raw = read_raw_tuple_bytes_from_relation(handle, tid, "ec_distann insert forward")?;
        let node = DistannNodeTuple::decode(&raw, graph_degree_r, code_len)?;
        forward.push(DistannInsertNeighbor { vec_id: fid, code: node.search_code });
        forward_tids.push(tid);
    }

    // 5. Assemble + append the new node record.
    let node = build_insert_node_tuple(
        new_vec_id,
        new_heap_tid,
        new_code.clone(),
        &forward,
        graph_degree_r,
        code_len,
    )?;
    let new_node_tid = append_node_record(handle, node.encode(graph_degree_r, code_len)?)?;

    // 6. Backlinks on each forward neighbor: full-reprune from the neighbor's
    //    vantage using head-sample source vectors, so a folded node earns an
    //    incoming edge from the built graph (evicting the neighbor's worst edge
    //    when at capacity) rather than being skipped. Head samples cover the
    //    built nodes, so their source vectors are available for the reprune.
    let source_map: std::collections::HashMap<u64, Vec<f32>> =
        sample_pairs.iter().cloned().collect();
    for &tid in &forward_tids {
        add_backlink(
            handle,
            tid,
            new_vec_id,
            &new_code,
            new_source_vector,
            &source_map,
            metadata.alpha,
            graph_degree_r,
            code_len,
        )?;
    }

    // 7. Rebuild the sorted directory with the new entry, based at the current
    //    relation end so its next_tids are correct after write_data_pages.
    let mut vec_ids: Vec<u64> = directory.iter().map(|(id, _)| *id).collect();
    let mut tids: Vec<ItemPointer> = directory.iter().map(|(_, tid)| *tid).collect();
    vec_ids.push(new_vec_id);
    tids.push(new_node_tid);
    let base_block = main_fork_block_count_handle(handle);
    let mut dir_chain = DataPageChain::with_base_block(base_block, pg_sys::BLCKSZ as usize);
    let new_directory_head = stage_directory_chain(&mut dir_chain, &vec_ids, &tids)?;
    write_data_pages(handle, &dir_chain);

    // 8. Publish the new directory + node count.
    metadata.directory_head = new_directory_head;
    metadata.node_count += 1;
    overwrite_metadata_page_handle(handle, &metadata);
    Ok(())
}

/// FR-083 M5 maintenance op: fold the interim delta buffer (D5) into the
/// persisted graph — graph-insert every buffered entry, then drain the buffer.
/// Bounds the buffer and gives inserted rows graph connectivity. Returns the
/// count folded. (Multi-node: the coordinator routes each fold to the
/// hash-owning node via the FR-083 write endpoint — a later wire-up.)
#[pg_extern]
fn ec_distann_fold_delta_into_graph(index_regclass: pg_sys::Oid) -> i64 {
    fold_delta_into_graph_impl(index_regclass).unwrap_or_else(|e| pgrx::error!("{e}"))
}

fn fold_delta_into_graph_impl(index_oid: pg_sys::Oid) -> Result<i64, String> {
    let guard = IndexRelationGuard::open(
        index_oid,
        pg_sys::RowExclusiveLock as pg_sys::LOCKMODE,
        "ec_distann_fold_delta_into_graph",
    );
    let handle = NonNull::new(guard.as_ptr())
        .ok_or_else(|| "ec_distann fold got a null index relation".to_owned())?;
    let metadata = read_metadata_from_index_handle(handle)?;
    if metadata.delta_buffer_head == ItemPointer::INVALID {
        return Ok(0);
    }
    let deltas = read_delta_chain(
        handle,
        metadata.delta_buffer_head,
        usize::from(metadata.dimensions),
    )?;
    let mut folded = 0_i64;
    for entry in &deltas {
        // SAFETY: the guard holds the index open for write for the call.
        unsafe {
            graph_insert_record(guard.as_ptr(), entry.vec_id, entry.heap_tid, &entry.vector)?;
        }
        folded += 1;
    }
    if folded > 0 {
        // Re-read (node_count/directory_head advanced per insert) and drain.
        let mut metadata = read_metadata_from_index_handle(handle)?;
        metadata.delta_buffer_head = ItemPointer::INVALID;
        overwrite_metadata_page_handle(handle, &metadata);
    }
    Ok(folded)
}

/// Append a node record on a freshly-extended page, WAL-logged. Returns its TID.
fn append_node_record(handle: RelationHandle, payload: Vec<u8>) -> Result<ItemPointer, String> {
    let buffer = LockedBufferGuard::read_main_locked_handle(
        handle,
        u32::MAX, // P_NEW
        pg_sys::ReadBufferMode::RBM_ZERO_AND_LOCK,
    )
    .ok_or_else(|| "ec_distann graph insert could not extend the relation".to_owned())?;
    let block_number = buffer.block_number();
    let mut wal_txn = wal::WalTxnScope::start_handle(handle);
    let offset_number = {
        let mut page = wal_txn.register_page(&buffer);
        page.init(0);
        page.add_item(&payload)
            .map_err(|e| format!("ec_distann graph insert node add failed: {e:?}"))?
    };
    wal_txn.finish();
    Ok(ItemPointer { block_number, offset_number })
}

/// Amend a forward neighbor's adjacency to include `new_vec_id`, WAL-logged in
/// place. A free slot → append (edge-preserving). At capacity → full-reprune
/// (`plan_insert_backlink`) from the neighbor's own vantage using head-sample
/// source vectors: the new node earns an incoming edge iff it is more
/// α-diverse-close than an existing edge, which is then evicted. A neighbor
/// whose current edges are not all head-sampled (e.g. a folded neighbor beyond
/// the head-index cap) falls back to append-if-free. Same-length rewrite.
#[allow(clippy::too_many_arguments)]
fn add_backlink(
    handle: RelationHandle,
    neighbor_tid: ItemPointer,
    new_vec_id: u64,
    new_code: &[u8],
    new_source_vector: &[f32],
    source_map: &std::collections::HashMap<u64, Vec<f32>>,
    alpha: f32,
    graph_degree_r: u16,
    code_len: usize,
) -> Result<(), String> {
    let buffer = LockedBufferGuard::read_main_handle(
        handle,
        neighbor_tid.block_number,
        pg_sys::ReadBufferMode::RBM_NORMAL,
        pg_sys::BUFFER_LOCK_EXCLUSIVE as i32,
    )
    .ok_or_else(|| {
        format!("ec_distann backlink: could not open block {}", neighbor_tid.block_number)
    })?;
    let r = usize::from(graph_degree_r);
    let vec_ids_off = distann_node_neighbor_vec_ids_offset(code_len);
    let codes_off = distann_node_neighbor_codes_offset(graph_degree_r, code_len);

    let mut wal_txn = wal::WalTxnScope::start_handle(handle);
    {
        let mut page = wal_txn.register_page(&buffer);
        page.visit_tuple_bytes_mut(neighbor_tid, "ec_distann backlink write", |raw| {
            if raw.first().copied() != Some(DISTANN_NODE_TAG) {
                return Err("ec_distann backlink: target is not a graph-node record".to_owned());
            }
            let neighbor_vec_id = u64::from_le_bytes(
                raw[super::tuple::DISTANN_NODE_VEC_ID_OFFSET
                    ..super::tuple::DISTANN_NODE_VEC_ID_OFFSET + 8]
                    .try_into()
                    .expect("vec_id bytes"),
            );
            let count = usize::from(u16::from_le_bytes(
                raw[DISTANN_NODE_NEIGHBOR_COUNT_OFFSET..DISTANN_NODE_NEIGHBOR_COUNT_OFFSET + 2]
                    .try_into()
                    .expect("neighbor count bytes"),
            ));
            // Read the current adjacency (vec_id + code per used slot).
            let read_slot = |raw: &[u8], slot: usize| -> (u64, Vec<u8>) {
                let voff = vec_ids_off + slot * 8;
                let vid = u64::from_le_bytes(raw[voff..voff + 8].try_into().expect("vid"));
                let coff = codes_off + slot * code_len;
                (vid, raw[coff..coff + code_len].to_vec())
            };
            // Idempotence.
            for slot in 0..count {
                if read_slot(raw, slot).0 == new_vec_id {
                    return Ok(());
                }
            }

            let write_adjacency = |raw: &mut [u8], edges: &[(u64, Vec<u8>)]| {
                for (slot, (vid, code)) in edges.iter().enumerate() {
                    let voff = vec_ids_off + slot * 8;
                    raw[voff..voff + 8].copy_from_slice(&vid.to_le_bytes());
                    let coff = codes_off + slot * code_len;
                    raw[coff..coff + code_len].copy_from_slice(code);
                }
                let new_count = u16::try_from(edges.len()).expect("edge count fits u16");
                raw[DISTANN_NODE_NEIGHBOR_COUNT_OFFSET..DISTANN_NODE_NEIGHBOR_COUNT_OFFSET + 2]
                    .copy_from_slice(&new_count.to_le_bytes());
            };

            if count < r {
                // Free slot: append.
                let mut edges: Vec<(u64, Vec<u8>)> =
                    (0..count).map(|s| read_slot(raw, s)).collect();
                edges.push((new_vec_id, new_code.to_vec()));
                write_adjacency(raw, &edges);
                return Ok(());
            }

            // Full: reprune from the neighbor's vantage, if we have the source
            // vectors (neighbor + all its current edges must be head-sampled).
            let Some(neighbor_vec) = source_map.get(&neighbor_vec_id) else {
                return Ok(()); // can't reprune without the vantage vector
            };
            let current: Vec<(u64, Vec<u8>)> = (0..count).map(|s| read_slot(raw, s)).collect();
            let mut current_candidates = Vec::with_capacity(count);
            for (vid, _) in &current {
                match source_map.get(vid) {
                    Some(v) => current_candidates.push(DistannForwardCandidate {
                        vec_id: *vid,
                        source_vector: v.clone(),
                    }),
                    None => return Ok(()), // an edge isn't sampled → skip reprune
                }
            }
            let target = DistannBacklinkTarget {
                vec_id: neighbor_vec_id,
                source_vector: neighbor_vec.clone(),
                current_neighbors: current_candidates,
            };
            let kept = plan_insert_backlink(&target, new_vec_id, new_source_vector, alpha, r)
                .map_err(|e| format!("ec_distann backlink reprune: {e}"))?;
            if !kept.contains(&new_vec_id) {
                return Ok(()); // new node not diverse-close enough; leave as-is
            }
            // Rebuild the adjacency for the kept set, carrying each edge's code
            // (existing codes from the record; the new node's from new_code).
            let edges: Vec<(u64, Vec<u8>)> = kept
                .iter()
                .map(|vid| {
                    if *vid == new_vec_id {
                        (*vid, new_code.to_vec())
                    } else {
                        let code = current
                            .iter()
                            .find(|(cid, _)| cid == vid)
                            .map(|(_, c)| c.clone())
                            .expect("kept edge came from current adjacency");
                        (*vid, code)
                    }
                })
                .collect();
            // Zero any now-unused trailing slots so stale bytes never read back.
            write_adjacency(raw, &edges);
            for slot in edges.len()..count {
                let voff = vec_ids_off + slot * 8;
                raw[voff..voff + 8].copy_from_slice(&0_u64.to_le_bytes());
                let coff = codes_off + slot * code_len;
                for b in &mut raw[coff..coff + code_len] {
                    *b = 0;
                }
            }
            Ok(())
        })?;
    }
    wal_txn.finish();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(vec_id: u64, v: &[f32]) -> DistannForwardCandidate {
        DistannForwardCandidate {
            vec_id,
            source_vector: v.to_vec(),
        }
    }

    #[test]
    fn select_respects_degree_bound_and_returns_valid_vec_ids() {
        let source = [1.0_f32, 0.0, 0.0];
        let cands = vec![
            candidate(10, &[0.99, 0.01, 0.0]),
            candidate(20, &[0.9, 0.1, 0.0]),
            candidate(30, &[0.0, 1.0, 0.0]),
            candidate(40, &[0.0, 0.0, 1.0]),
            candidate(50, &[-1.0, 0.0, 0.0]),
        ];
        let kept = select_insert_forward_neighbors(&source, &cands, 1.2, 3).unwrap();
        assert!(kept.len() <= 3, "degree bound respected");
        assert!(!kept.is_empty(), "at least one neighbor selected");
        let valid: std::collections::HashSet<u64> = cands.iter().map(|c| c.vec_id).collect();
        for id in &kept {
            assert!(valid.contains(id), "kept vec_id {id} is a real candidate");
        }
        // The nearest candidate (10) must be selected.
        assert!(kept.contains(&10), "nearest candidate kept");
        // No duplicates.
        let uniq: std::collections::HashSet<u64> = kept.iter().copied().collect();
        assert_eq!(uniq.len(), kept.len(), "no duplicate edges");
    }

    #[test]
    fn rank_candidates_keeps_closest() {
        let new_vec = [1.0_f32, 0.0, 0.0];
        let samples = vec![
            (10_u64, vec![0.99, 0.01, 0.0]), // closest
            (20, vec![0.0, 1.0, 0.0]),
            (30, vec![0.9, 0.1, 0.0]),   // 2nd closest
            (40, vec![-1.0, 0.0, 0.0]),  // farthest
        ];
        let ranked = rank_insert_candidates(&new_vec, &samples, 2).unwrap();
        assert_eq!(ranked.len(), 2, "limit respected");
        assert_eq!(ranked[0].vec_id, 10, "closest first");
        assert_eq!(ranked[1].vec_id, 30, "2nd closest second");
        // Feeding into forward selection yields valid, degree-bounded edges.
        let fwd = select_insert_forward_neighbors(&new_vec, &ranked, 1.2, 4).unwrap();
        assert!(fwd.iter().all(|id| [10, 30].contains(id)));

        assert!(
            rank_insert_candidates(&[], &samples, 2).is_err(),
            "empty vector rejected"
        );
        let bad = vec![(1_u64, vec![1.0, 0.0])];
        assert!(
            rank_insert_candidates(&new_vec, &bad, 2).is_err(),
            "dim mismatch rejected"
        );
    }

    #[test]
    fn build_node_tuple_round_trips_and_pads() {
        let r: u16 = 4;
        let code_len = 8;
        let fwd = vec![
            DistannInsertNeighbor { vec_id: 111, code: vec![1_u8; code_len] },
            DistannInsertNeighbor { vec_id: 222, code: vec![2_u8; code_len] },
        ];
        let node = build_insert_node_tuple(
            999,
            ItemPointer { block_number: 3, offset_number: 7 },
            vec![9_u8; code_len],
            &fwd,
            r,
            code_len,
        )
        .unwrap();
        assert_eq!(node.vec_id, 999);
        assert_eq!(node.neighbor_count, 2, "count = real neighbors, not padding");
        assert_eq!(node.neighbor_vec_ids.len(), r as usize, "fixed R slots");
        assert_eq!(node.neighbor_vec_ids[0], 111);
        assert_eq!(node.neighbor_vec_ids[2], 0, "slots past count are zero-padded");
        assert!(!node.tombstoned);
        // Round-trips through the on-disk encoding at fixed length.
        let encoded = node.encode(r, code_len).unwrap();
        assert_eq!(encoded.len(), DistannNodeTuple::encoded_len(r, code_len));
        assert_eq!(DistannNodeTuple::decode(&encoded, r, code_len).unwrap(), node);
    }

    #[test]
    fn build_node_tuple_rejects_bad_shape() {
        let code_len = 8;
        // Too many neighbors for R.
        let many: Vec<_> = (0..5)
            .map(|i| DistannInsertNeighbor { vec_id: i, code: vec![0_u8; code_len] })
            .collect();
        assert!(
            build_insert_node_tuple(1, ItemPointer::INVALID, vec![0; code_len], &many, 4, code_len)
                .is_err(),
            "neighbors > R rejected"
        );
        // Wrong search_code length.
        assert!(
            build_insert_node_tuple(1, ItemPointer::INVALID, vec![0; 4], &[], 4, code_len).is_err(),
            "search_code len mismatch rejected"
        );
        // Wrong neighbor code length.
        let bad = vec![DistannInsertNeighbor { vec_id: 1, code: vec![0_u8; 4] }];
        assert!(
            build_insert_node_tuple(1, ItemPointer::INVALID, vec![0; code_len], &bad, 4, code_len)
                .is_err(),
            "neighbor code len mismatch rejected"
        );
    }

    #[test]
    fn backlink_appends_when_free() {
        // Neighbor 100 has 2 of max 4 edges; the new node 999 is appended.
        let target = DistannBacklinkTarget {
            vec_id: 100,
            source_vector: vec![1.0, 0.0, 0.0],
            current_neighbors: vec![
                candidate(10, &[0.9, 0.1, 0.0]),
                candidate(20, &[0.8, 0.2, 0.0]),
            ],
        };
        let kept = plan_insert_backlink(&target, 999, &[0.95, 0.05, 0.0], 1.2, 4).unwrap();
        assert_eq!(kept, vec![10, 20, 999], "free slot -> append, edges preserved");
    }

    #[test]
    fn backlink_reprunes_when_full() {
        // Neighbor 100 is at capacity (3/3); adding 999 re-prunes the union to 3.
        let target = DistannBacklinkTarget {
            vec_id: 100,
            source_vector: vec![1.0, 0.0, 0.0],
            current_neighbors: vec![
                candidate(10, &[0.99, 0.01, 0.0]),
                candidate(20, &[0.0, 1.0, 0.0]),
                candidate(30, &[-1.0, 0.0, 0.0]),
            ],
        };
        let kept = plan_insert_backlink(&target, 999, &[0.98, 0.02, 0.0], 1.2, 3).unwrap();
        assert!(kept.len() <= 3, "degree bound respected after re-prune");
        let uniq: std::collections::HashSet<u64> = kept.iter().copied().collect();
        assert_eq!(uniq.len(), kept.len(), "no duplicate edges");
    }

    #[test]
    fn backlink_idempotent_when_already_linked() {
        let target = DistannBacklinkTarget {
            vec_id: 100,
            source_vector: vec![1.0, 0.0, 0.0],
            current_neighbors: vec![candidate(999, &[0.9, 0.1, 0.0]), candidate(10, &[0.8, 0.2, 0.0])],
        };
        let kept = plan_insert_backlink(&target, 999, &[0.9, 0.1, 0.0], 1.2, 4).unwrap();
        assert_eq!(kept, vec![999, 10], "already-linked -> unchanged (idempotent)");
    }

    #[test]
    fn select_rejects_bad_input() {
        let source = [1.0_f32, 0.0];
        assert!(select_insert_forward_neighbors(&[], &[], 1.2, 3).is_err(), "empty source");
        assert!(
            select_insert_forward_neighbors(&source, &[], 0.5, 3).is_err(),
            "alpha < 1.0"
        );
        assert!(
            select_insert_forward_neighbors(&source, &[], 1.2, 0).is_err(),
            "max_degree 0"
        );
        assert_eq!(
            select_insert_forward_neighbors(&source, &[], 1.2, 3).unwrap(),
            Vec::<u64>::new(),
            "no candidates -> no neighbors"
        );
        // Dimension mismatch.
        let bad = vec![candidate(1, &[1.0, 0.0, 0.0])];
        assert!(
            select_insert_forward_neighbors(&source, &bad, 1.2, 3).is_err(),
            "dimension mismatch rejected"
        );
    }
}
