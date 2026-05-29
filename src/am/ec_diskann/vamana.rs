//! In-memory Vamana α-pruning graph construction.
//!
//! Implements the algorithmic core of `ec_diskann`'s build pipeline:
//! [`GreedySearch`] (best-first traversal with bounded frontier),
//! [`RobustPrune`] (α-dominance candidate pruning), and
//! [`build_vamana_graph`] (the single-pass, growing-alpha build driver from
//! the Task 65 performance overhaul).
//!
//! Distance is abstract: callers pass an `&impl Fn(u32, u32) -> f32`
//! (build-time) or `&impl Fn(u32) -> f32` (query-time) so the algorithm
//! is testable with synthetic distances and reusable for both the
//! `score_ip_codes_lite` candidate-vs-candidate path and the
//! `PqFastScan` query-vs-candidate path that phase-5 integration will
//! plug in.
//!
//! The graph itself is an adjacency list keyed by dense `u32` node id.
//! Mapping node id ↔ heap TID is the integration layer's job (phase
//! 5C); this module just builds the graph.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::{cmp::Reverse, collections::BinaryHeap, time::Instant};

/// In-memory adjacency list. `neighbors[i]` is the out-edge set of node `i`.
///
/// Invariant: `neighbors[i].len() <= max_degree`. Edges are stored
/// distance-unordered (the build path repeatedly re-prunes, so any
/// invariant on order is paid for separately at persistence time).
#[derive(Debug, Clone)]
pub struct VamanaGraph {
    pub neighbors: Vec<Vec<u32>>,
    pub max_degree: usize,
}

impl VamanaGraph {
    pub fn empty(node_count: usize, max_degree: usize) -> Self {
        Self {
            neighbors: vec![Vec::new(); node_count],
            max_degree,
        }
    }

    pub fn node_count(&self) -> usize {
        self.neighbors.len()
    }

    pub fn out_degree(&self, node: u32) -> usize {
        self.neighbors[node as usize].len()
    }
}

pub trait VamanaGraphView {
    fn node_count(&self) -> usize;
    fn neighbors(&self, node: u32) -> &[u32];
}

impl VamanaGraphView for VamanaGraph {
    fn node_count(&self) -> usize {
        self.node_count()
    }

    fn neighbors(&self, node: u32) -> &[u32] {
        &self.neighbors[node as usize]
    }
}

/// Aggregate summary for build-time counters captured by
/// [`build_vamana_graph_with_stats`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MetricSummary {
    pub count: usize,
    pub min: usize,
    pub mean: f64,
    pub p50: usize,
    pub p95: usize,
    pub p99: usize,
    pub max: usize,
}

impl MetricSummary {
    fn from_values(values: &[usize]) -> Self {
        if values.is_empty() {
            return Self::default();
        }
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        let sum: usize = sorted.iter().sum();
        Self {
            count: sorted.len(),
            min: sorted[0],
            mean: sum as f64 / sorted.len() as f64,
            p50: percentile(&sorted, 0.50),
            p95: percentile(&sorted, 0.95),
            p99: percentile(&sorted, 0.99),
            max: *sorted.last().expect("non-empty sorted values"),
        }
    }
}

