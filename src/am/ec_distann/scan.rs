//! FR-081 query orchestration for `ec_distann`: head-index descent followed
//! by at most H batched hop rounds, each expanding the best BW unvisited
//! frontier candidates through the expansion seam.
//!
//! # The frozen expansion seam (M0 design output)
//!
//! [`DistannNodeExpander::expand_nodes`] mirrors the FR-079 wire contract of
//! `ec_distann_expand_nodes(index, epoch_fingerprint, query, vec_ids,
//! code_threshold, candidate_limit)` exactly: request = a batch of vec_ids (≤
//! BW), an optional code-score floor, and an optional owner-side candidate
//! limit; response = one entry per requested vec_id, in request order, carrying
//! `(vec_id, exact_dist, is_tombstone, neighbor_vec_ids,
//! neighbor_code_dists)`. The M2 remote form groups the
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

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

use crate::storage::page::ItemPointer;

use super::expand_error::DistannExpandError;

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
    /// Owner-side timing returned by the physical expansion endpoint. Zero
    /// for coordinator-local expansions and test doubles.
    pub(crate) owner_total_ns: i64,
    pub(crate) owner_open_validate_ns: i64,
    pub(crate) owner_graph_read_ns: i64,
    pub(crate) owner_score_ns: i64,
    pub(crate) owner_response_encode_ns: i64,
    pub(crate) owner_response_bytes: i64,
    pub(crate) coordinator_rpc_ns: i64,
    pub(crate) coordinator_decode_ns: i64,
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
        candidate_limit: Option<usize>,
    ) -> Result<Vec<DistannExpandedNode>, DistannExpandError>;
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
    /// L, the maximum retained unexpanded candidate heap used to derive the
    /// owner-side threshold. It is separate from BW x H.
    pub(crate) candidate_heap_limit: usize,
    pub(crate) hop_rounds: usize,
    pub(crate) top_k: usize,
    /// NFR-020 fault injection: fail at the start of this 0-based hop round
    /// (after earlier rounds executed). `None` disables injection.
    pub(crate) debug_fail_hop_round: Option<usize>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct DistannScanCounters {
    pub(crate) rounds_executed: usize,
    pub(crate) records_expanded: usize,
    pub(crate) neighbors_code_scored: usize,
    pub(crate) pushdown_rounds_with_threshold: usize,
    pub(crate) early_exit: bool,
    pub(crate) beam_exhausted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DistannScanHit {
    pub(crate) vec_id: u64,
    pub(crate) heap_tid: ItemPointer,
    pub(crate) exact_dist: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BeamCandidate {
    dist: f32,
    vec_id: u64,
}

/// The coordinator-derived Algorithm 1 controls for one hop round. `threshold`
/// is expressed as an IP-score floor, while the beam stores `-ip` distances.
#[derive(Debug, Clone, Copy, PartialEq)]
struct PushdownLimits {
    threshold: Option<f32>,
    candidate_limit: usize,
}

fn derive_pushdown_limits(
    beam: &BinaryHeap<BeamCandidate>,
    expanded: &HashSet<u64>,
    candidate_heap_limit: usize,
    beam_width: usize,
) -> PushdownLimits {
    // Derive the threshold from the L-th best live retained candidate, not
    // from the total future expansion budget. This is the bounded-heap form
    // of Algorithm 1.
    let mut live = beam
        .iter()
        .filter(|candidate| !expanded.contains(&candidate.vec_id))
        .copied()
        .collect::<Vec<_>>();
    live.sort_unstable_by(|left, right| {
        left.dist
            .total_cmp(&right.dist)
            .then_with(|| left.vec_id.cmp(&right.vec_id))
    });
    let threshold = if candidate_heap_limit > 0 && live.len() >= candidate_heap_limit {
        live.get(candidate_heap_limit - 1).map(|candidate| -candidate.dist)
    } else {
        None
    };
    // l is per expanded response. Keeping l near L/BW prevents one owner
    // response from receiving the full remaining BW x H expansion budget.
    let candidate_limit = candidate_heap_limit
        .saturating_add(beam_width.saturating_sub(1))
        .checked_div(beam_width.max(1))
        .unwrap_or(1)
        .max(1);
    PushdownLimits {
        threshold,
        candidate_limit,
    }
}

fn retain_best_candidates(beam: &mut BinaryHeap<BeamCandidate>, limit: usize) {
    if beam.len() <= limit {
        return;
    }
    let mut candidates = beam.drain().collect::<Vec<_>>();
    candidates.sort_unstable_by(|left, right| {
        left.dist
            .total_cmp(&right.dist)
            .then_with(|| left.vec_id.cmp(&right.vec_id))
    });
    candidates.truncate(limit);
    beam.extend(candidates);
}

/// Apply the owner-side Algorithm 1 filter and deterministic top-`l` window.
/// Ties at the threshold are retained before the limit is applied, and the
/// vec_id tie-break makes the wire response deterministic.
pub(crate) fn prune_and_limit_neighbors(
    neighbor_vec_ids: &[u64],
    neighbor_code_dists: &[f32],
    code_threshold: Option<f32>,
    candidate_limit: Option<usize>,
) -> Result<(Vec<u64>, Vec<f32>), DistannExpandError> {
    if neighbor_vec_ids.len() != neighbor_code_dists.len() {
        return Err(DistannExpandError::Internal(
            "ec_distann expansion neighbor arrays are not index-aligned".to_owned(),
        ));
    }
    let mut neighbors = neighbor_vec_ids
        .iter()
        .copied()
        .zip(neighbor_code_dists.iter().copied())
        .filter(|(_, code_dist)| code_threshold.map_or(true, |threshold| -*code_dist >= threshold))
        .collect::<Vec<_>>();
    if candidate_limit.is_some() {
        neighbors.sort_unstable_by(|left, right| {
            left.1
                .total_cmp(&right.1)
                .then_with(|| left.0.cmp(&right.0))
        });
        neighbors.truncate(candidate_limit.unwrap_or(0));
    }
    #[cfg(feature = "distann-head-attribution-benchmark")]
    super::stage_counters::record_work(
        super::stage_counters::DistannMaterializationWork::NeighborsPruned,
        neighbor_vec_ids.len().saturating_sub(neighbors.len()),
    );
    let (ids, dists): (Vec<_>, Vec<_>) = neighbors.into_iter().unzip();
    Ok((ids, dists))
}

impl Eq for BeamCandidate {}

impl Ord for BeamCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max-heap. Reverse the total distance order so the
        // best (smallest) code distance is popped first, with a stable vec-id
        // tie-break that makes equal-distance traversal deterministic.
        other
            .dist
            .total_cmp(&self.dist)
            .then_with(|| other.vec_id.cmp(&self.vec_id))
    }
}

