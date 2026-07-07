//! FR-081 query orchestration for `ec_distann`: head-index descent followed
//! by at most H batched hop rounds, each expanding the best BW unvisited
//! frontier candidates through the expansion seam.
//!
//! # The frozen expansion seam (M0 design output)
//!
//! [`DistannNodeExpander::expand_nodes`] mirrors the FR-079 wire contract of
//! `ec_distann_expand_nodes(index, epoch_fingerprint, query, vec_ids,
//! code_threshold)` exactly: request = a batch of vec_ids (≤ BW) plus an
//! optional code-score floor; response = one entry per requested vec_id, in
//! request order, carrying `(vec_id, exact_dist, is_tombstone,
//! neighbor_vec_ids, neighbor_code_dists)`. The M2 remote form groups the
//! batch by owning node (FR-078) and issues one pooled SQL call per node;
//! this loop does not change. `heap_tid` on the response entry is a
//! LOCAL-ONLY materialization convenience — it is deliberately NOT part of
//! the wire contract (remote materialization resolves rows through the
//! FR-078 co-placement machinery instead).
//!
//! Loop invariants (FR-081): results come only from expanded records; a
//! vec_id is never expanded twice; BW×H is the hard expansion cap
//! (NFR-019); beam exhaustion before k results is a complete result; the
//! convergence early-exit stops when the best unvisited code distance
//! cannot improve the current kth exact distance.

use std::collections::HashSet;

use crate::storage::page::ItemPointer;

/// One expanded record, mirroring an `ec_distann_expand_nodes` response row
/// (plus the local-only `heap_tid`; see the module docs).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DistannExpandedNode {
    pub(crate) vec_id: u64,
    /// `-ip(query, co-placed full-precision vector)`; `None` only for
    /// tombstones, whose vector read MAY be skipped (FR-079).
    pub(crate) exact_dist: Option<f32>,
    pub(crate) is_tombstone: bool,
    /// Local-only materialization handle; NOT in the FR-079 wire contract.
    pub(crate) heap_tid: ItemPointer,
    pub(crate) neighbor_vec_ids: Vec<u64>,
    /// Code-approximated `-ip` per neighbor, index-aligned with
    /// `neighbor_vec_ids` (embedded-code scoring, FR-076).
    pub(crate) neighbor_code_dists: Vec<f32>,
}

/// The frozen local/remote expansion seam (FR-079 shape; see module docs).
pub(crate) trait DistannNodeExpander {
    /// Expand `vec_ids` (≤ BW): response entries preserve request order and
    /// cover every requested vec_id (FR-079-AC-1). A vec_id that cannot be
    /// resolved is an error (placement/structural fault), never a silent
    /// omission.
    fn expand_nodes(
        &mut self,
        vec_ids: &[u64],
        code_threshold: Option<f32>,
    ) -> Result<Vec<DistannExpandedNode>, String>;
}