fn percentile(sorted: &[usize], p: f64) -> usize {
    debug_assert!(!sorted.is_empty());
    let idx = ((sorted.len() - 1) as f64 * p).ceil() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Per-pass Vamana build diagnostics. The vectors are summarized after
/// each pass so callers can inspect candidate generation without storing
/// per-node detail for large corpora.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VamanaBuildPassStats {
    pub alpha: f32,
    pub pivot_count: usize,
    pub elapsed_ms: u128,
    pub greedy_search_ms: u128,
    pub candidate_pool_ms: u128,
    pub robust_prune_ms: u128,
    pub backlink_ms: u128,
    pub greedy_distance_calls: usize,
    pub candidate_pool_distance_calls: usize,
    pub robust_prune_distance_calls: usize,
    pub backlink_distance_calls: usize,
    pub visited: MetricSummary,
    pub existing_neighbors: MetricSummary,
    pub candidate_pool: MetricSummary,
    pub selected_neighbors: MetricSummary,
    pub backlinks_added: usize,
    pub reprunes: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VamanaBuildStats {
    pub node_count: usize,
    pub medoid: u32,
    pub max_degree: usize,
    pub list_size: usize,
    pub alpha_final: f32,
    pub seed: u64,
    pub passes: Vec<VamanaBuildPassStats>,
    pub final_out_degree: MetricSummary,
    pub final_in_degree: MetricSummary,
}

/// Result of [`greedy_search`]: the final frontier (top-`L` candidates by
/// distance) and the full visited set.
#[derive(Debug, Clone)]
pub struct GreedySearchResult {
    /// Candidates in distance order, ascending. At most `L` entries.
    pub frontier: Vec<Candidate>,
    /// Every node id whose neighbors were expanded.
    pub visited: Vec<u32>,
}

/// Reusable build-time scratch for Vamana search and candidate de-duplication.
#[derive(Debug, Clone)]
pub struct SearchScratch {
    in_frontier: BitSet,
    visited: BitSet,
    seen_candidates: BitSet,
    unexpanded: BinaryHeap<Reverse<Candidate>>,
    retained: BinaryHeap<Candidate>,
    visited_order: Vec<u32>,
}

impl SearchScratch {
    pub fn new(node_count: usize, list_size: usize) -> Self {
        Self {
            in_frontier: BitSet::new(node_count),
            visited: BitSet::new(node_count),
            seen_candidates: BitSet::new(node_count),
            unexpanded: BinaryHeap::with_capacity(list_size.saturating_add(1)),
            retained: BinaryHeap::with_capacity(list_size.saturating_add(1)),
            visited_order: Vec::with_capacity(list_size),
        }
    }

    fn clear_search(&mut self) {
        self.in_frontier.clear();
        self.visited.clear();
        self.unexpanded.clear();
        self.retained.clear();
        self.visited_order.clear();
    }

    fn clear_candidates(&mut self) {
        self.seen_candidates.clear();
    }
}

#[derive(Debug, Clone)]
struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    fn new(bits: usize) -> Self {
        Self {
            words: vec![0; bits.div_ceil(64)],
        }
    }

    fn clear(&mut self) {
        self.words.fill(0);
    }

    fn contains(&self, bit: u32) -> bool {
        let bit = bit as usize;
        let word = bit / 64;
        let mask = 1u64 << (bit % 64);
        self.words
            .get(word)
            .map(|value| value & mask != 0)
            .unwrap_or(false)
    }

    fn insert(&mut self, bit: u32) {
        let bit = bit as usize;
        let word = bit / 64;
        let mask = 1u64 << (bit % 64);
        if let Some(value) = self.words.get_mut(word) {
            *value |= mask;
        }
    }

    fn remove(&mut self, bit: u32) {
        let bit = bit as usize;
        let word = bit / 64;
        let mask = 1u64 << (bit % 64);
        if let Some(value) = self.words.get_mut(word) {
            *value &= !mask;
        }
    }
}

/// `(node id, distance)` pair. `Ord` is by distance ascending so a
/// `BinaryHeap<Reverse<Candidate>>` becomes a min-heap.
#[derive(Debug, Clone, Copy)]
pub struct Candidate {
    pub node: u32,
    pub distance: f32,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance && self.node == other.node
    }
}
impl Eq for Candidate {}
impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // NaN treated as +infinity so it sorts last.
        self.distance
            .partial_cmp(&other.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| self.node.cmp(&other.node))
    }
}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Greedy best-first search from `start` toward the implicit query whose
/// distance to any node id `n` is `query_dist(n)`. Returns the top-`L`
/// frontier and the visited set.
///
/// Loop invariant: every node in `frontier.visited == false` is a
/// candidate for expansion. We pop the closest unexpanded node, add its
/// neighbors to the frontier, then truncate to `L`.
pub fn greedy_search<D>(
    graph: &VamanaGraph,
    start: u32,
    list_size: usize,
    query_dist: D,
) -> GreedySearchResult
where
    D: Fn(u32) -> f32,
{
    greedy_search_view(graph, start, list_size, query_dist)
}

pub fn greedy_search_view<G, D>(
    graph: &G,
    start: u32,
    list_size: usize,
    query_dist: D,
) -> GreedySearchResult
where
    G: VamanaGraphView + ?Sized,
    D: Fn(u32) -> f32,
{
    let mut scratch = SearchScratch::new(graph.node_count(), list_size);
    greedy_search_view_with_scratch(graph, start, list_size, &mut scratch, query_dist)
}