impl PartialOrd for BeamCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Eager FR-081 orchestration: runs to completion at rescan; `amgettuple`
/// is a cursor over the returned hits (ADR-056 pattern).
pub(crate) fn distann_orchestrated_search<E: DistannNodeExpander>(
    seeds: &[DistannSeedCandidate],
    expander: &mut E,
    params: DistannOrchestrationParams,
) -> Result<(Vec<DistannScanHit>, DistannScanCounters), DistannExpandError> {
    distann_orchestrated_search_with_pushdown(seeds, expander, params, true)
}

fn distann_orchestrated_search_with_pushdown<E: DistannNodeExpander>(
    seeds: &[DistannSeedCandidate],
    expander: &mut E,
    params: DistannOrchestrationParams,
    pushdown: bool,
) -> Result<(Vec<DistannScanHit>, DistannScanCounters), DistannExpandError> {
    let mut counters = DistannScanCounters::default();
    if params.beam_width == 0 || params.hop_rounds == 0 {
        return Err(DistannExpandError::BadInput(
            "ec_distann scan requires beam_width >= 1 and hop_rounds >= 1".to_owned(),
        ));
    }
    if params.candidate_heap_limit == 0 {
        return Err(DistannExpandError::BadInput(
            "ec_distann scan requires candidate_heap_limit >= 1".to_owned(),
        ));
    }

    // Beam pool ordered by code distance; `enqueued` dedupes by vec_id
    // (FR-081: visited-set dedupe is by vec_id, expansion at most once).
    let beam_capacity = seeds.len() + params.beam_width * params.hop_rounds * 8;
    let mut beam = BinaryHeap::with_capacity(beam_capacity);
    let mut enqueued: HashSet<u64> = HashSet::with_capacity(beam_capacity);
    for seed in seeds {
        if enqueued.insert(seed.vec_id) {
            beam.push(BeamCandidate {
                dist: seed.dist,
                vec_id: seed.vec_id,
            });
        }
    }
    if pushdown {
        retain_best_candidates(&mut beam, params.candidate_heap_limit);
    }
    let mut expanded: HashSet<u64> = HashSet::new();
    let mut hits: Vec<DistannScanHit> = Vec::new();

    let mut batch: Vec<u64> = Vec::with_capacity(params.beam_width);
    for _round in 0..params.hop_rounds {
        let limits = if pushdown {
            derive_pushdown_limits(
                &beam,
                &expanded,
                params.candidate_heap_limit,
                params.beam_width,
            )
        } else {
            PushdownLimits {
                threshold: None,
                candidate_limit: 0,
            }
        };
        // Pop the best BW unvisited candidates by code distance. Maintaining
        // the heap avoids re-sorting the entire accumulated frontier on every
        // hop round.
        batch.clear();
        let mut best_unvisited = None;
        while let Some(candidate) = beam.pop() {
            if expanded.contains(&candidate.vec_id) {
                continue;
            }
            if best_unvisited.is_none() {
                best_unvisited = Some(candidate);
            }
            batch.push(candidate.vec_id);
            if batch.len() >= params.beam_width {
                break;
            }
        }
        if batch.is_empty() {
            counters.beam_exhausted = true;
            break;
        }

        // Convergence early-exit: the best unvisited code candidate cannot
        // improve the current kth exact result under the same total order used
        // for final output. Comparing vec_id when distances tie is essential:
        // otherwise iterative deepening could serve an equal-distance row with
        // a larger vec_id before discovering the true boundary row.
        if hits.len() >= params.top_k {
            let kth = kth_exact_hit(&mut hits, params.top_k);
            let best = best_unvisited.expect("non-empty batch has a best");
            if best
                .dist
                .total_cmp(&kth.exact_dist)
                .then_with(|| best.vec_id.cmp(&kth.vec_id))
                != Ordering::Less
            {
                counters.early_exit = true;
                break;
            }
        }

        // NFR-020 fault injection: a mid-beam round failure must discard the
        // partial beam and error rather than surface a partial result. Fires
        // after earlier rounds executed (counters.rounds_executed == round idx).
        if params.debug_fail_hop_round == Some(counters.rounds_executed) {
            return Err(DistannExpandError::Internal(format!(
                "ec_distann injected hop-round failure at round {} (ec_distann.debug_fail_hop_round)",
                counters.rounds_executed
            )));
        }

        if limits.threshold.is_some() {
            counters.pushdown_rounds_with_threshold += 1;
            #[cfg(feature = "distann-head-attribution-benchmark")]
            super::stage_counters::record_work(
                super::stage_counters::DistannMaterializationWork::PushdownRoundsWithThreshold,
                1,
            );
        }
        let responses = expander.expand_nodes(
            &batch,
            limits.threshold,
            pushdown.then_some(limits.candidate_limit),
        )?;
        if responses.len() != batch.len() {
            return Err(DistannExpandError::Internal(format!(
                "ec_distann expansion returned {} entries for {} requested vec_ids",
                responses.len(),
                batch.len()
            )));
        }
        counters.rounds_executed += 1;
        #[cfg(feature = "distann-head-attribution-benchmark")]
        {
            super::stage_counters::record_work(
                super::stage_counters::DistannMaterializationWork::TraversalHopRounds,
                1,
            );
            super::stage_counters::record_work(
                super::stage_counters::DistannMaterializationWork::TraversalBatchWidth,
                batch.len(),
            );
            super::stage_counters::record_work(
                super::stage_counters::DistannMaterializationWork::TraversalNodesRequested,
                batch.len(),
            );
            super::stage_counters::record_work(
                super::stage_counters::DistannMaterializationWork::TraversalNodesReturned,
                responses.len(),
            );
        }
        for (requested, response) in batch.iter().zip(responses.iter()) {
            if response.vec_id != *requested {
                return Err(DistannExpandError::Internal(format!(
                    "ec_distann expansion order violation: requested vec_id {requested:#x}, got {:#x}",
                    response.vec_id
                )));
            }
            let inserted = expanded.insert(response.vec_id);
            #[cfg(feature = "distann-head-attribution-benchmark")]
            super::stage_counters::record_work(
                super::stage_counters::DistannMaterializationWork::TraversalRepeatedNodes,
                usize::from(!inserted),
            );
            #[cfg(not(feature = "distann-head-attribution-benchmark"))]
            let _ = inserted;
            counters.records_expanded += 1;

            if !response.is_tombstone {
                let exact_dist = response.exact_dist.ok_or_else(|| {
                    DistannExpandError::Internal(format!(
                        "ec_distann expansion returned no exact distance for live vec_id {:#x}",
                        response.vec_id
                    ))
                })?;
                hits.push(DistannScanHit {
                    vec_id: response.vec_id,
                    heap_tid: response.heap_tid,
                    exact_dist,
                });
            }

            if response.neighbor_vec_ids.len() != response.neighbor_code_dists.len() {
                return Err(DistannExpandError::Internal(
                    "ec_distann expansion neighbor arrays are not index-aligned".to_owned(),
                ));
            }
            counters.neighbors_code_scored += response.neighbor_vec_ids.len();
            for (neighbor_vec_id, code_dist) in response
                .neighbor_vec_ids
                .iter()
                .zip(response.neighbor_code_dists.iter())
            {
                if enqueued.insert(*neighbor_vec_id) {
                    #[cfg(feature = "distann-head-attribution-benchmark")]
                    super::stage_counters::record_work(
                        super::stage_counters::DistannMaterializationWork::TraversalFrontierInsertions,
                        1,
                    );
                    #[cfg(feature = "distann-head-attribution-benchmark")]
                    let frontier_started = std::time::Instant::now();
                    beam.push(BeamCandidate {
                        dist: *code_dist,
                        vec_id: *neighbor_vec_id,
                    });
                    #[cfg(feature = "distann-head-attribution-benchmark")]
                    super::stage_counters::record(
                        super::stage_counters::DistannQueryStage::TraversalFrontierInsert,
                        frontier_started.elapsed(),
                    );
                }
            }
        }
        if pushdown {
            retain_best_candidates(&mut beam, params.candidate_heap_limit);
        }
    }

    debug_assert!(
        counters.records_expanded <= params.beam_width * params.hop_rounds,
        "BW x H expansion cap violated"
    );
    hits.sort_unstable_by(|left, right| {
        left.exact_dist
            .total_cmp(&right.exact_dist)
            .then_with(|| left.vec_id.cmp(&right.vec_id))
    });
    Ok((hits, counters))
}

