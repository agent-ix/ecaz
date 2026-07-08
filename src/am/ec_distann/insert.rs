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

use crate::am::{robust_prune, Candidate};
use crate::storage::page::ItemPointer;

use super::tuple::DistannNodeTuple;

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
