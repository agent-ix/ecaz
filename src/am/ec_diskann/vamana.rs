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
use rayon::prelude::*;
use std::{cmp::Reverse, collections::BinaryHeap, sync::Arc, time::Instant};

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

/// Serial build-side neighbor cache.
///
/// Task 65b's parallel build work needs one narrow place to replace direct
/// adjacency-list reads and writes with a shared/coordinated neighbor store.
/// This first slice keeps the storage in ordinary `Vec<Vec<u32>>` form so the
/// single-threaded build remains deterministic and behavior-equivalent, while all
/// build-loop graph mutation now flows through this type.
#[derive(Debug, Clone)]
struct BuilderNeighborCache {
    neighbors: Vec<Arc<[u32]>>,
    max_degree: usize,
}

impl BuilderNeighborCache {
    fn empty(node_count: usize, max_degree: usize) -> Self {
        let empty: Arc<[u32]> = Arc::from(Vec::<u32>::new().into_boxed_slice());
        Self {
            neighbors: vec![empty; node_count],
            max_degree,
        }
    }

    fn out_degree(&self, node: u32) -> usize {
        self.neighbors[node as usize].len()
    }

    fn contains_neighbor(&self, node: u32, neighbor: u32) -> bool {
        self.neighbors[node as usize].contains(&neighbor)
    }

    fn push_neighbor(&mut self, node: u32, neighbor: u32) {
        let node_idx = node as usize;
        debug_assert!(
            self.neighbors[node_idx].len() < self.max_degree,
            "push_neighbor called for full Vamana adjacency list"
        );
        let mut updated = self.neighbors[node_idx].to_vec();
        updated.push(neighbor);
        self.neighbors[node_idx] = Arc::from(updated.into_boxed_slice());
    }

    fn set_neighbors(&mut self, node: u32, neighbors: &[u32]) {
        debug_assert!(
            neighbors.len() <= self.max_degree,
            "Vamana adjacency list exceeds max_degree"
        );
        self.neighbors[node as usize] = Arc::from(neighbors.to_vec().into_boxed_slice());
    }

    fn replace_neighbors(&mut self, node: u32, neighbors: Vec<u32>) {
        debug_assert!(
            neighbors.len() <= self.max_degree,
            "Vamana adjacency list exceeds max_degree"
        );
        self.neighbors[node as usize] = Arc::from(neighbors.into_boxed_slice());
    }

    fn iter_adjacency(&self) -> impl Iterator<Item = &[u32]> {
        self.neighbors.iter().map(Arc::as_ref)
    }

    fn snapshot(&self) -> BuilderNeighborSnapshot {
        BuilderNeighborSnapshot {
            neighbors: Arc::from(self.neighbors.clone().into_boxed_slice()),
        }
    }

    fn into_graph(self) -> VamanaGraph {
        VamanaGraph {
            neighbors: self
                .neighbors
                .into_iter()
                .map(|neighbors| neighbors.to_vec())
                .collect(),
            max_degree: self.max_degree,
        }
    }
}

impl VamanaGraphView for BuilderNeighborCache {
    fn node_count(&self) -> usize {
        self.neighbors.len()
    }

    fn neighbors(&self, node: u32) -> &[u32] {
        &self.neighbors[node as usize]
    }
}

#[derive(Debug, Clone)]
struct BuilderNeighborSnapshot {
    neighbors: Arc<[Arc<[u32]>]>,
}