/// FR-082-AC-2 restart-once: run a coordinator scan attempt; if any hop round
/// fails with a **retriable epoch mismatch**, discard all partial state (the
/// closure rebuilds it), let the caller refresh its epoch view (the closure
/// re-reads it), and restart the whole attempt exactly once. A second epoch
/// mismatch — or any other error — propagates. Non-epoch errors on the first
/// attempt propagate immediately (no restart). NFR-019 per-attempt accounting is
/// reset naturally because each attempt runs the orchestration fresh.
///
/// This is the coordinator half of the epoch-swap consistency contract; the
/// closure's "refresh" reads whatever the active epoch source provides (the
/// roster/epoch GUCs today; the persisted epoch manifest once FR-082's lifecycle
/// lands — see reviews/task-165/014-fr082-lifecycle-design).
pub(crate) fn run_scan_attempt_with_restart<T>(
    mut attempt: impl FnMut() -> Result<T, DistannExpandError>,
) -> Result<T, DistannExpandError> {
    match attempt() {
        Err(DistannExpandError::EpochMismatch(_)) => attempt(),
        other => other,
    }
}

fn kth_exact_hit(hits: &mut [DistannScanHit], top_k: usize) -> DistannScanHit {
    debug_assert!(hits.len() >= top_k && top_k >= 1);
    let (_, kth, _) = hits.select_nth_unstable_by(top_k - 1, |left, right| {
        left.exact_dist
            .total_cmp(&right.exact_dist)
            .then_with(|| left.vec_id.cmp(&right.vec_id))
    });
    *kth
}