pub fn greedy_search_view_with_scratch<G, D>(
    graph: &G,
    start: u32,
    list_size: usize,
    scratch: &mut SearchScratch,
    query_dist: D,
) -> GreedySearchResult
where
    G: VamanaGraphView + ?Sized,
    D: Fn(u32) -> f32,
{
    scratch.clear_search();
    let start_dist = query_dist(start);
    let start_candidate = Candidate {
        node: start,
        distance: start_dist,
    };
    push_frontier_candidate(scratch, list_size, start_candidate);

    loop {
        let next = loop {
            let Some(Reverse(candidate)) = scratch.unexpanded.pop() else {
                break None;
            };
            if scratch.in_frontier.contains(candidate.node)
                && !scratch.visited.contains(candidate.node)
            {
                break Some(candidate);
            }
        };
        let Some(picked) = next else {
            break;
        };
        scratch.visited.insert(picked.node);
        scratch.visited_order.push(picked.node);

        for &neighbor in graph.neighbors(picked.node) {
            if scratch.in_frontier.contains(neighbor) {
                continue;
            }
            let d = query_dist(neighbor);
            let candidate = Candidate {
                node: neighbor,
                distance: d,
            };
            push_frontier_candidate(scratch, list_size, candidate);
        }
    }

    let mut frontier: Vec<Candidate> = scratch
        .retained
        .iter()
        .copied()
        .filter(|candidate| scratch.in_frontier.contains(candidate.node))
        .collect();
    frontier.sort_unstable();
    GreedySearchResult {
        frontier,
        visited: std::mem::take(&mut scratch.visited_order),
    }
}

/// Build-path variant of [`greedy_search_view_with_scratch`] that skips
/// constructing the sorted top-`L` frontier. The build driver only reads
/// the visited list, so the frontier build is pure waste in that hot path.
pub fn greedy_search_visited_with_scratch<G, D>(
    graph: &G,
    start: u32,
    list_size: usize,
    scratch: &mut SearchScratch,
    query_dist: D,
) -> Vec<u32>
where
    G: VamanaGraphView + ?Sized,
    D: Fn(u32) -> f32,
{
    scratch.clear_search();
    let start_dist = query_dist(start);
    let start_candidate = Candidate {
        node: start,
        distance: start_dist,
    };
    push_frontier_candidate(scratch, list_size, start_candidate);

    loop {
        let next = loop {
            let Some(Reverse(candidate)) = scratch.unexpanded.pop() else {
                break None;
            };
            if scratch.in_frontier.contains(candidate.node)
                && !scratch.visited.contains(candidate.node)
            {
                break Some(candidate);
            }
        };
        let Some(picked) = next else {
            break;
        };
        scratch.visited.insert(picked.node);
        scratch.visited_order.push(picked.node);

        for &neighbor in graph.neighbors(picked.node) {
            if scratch.in_frontier.contains(neighbor) {
                continue;
            }
            let d = query_dist(neighbor);
            let candidate = Candidate {
                node: neighbor,
                distance: d,
            };
            push_frontier_candidate(scratch, list_size, candidate);
        }
    }

    std::mem::take(&mut scratch.visited_order)
}

fn push_frontier_candidate(scratch: &mut SearchScratch, list_size: usize, candidate: Candidate) {
    if list_size == 0 || scratch.in_frontier.contains(candidate.node) {
        return;
    }

    if scratch.retained.len() < list_size {
        scratch.in_frontier.insert(candidate.node);
        scratch.retained.push(candidate);
        scratch.unexpanded.push(Reverse(candidate));
        return;
    }

    let Some(&worst) = scratch.retained.peek() else {
        return;
    };
    if candidate >= worst {
        return;
    }

    let evicted = scratch
        .retained
        .pop()
        .expect("peeked retained frontier should be poppable");
    scratch.in_frontier.remove(evicted.node);
    scratch.in_frontier.insert(candidate.node);
    scratch.retained.push(candidate);
    scratch.unexpanded.push(Reverse(candidate));
}

/// α-pruning: select up to `max_degree` neighbors from `candidates` for
/// node `pivot` such that no kept neighbor is α-dominated by an earlier
/// kept neighbor.
///
/// `dist(a, b)` returns the build-time distance. Must be nonnegative
/// for the α-inequality (`α · d(p*, v) > d(p, v)`) to be well-defined —
/// this is why ec_diskann wraps inner product as
/// `d = max(0, -ip + C)` (see design doc §"Distance function").
pub fn robust_prune<D>(
    pivot: u32,
    mut candidates: Vec<Candidate>,
    max_alpha: f32,
    max_degree: usize,
    dist: D,
) -> Vec<u32>
where
    D: Fn(u32, u32) -> f32,
{
    candidates.retain(|c| c.node != pivot);
    if candidates.is_empty() {
        return Vec::new();
    }
    candidates.sort();

    let mut alpha = 1.0f32;
    loop {
        let best = robust_prune_at_alpha(&candidates, alpha, max_degree, &dist);
        if best.len() >= max_degree || alpha >= max_alpha {
            return best;
        }
        alpha = (alpha * 1.2).min(max_alpha);
    }
}