impl VamanaGraphView for BuilderNeighborSnapshot {
    fn node_count(&self) -> usize {
        self.neighbors.len()
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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct VamanaParallelBuildStats {
    pub epochs: usize,
    pub batch_size: usize,
    pub proposal_ms: u128,
    pub reducer_ms: u128,
    pub same_epoch_candidate_reads: usize,
    pub total_candidate_reads: usize,
}

impl VamanaParallelBuildStats {
    pub fn stale_read_fraction(self) -> f64 {
        if self.total_candidate_reads == 0 {
            0.0
        } else {
            self.same_epoch_candidate_reads as f64 / self.total_candidate_reads as f64
        }
    }
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

fn robust_prune_final_alpha<D>(
    pivot: u32,
    mut candidates: Vec<Candidate>,
    alpha: f32,
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
    robust_prune_at_alpha(&candidates, alpha, max_degree, &dist)
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
    for candidate in candidates {
        if result.len() >= max_degree {
            break;
        }
        let dominated = result.iter().any(|&kept| {
            // Keep v iff α · d(p*, v) > d(p, v) — i.e. v is not
            // α-dominated by p*. Using <= for the drop side so that
            // exact ties prune, matching pgvectorscale behavior.
            alpha * dist(kept, candidate.node) <= candidate.distance
        });
        if !dominated {
            result.push(candidate.node);
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
    let mut neighbor_cache = BuilderNeighborCache::empty(node_count, max_degree);

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
        let visited = greedy_search_visited_with_scratch(
            &neighbor_cache,
            medoid,
            list_size,
            &mut scratch,
            |n| dist(n, i),
        );
        greedy_search_ms += greedy_started.elapsed().as_millis();
        let visited_count = visited.len();
        let existing_neighbor_count = neighbor_cache.out_degree(i);
        let candidate_pool_started = Instant::now();
        let mut candidates = candidate_pool_for_prune(
            i,
            visited,
            neighbor_cache.neighbors(i).iter().copied(),
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
        neighbor_cache.set_neighbors(i, &pruned);

        let backlink_started = Instant::now();
        for j in pruned {
            if neighbor_cache.contains_neighbor(j, i) {
                continue;
            }
            if neighbor_cache.out_degree(j) < max_degree {
                neighbor_cache.push_neighbor(j, i);
                backlinks_added += 1;
            } else {
                // Re-prune j's neighborhood including i.
                let combined: Vec<Candidate> = neighbor_cache
                    .neighbors(j)
                    .iter()
                    .copied()
                    .chain(std::iter::once(i))
                    .map(|n| Candidate {
                        node: n,
                        distance: dist(j, n),
                    })
                    .collect();
                let repruned = robust_prune(j, combined, alpha_final, max_degree, dist);
                neighbor_cache.replace_neighbors(j, repruned);
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

    let final_out_degrees: Vec<usize> = neighbor_cache
        .iter_adjacency()
        .map(|neighbors| neighbors.len())
        .collect();
    let mut final_in_degrees = vec![0usize; neighbor_cache.node_count()];
    for neighbors in neighbor_cache.iter_adjacency() {
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
    let graph = neighbor_cache.into_graph();

    (graph, stats)
}

pub fn build_vamana_graph_with_parallel_epochs<D>(
    node_count: usize,
    medoid: u32,
    max_degree: usize,
    list_size: usize,
    alpha_final: f32,
    seed: u64,
    batch_size: usize,
    dist: D,
) -> (VamanaGraph, VamanaBuildStats, VamanaParallelBuildStats)
where
    D: Fn(u32, u32) -> f32 + Copy + Send + Sync,
{
    assert!(batch_size > 0, "parallel Vamana batch size must be > 0");
    let mut neighbor_cache = BuilderNeighborCache::empty(node_count, max_degree);

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut permutation: Vec<u32> = (0..node_count as u32).collect();
    let mut passes = Vec::with_capacity(1);

    let pass_started = Instant::now();
    for i in (1..permutation.len()).rev() {
        let j = rng.gen_range(0..=i);
        permutation.swap(i, j);
    }

    let mut pivot_ordinals = vec![usize::MAX; node_count];
    for (ordinal, &node) in permutation.iter().enumerate() {
        pivot_ordinals[node as usize] = ordinal;
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
    let mut parallel_stats = VamanaParallelBuildStats {
        batch_size,
        ..VamanaParallelBuildStats::default()
    };
    // Batch size 1 is the serial-equivalence scaffold. True parallel batches
    // use the final-alpha prune directly to avoid repeating reducer distance
    // work for an intermediate alpha that is not required for fallback parity.
    let grow_alpha = batch_size == 1;
    let mut incoming_backlinks: Vec<Vec<u32>> = vec![Vec::new(); node_count];
    let mut touched_backlink_targets: Vec<u32> = Vec::new();

    for (epoch_idx, epoch) in permutation.chunks(batch_size).enumerate() {
        let epoch_start = epoch_idx * batch_size;
        let snapshot = neighbor_cache.snapshot();
        let proposal_started = Instant::now();
        let mut proposals: Vec<VamanaPivotProposal> = epoch
            .par_iter()
            .enumerate()
            .map(|(epoch_offset, &pivot)| {
                propose_vamana_pivot(
                    pivot,
                    epoch_idx,
                    epoch_start + epoch_offset,
                    epoch_start,
                    &pivot_ordinals,
                    &snapshot,
                    medoid,
                    node_count,
                    max_degree,
                    list_size,
                    alpha_final,
                    grow_alpha,
                    dist,
                )
            })
            .collect();
        parallel_stats.proposal_ms += proposal_started.elapsed().as_millis();
        proposals.sort_by_key(|proposal| proposal.ordinal);

        let reducer_started = Instant::now();
        for proposal in proposals {
            debug_assert_eq!(proposal.epoch, parallel_stats.epochs);
            greedy_search_ms += proposal.greedy_search_ms;
            candidate_pool_ms += proposal.candidate_pool_ms;
            robust_prune_ms += proposal.robust_prune_ms;
            parallel_stats.same_epoch_candidate_reads += proposal.same_epoch_candidate_reads;
            parallel_stats.total_candidate_reads += proposal.total_candidate_reads;
            visited_counts.push(proposal.visited_count);
            existing_neighbor_counts.push(proposal.existing_neighbor_count);
            candidate_pool_counts.push(proposal.candidate_count);
            selected_neighbor_counts.push(proposal.pruned.len());

            for &target in &proposal.pruned {
                let target_idx = target as usize;
                debug_assert!(
                    target_idx < incoming_backlinks.len(),
                    "proposal target outside graph"
                );
                if incoming_backlinks[target_idx].is_empty() {
                    touched_backlink_targets.push(target);
                }
                incoming_backlinks[target_idx].push(proposal.pivot);
            }
            neighbor_cache.replace_neighbors(proposal.pivot, proposal.pruned);
        }

        let backlink_started = Instant::now();
        let backlink_plans: Vec<VamanaBacklinkBatchPlan> = touched_backlink_targets
            .par_iter()
            .map(|&target| {
                let target_idx = target as usize;
                plan_vamana_backlink_batch(
                    target,
                    neighbor_cache.neighbors(target),
                    &incoming_backlinks[target_idx],
                    max_degree,
                    alpha_final,
                    grow_alpha,
                    dist,
                )
            })
            .collect();
        for plan in backlink_plans {
            backlinks_added += plan.stats.backlinks_added;
            reprunes += plan.stats.reprunes;
            if let Some(neighbors) = plan.neighbors {
                neighbor_cache.replace_neighbors(plan.target, neighbors);
            }
        }
        for &target in &touched_backlink_targets {
            incoming_backlinks[target as usize].clear();
        }
        touched_backlink_targets.clear();
        backlink_ms += backlink_started.elapsed().as_millis();
        parallel_stats.reducer_ms += reducer_started.elapsed().as_millis();
        parallel_stats.epochs += 1;
    }

    passes.push(VamanaBuildPassStats {
        alpha: alpha_final,
        pivot_count: permutation.len(),
        elapsed_ms: pass_started.elapsed().as_millis(),
        greedy_search_ms,
        candidate_pool_ms,
        robust_prune_ms,
        backlink_ms,
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

    let final_out_degrees: Vec<usize> = neighbor_cache
        .iter_adjacency()
        .map(|neighbors| neighbors.len())
        .collect();
    let mut final_in_degrees = vec![0usize; neighbor_cache.node_count()];
    for neighbors in neighbor_cache.iter_adjacency() {
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
    let graph = neighbor_cache.into_graph();

    (graph, stats, parallel_stats)
}

#[derive(Debug, Clone)]
struct VamanaPivotProposal {
    epoch: usize,
    ordinal: usize,
    pivot: u32,
    pruned: Vec<u32>,
    visited_count: usize,
    existing_neighbor_count: usize,
    candidate_count: usize,
    greedy_search_ms: u128,
    candidate_pool_ms: u128,
    robust_prune_ms: u128,
    same_epoch_candidate_reads: usize,
    total_candidate_reads: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct VamanaProposalCommitStats {
    backlinks_added: usize,
    reprunes: usize,
    backlink_ms: u128,
}

struct VamanaBacklinkBatchPlan {
    target: u32,
    neighbors: Option<Vec<u32>>,
    stats: VamanaProposalCommitStats,
}

fn commit_vamana_pivot_proposal<D>(
    neighbor_cache: &mut BuilderNeighborCache,
    proposal: VamanaPivotProposal,
    max_degree: usize,
    alpha_final: f32,
    dist: D,
) -> VamanaProposalCommitStats
where
    D: Fn(u32, u32) -> f32 + Copy,
{
    let mut stats = VamanaProposalCommitStats::default();
    neighbor_cache.set_neighbors(proposal.pivot, &proposal.pruned);

    let backlink_started = Instant::now();
    for j in proposal.pruned {
        if neighbor_cache.contains_neighbor(j, proposal.pivot) {
            continue;
        }
        if neighbor_cache.out_degree(j) < max_degree {
            neighbor_cache.push_neighbor(j, proposal.pivot);
            stats.backlinks_added += 1;
        } else {
            let combined: Vec<Candidate> = neighbor_cache
                .neighbors(j)
                .iter()
                .copied()
                .chain(std::iter::once(proposal.pivot))
                .map(|n| Candidate {
                    node: n,
                    distance: dist(j, n),
                })
                .collect();
            let repruned = robust_prune(j, combined, alpha_final, max_degree, dist);
            neighbor_cache.replace_neighbors(j, repruned);
            stats.reprunes += 1;
        }
    }
    stats.backlink_ms = backlink_started.elapsed().as_millis();
    stats
}

fn commit_vamana_backlink_batch<D>(
    neighbor_cache: &mut BuilderNeighborCache,
    target: u32,
    proposed_sources: &[u32],
    max_degree: usize,
    alpha_final: f32,
    grow_alpha: bool,
    dist: D,
) -> VamanaProposalCommitStats
where
    D: Fn(u32, u32) -> f32 + Copy,
{
    let plan = plan_vamana_backlink_batch(
        target,
        neighbor_cache.neighbors(target),
        proposed_sources,
        max_degree,
        alpha_final,
        grow_alpha,
        dist,
    );
    if let Some(neighbors) = plan.neighbors {
        neighbor_cache.replace_neighbors(plan.target, neighbors);
    }
    plan.stats
}

fn plan_vamana_backlink_batch<D>(
    target: u32,
    existing: &[u32],
    proposed_sources: &[u32],
    max_degree: usize,
    alpha_final: f32,
    grow_alpha: bool,
    dist: D,
) -> VamanaBacklinkBatchPlan
where
    D: Fn(u32, u32) -> f32 + Copy,
{
    let mut stats = VamanaProposalCommitStats::default();
    if proposed_sources.is_empty() {
        return VamanaBacklinkBatchPlan {
            target,
            neighbors: None,
            stats,
        };
    }

    let mut additions = Vec::with_capacity(proposed_sources.len());
    for &source in proposed_sources {
        if existing.contains(&source) || additions.contains(&source) {
            continue;
        }
        additions.push(source);
    }
    if additions.is_empty() {
        return VamanaBacklinkBatchPlan {
            target,
            neighbors: None,
            stats,
        };
    }

    if existing.len() + additions.len() <= max_degree {
        let mut updated = Vec::with_capacity(existing.len() + additions.len());
        updated.extend_from_slice(existing);
        stats.backlinks_added += additions.len();
        updated.extend(additions);
        return VamanaBacklinkBatchPlan {
            target,
            neighbors: Some(updated),
            stats,
        };
    }

    let combined: Vec<Candidate> = existing
        .iter()
        .copied()
        .chain(additions)
        .map(|node| Candidate {
            node,
            distance: dist(target, node),
        })
        .collect();
    let repruned = if grow_alpha {
        robust_prune(target, combined, alpha_final, max_degree, dist)
    } else {
        robust_prune_final_alpha(target, combined, alpha_final, max_degree, dist)
    };
    stats.reprunes += 1;
    VamanaBacklinkBatchPlan {
        target,
        neighbors: Some(repruned),
        stats,
    }
}

#[allow(clippy::too_many_arguments)]
fn propose_vamana_pivot<D>(
    pivot: u32,
    epoch: usize,
    ordinal: usize,
    epoch_start: usize,
    pivot_ordinals: &[usize],
    snapshot: &impl VamanaGraphView,
    medoid: u32,
    node_count: usize,
    max_degree: usize,
    list_size: usize,
    alpha_final: f32,
    grow_alpha: bool,
    dist: D,
) -> VamanaPivotProposal
where
    D: Fn(u32, u32) -> f32 + Copy,
{
    let mut scratch = SearchScratch::new(node_count, list_size);
    let greedy_started = Instant::now();
    let visited =
        greedy_search_visited_with_scratch(snapshot, medoid, list_size, &mut scratch, |n| {
            dist(n, pivot)
        });
    let greedy_search_ms = greedy_started.elapsed().as_millis();
    let visited_count = visited.len();
    let existing_neighbor_count = snapshot.neighbors(pivot).len();

    let candidate_pool_started = Instant::now();
    let existing_neighbors: Vec<u32> = snapshot.neighbors(pivot).to_vec();
    let total_candidate_reads = visited_count + existing_neighbors.len();
    let same_epoch_candidate_reads = visited
        .iter()
        .chain(existing_neighbors.iter())
        .filter(|&&node| {
            let Some(&node_ordinal) = pivot_ordinals.get(node as usize) else {
                return false;
            };
            (epoch_start..ordinal).contains(&node_ordinal)
        })
        .count();
    let candidates = candidate_pool_for_prune(
        pivot,
        visited,
        existing_neighbors,
        node_count,
        &mut scratch,
        |node, pivot| dist(node, pivot),
    );
    let candidate_pool_ms = candidate_pool_started.elapsed().as_millis();
    let candidate_count = candidates.len();

    let robust_prune_started = Instant::now();
    let pruned = if grow_alpha {
        robust_prune(pivot, candidates, alpha_final, max_degree, dist)
    } else {
        robust_prune_final_alpha(pivot, candidates, alpha_final, max_degree, dist)
    };
    let robust_prune_ms = robust_prune_started.elapsed().as_millis();

    VamanaPivotProposal {
        epoch,
        ordinal,
        pivot,
        pruned,
        visited_count,
        existing_neighbor_count,
        candidate_count,
        greedy_search_ms,
        candidate_pool_ms,
        robust_prune_ms,
        same_epoch_candidate_reads,
        total_candidate_reads,
    }
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
    fn builder_neighbor_cache_preserves_adjacency_operations() {
        let mut cache = BuilderNeighborCache::empty(4, 3);
        cache.push_neighbor(0, 1);
        cache.push_neighbor(0, 2);
        assert_eq!(cache.neighbors(0), &[1, 2]);
        assert_eq!(cache.out_degree(0), 2);
        assert!(cache.contains_neighbor(0, 1));

        cache.set_neighbors(1, &[0, 2]);
        assert_eq!(cache.neighbors(1), &[0, 2]);

        cache.replace_neighbors(1, vec![3]);
        assert_eq!(cache.neighbors(1), &[3]);

        let graph = cache.into_graph();
        assert_eq!(graph.max_degree, 3);
        assert_eq!(graph.neighbors[0], vec![1, 2]);
        assert_eq!(graph.neighbors[1], vec![3]);
    }

    #[test]
    fn task65b_snapshot_rows_do_not_observe_later_reducer_writes() {
        let mut cache = BuilderNeighborCache::empty(4, 3);
        cache.set_neighbors(0, &[1, 2]);
        let snapshot = cache.snapshot();

        cache.push_neighbor(0, 3);
        cache.set_neighbors(1, &[0]);

        assert_eq!(
            snapshot.neighbors(0),
            &[1, 2],
            "epoch snapshot must retain the row shape seen by proposal workers"
        );
        assert!(
            snapshot.neighbors(1).is_empty(),
            "new reducer writes must replace live rows without mutating snapshots"
        );
        assert_eq!(cache.neighbors(0), &[1, 2, 3]);
        assert_eq!(cache.neighbors(1), &[0]);
    }

    fn task65b_model_proposal(
        epoch: usize,
        ordinal: usize,
        pivot: u32,
        pruned: Vec<u32>,
    ) -> VamanaPivotProposal {
        VamanaPivotProposal {
            epoch,
            ordinal,
            pivot,
            visited_count: 0,
            existing_neighbor_count: 0,
            candidate_count: pruned.len(),
            greedy_search_ms: 0,
            candidate_pool_ms: 0,
            robust_prune_ms: 0,
            same_epoch_candidate_reads: 0,
            total_candidate_reads: 0,
            pruned,
        }
    }

    fn task65b_commit_ordered_proposals(
        mut proposals: Vec<VamanaPivotProposal>,
        max_degree: usize,
        dist: impl Fn(u32, u32) -> f32 + Copy,
    ) -> Vec<Vec<u32>> {
        proposals.sort_by_key(|proposal| proposal.ordinal);
        let mut cache = BuilderNeighborCache::empty(4, max_degree);
        for proposal in proposals {
            commit_vamana_pivot_proposal(&mut cache, proposal, max_degree, 1.2, dist);
        }
        cache.into_graph().neighbors
    }

    #[test]
    fn task65b_epoch_proposals_read_epoch_snapshot_not_live_reducer_state() {
        let mut live = BuilderNeighborCache::empty(4, 3);
        live.set_neighbors(0, &[2]);
        let snapshot = live.clone();

        let committed = task65b_model_proposal(0, 0, 1, vec![3]);
        commit_vamana_pivot_proposal(&mut live, committed, 3, 1.2, |a, b| {
            (a as i32 - b as i32).unsigned_abs() as f32
        });

        assert_eq!(live.neighbors(1), &[3]);
        assert!(
            snapshot.neighbors(1).is_empty(),
            "epoch snapshot must not observe reducer commits from the same epoch"
        );

        let pivot_ordinals = [0, 1, 2, 3];
        let proposal = propose_vamana_pivot(
            1,
            0,
            1,
            0,
            &pivot_ordinals,
            &snapshot,
            0,
            4,
            3,
            4,
            1.2,
            true,
            |a, b| (a as i32 - b as i32).unsigned_abs() as f32,
        );
        assert_eq!(proposal.existing_neighbor_count, 0);
        assert_eq!(
            proposal.same_epoch_candidate_reads, 1,
            "the medoid has an earlier ordinal in this epoch and should be counted as a stale-read candidate"
        );
    }

    #[test]
    fn task65b_ordered_reducer_ignores_proposal_completion_order() {
        let dist = |a: u32, b: u32| (a as i32 - b as i32).unsigned_abs() as f32;
        let ordered = vec![
            task65b_model_proposal(0, 0, 0, vec![2]),
            task65b_model_proposal(0, 1, 1, vec![2]),
            task65b_model_proposal(0, 2, 3, vec![2]),
        ];
        let mut completed_out_of_order = vec![
            task65b_model_proposal(0, 2, 3, vec![2]),
            task65b_model_proposal(0, 1, 1, vec![2]),
            task65b_model_proposal(0, 0, 0, vec![2]),
        ];
        completed_out_of_order.sort_by_key(|proposal| proposal.ordinal);

        let mut ordered_cache = BuilderNeighborCache::empty(4, 3);
        for proposal in ordered {
            commit_vamana_pivot_proposal(&mut ordered_cache, proposal, 3, 1.2, dist);
        }
        let mut reduced_cache = BuilderNeighborCache::empty(4, 3);
        for proposal in completed_out_of_order {
            commit_vamana_pivot_proposal(&mut reduced_cache, proposal, 3, 1.2, dist);
        }

        assert_eq!(
            ordered_cache.into_graph().neighbors,
            reduced_cache.into_graph().neighbors
        );
    }

    #[test]
    fn task65b_reducer_is_invariant_across_all_three_proposal_arrival_orders() {
        let dist = |a: u32, b: u32| (a as i32 - b as i32).unsigned_abs() as f32;
        let proposals = [
            task65b_model_proposal(0, 0, 0, vec![2]),
            task65b_model_proposal(0, 1, 1, vec![2]),
            task65b_model_proposal(0, 2, 3, vec![2]),
        ];
        let expected = task65b_commit_ordered_proposals(proposals.to_vec(), 3, dist);

        for order in [
            [0usize, 1usize, 2usize],
            [0usize, 2usize, 1usize],
            [1usize, 0usize, 2usize],
            [1usize, 2usize, 0usize],
            [2usize, 0usize, 1usize],
            [2usize, 1usize, 0usize],
        ] {
            let scheduled = order
                .into_iter()
                .map(|idx| proposals[idx].clone())
                .collect::<Vec<_>>();
            assert_eq!(
                task65b_commit_ordered_proposals(scheduled, 3, dist),
                expected,
                "arrival order {order:?} should reduce to the same graph"
            );
        }
    }

    #[test]
    fn task65b_epoch_boundary_controls_snapshot_visibility() {
        let dist = |a: u32, b: u32| (a as i32 - b as i32).unsigned_abs() as f32;
        let pivot_ordinals = [0usize, 1usize, 2usize, 3usize];
        let mut live = BuilderNeighborCache::empty(4, 3);

        let epoch0_snapshot = live.snapshot();
        let epoch0_proposal = propose_vamana_pivot(
            1,
            0,
            1,
            0,
            &pivot_ordinals,
            &epoch0_snapshot,
            0,
            4,
            3,
            4,
            1.2,
            true,
            dist,
        );
        commit_vamana_pivot_proposal(&mut live, epoch0_proposal, 3, 1.2, dist);
        assert!(!live.neighbors(1).is_empty());

        let stale_epoch0_proposal = propose_vamana_pivot(
            1,
            0,
            1,
            0,
            &pivot_ordinals,
            &epoch0_snapshot,
            0,
            4,
            3,
            4,
            1.2,
            true,
            dist,
        );
        assert_eq!(
            stale_epoch0_proposal.existing_neighbor_count, 0,
            "same-epoch proposals must keep reading the pre-reducer snapshot"
        );

        let epoch1_snapshot = live.snapshot();
        let epoch1_proposal = propose_vamana_pivot(
            1,
            1,
            1,
            1,
            &pivot_ordinals,
            &epoch1_snapshot,
            0,
            4,
            3,
            4,
            1.2,
            true,
            dist,
        );
        assert!(
            epoch1_proposal.existing_neighbor_count > 0,
            "next-epoch proposals should observe prior epoch reducer commits"
        );
    }

    #[test]
    fn task65b_batched_backlinks_reprune_each_touched_target_once() {
        let dist = |a: u32, b: u32| (a as i32 - b as i32).unsigned_abs() as f32;
        let mut append_cache = BuilderNeighborCache::empty(6, 3);
        append_cache.set_neighbors(0, &[1]);
        let append_stats =
            commit_vamana_backlink_batch(&mut append_cache, 0, &[2, 3], 3, 1.2, true, dist);
        assert_eq!(append_stats.backlinks_added, 2);
        assert_eq!(append_stats.reprunes, 0);
        assert_eq!(append_cache.neighbors(0), &[1, 2, 3]);

        let mut reprune_cache = BuilderNeighborCache::empty(6, 2);
        reprune_cache.set_neighbors(0, &[1, 2]);
        let reprune_stats =
            commit_vamana_backlink_batch(&mut reprune_cache, 0, &[3, 4, 5], 2, 1.2, true, dist);
        assert_eq!(reprune_stats.backlinks_added, 0);
        assert_eq!(
            reprune_stats.reprunes, 1,
            "overflow backlinks for one target should collapse into one re-prune"
        );
        let neighbors = reprune_cache.neighbors(0);
        assert!(neighbors.len() <= 2);
        assert!(
            neighbors.windows(2).all(|pair| pair[0] != pair[1]),
            "batched re-prune must not introduce duplicate neighbors"
        );
    }

    #[test]
    fn task65b_parallel_epoch_batch_one_matches_serial_graph_exactly() {
        let points = synth_2d(48, 13);
        let dist = l2(&points);
        let medoid = approximate_medoid(points.len(), points.len(), 7, dist);
        let (serial, serial_stats) =
            build_vamana_graph_with_stats(points.len(), medoid, 6, 12, 1.2, 11, dist);
        let (parallel, parallel_stats, epoch_stats) =
            build_vamana_graph_with_parallel_epochs(points.len(), medoid, 6, 12, 1.2, 11, 1, dist);

        assert_eq!(serial.neighbors, parallel.neighbors);
        assert_eq!(
            serial_stats.final_out_degree,
            parallel_stats.final_out_degree
        );
        assert_eq!(serial_stats.final_in_degree, parallel_stats.final_in_degree);
        assert_eq!(epoch_stats.epochs, points.len());
        assert_eq!(epoch_stats.batch_size, 1);
        assert_eq!(epoch_stats.same_epoch_candidate_reads, 0);
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