/// Head-index descent output seeding the hop-round frontier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DistannSeedCandidate {
    pub(crate) vec_id: u64,
    /// `-ip` over the head sample's full-precision vector.
    pub(crate) dist: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DistannOrchestrationParams {
    pub(crate) beam_width: usize,
    pub(crate) hop_rounds: usize,
    pub(crate) top_k: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct DistannScanCounters {
    pub(crate) rounds_executed: usize,
    pub(crate) records_expanded: usize,
    pub(crate) neighbors_code_scored: usize,
    pub(crate) early_exit: bool,
    pub(crate) beam_exhausted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DistannScanHit {
    pub(crate) vec_id: u64,
    pub(crate) heap_tid: ItemPointer,
    pub(crate) exact_dist: f32,
}

/// Eager FR-081 orchestration: runs to completion at rescan; `amgettuple`
/// is a cursor over the returned hits (ADR-056 pattern).
pub(crate) fn distann_orchestrated_search<E: DistannNodeExpander>(
    seeds: &[DistannSeedCandidate],
    expander: &mut E,
    params: DistannOrchestrationParams,
) -> Result<(Vec<DistannScanHit>, DistannScanCounters), String> {
    let mut counters = DistannScanCounters::default();
    if params.beam_width == 0 || params.hop_rounds == 0 {
        return Err("ec_distann scan requires beam_width >= 1 and hop_rounds >= 1".to_owned());
    }

    // Beam pool ordered by code distance; `enqueued` dedupes by vec_id
    // (FR-081: visited-set dedupe is by vec_id, expansion at most once).
    let mut beam: Vec<(f32, u64)> = Vec::with_capacity(
        seeds.len() + params.beam_width * params.hop_rounds * 8,
    );
    let mut enqueued: HashSet<u64> = HashSet::with_capacity(beam.capacity());
    for seed in seeds {
        if enqueued.insert(seed.vec_id) {
            beam.push((seed.dist, seed.vec_id));
        }
    }
    let mut expanded: HashSet<u64> = HashSet::new();
    let mut hits: Vec<DistannScanHit> = Vec::new();

    let mut batch: Vec<u64> = Vec::with_capacity(params.beam_width);
    for _ in 0..params.hop_rounds {
        // Best BW unvisited candidates by code distance.
        beam.sort_unstable_by(|left, right| left.0.total_cmp(&right.0));
        batch.clear();
        let mut best_unvisited_dist = None;
        for (dist, vec_id) in beam.iter() {
            if expanded.contains(vec_id) {
                continue;
            }
            if best_unvisited_dist.is_none() {
                best_unvisited_dist = Some(*dist);
            }
            batch.push(*vec_id);
            if batch.len() >= params.beam_width {
                break;
            }
        }
        if batch.is_empty() {
            counters.beam_exhausted = true;
            break;
        }

        // Convergence early-exit: the best unvisited code distance cannot
        // improve the current kth exact distance (D9).
        if hits.len() >= params.top_k {
            let kth = kth_exact_dist(&mut hits, params.top_k);
            if best_unvisited_dist.expect("non-empty batch has a best") >= kth {
                counters.early_exit = true;
                break;
            }
        }

        let responses = expander.expand_nodes(&batch, None)?;
        if responses.len() != batch.len() {
            return Err(format!(
                "ec_distann expansion returned {} entries for {} requested vec_ids",
                responses.len(),
                batch.len()
            ));
        }
        counters.rounds_executed += 1;
        for (requested, response) in batch.iter().zip(responses.iter()) {
            if response.vec_id != *requested {
                return Err(format!(
                    "ec_distann expansion order violation: requested vec_id {requested:#x}, got {:#x}",
                    response.vec_id
                ));
            }
            expanded.insert(response.vec_id);
            counters.records_expanded += 1;

            if !response.is_tombstone {
                let exact_dist = response.exact_dist.ok_or_else(|| {
                    format!(
                        "ec_distann expansion returned no exact distance for live vec_id {:#x}",
                        response.vec_id
                    )
                })?;
                hits.push(DistannScanHit {
                    vec_id: response.vec_id,
                    heap_tid: response.heap_tid,
                    exact_dist,
                });
            }

            if response.neighbor_vec_ids.len() != response.neighbor_code_dists.len() {
                return Err(
                    "ec_distann expansion neighbor arrays are not index-aligned".to_owned()
                );
            }
            counters.neighbors_code_scored += response.neighbor_vec_ids.len();
            for (neighbor_vec_id, code_dist) in response
                .neighbor_vec_ids
                .iter()
                .zip(response.neighbor_code_dists.iter())
            {
                if enqueued.insert(*neighbor_vec_id) {
                    beam.push((*code_dist, *neighbor_vec_id));
                }
            }
        }
    }

    debug_assert!(
        counters.records_expanded <= params.beam_width * params.hop_rounds,
        "BW x H expansion cap violated"
    );
    hits.sort_unstable_by(|left, right| left.exact_dist.total_cmp(&right.exact_dist));
    Ok((hits, counters))
}

fn kth_exact_dist(hits: &mut [DistannScanHit], top_k: usize) -> f32 {
    debug_assert!(hits.len() >= top_k && top_k >= 1);
    let (_, kth, _) = hits.select_nth_unstable_by(top_k - 1, |left, right| {
        left.exact_dist.total_cmp(&right.exact_dist)
    });
    kth.exact_dist
}

#[cfg(test)]
mod tests {
    use super::{
        distann_orchestrated_search, DistannExpandedNode, DistannNodeExpander,
        DistannOrchestrationParams, DistannSeedCandidate,
    };
    use crate::storage::page::ItemPointer;
    use std::collections::HashMap;

    fn tid(offset: u16) -> ItemPointer {
        ItemPointer {
            block_number: 1,
            offset_number: offset,
        }
    }

    /// Mock graph: node -> (exact_dist, tombstone, neighbors[(id, code_dist)]).
    struct MockExpander {
        nodes: HashMap<u64, (f32, bool, Vec<(u64, f32)>)>,
        calls: Vec<Vec<u64>>,
    }

    impl DistannNodeExpander for MockExpander {
        fn expand_nodes(
            &mut self,
            vec_ids: &[u64],
            _code_threshold: Option<f32>,
        ) -> Result<Vec<DistannExpandedNode>, String> {
            self.calls.push(vec_ids.to_vec());
            vec_ids
                .iter()
                .map(|vec_id| {
                    let (exact, tombstone, neighbors) = self
                        .nodes
                        .get(vec_id)
                        .ok_or_else(|| format!("unknown vec_id {vec_id}"))?;
                    Ok(DistannExpandedNode {
                        vec_id: *vec_id,
                        exact_dist: (!tombstone).then_some(*exact),
                        is_tombstone: *tombstone,
                        heap_tid: tid(*vec_id as u16),
                        neighbor_vec_ids: neighbors.iter().map(|(id, _)| *id).collect(),
                        neighbor_code_dists: neighbors.iter().map(|(_, d)| *d).collect(),
                    })
                })
                .collect()
        }
    }

    fn params(bw: usize, h: usize, k: usize) -> DistannOrchestrationParams {
        DistannOrchestrationParams {
            beam_width: bw,
            hop_rounds: h,
            top_k: k,
        }
    }

    #[test]
    fn distann_orchestration_expands_no_vec_id_twice_and_respects_cap() {
        // Cycle 1 <-> 2 with self-loops: without the visited set the loop
        // would re-expand forever (FR-081-AC-2/AC-3).
        let mut expander = MockExpander {
            nodes: HashMap::from([
                (1, (-0.9, false, vec![(2, -0.8), (1, -0.9)])),
                (2, (-0.8, false, vec![(1, -0.9), (2, -0.8)])),
            ]),
            calls: Vec::new(),
        };
        let seeds = [DistannSeedCandidate {
            vec_id: 1,
            dist: -0.9,
        }];
        let (hits, counters) =
            distann_orchestrated_search(&seeds, &mut expander, params(2, 8, 10))
                .expect("search should succeed");
        assert_eq!(counters.records_expanded, 2);
        assert!(counters.records_expanded <= 2 * 8);
        assert!(counters.beam_exhausted, "cycle exhausts the beam");
        let mut all_expanded: Vec<u64> = expander.calls.concat();
        all_expanded.sort_unstable();
        all_expanded.dedup();
        assert_eq!(all_expanded, vec![1, 2], "no vec_id expanded twice");
        assert_eq!(hits.len(), 2);
        assert!(hits[0].exact_dist <= hits[1].exact_dist);
    }

    #[test]
    fn distann_orchestration_returns_sub_k_on_exhaustion_as_complete() {
        let mut expander = MockExpander {
            nodes: HashMap::from([(1, (-0.5, false, vec![]))]),
            calls: Vec::new(),
        };
        let seeds = [DistannSeedCandidate {
            vec_id: 1,
            dist: -0.5,
        }];
        let (hits, counters) =
            distann_orchestrated_search(&seeds, &mut expander, params(4, 8, 10))
                .expect("search should succeed");
        assert_eq!(hits.len(), 1, "fewer-than-k is a complete result");
        assert!(counters.beam_exhausted);
        assert!(!counters.early_exit);
    }

    #[test]
    fn distann_orchestration_excludes_tombstones_but_traverses_their_edges() {
        // 1 (tombstone) -> 2 (live): the tombstone must not appear in
        // results but its adjacency must remain usable (FR-076-AC-4).
        let mut expander = MockExpander {
            nodes: HashMap::from([
                (1, (0.0, true, vec![(2, -0.7)])),
                (2, (-0.7, false, vec![])),
            ]),
            calls: Vec::new(),
        };
        let seeds = [DistannSeedCandidate {
            vec_id: 1,
            dist: -0.9,
        }];
        let (hits, _) = distann_orchestrated_search(&seeds, &mut expander, params(2, 4, 10))
            .expect("search should succeed");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].vec_id, 2);
    }

    #[test]
    fn distann_orchestration_early_exits_when_beam_cannot_improve_kth() {
        // k=1 satisfied by node 1 (exact -0.9); remaining frontier code
        // dist (-0.1) cannot improve it -> early exit before expanding 2.
        let mut expander = MockExpander {
            nodes: HashMap::from([
                (1, (-0.9, false, vec![(2, -0.1)])),
                (2, (-0.1, false, vec![])),
            ]),
            calls: Vec::new(),
        };
        let seeds = [DistannSeedCandidate {
            vec_id: 1,
            dist: -0.9,
        }];
        let (hits, counters) =
            distann_orchestrated_search(&seeds, &mut expander, params(1, 8, 1))
                .expect("search should succeed");
        assert!(counters.early_exit);
        assert_eq!(counters.records_expanded, 1);
        assert_eq!(hits[0].vec_id, 1);
    }

    #[test]
    fn distann_orchestration_hop_rounds_cap_bounds_expansions() {
        // Long chain 1 -> 2 -> 3 -> ...: H=2, BW=1 must stop at 2 records.
        let mut nodes = HashMap::new();
        for id in 1..=10_u64 {
            nodes.insert(
                id,
                (
                    -1.0 + id as f32 * 0.01,
                    false,
                    vec![(id + 1, -1.0 + (id + 1) as f32 * 0.01)],
                ),
            );
        }
        nodes.insert(11, (-0.5, false, vec![]));
        let mut expander = MockExpander {
            nodes,
            calls: Vec::new(),
        };
        let seeds = [DistannSeedCandidate {
            vec_id: 1,
            dist: -1.0,
        }];
        let (_, counters) = distann_orchestrated_search(&seeds, &mut expander, params(1, 2, 100))
            .expect("search should succeed");
        assert_eq!(counters.rounds_executed, 2);
        assert_eq!(counters.records_expanded, 2, "BW x H = 2 hard cap");
    }
}