fn robust_prune_at_alpha<D>(
    candidates: &[Candidate],
    alpha: f32,
    max_degree: usize,
    dist: &D,
) -> Vec<u32>
where
    D: Fn(u32, u32) -> f32,
{
    let mut result: Vec<u32> = Vec::with_capacity(max_degree);
    let mut active = vec![true; candidates.len()];
    let mut remaining = candidates.len();
    let mut cursor = 0usize;
    while remaining > 0 && result.len() < max_degree {
        while cursor < active.len() && !active[cursor] {
            cursor += 1;
        }
        if cursor == active.len() {
            break;
        }

        let pivot_star = candidates[cursor];
        active[cursor] = false;
        remaining -= 1;
        result.push(pivot_star.node);

        for idx in cursor + 1..candidates.len() {
            if !active[idx] {
                continue;
            }
            let v = candidates[idx];
            // Keep v iff α · d(p*, v) > d(p, v) — i.e. v is not
            // α-dominated by p*. Using <= for the drop side so that
            // exact ties prune, matching pgvectorscale behavior.
            if alpha * dist(pivot_star.node, v.node) <= v.distance {
                active[idx] = false;
                remaining -= 1;
            }
        }
    }
    result
}

fn candidate_pool_for_prune<D>(
    pivot: u32,
    visited: impl IntoIterator<Item = u32>,
    existing_neighbors: impl IntoIterator<Item = u32>,
    node_count: usize,
    scratch: &mut SearchScratch,
    dist: D,
) -> Vec<Candidate>
where
    D: Fn(u32, u32) -> f32,
{
    scratch.clear_candidates();
    let mut candidates = Vec::new();

    for node in visited.into_iter().chain(existing_neighbors) {
        if node == pivot {
            continue;
        }
        debug_assert!(
            (node as usize) < node_count,
            "candidate node id {} outside graph size {}",
            node,
            node_count
        );
        if (node as usize) >= node_count || scratch.seen_candidates.contains(node) {
            continue;
        }
        scratch.seen_candidates.insert(node);
        candidates.push(Candidate {
            node,
            distance: dist(node, pivot),
        });
    }

    candidates
}

fn append_candidates_for_prune<D>(
    pivot: u32,
    candidates: &mut Vec<Candidate>,
    extra: impl IntoIterator<Item = u32>,
    node_count: usize,
    scratch: &mut SearchScratch,
    dist: D,
) where
    D: Fn(u32, u32) -> f32,
{
    scratch.clear_candidates();
    for candidate in candidates.iter() {
        if (candidate.node as usize) < node_count {
            scratch.seen_candidates.insert(candidate.node);
        }
    }

    for node in extra {
        if node == pivot || (node as usize) >= node_count || scratch.seen_candidates.contains(node)
        {
            continue;
        }
        scratch.seen_candidates.insert(node);
        candidates.push(Candidate {
            node,
            distance: dist(node, pivot),
        });
    }
}

/// Approximate medoid via random-sample sum-of-distances. `S = min(cap,
/// node_count)` indices are drawn uniformly without replacement; the
/// medoid is the sample with the smallest sum of distances to the
/// other samples. Cost: O(S²) distance evaluations.
pub fn approximate_medoid<D>(node_count: usize, sample_cap: usize, seed: u64, dist: D) -> u32
where
    D: Fn(u32, u32) -> f32,
{
    assert!(node_count > 0, "approximate_medoid requires node_count > 0");
    let sample_size = sample_cap.min(node_count);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // Reservoir sample for unbiased uniform-without-replacement.
    let mut samples: Vec<u32> = (0..sample_size as u32).collect();
    for i in sample_size..node_count {
        let j = rng.gen_range(0..=i);
        if j < sample_size {
            samples[j] = i as u32;
        }
    }

    let mut best_node = samples[0];
    let mut best_sum = f32::INFINITY;
    for &s in &samples {
        let mut sum = 0.0f32;
        for &t in &samples {
            if s != t {
                sum += dist(s, t);
            }
        }
        if sum < best_sum {
            best_sum = sum;
            best_node = s;
        }
    }
    best_node
}

/// Build a Vamana graph in one shuffled pivot pass. Robust pruning grows
/// α from 1.0 up to `alpha_final` while looking for enough neighbors.
/// `medoid` is the entry point; `list_size` is the search-list bound `L`;
/// `max_degree` is `R`.
///
/// `dist(a, b)` is the build-time distance between two node ids; it
/// must be nonnegative (see [`robust_prune`]). The same `dist` is used
/// for greedy search by binding the second arg to the pivot via the
/// closure.
pub fn build_vamana_graph<D>(
    node_count: usize,
    medoid: u32,
    max_degree: usize,
    list_size: usize,
    alpha_final: f32,
    seed: u64,
    dist: D,
) -> VamanaGraph
where
    D: Fn(u32, u32) -> f32 + Copy,
{
    build_vamana_graph_with_stats(
        node_count,
        medoid,
        max_degree,
        list_size,
        alpha_final,
        seed,
        dist,
    )
    .0
}

