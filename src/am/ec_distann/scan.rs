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
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::storage::page::ItemPointer;

use super::expand_error::DistannExpandError;

/// One expanded record, mirroring an `ec_distann_expand_nodes` response row
/// (plus local and benchmark-only materialization handles).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DistannExpandedNode {
    pub(crate) vec_id: u64,
    /// `-ip(query, co-placed full-precision vector)`; `None` only for
    /// tombstones, whose vector read MAY be skipped (FR-079).
    pub(crate) exact_dist: Option<f32>,
    pub(crate) is_tombstone: bool,
    /// Local-only materialization handle; NOT in the FR-079 wire contract.
    pub(crate) heap_tid: ItemPointer,
    /// MAT-22 owner-side locator. This is kept separate from `heap_tid` so a
    /// remote owner's TID is never mistaken for a coordinator-local fetch.
    pub(crate) owner_heap_tid: ItemPointer,
    pub(crate) neighbor_vec_ids: Vec<u64>,
    /// Code-approximated `-ip` per neighbor, index-aligned with
    /// `neighbor_vec_ids` (embedded-code scoring, FR-076).
    pub(crate) neighbor_code_dists: Vec<f32>,
    /// Number of owner-side candidates removed by the threshold or batch L
    /// limit. Remote benchmark attribution records the same quantity at the
    /// owner; this field is used by local orchestration tests and counters.
    pub(crate) neighbors_pruned: usize,
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
pub(crate) struct DistannRoundCounters {
    pub(crate) round: usize,
    pub(crate) requested_nodes: usize,
    pub(crate) expanded_nodes: usize,
    pub(crate) transport_wait_ns: u64,
    pub(crate) straggler_spread_ns: u64,
    pub(crate) request_bytes: usize,
    pub(crate) response_bytes: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DistannScanCounters {
    pub(crate) rounds_executed: usize,
    pub(crate) records_expanded: usize,
    pub(crate) neighbors_code_scored: usize,
    pub(crate) neighbors_pruned: usize,
    pub(crate) pushdown_rounds_with_threshold: usize,
    pub(crate) early_exit: bool,
    pub(crate) beam_exhausted: bool,
    /// Per-hop attribution required by Task 206. This is populated for both
    /// local and physical expanders; local transport fields remain zero.
    pub(crate) rounds: Vec<DistannRoundCounters>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DistannScanHit {
    pub(crate) vec_id: u64,
    pub(crate) heap_tid: ItemPointer,
    pub(crate) owner_heap_tid: ItemPointer,
    pub(crate) exact_dist: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BeamCandidate {
    dist: f32,
    vec_id: u64,
    origin_mask: u32,
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
    PushdownLimits {
        threshold,
        // Algorithm 2 passes the candidate heap size L unchanged to Node
        // Scoring. The owner applies this once to the merged batch response.
        candidate_limit: candidate_heap_limit.max(1),
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

/// Apply the owner-side Algorithm 1 score filter. Batch-level sorting and the
/// L limit are applied by [`prune_and_limit_neighbor_batch`]. Threshold ties
/// are retained deliberately, and the vec_id tie-break makes results
/// deterministic.
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

/// Apply the Algorithm 1 limit once to the merged candidate set for a whole
/// owner response batch. The wire response retains one row per requested key,
/// so retained candidates are redistributed into their originating response
/// row after the global top-L selection.
pub(crate) fn prune_and_limit_neighbor_batch(
    responses: &mut [DistannExpandedNode],
    code_threshold: Option<f32>,
    candidate_limit: Option<usize>,
) -> Result<(), DistannExpandError> {
    for response in responses.iter_mut() {
        let before = response.neighbor_vec_ids.len();
        let (ids, dists) = prune_and_limit_neighbors(
            &response.neighbor_vec_ids,
            &response.neighbor_code_dists,
            code_threshold,
            None,
        )?;
        response.neighbor_vec_ids = ids;
        response.neighbor_code_dists = dists;
        response.neighbors_pruned = response
            .neighbors_pruned
            .saturating_add(before.saturating_sub(response.neighbor_vec_ids.len()));
    }
    let Some(limit) = candidate_limit else {
        return Ok(());
    };
    let mut merged = responses
        .iter()
        .enumerate()
        .flat_map(|(response_idx, response)| {
            response
                .neighbor_vec_ids
                .iter()
                .copied()
                .zip(response.neighbor_code_dists.iter().copied())
                .enumerate()
                .map(move |(neighbor_idx, (vec_id, dist))| {
                    (dist, vec_id, response_idx, neighbor_idx)
                })
        })
        .collect::<Vec<_>>();
    #[cfg(feature = "distann-head-attribution-benchmark")]
    let before = merged.len();
    merged.sort_unstable_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
            .then_with(|| left.3.cmp(&right.3))
    });
    merged.truncate(limit);
    let mut keep = responses
        .iter()
        .map(|response| vec![false; response.neighbor_vec_ids.len()])
        .collect::<Vec<_>>();
    for (_, _, response_idx, neighbor_idx) in merged {
        keep[response_idx][neighbor_idx] = true;
    }
    for (response_idx, response) in responses.iter_mut().enumerate() {
        let before = response.neighbor_vec_ids.len();
        let mut retained = response
            .neighbor_vec_ids
            .iter()
            .copied()
            .zip(response.neighbor_code_dists.iter().copied())
            .enumerate()
            .filter_map(|(neighbor_idx, pair)| keep[response_idx][neighbor_idx].then_some(pair))
            .collect::<Vec<_>>();
        retained.sort_unstable_by(|left, right| {
            left.1
                .total_cmp(&right.1)
                .then_with(|| left.0.cmp(&right.0))
        });
        response.neighbor_vec_ids = retained.iter().map(|(id, _)| *id).collect();
        response.neighbor_code_dists = retained.iter().map(|(_, dist)| *dist).collect();
        response.neighbors_pruned = response
            .neighbors_pruned
            .saturating_add(before.saturating_sub(response.neighbor_vec_ids.len()));
    }
    #[cfg(feature = "distann-head-attribution-benchmark")]
    super::stage_counters::record_work(
        super::stage_counters::DistannMaterializationWork::NeighborsPruned,
        before.saturating_sub(limit),
    );
    Ok(())
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
    let expansion_budget = params
        .beam_width
        .checked_mul(params.hop_rounds)
        .ok_or_else(|| {
            DistannExpandError::BadInput(
                "ec_distann scan beam_width * hop_rounds exceeds usize".to_owned(),
            )
        })?;

    // Beam pool ordered by code distance; `enqueued` dedupes by vec_id
    // (FR-081: visited-set dedupe is by vec_id, expansion at most once).
    let beam_capacity = seeds.len() + params.beam_width * params.hop_rounds * 8;
    let mut beam = BinaryHeap::with_capacity(beam_capacity);
    let mut enqueued: HashSet<u64> = HashSet::with_capacity(beam_capacity);
    #[cfg(feature = "distann-head-attribution-benchmark")]
    super::stage_counters::seed_trace_start(
        &seeds.iter().map(|seed| seed.vec_id).collect::<Vec<_>>(),
    );
    let mut origins: HashMap<u64, u32> = HashMap::with_capacity(beam_capacity);
    for (seed_index, seed) in seeds.iter().enumerate() {
        if enqueued.insert(seed.vec_id) {
            let origin_mask = u32::try_from(seed_index)
                .ok()
                .and_then(|index| 1_u32.checked_shl(index))
                .unwrap_or(0);
            origins.insert(seed.vec_id, origin_mask);
            beam.push(BeamCandidate {
                dist: seed.dist,
                vec_id: seed.vec_id,
                origin_mask,
            });
        }
    }
    if pushdown {
        retain_best_candidates(&mut beam, params.candidate_heap_limit);
    }
    let mut expanded: HashSet<u64> = HashSet::new();
    let mut hits: Vec<DistannScanHit> = Vec::new();

    let mut batch: Vec<u64> = Vec::with_capacity(params.beam_width);
    let mut batch_origins: Vec<u32> = Vec::with_capacity(params.beam_width);
    for _round in 0..params.hop_rounds {
        let limits = if pushdown {
            derive_pushdown_limits(
                &beam,
                &expanded,
                params.candidate_heap_limit,
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
        batch_origins.clear();
        let mut best_unvisited = None;
        while let Some(candidate) = beam.pop() {
            if expanded.contains(&candidate.vec_id) {
                continue;
            }
            if best_unvisited.is_none() {
                best_unvisited = Some(candidate);
            }
            batch.push(candidate.vec_id);
            batch_origins.push(
                origins
                    .get(&candidate.vec_id)
                    .copied()
                    .unwrap_or(candidate.origin_mask),
            );
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
        let owner_times = responses
            .iter()
            .map(|response| response.owner_total_ns.max(0) as u64)
            .filter(|value| *value > 0)
            .collect::<Vec<_>>();
        let max_owner = owner_times.iter().copied().max().unwrap_or(0);
        let min_owner = owner_times.iter().copied().min().unwrap_or(0);
        let transport_wait_ns = responses
            .iter()
            .map(|response| {
                response
                    .coordinator_rpc_ns
                    .saturating_sub(response.owner_total_ns)
                    .saturating_sub(response.coordinator_decode_ns)
                    .max(0) as u64
            })
            .max()
            .unwrap_or(0);
        let response_bytes = responses
            .iter()
            .map(|response| usize::try_from(response.owner_response_bytes.max(0)).unwrap_or(usize::MAX))
            .sum();
        counters.rounds.push(DistannRoundCounters {
            round: counters.rounds_executed - 1,
            requested_nodes: batch.len(),
            expanded_nodes: responses.len(),
            transport_wait_ns,
            straggler_spread_ns: max_owner.saturating_sub(min_owner),
            // The wire request is one vec_id u64 per requested node plus the
            // fixed bind/query envelope. Keep the estimate explicit: the
            // physical endpoint's exact response bytes are authoritative.
            request_bytes: batch.len().saturating_mul(std::mem::size_of::<u64>()),
            response_bytes,
        });
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
        for ((requested, origin_mask), response) in batch
            .iter()
            .zip(batch_origins.iter())
            .zip(responses.iter())
        {
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
                    owner_heap_tid: response.owner_heap_tid,
                    exact_dist,
                });
                #[cfg(feature = "distann-head-attribution-benchmark")]
                super::stage_counters::seed_trace_hit(response.vec_id, *origin_mask);
            }

            #[cfg(feature = "distann-head-attribution-benchmark")]
            super::stage_counters::seed_trace_expanded(*origin_mask);

            if response.neighbor_vec_ids.len() != response.neighbor_code_dists.len() {
                return Err(DistannExpandError::Internal(
                    "ec_distann expansion neighbor arrays are not index-aligned".to_owned(),
                ));
            }
            counters.neighbors_code_scored += response.neighbor_vec_ids.len();
            counters.neighbors_pruned += response.neighbors_pruned;
            for (neighbor_vec_id, code_dist) in response
                .neighbor_vec_ids
                .iter()
                .zip(response.neighbor_code_dists.iter())
            {
                let prior_origin_mask = origins.get(neighbor_vec_id).copied().unwrap_or(0);
                let merged_origin_mask = prior_origin_mask | *origin_mask;
                if merged_origin_mask != prior_origin_mask {
                    origins.insert(*neighbor_vec_id, merged_origin_mask);
                }
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
                        origin_mask: merged_origin_mask,
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

    if counters.records_expanded > expansion_budget {
        return Err(DistannExpandError::Internal(format!(
            "ec_distann expansion cap violated: expanded {} records, budget is {} (BW={} H={})",
            counters.records_expanded, expansion_budget, params.beam_width, params.hop_rounds
        )));
    }
    debug_assert!(counters.records_expanded <= expansion_budget);
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
        prune_and_limit_neighbor_batch, prune_and_limit_neighbors, run_scan_attempt_with_restart,
        BeamCandidate, DistannExpandError, DistannExpandedNode, DistannNodeExpander,
        DistannOrchestrationParams, DistannScanHit, DistannSeedCandidate,
        retain_best_candidates,
    };
    use crate::storage::page::ItemPointer;
    use std::cell::Cell;
    use std::collections::{BinaryHeap, HashMap};

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
            let mut responses = vec_ids
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
                        None,
                        None,
                    )?;
                    Ok(DistannExpandedNode {
                        vec_id: *vec_id,
                        exact_dist: (!tombstone).then_some(*exact),
                        is_tombstone: *tombstone,
                        heap_tid: tid(*vec_id as u16),
                        owner_heap_tid: ItemPointer::INVALID,
                        neighbor_vec_ids,
                        neighbor_code_dists,
                        neighbors_pruned: 0,
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
                .collect::<Result<Vec<_>, DistannExpandError>>()?;
            prune_and_limit_neighbor_batch(&mut responses, code_threshold, candidate_limit)?;
            Ok(responses)
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

    #[cfg(feature = "distann-head-attribution-benchmark")]
    #[test]
    fn gateway_trace_records_shared_expansion_origins() {
        let mut expander = MockExpander {
            nodes: HashMap::from([
                (1, (-0.9, false, vec![(3, -0.7)])),
                (2, (-0.8, false, vec![(3, -0.6)])),
                (3, (-0.5, false, vec![])),
            ]),
            calls: Vec::new(),
        };
        let seeds = [
            DistannSeedCandidate {
                vec_id: 1,
                dist: -0.9,
            },
            DistannSeedCandidate {
                vec_id: 2,
                dist: -0.8,
            },
        ];
        let (result, trace) = super::super::stage_counters::with_seed_trace(|| {
            distann_orchestrated_search(&seeds, &mut expander, params(2, 4, 10))
        });
        result.expect("gateway trace search should succeed");
        assert_eq!(trace.seed_ids, vec![1, 2]);
        assert_eq!(trace.seed_expanded_counts, vec![2, 2]);
        assert_eq!(trace.seed_hit_counts, vec![2, 2]);
        assert_eq!(trace.expanded_unique, 3);
        assert_eq!(trace.expanded_overlap, 1);
        assert!(trace.hit_origin_masks.contains(&3));
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
    fn distann_orchestration_accepts_maximum_beam_width_without_budget_overflow() {
        let mut expander = MockExpander {
            nodes: HashMap::from([(1, (-1.0, false, vec![]))]),
            calls: Vec::new(),
        };
        let seeds = [DistannSeedCandidate {
            vec_id: 1,
            dist: -1.0,
        }];
        let (_, counters) = distann_orchestrated_search(
            &seeds,
            &mut expander,
            params(256, 1, 1),
        )
        .expect("BW=256 should remain within the checked budget");
        assert!(counters.records_expanded <= 256);
        assert_eq!(counters.rounds.len(), counters.rounds_executed);
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
            // The bounded path prunes this low-scoring tombstone. The
            // unbounded reference still traverses it, so full hit equality
            // proves that pruning did not alter the returned live rows.
            (3, (-0.2, true, vec![])),
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
        assert_eq!(reference, candidate);
        assert!(counters.pushdown_rounds_with_threshold > 0);
        assert!(counters.neighbors_pruned > 0);
        let (ids, _) = prune_and_limit_neighbors(
            &[2, 3],
            &[-0.9, -0.1],
            Some(0.5),
            Some(1),
        )
        .expect("aligned neighbor arrays");
        assert_eq!(ids, vec![2], "the active bounded response prunes a neighbor");
    }

    #[test]
    fn bounded_candidate_heap_never_exceeds_l() {
        let mut beam = BinaryHeap::new();
        for vec_id in 0..16 {
            beam.push(BeamCandidate {
                dist: -(vec_id as f32),
                vec_id,
                origin_mask: 0,
            });
        }
        retain_best_candidates(&mut beam, 4);
        assert_eq!(beam.len(), 4);
        assert!(beam.iter().all(|candidate| candidate.vec_id >= 12));
    }
}