#[cfg(test)]
mod tests {
    use super::{
        distann_orchestrated_search, distann_orchestrated_search_with_pushdown,
        prune_and_limit_neighbors, run_scan_attempt_with_restart, DistannExpandError,
        DistannExpandedNode, DistannNodeExpander, DistannOrchestrationParams, DistannScanHit,
        DistannSeedCandidate,
    };
    use crate::storage::page::ItemPointer;
    use std::cell::Cell;
    use std::collections::HashMap;

    #[test]
    fn restart_success_on_first_attempt_runs_once() {
        let calls = Cell::new(0);
        let result = run_scan_attempt_with_restart(|| {
            calls.set(calls.get() + 1);
            Ok::<_, DistannExpandError>(42)
        });
        assert_eq!(result.unwrap(), 42);
        assert_eq!(calls.get(), 1, "no restart when the first attempt succeeds");
    }

    #[test]
    fn restart_once_after_epoch_mismatch_then_succeeds() {
        // FR-082-AC-2: one epoch mismatch triggers exactly one restart, which
        // succeeds under the refreshed epoch.
        let calls = Cell::new(0);
        let result = run_scan_attempt_with_restart(|| {
            calls.set(calls.get() + 1);
            if calls.get() == 1 {
                Err(DistannExpandError::EpochMismatch("stale".to_owned()))
            } else {
                Ok(7)
            }
        });
        assert_eq!(result.unwrap(), 7);
        assert_eq!(calls.get(), 2, "exactly one restart");
    }