pub fn build_vamana_graph_with_stats<D>(
    node_count: usize,
    medoid: u32,
    max_degree: usize,
    list_size: usize,
    alpha_final: f32,
    seed: u64,
    dist: D,
) -> (VamanaGraph, VamanaBuildStats)
where
    D: Fn(u32, u32) -> f32 + Copy,
{
    build_vamana_graph_with_pass1_extra_candidates(
        node_count,
        medoid,
        max_degree,
        list_size,
        alpha_final,
        seed,
        &[],
        dist,
    )
}

pub fn build_vamana_graph_with_pass1_extra_candidates<D>(
    node_count: usize,
    medoid: u32,
    max_degree: usize,
    list_size: usize,
    alpha_final: f32,
    seed: u64,
    pass1_extra_candidates: &[Vec<u32>],
    dist: D,
) -> (VamanaGraph, VamanaBuildStats)
where
    D: Fn(u32, u32) -> f32 + Copy,
{
    debug_assert!(
        pass1_extra_candidates.is_empty() || pass1_extra_candidates.len() == node_count,
        "pass1 extra candidates must be empty or have one entry per node"
    );
    let mut graph = VamanaGraph::empty(node_count, max_degree);

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut permutation: Vec<u32> = (0..node_count as u32).collect();
    let mut passes = Vec::with_capacity(1);

    let pass_started = Instant::now();
    for i in (1..permutation.len()).rev() {
        let j = rng.gen_range(0..=i);
        permutation.swap(i, j);
    }

    let mut visited_counts = Vec::with_capacity(permutation.len());
    let mut existing_neighbor_counts = Vec::with_capacity(permutation.len());
    let mut candidate_pool_counts = Vec::with_capacity(permutation.len());
    let mut selected_neighbor_counts = Vec::with_capacity(permutation.len());
    let mut backlinks_added = 0usize;
    let mut reprunes = 0usize;
    let mut greedy_search_ms = 0u128;
    let mut candidate_pool_ms = 0u128;
    let mut robust_prune_ms = 0u128;
    let mut backlink_ms = 0u128;
    let mut scratch = SearchScratch::new(node_count, list_size);

    for &i in &permutation {
        // Greedy search from medoid toward i; collect visited set
        // plus i's existing out-neighbors as the candidate pool.
        // Vamana's RobustPrune folds the old neighborhood into
        // the fresh visited set so later passes refine, rather
        // than replace, prior graph structure.
        let greedy_started = Instant::now();
        let visited =
            greedy_search_visited_with_scratch(&graph, medoid, list_size, &mut scratch, |n| {
                dist(n, i)
            });
        greedy_search_ms += greedy_started.elapsed().as_millis();
        let visited_count = visited.len();
        let existing_neighbor_count = graph.neighbors[i as usize].len();
        let candidate_pool_started = Instant::now();
        let mut candidates = candidate_pool_for_prune(
            i,
            visited,
            graph.neighbors[i as usize].iter().copied(),
            node_count,
            &mut scratch,
            |node, pivot| dist(node, pivot),
        );
        if let Some(extra) = pass1_extra_candidates.get(i as usize) {
            append_candidates_for_prune(
                i,
                &mut candidates,
                extra.iter().copied(),
                node_count,
                &mut scratch,
                |node, pivot| dist(node, pivot),
            );
        }
        candidate_pool_ms += candidate_pool_started.elapsed().as_millis();
        let candidate_count = candidates.len();

        let robust_prune_started = Instant::now();
        let pruned = robust_prune(i, candidates, alpha_final, max_degree, dist);
        robust_prune_ms += robust_prune_started.elapsed().as_millis();
        let selected_count = pruned.len();
        graph.neighbors[i as usize].clone_from(&pruned);

        let backlink_started = Instant::now();
        for j in pruned {
            let neighbors_j = &mut graph.neighbors[j as usize];
            if neighbors_j.contains(&i) {
                continue;
            }
            if neighbors_j.len() < max_degree {
                neighbors_j.push(i);
                backlinks_added += 1;
            } else {
                // Re-prune j's neighborhood including i.
                let mut combined: Vec<Candidate> = neighbors_j
                    .iter()
                    .copied()
                    .chain(std::iter::once(i))
                    .map(|n| Candidate {
                        node: n,
                        distance: dist(j, n),
                    })
                    .collect();
                combined.sort();
                let repruned = robust_prune(j, combined, alpha_final, max_degree, dist);
                graph.neighbors[j as usize] = repruned;
                reprunes += 1;
            }
        }
        backlink_ms += backlink_started.elapsed().as_millis();

        visited_counts.push(visited_count);
        existing_neighbor_counts.push(existing_neighbor_count);
        candidate_pool_counts.push(candidate_count);
        selected_neighbor_counts.push(selected_count);
    }

    passes.push(VamanaBuildPassStats {
        alpha: alpha_final,
        pivot_count: permutation.len(),
        elapsed_ms: pass_started.elapsed().as_millis(),
        greedy_search_ms,
        candidate_pool_ms,
        robust_prune_ms,
        backlink_ms,
        // Per-call counters dropped from the hot path; fields preserved for
        // log-line compatibility, populated to 0 in release builds.
        greedy_distance_calls: 0,
        candidate_pool_distance_calls: 0,
        robust_prune_distance_calls: 0,
        backlink_distance_calls: 0,
        visited: MetricSummary::from_values(&visited_counts),
        existing_neighbors: MetricSummary::from_values(&existing_neighbor_counts),
        candidate_pool: MetricSummary::from_values(&candidate_pool_counts),
        selected_neighbors: MetricSummary::from_values(&selected_neighbor_counts),
        backlinks_added,
        reprunes,
    });

    let final_out_degrees: Vec<usize> = graph.neighbors.iter().map(Vec::len).collect();
    let mut final_in_degrees = vec![0usize; graph.node_count()];
    for neighbors in &graph.neighbors {
        for &neighbor in neighbors {
            if let Some(count) = final_in_degrees.get_mut(neighbor as usize) {
                *count += 1;
            }
        }
    }
    let stats = VamanaBuildStats {
        node_count,
        medoid,
        max_degree,
        list_size,
        alpha_final,
        seed,
        passes,
        final_out_degree: MetricSummary::from_values(&final_out_degrees),
        final_in_degree: MetricSummary::from_values(&final_in_degrees),
    };

    (graph, stats)
}

