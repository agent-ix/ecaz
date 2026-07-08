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