    #[test]
    fn restart_second_epoch_mismatch_errors() {
        // A second mismatch fails the query (attempts capped at two).
        let calls = Cell::new(0);
        let result = run_scan_attempt_with_restart(|| {
            calls.set(calls.get() + 1);
            Err::<i32, _>(DistannExpandError::EpochMismatch(format!(
                "mismatch {}",
                calls.get()
            )))
        });
        assert!(matches!(result, Err(DistannExpandError::EpochMismatch(_))));
        assert_eq!(calls.get(), 2, "capped at two attempts");
    }

    #[test]
    fn restart_non_epoch_error_does_not_restart() {
        // A structural/placement error is not retriable — it propagates at once.
        let calls = Cell::new(0);
        let result = run_scan_attempt_with_restart(|| {
            calls.set(calls.get() + 1);
            Err::<i32, _>(DistannExpandError::Placement("not owned".to_owned()))
        });
        assert!(matches!(result, Err(DistannExpandError::Placement(_))));
        assert_eq!(calls.get(), 1, "no restart on a non-epoch error");
    }

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
            code_threshold: Option<f32>,
            candidate_limit: Option<usize>,
        ) -> Result<Vec<DistannExpandedNode>, DistannExpandError> {
            self.calls.push(vec_ids.to_vec());
            vec_ids
                .iter()
                .map(|vec_id| {
                    let (exact, tombstone, neighbors) =
                        self.nodes.get(vec_id).ok_or_else(|| {
                            DistannExpandError::Internal(format!("unknown vec_id {vec_id}"))
                        })?;
                    let neighbor_vec_ids = neighbors.iter().map(|(id, _)| *id).collect::<Vec<_>>();
                    let neighbor_code_dists = neighbors.iter().map(|(_, d)| *d).collect::<Vec<_>>();
                    let (neighbor_vec_ids, neighbor_code_dists) = prune_and_limit_neighbors(
                        &neighbor_vec_ids,
                        &neighbor_code_dists,
                        code_threshold,
                        candidate_limit,
                    )?;
                    Ok(DistannExpandedNode {
                        vec_id: *vec_id,
                        exact_dist: (!tombstone).then_some(*exact),
                        is_tombstone: *tombstone,
                        heap_tid: tid(*vec_id as u16),
                        neighbor_vec_ids,
                        neighbor_code_dists,
                        owner_total_ns: 0,
                        owner_open_validate_ns: 0,
                        owner_graph_read_ns: 0,
                        owner_score_ns: 0,
                        owner_response_encode_ns: 0,
                        owner_response_bytes: 0,
                        coordinator_rpc_ns: 0,
                        coordinator_decode_ns: 0,
                    })
                })
                .collect()
        }
    }

    fn params(bw: usize, h: usize, k: usize) -> DistannOrchestrationParams {
        DistannOrchestrationParams {
            beam_width: bw,
            candidate_heap_limit: bw.saturating_mul(4).max(1),
            hop_rounds: h,
            top_k: k,
            debug_fail_hop_round: None,
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
        let (hits, counters) = distann_orchestrated_search(&seeds, &mut expander, params(2, 8, 10))
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
        let (hits, counters) = distann_orchestrated_search(&seeds, &mut expander, params(4, 8, 10))
            .expect("search should succeed");
        assert_eq!(hits.len(), 1, "fewer-than-k is a complete result");
        assert!(counters.beam_exhausted);
        assert!(!counters.early_exit);
    }

    #[test]
    fn distann_orchestration_orders_equal_exact_scores_by_vec_id() {
        let mut expander = MockExpander {
            nodes: HashMap::from([(9, (-0.5, false, vec![])), (3, (-0.5, false, vec![]))]),
            calls: Vec::new(),
        };
        let seeds = [
            DistannSeedCandidate {
                vec_id: 9,
                dist: -0.5,
            },
            DistannSeedCandidate {
                vec_id: 3,
                dist: -0.5,
            },
        ];
        let (hits, counters) = distann_orchestrated_search(&seeds, &mut expander, params(2, 2, 2))
            .expect("equal-score search should succeed");
        assert!(counters.beam_exhausted);
        assert_eq!(
            hits.iter().map(|hit| hit.vec_id).collect::<Vec<_>>(),
            vec![3, 9]
        );
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
        let (hits, counters) = distann_orchestrated_search(&seeds, &mut expander, params(1, 8, 1))
            .expect("search should succeed");
        assert!(counters.early_exit);
        assert_eq!(counters.records_expanded, 1);
        assert_eq!(hits[0].vec_id, 1);
    }

    #[test]
    fn distann_orchestration_does_not_early_exit_before_equal_score_tie_break() {
        let mut expander = MockExpander {
            nodes: HashMap::from([
                (9, (-0.5, false, vec![(3, -0.5)])),
                (3, (-0.5, false, vec![])),
            ]),
            calls: Vec::new(),
        };
        let seeds = [DistannSeedCandidate {
            vec_id: 9,
            dist: -0.5,
        }];
        let (hits, counters) = distann_orchestrated_search(&seeds, &mut expander, params(1, 4, 1))
            .expect("equal-score boundary search should succeed");
        assert!(!counters.early_exit);
        assert_eq!(counters.records_expanded, 2);
        assert_eq!(
            hits.iter().map(|hit| hit.vec_id).collect::<Vec<_>>(),
            vec![3, 9]
        );
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

    #[test]
    fn owner_pushdown_keeps_threshold_ties_and_deterministic_limit_order() {
        let (ids, dists) = prune_and_limit_neighbors(
            &[40, 10, 30, 20],
            &[-0.5, -0.7, -0.7, -0.2],
            Some(0.7),
            Some(2),
        )
        .expect("aligned neighbor arrays");
        assert_eq!(ids, vec![10, 30]);
        assert_eq!(dists, vec![-0.7, -0.7]);
    }

    #[test]
    fn algorithm1_pushdown_is_order_identical_to_unbounded_path_with_tombstone() {
        // Two owner-like frontier regions, equal-score ties, and a tombstone
        // whose exact distance is absent. The bounded owner responses must
        // produce the same ordered live result IDs as the None/None path.
        let nodes = HashMap::from([
            (1, (-0.90, false, vec![(2, -0.80), (3, -0.80), (4, -0.10)])),
            (2, (0.0, true, vec![(5, -0.75), (6, -0.75)])),
            (3, (-0.70, false, vec![(7, -0.74), (8, -0.20)])),
            (4, (-0.20, false, vec![(9, -0.73)])),
            (5, (-0.60, false, vec![])),
            (6, (-0.59, false, vec![])),
            (7, (-0.58, false, vec![])),
            (8, (-0.10, false, vec![])),
            (9, (-0.05, false, vec![])),
        ]);
        let seeds = [DistannSeedCandidate {
            vec_id: 1,
            dist: -0.90,
        }];
        let mut unbounded = MockExpander {
            nodes: nodes.clone(),
            calls: Vec::new(),
        };
        let mut pushed = MockExpander {
            nodes,
            calls: Vec::new(),
        };
        let params = params(2, 4, 5);
        let (reference, _) =
            distann_orchestrated_search_with_pushdown(&seeds, &mut unbounded, params, false)
                .expect("unbounded reference path");
        let (candidate, _) =
            distann_orchestrated_search(&seeds, &mut pushed, params).expect("Algorithm 1 path");
        assert_eq!(
            reference.iter().map(|hit| hit.vec_id).collect::<Vec<_>>(),
            candidate.iter().map(|hit| hit.vec_id).collect::<Vec<_>>()
        );
        assert_eq!(reference, candidate);
        assert!(pushed.calls.len() <= 4);
    }

    #[test]
    fn bounded_candidate_heap_activates_pushdown_without_changing_top_k_identity() {
        let nodes = HashMap::from([
            (1, (-1.0, false, vec![(2, -0.9), (3, -0.1)])),
            (10, (-0.5, false, vec![])),
            (2, (-0.95, false, vec![])),
            (3, (-0.2, false, vec![])),
        ]);
        let seeds = [
            DistannSeedCandidate { vec_id: 1, dist: -1.0 },
            DistannSeedCandidate { vec_id: 10, dist: -0.5 },
        ];
        let params = DistannOrchestrationParams {
            beam_width: 2,
            candidate_heap_limit: 2,
            hop_rounds: 3,
            top_k: 2,
            debug_fail_hop_round: None,
        };
        let mut reference = MockExpander {
            nodes: nodes.clone(),
            calls: Vec::new(),
        };
        let mut candidate = MockExpander {
            nodes,
            calls: Vec::new(),
        };
        let (reference, _) =
            distann_orchestrated_search_with_pushdown(&seeds, &mut reference, params, false)
                .expect("unbounded reference path");
        let (candidate, counters) =
            distann_orchestrated_search(&seeds, &mut candidate, params).expect("bounded path");
        let top_k = |hits: &[DistannScanHit]| {
            hits.iter()
                .take(params.top_k)
                .map(|hit| hit.vec_id)
                .collect::<Vec<_>>()
        };
        assert_eq!(top_k(&reference), top_k(&candidate));
        assert!(counters.pushdown_rounds_with_threshold > 0);
        let (ids, _) = prune_and_limit_neighbors(
            &[2, 3],
            &[-0.9, -0.1],
            Some(0.5),
            Some(1),
        )
        .expect("aligned neighbor arrays");
        assert_eq!(ids, vec![2], "the active bounded response prunes a neighbor");
    }
}