/// BFS from `start`, returning the set of reachable node ids.
///
/// Phase-5 connectivity test uses this to assert that the medoid can
/// reach the bulk of the graph after build (see ADR-046 step 1 — a
/// disconnected medoid breaks both scan and live insert).
pub fn bfs_reachable(graph: &VamanaGraph, start: u32) -> Vec<u32> {
    let n = graph.node_count();
    let mut seen = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    let mut order = Vec::new();

    seen[start as usize] = true;
    queue.push_back(start);
    while let Some(node) = queue.pop_front() {
        order.push(node);
        for &neighbor in &graph.neighbors[node as usize] {
            if !seen[neighbor as usize] {
                seen[neighbor as usize] = true;
                queue.push_back(neighbor);
            }
        }
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic 2D dataset; distance = squared L2.
    fn synth_2d(n: usize, seed: u64) -> Vec<(f32, f32)> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        (0..n)
            .map(|_| (rng.gen::<f32>(), rng.gen::<f32>()))
            .collect()
    }

    fn l2(points: &[(f32, f32)]) -> impl Fn(u32, u32) -> f32 + Copy + '_ {
        move |a, b| {
            let (ax, ay) = points[a as usize];
            let (bx, by) = points[b as usize];
            let dx = ax - bx;
            let dy = ay - by;
            dx * dx + dy * dy
        }
    }

    #[test]
    fn robust_prune_respects_max_degree() {
        let candidates: Vec<Candidate> = (1..=20u32)
            .map(|n| Candidate {
                node: n,
                distance: n as f32,
            })
            .collect();
        let dist = |_a: u32, _b: u32| 100.0; // All non-pivot dists are huge → no α-domination.
        let kept = robust_prune(0, candidates, 1.2, 8, dist);
        assert!(kept.len() <= 8);
        assert!(!kept.contains(&0), "pivot must be excluded");
    }

    #[test]
    fn miri_robust_prune_excludes_alpha_dominated() {
        // Pivot is node 0. Candidates 1, 2, 3 all near pivot, but
        // candidates 2 and 3 are very close to candidate 1 (so 1
        // α-dominates them with α = 1.2).
        let candidates = vec![
            Candidate {
                node: 1,
                distance: 1.0,
            },
            Candidate {
                node: 2,
                distance: 1.5,
            },
            Candidate {
                node: 3,
                distance: 1.6,
            },
        ];
        let dist = |a: u32, b: u32| match (a.min(b), a.max(b)) {
            (1, 2) => 0.1, // 1.2 * 0.1 = 0.12 < 1.5 → 2 is dominated by 1
            (1, 3) => 0.1, // 1.2 * 0.1 = 0.12 < 1.6 → 3 is dominated by 1
            _ => 100.0,
        };
        let kept = robust_prune(0, candidates, 1.2, 8, dist);
        assert_eq!(kept, vec![1], "1 should dominate 2 and 3 at α=1.2");
    }

    #[test]
    fn candidate_pool_includes_existing_out_neighbors() {
        let mut scratch = SearchScratch::new(4, 4);
        let pool =
            candidate_pool_for_prune(0, vec![1, 2, 2], vec![2, 3, 0], 4, &mut scratch, |a, b| {
                (a as i32 - b as i32).unsigned_abs() as f32
            });
        let nodes: Vec<u32> = pool.iter().map(|c| c.node).collect();
        assert_eq!(nodes, vec![1, 2, 3]);
        assert_eq!(
            pool.iter().map(|c| c.distance).collect::<Vec<_>>(),
            vec![1.0, 2.0, 3.0]
        );
    }

    #[test]
    fn append_candidates_for_prune_deduplicates_extra_nodes() {
        let mut scratch = SearchScratch::new(4, 4);
        let mut pool = candidate_pool_for_prune(0, vec![1], Vec::new(), 4, &mut scratch, |a, b| {
            (a as i32 - b as i32).unsigned_abs() as f32
        });
        append_candidates_for_prune(
            0,
            &mut pool,
            vec![1, 2, 2, 0, 3],
            4,
            &mut scratch,
            |a, b| (a as i32 - b as i32).unsigned_abs() as f32,
        );
        let nodes: Vec<u32> = pool.iter().map(|c| c.node).collect();
        assert_eq!(nodes, vec![1, 2, 3]);
    }

    #[test]
    fn miri_greedy_search_finds_nearest() {
        // Linear-chain graph: 0 - 1 - 2 - ... - 19. Distance to node 19
        // is just the index. Search from node 0 should converge to 19.
        let n = 20;
        let mut graph = VamanaGraph::empty(n, 4);
        for i in 0..n - 1 {
            graph.neighbors[i].push((i + 1) as u32);
            graph.neighbors[i + 1].push(i as u32);
        }
        let target = 19u32;
        let result = greedy_search(&graph, 0, 4, |n| (target as i32 - n as i32).abs() as f32);
        assert_eq!(result.frontier.first().map(|c| c.node), Some(target));
        assert_eq!(result.frontier.first().map(|c| c.distance), Some(0.0));
    }

    #[test]
    fn build_small_graph_is_connected() {
        let points = synth_2d(100, 7);
        let dist = l2(&points);
        let medoid = approximate_medoid(points.len(), 100, 7, dist);
        let (graph, stats) =
            build_vamana_graph_with_stats(points.len(), medoid, 8, 32, 1.2, 11, dist);

        // Every node has at least one neighbor.
        for (i, neighbors) in graph.neighbors.iter().enumerate() {
            assert!(
                !neighbors.is_empty(),
                "node {} has no out-edges after build",
                i
            );
            assert!(
                neighbors.len() <= graph.max_degree,
                "node {} exceeds max_degree",
                i
            );
        }

        // BFS from medoid reaches every node.
        let reachable = bfs_reachable(&graph, medoid);
        assert_eq!(
            reachable.len(),
            graph.node_count(),
            "medoid must reach every node; got {}/{}",
            reachable.len(),
            graph.node_count()
        );
        assert_eq!(stats.passes.len(), 1);
        assert_eq!(stats.passes[0].pivot_count, points.len());
        assert_eq!(stats.final_out_degree.count, points.len());
        assert_eq!(stats.final_in_degree.count, points.len());
    }

    #[test]
    fn build_stats_include_pass1_extra_candidates() {
        let points = synth_2d(50, 7);
        let dist = l2(&points);
        let medoid = approximate_medoid(points.len(), 50, 7, dist);
        let extra = vec![vec![1, 2, 3, 4]; points.len()];
        let (_graph, stats) = build_vamana_graph_with_pass1_extra_candidates(
            points.len(),
            medoid,
            8,
            16,
            1.2,
            11,
            &extra,
            dist,
        );
        assert!(
            stats.passes[0].candidate_pool.mean > stats.passes[0].visited.mean,
            "pass-1 extra candidates should enlarge the candidate pool"
        );
    }

    #[test]
    fn miri_build_small_graph_is_connected() {
        let points = synth_2d(16, 7);
        let dist = l2(&points);
        let medoid = approximate_medoid(points.len(), points.len(), 7, dist);
        let (graph, stats) =
            build_vamana_graph_with_stats(points.len(), medoid, 4, 8, 1.2, 11, dist);

        for (i, neighbors) in graph.neighbors.iter().enumerate() {
            assert!(!neighbors.is_empty(), "node {i} should have out-edges");
            assert!(neighbors.len() <= graph.max_degree, "node {i} exceeds R");
        }
        assert_eq!(bfs_reachable(&graph, medoid).len(), graph.node_count());
        assert_eq!(stats.passes.len(), 1);
        assert_eq!(stats.final_out_degree.count, points.len());
        assert_eq!(stats.final_in_degree.count, points.len());
    }

    #[test]
    fn miri_build_stats_include_pass1_extra_candidates() {
        let points = synth_2d(12, 7);
        let dist = l2(&points);
        let medoid = approximate_medoid(points.len(), points.len(), 7, dist);
        let extra = vec![vec![1, 2, 3, 4]; points.len()];
        let (_graph, stats) = build_vamana_graph_with_pass1_extra_candidates(
            points.len(),
            medoid,
            4,
            8,
            1.2,
            11,
            &extra,
            dist,
        );
        assert!(
            stats.passes[0].candidate_pool.mean > stats.passes[0].visited.mean,
            "pass-1 extra candidates should enlarge the candidate pool"
        );
    }

    #[test]
    fn approximate_medoid_within_10pct_of_exact() {
        let points = synth_2d(200, 3);
        let dist = l2(&points);

        // Exact medoid via O(N²).
        let mut best_exact = 0u32;
        let mut best_sum = f32::INFINITY;
        for i in 0..points.len() as u32 {
            let mut sum = 0.0f32;
            for j in 0..points.len() as u32 {
                if i != j {
                    sum += dist(i, j);
                }
            }
            if sum < best_sum {
                best_sum = sum;
                best_exact = i;
            }
        }

        // Approximate medoid with full-population sample (deterministic
        // since cap == N).
        let approx = approximate_medoid(points.len(), points.len(), 3, dist);
        assert_eq!(approx, best_exact, "full-population sample == exact");

        // Sub-sampled medoid: average distance from approx should be
        // within 10% of exact.
        let approx_sub = approximate_medoid(points.len(), 100, 3, dist);
        let sum_for = |node: u32| {
            (0..points.len() as u32)
                .filter(|j| *j != node)
                .map(|j| dist(node, j))
                .sum::<f32>()
        };
        let exact_sum = sum_for(best_exact);
        let approx_sum = sum_for(approx_sub);
        let ratio = approx_sum / exact_sum;
        assert!(
            ratio <= 1.10,
            "approx medoid sum {} > 1.10 * exact {} (ratio {:.3})",
            approx_sum,
            exact_sum,
            ratio
        );
    }

    #[test]
    fn build_recall_at_10_meets_baseline() {
        // 500 random 2D points; build with R=16, L=64, α=1.2; query 50
        // random points and check the top-10 nearest from the graph
        // overlap the brute-force top-10 by ≥ 80%. This is a sanity
        // floor, not the production target — the production target is
        // measured on real PqFastScan codes at 1536d in phase 6.
        let n = 500;
        let points = synth_2d(n, 17);
        let dist = l2(&points);
        let medoid = approximate_medoid(n, 200, 17, dist);
        let graph = build_vamana_graph(n, medoid, 16, 64, 1.2, 23, dist);

        let queries = synth_2d(50, 99);
        let mut total_overlap = 0;
        let mut total_compared = 0;
        for q in &queries {
            let qx = q.0;
            let qy = q.1;
            let qdist = |n: u32| {
                let (px, py) = points[n as usize];
                let dx = px - qx;
                let dy = py - qy;
                dx * dx + dy * dy
            };

            let mut all: Vec<u32> = (0..n as u32).collect();
            all.sort_by(|&a, &b| qdist(a).partial_cmp(&qdist(b)).unwrap());
            let exact_top10: Vec<u32> = all.into_iter().take(10).collect();

            let result = greedy_search(&graph, medoid, 64, qdist);
            let approx_top10: Vec<u32> = result.frontier.iter().take(10).map(|c| c.node).collect();

            let overlap = approx_top10
                .iter()
                .filter(|n| exact_top10.contains(n))
                .count();
            total_overlap += overlap;
            total_compared += exact_top10.len();
        }

        let recall = total_overlap as f32 / total_compared as f32;
        assert!(
            recall >= 0.80,
            "synthetic Recall@10 {:.3} < 0.80 baseline",
            recall
        );
    }
}
