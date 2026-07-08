//! FR-077 sharded closure-overlap build + stitch (Task 163 M1).
//!
//! Replaces the monolithic Vamana graph-construction core with:
//!   1. **Closure-overlap shard assignment** (`assign_shards`): spherical
//!      k-means over the corpus, then a distance-ratio closure band — a
//!      vector whose distance to a non-primary centroid is within
//!      `(1 + closure_epsilon)` of its best centroid distance is duplicated
//!      into that shard as well. The distance-ratio machinery ADR-085 cites
//!      lives on the unmerged `task-144-spire-closure-ratio-pruning` branch,
//!      so this is a fresh implementation against the already-plumbed
//!      `closure_epsilon` reloption (FR-077 "implement the ε band fresh").
//!   2. **Per-shard Vamana builds** (`build_shard_graphs`): each shard is an
//!      independent seed-deterministic Vamana over the shared core
//!      (`build_vamana_graph_with_stats`), built in parallel; shard output
//!      adjacency is mapped back to the global vec-id space and sorted by
//!      global node id so the stitch can stream it.
//!   3. **Streaming stitch** (`stitch_shard_graphs`): group by global node
//!      (ADR-085 D8 — one node group + the prune working set held at a time),
//!      union the per-shard neighbor lists, and `robust_prune` the union to
//!      at most `graph_degree` edges under the global distance. A node that
//!      appears in exactly one shard passes through unchanged (stitch
//!      idempotence, FR-077-AC-2).
//!   4. **Reachability repair** (`repair_reachability`): a deterministic
//!      guard that guarantees FR-077-CON-3 (every node reachable from the
//!      entry medoid) by adding a single in-edge from the nearest reached
//!      node to each stranded node. At corpus scale this fires ~never; the
//!      repair count is reported in the stitch stats.
//!
//! The whole pipeline is deterministic under a fixed seed (FR-077): identical
//! corpus + seed + options yield an identical stitched graph — the invariant
//! the M2 single-vs-multinode result-identity test (FR-081-AC-1) depends on.
//!
//! The output is a `VamanaGraph` in the global node-id space, byte-identical
//! in shape to the monolithic build's output, so the record / directory /
//! head-sample staging in `ambuild.rs` is unchanged (FR-077 non-goal: no
//! head-index changes beyond consuming multi-shard entry samples).

use std::collections::BTreeSet;

use rayon::prelude::*;

use crate::am::common::training::train_spherical_kmeans;
use crate::am::ec_diskann::source_inner_product_distance;
use crate::am::{
    approximate_medoid, bfs_reachable, build_vamana_graph_with_stats, robust_prune, Candidate,
    VamanaGraph,
};

/// Same medoid sample cap as the monolithic build (`ambuild.rs`).
const DISTANN_MEDOID_SAMPLE_CAP: usize = 1000;

/// k-means iterations for shard assignment. Shard boundaries only need to be
/// spatially coherent enough for closure overlap to stitch back to global
/// quality, so a modest iteration budget keeps build cost down; determinism
/// holds regardless of the count.
const DISTANN_SHARD_KMEANS_ITERATIONS: usize = 10;

/// Golden-ratio odd constant used to derive independent per-shard build seeds
/// from the base seed (keeps each shard's Vamana insertion order distinct yet
/// deterministic).
const DISTANN_SHARD_SEED_STRIDE: u64 = 0x9E37_79B9_7F4A_7C15;

/// Build-time diagnostics surfaced to the epoch/packet manifest (FR-077-AC-3,
/// ADR-085 D8). All counts are over the global node space.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct ShardBuildStats {
    /// Number of build shards actually used.
    pub(super) shard_count: usize,
    /// Sum of shard memberships divided by node count (1.0 = no overlap).
    pub(super) duplication_factor: f64,
    /// Largest single shard membership (load-balance sanity).
    pub(super) max_shard_size: usize,
    /// Total unioned candidate edges considered across the stitch, before
    /// `robust_prune`.
    pub(super) stitch_edges_before_prune: u64,
    /// Total edges emitted after `robust_prune`.
    pub(super) stitch_edges_after_prune: u64,
    /// Peak *incremental* stitch working set in node ids: the largest
    /// single-node neighbor union the merge held at once. The merge never holds
    /// all unions simultaneously — it streams one node group at a time.
    pub(super) stitch_peak_union_len: usize,
    /// Total node-ids retained across all shard outputs during the stitch
    /// (sum of every shard's node list + adjacency). This v1 holds the shard
    /// graphs in memory rather than spilling them to sorted files, so this —
    /// NOT `stitch_peak_union_len` — is the honest peak-memory figure for the
    /// stitch input (ADR-085 D8 / FR-077-CON-4). It is bounded by
    /// `duplication_factor * node_count * (graph_degree + 1)` and is a small
    /// fraction of the already-resident source vectors; the strict "streamed by
    /// vec_id group" D8 bound (spill shard outputs to disk, merge from cursors)
    /// is a tracked follow-up, not required for M1 correctness.
    pub(super) shard_output_retained_node_ids: u64,
    /// Reachability repairs applied (FR-077-CON-3 guard); expected 0 at scale.
    pub(super) reachability_repairs: usize,
}

/// A single shard's Vamana output, projected into the global node-id space.
/// `nodes` is sorted ascending so the stitch can k-way-merge by node id and
/// hold only one node group at a time (ADR-085 D8).
struct ShardGraph {
    /// Global node ids that belong to this shard, ascending.
    nodes: Vec<u32>,
    /// Parallel to `nodes`: each entry is the shard-local Vamana adjacency of
    /// that node, remapped to global node ids.
    neighbors: Vec<Vec<u32>>,
}

/// Result of closure-overlap shard assignment.
struct ShardAssignment {
    /// Per shard: the global node ids assigned to it (primary + closure), each
    /// list ascending.
    members: Vec<Vec<u32>>,
    duplication_factor: f64,
    max_shard_size: usize,
}

/// Pick the shard count for `node_count` nodes given the requested reloption
/// (`0` = auto). Auto scales gently with corpus size so shards stay large
/// enough for a coherent per-shard Vamana while still parallelizing big
/// builds; `1` (or any explicit request) is honored verbatim, which keeps the
/// monolithic fallback selectable (FR-077 / ADR-085 Consequences).
pub(super) fn resolve_shard_count(node_count: usize, requested: usize) -> usize {
    if requested >= 1 {
        return requested.min(node_count.max(1));
    }
    if node_count <= 20_000 {
        1
    } else {
        // ~one shard per 25k rows, capped so per-shard builds stay meaty.
        (node_count / 25_000).clamp(2, 16)
    }
}

/// Full sharded build. Returns a global-space `VamanaGraph`, the global entry
/// medoid, and manifest stats. Callers guarantee `shard_count >= 2`; the
/// single-shard case stays on the monolithic path in `ambuild.rs`.
pub(super) fn build_sharded_graph(
    source_refs: &[&[f32]],
    dimensions: usize,
    graph_degree: usize,
    build_list_size: usize,
    alpha: f32,
    shard_count: usize,
    closure_epsilon: f32,
    seed: u64,
) -> Result<(VamanaGraph, u32, ShardBuildStats), String> {
    let node_count = source_refs.len();
    debug_assert!(shard_count >= 2, "monolithic build handled by caller");

    let dist = |left: u32, right: u32| -> f32 {
        source_inner_product_distance(source_refs[left as usize], source_refs[right as usize])
    };

    // Global entry medoid — identical rule to the monolithic build so the
    // stitched graph shares an entry point with the fallback path.
    let medoid = approximate_medoid(node_count, DISTANN_MEDOID_SAMPLE_CAP, seed, dist);

    let assignment = assign_shards(source_refs, dimensions, shard_count, closure_epsilon, seed)?;
    let shard_graphs = build_shard_graphs(
        &assignment.members,
        graph_degree,
        build_list_size,
        alpha,
        seed,
        &dist,
    );

    // Honest D8/CON-4 accounting: the shard outputs are held in memory during
    // the stitch (v1 does not spill them to sorted files), so their total size
    // is the real peak-memory figure for the stitch input.
    let shard_output_retained_node_ids: u64 = shard_graphs
        .iter()
        .map(|shard| {
            shard.nodes.len() as u64
                + shard.neighbors.iter().map(|n| n.len() as u64).sum::<u64>()
        })
        .sum();

    let (graph, mut stats) =
        stitch_shard_graphs(node_count, &shard_graphs, graph_degree, alpha, &dist);
    stats.shard_count = shard_count;
    stats.duplication_factor = assignment.duplication_factor;
    stats.max_shard_size = assignment.max_shard_size;
    stats.shard_output_retained_node_ids = shard_output_retained_node_ids;

    let mut graph = graph;
    stats.reachability_repairs = repair_reachability(&mut graph, medoid, graph_degree, &dist)?;

    Ok((graph, medoid, stats))
}

/// Spherical k-means + closure-overlap band. A node is assigned to its nearest
/// centroid (primary) and to every centroid whose `1 - ip` distance is within
/// `(1 + closure_epsilon)` of the primary distance (FR-077 closure overlap).
fn assign_shards(
    source_refs: &[&[f32]],
    dimensions: usize,
    shard_count: usize,
    closure_epsilon: f32,
    seed: u64,
) -> Result<ShardAssignment, String> {
    let node_count = source_refs.len();
    let model = train_spherical_kmeans(
        "ec_distann sharded build",
        source_refs,
        dimensions,
        shard_count,
        seed,
        DISTANN_SHARD_KMEANS_ITERATIONS,
    )?;
    let centroids = &model.centroids;
    let ratio = 1.0_f32 + closure_epsilon.max(0.0);

    let mut members: Vec<Vec<u32>> = vec![Vec::new(); shard_count];
    let mut total_memberships: u64 = 0;
    for (node, source) in source_refs.iter().enumerate() {
        // Distance to every centroid; primary = argmin.
        let mut best = f32::INFINITY;
        for centroid in centroids.iter() {
            let d = source_inner_product_distance(source, centroid);
            if d < best {
                best = d;
            }
        }
        let band = best * ratio;
        let mut assigned_any = false;
        for (shard, centroid) in centroids.iter().enumerate() {
            let d = source_inner_product_distance(source, centroid);
            // `<= band` includes the primary (d == best) and every centroid in
            // the closure band. `band` is finite and >= best, so at least the
            // primary always qualifies.
            if d <= band {
                members[shard].push(node as u32);
                total_memberships += 1;
                assigned_any = true;
            }
        }
        // Defensive: floating-point could in principle exclude all shards if
        // `best` were non-finite; keep every node in at least its primary.
        if !assigned_any {
            let primary = nearest_centroid_index(source, centroids);
            members[primary].push(node as u32);
            total_memberships += 1;
        }
    }

    // `members` are already ascending (nodes pushed in increasing order).
    let max_shard_size = members.iter().map(Vec::len).max().unwrap_or(0);
    let duplication_factor = if node_count == 0 {
        1.0
    } else {
        total_memberships as f64 / node_count as f64
    };

    Ok(ShardAssignment {
        members,
        duplication_factor,
        max_shard_size,
    })
}

fn nearest_centroid_index(source: &[f32], centroids: &[Vec<f32>]) -> usize {
    let mut best = f32::INFINITY;
    let mut best_index = 0;
    for (index, centroid) in centroids.iter().enumerate() {
        let d = source_inner_product_distance(source, centroid);
        if d < best {
            best = d;
            best_index = index;
        }
    }
    best_index
}

/// Build one independent Vamana per shard (parallel; each is pure and seeded,
/// so parallel and sequential yield identical output). Shard-local node ids
/// are remapped to global ids on the way out.
fn build_shard_graphs<D>(
    members: &[Vec<u32>],
    graph_degree: usize,
    build_list_size: usize,
    alpha: f32,
    seed: u64,
    dist: &D,
) -> Vec<ShardGraph>
where
    D: Fn(u32, u32) -> f32 + Sync,
{
    members
        .par_iter()
        .enumerate()
        .map(|(shard_index, global_ids)| {
            build_one_shard(shard_index, global_ids, graph_degree, build_list_size, alpha, seed, dist)
        })
        .collect()
}

fn build_one_shard<D>(
    shard_index: usize,
    global_ids: &[u32],
    graph_degree: usize,
    build_list_size: usize,
    alpha: f32,
    seed: u64,
    dist: &D,
) -> ShardGraph
where
    D: Fn(u32, u32) -> f32,
{
    let shard_len = global_ids.len();
    if shard_len == 0 {
        return ShardGraph {
            nodes: Vec::new(),
            neighbors: Vec::new(),
        };
    }
    // Shard-local distance maps local index -> global id -> global distance.
    let local_dist = |left: u32, right: u32| -> f32 {
        dist(global_ids[left as usize], global_ids[right as usize])
    };
    // Independent per-shard seed keeps each shard's insertion order distinct
    // yet deterministic.
    let shard_seed = seed.wrapping_add((shard_index as u64).wrapping_mul(DISTANN_SHARD_SEED_STRIDE));
    let medoid = approximate_medoid(shard_len, DISTANN_MEDOID_SAMPLE_CAP, shard_seed, local_dist);
    let (graph, _stats) = build_vamana_graph_with_stats(
        shard_len,
        medoid,
        graph_degree,
        build_list_size,
        alpha,
        shard_seed,
        local_dist,
    );

    // Remap adjacency to global ids. `nodes` is `global_ids` (already
    // ascending); neighbor lists translate local -> global.
    let neighbors = graph
        .neighbors
        .iter()
        .map(|local_neighbors| {
            local_neighbors
                .iter()
                .map(|&local| global_ids[local as usize])
                .collect::<Vec<u32>>()
        })
        .collect();

    ShardGraph {
        nodes: global_ids.to_vec(),
        neighbors,
    }
}

/// Streaming stitch (ADR-085 D8): merge the sorted shard adjacency streams by
/// global node id, holding one node group + the prune working set at a time.
/// Single-shard nodes pass through unchanged (idempotence); multi-shard nodes
/// have their neighbor union `robust_prune`d to `graph_degree`.
///
/// Exposed to the property suite (`stitch_shard_graphs` is the pure core of
/// FR-077-CON-1..3 and FR-077-AC-2).
fn stitch_shard_graphs<D>(
    node_count: usize,
    shard_graphs: &[ShardGraph],
    graph_degree: usize,
    alpha: f32,
    dist: &D,
) -> (VamanaGraph, ShardBuildStats)
where
    D: Fn(u32, u32) -> f32,
{
    let mut graph = VamanaGraph::empty(node_count, graph_degree);
    let mut cursors = vec![0_usize; shard_graphs.len()];

    let mut edges_before: u64 = 0;
    let mut edges_after: u64 = 0;
    let mut peak_union_len = 0_usize;

    for node in 0..node_count as u32 {
        // Gather this node's neighbor lists from every shard that owns it,
        // advancing each shard cursor. Streams are sorted by node id, so the
        // cursor only moves forward.
        let mut membership = 0_usize;
        let mut union: BTreeSet<u32> = BTreeSet::new();
        let mut passthrough: Option<Vec<u32>> = None;
        for (shard_index, shard) in shard_graphs.iter().enumerate() {
            let cursor = &mut cursors[shard_index];
            while *cursor < shard.nodes.len() && shard.nodes[*cursor] < node {
                *cursor += 1;
            }
            if *cursor < shard.nodes.len() && shard.nodes[*cursor] == node {
                let neighbors = &shard.neighbors[*cursor];
                membership += 1;
                if membership == 1 {
                    // Remember the first shard's list verbatim in case this is
                    // the only shard (idempotent passthrough).
                    passthrough = Some(neighbors.clone());
                }
                for &neighbor in neighbors {
                    if neighbor != node {
                        union.insert(neighbor);
                    }
                }
                *cursor += 1;
            }
        }

        peak_union_len = peak_union_len.max(union.len());

        let final_neighbors = match membership {
            0 => Vec::new(),
            1 => {
                // Single-shard node: pass the record through unchanged
                // (FR-077 stitch idempotence). Filter any self-loop for safety.
                let mut list = passthrough.unwrap_or_default();
                list.retain(|&n| n != node);
                edges_before += list.len() as u64;
                list
            }
            _ => {
                edges_before += union.len() as u64;
                let candidates: Vec<Candidate> = union
                    .iter()
                    .map(|&n| Candidate {
                        node: n,
                        distance: dist(node, n),
                    })
                    .collect();
                robust_prune(node, candidates, alpha, graph_degree, dist)
            }
        };
        edges_after += final_neighbors.len() as u64;
        graph.neighbors[node as usize] = final_neighbors;
    }

    let stats = ShardBuildStats {
        shard_count: shard_graphs.len(),
        duplication_factor: 1.0,
        max_shard_size: 0,
        stitch_edges_before_prune: edges_before,
        stitch_edges_after_prune: edges_after,
        stitch_peak_union_len: peak_union_len,
        shard_output_retained_node_ids: 0,
        reachability_repairs: 0,
    };
    (graph, stats)
}

/// Guarantee FR-077-CON-3: every node reachable from `medoid`, while never
/// violating FR-077-CON-1 (out-degree ≤ `graph_degree`). Runs BFS over
/// out-edges; each stranded node gets one in-edge from the nearest already-
/// reached node that can accept the edge **without exceeding R** — a node with
/// a free slot (append) or with an evictable non-protected neighbor (replace
/// its farthest such neighbor). Repair edges are protected so a later repair
/// on the same source never re-strands an earlier node (monotone convergence),
/// and reachability is propagated from each repaired node so no node is
/// repaired that is already navigable through a repaired predecessor.
///
/// Such an accepting source always exists for `graph_degree >= 2`: the number
/// of protected repair edges never exceeds the number of reached nodes (each
/// repair marks ≥1 new node reached), which is `< reached_count * graph_degree`
/// total slots, so some reached node always has a free or non-protected slot.
/// `graph_degree` is reloption-bounded to ≥ 4 (`ECDISTANN_MIN_GRAPH_DEGREE`),
/// so the "no accepting source" branch is unreachable in practice; it is an
/// `Err` rather than a silent CON-1 violation. Deterministic (stranded nodes
/// processed ascending). Returns the repair count (expected 0 at corpus scale).
fn repair_reachability<D>(
    graph: &mut VamanaGraph,
    medoid: u32,
    graph_degree: usize,
    dist: &D,
) -> Result<usize, String>
where
    D: Fn(u32, u32) -> f32,
{
    let node_count = graph.node_count();
    if node_count == 0 {
        return Ok(0);
    }
    let reached_order = bfs_reachable(graph, medoid);
    if reached_order.len() == node_count {
        return Ok(0);
    }

    let mut reached = vec![false; node_count];
    for &n in &reached_order {
        reached[n as usize] = true;
    }
    let mut reached_list: Vec<u32> = reached_order;
    // Edges added by the repair must never be evicted, else a later repair on
    // the same source could re-strand an earlier node.
    let mut protected: Vec<Vec<u32>> = vec![Vec::new(); node_count];

    let mut repairs = 0_usize;
    for node in 0..node_count as u32 {
        if reached[node as usize] {
            continue;
        }
        // With reachability propagation, an unreached node has no in-edge from
        // any reached node, so a fresh in-edge is always needed. Pick the
        // nearest reached source that can accept it without exceeding R: it has
        // a free slot, or a non-protected neighbor we can evict.
        let mut best_src: Option<(u32, f32, bool)> = None; // (src, dist, has_room)
        for &candidate in &reached_list {
            let adjacency = &graph.neighbors[candidate as usize];
            let has_room = adjacency.len() < graph_degree;
            let has_evictable = !has_room
                && adjacency
                    .iter()
                    .any(|n| !protected[candidate as usize].contains(n));
            if has_room || has_evictable {
                let d = dist(candidate, node);
                if best_src.map_or(true, |(_, bd, _)| d < bd) {
                    best_src = Some((candidate, d, has_room));
                }
            }
        }

        let (src, has_room) = match best_src {
            Some((src, _, has_room)) => (src, has_room),
            // Unreachable for graph_degree >= 2 (see fn docs); never exceed R.
            None => {
                return Err(format!(
                    "ec_distann reachability repair could not place an in-edge to node {node} \
                     without exceeding graph_degree {graph_degree} (degenerate saturation)"
                ))
            }
        };

        if has_room {
            graph.neighbors[src as usize].push(node);
        } else {
            // Evict the farthest non-protected neighbor (exists: has_evictable).
            let protected_src = &protected[src as usize];
            let victim = graph.neighbors[src as usize]
                .iter()
                .enumerate()
                .filter(|(_, n)| !protected_src.contains(n))
                .map(|(i, &n)| (i, dist(src, n)))
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .expect("has_evictable guarantees a non-protected neighbor");
            graph.neighbors[src as usize][victim] = node;
        }
        protected[src as usize].push(node);
        repairs += 1;
        debug_assert!(graph.neighbors[src as usize].len() <= graph_degree);

        // Propagate reachability from the newly-connected node: everything it
        // can now reach is reached too, so no node navigable through a repaired
        // predecessor is repaired again.
        let mut stack = vec![node];
        reached[node as usize] = true;
        reached_list.push(node);
        while let Some(cur) = stack.pop() {
            for &neighbor in &graph.neighbors[cur as usize] {
                if !reached[neighbor as usize] {
                    reached[neighbor as usize] = true;
                    reached_list.push(neighbor);
                    stack.push(neighbor);
                }
            }
        }
    }

    Ok(repairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    /// Deterministic unit-normalized random corpus for the property suite.
    fn random_corpus(node_count: usize, dimensions: usize, seed: u64) -> Vec<Vec<f32>> {
        let mut rng = StdRng::seed_from_u64(seed);
        (0..node_count)
            .map(|_| {
                let mut v: Vec<f32> = (0..dimensions).map(|_| rng.gen_range(-1.0_f32..1.0)).collect();
                let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-6);
                v.iter_mut().for_each(|x| *x /= norm);
                v
            })
            .collect()
    }

    fn refs(corpus: &[Vec<f32>]) -> Vec<&[f32]> {
        corpus.iter().map(Vec::as_slice).collect()
    }

    fn build(
        corpus: &[Vec<f32>],
        dimensions: usize,
        graph_degree: usize,
        shard_count: usize,
        closure_epsilon: f32,
        seed: u64,
    ) -> (VamanaGraph, u32, ShardBuildStats) {
        let refs = refs(corpus);
        build_sharded_graph(
            &refs,
            dimensions,
            graph_degree,
            /* build_list_size */ 64,
            /* alpha */ 1.2,
            shard_count,
            closure_epsilon,
            seed,
        )
        .expect("sharded build should succeed")
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(48))]

        // FR-077-CON-1: post-stitch out-degree <= graph_degree for every node.
        #[test]
        fn tc038_degree_bounded(
            node_count in 40_usize..160,
            dimensions in 4_usize..12,
            graph_degree in 4_usize..24,
            shard_count in 2_usize..6,
            eps in 0.0_f32..0.4,
            seed in any::<u64>(),
        ) {
            let corpus = random_corpus(node_count, dimensions, seed);
            let (graph, _medoid, _stats) =
                build(&corpus, dimensions, graph_degree, shard_count, eps, seed);
            for node in 0..node_count {
                prop_assert!(
                    graph.neighbors[node].len() <= graph_degree,
                    "node {} degree {} exceeds R {}",
                    node,
                    graph.neighbors[node].len(),
                    graph_degree
                );
            }
        }

        // FR-077-CON-2: every vec_id appears exactly once (the graph has
        // exactly node_count adjacency slots, each a valid distinct node id),
        // and no self-loops or dangling neighbor ids.
        #[test]
        fn tc038_uniqueness_and_valid_edges(
            node_count in 40_usize..160,
            dimensions in 4_usize..12,
            graph_degree in 4_usize..24,
            shard_count in 2_usize..6,
            eps in 0.0_f32..0.4,
            seed in any::<u64>(),
        ) {
            let corpus = random_corpus(node_count, dimensions, seed);
            let (graph, _medoid, _stats) =
                build(&corpus, dimensions, graph_degree, shard_count, eps, seed);
            prop_assert_eq!(graph.node_count(), node_count);
            for node in 0..node_count {
                let neighbors = &graph.neighbors[node];
                let mut seen = std::collections::HashSet::new();
                for &nb in neighbors {
                    prop_assert!((nb as usize) < node_count, "dangling neighbor {}", nb);
                    prop_assert_ne!(nb as usize, node, "self-loop at {}", node);
                    prop_assert!(seen.insert(nb), "duplicate neighbor {} at {}", nb, node);
                }
            }
        }

        // FR-077-CON-3: every node reachable from the entry medoid.
        #[test]
        fn tc038_medoid_reachability(
            node_count in 40_usize..160,
            dimensions in 4_usize..12,
            graph_degree in 4_usize..24,
            shard_count in 2_usize..6,
            eps in 0.0_f32..0.4,
            seed in any::<u64>(),
        ) {
            let corpus = random_corpus(node_count, dimensions, seed);
            let (graph, medoid, _stats) =
                build(&corpus, dimensions, graph_degree, shard_count, eps, seed);
            let reached = bfs_reachable(&graph, medoid);
            prop_assert_eq!(
                reached.len(),
                node_count,
                "only {} of {} nodes reachable from medoid",
                reached.len(),
                node_count
            );
        }

        // FR-077 determinism: identical corpus + seed + options => identical
        // stitched graph.
        #[test]
        fn tc038_determinism(
            node_count in 40_usize..160,
            dimensions in 4_usize..12,
            graph_degree in 4_usize..24,
            shard_count in 2_usize..6,
            eps in 0.0_f32..0.4,
            seed in any::<u64>(),
        ) {
            let corpus = random_corpus(node_count, dimensions, seed);
            let (graph_a, medoid_a, stats_a) =
                build(&corpus, dimensions, graph_degree, shard_count, eps, seed);
            let (graph_b, medoid_b, stats_b) =
                build(&corpus, dimensions, graph_degree, shard_count, eps, seed);
            prop_assert_eq!(medoid_a, medoid_b);
            prop_assert_eq!(stats_a, stats_b);
            prop_assert_eq!(graph_a.neighbors, graph_b.neighbors);
        }

        // FR-077-CON (alpha-prune invariant, TC-038 / spec/tests.md): the
        // stitched *union* preserves robust_prune's alpha-diversity. For every
        // multi-shard node (the nodes the stitch actually re-prunes over their
        // unioned edge lists), the kept neighbor list satisfies, in kept order,
        // that no earlier-kept (closer) neighbor alpha-dominates a later one
        // under the GLOBAL distance: alpha * dist(nb[i], nb[j]) > dist(node,
        // nb[j]) for i < j. Single-shard passthrough nodes carry the per-shard
        // Vamana output (backlink re-prunes reorder that list, so it is not a
        // single-prune kept order) and are exempt; the reachability repair runs
        // after this, so it is checked on the pre-repair stitch output.
        #[test]
        fn tc038_alpha_prune_invariant(
            node_count in 40_usize..160,
            dimensions in 4_usize..12,
            graph_degree in 4_usize..24,
            shard_count in 2_usize..6,
            eps in 0.0_f32..0.4,
            seed in any::<u64>(),
        ) {
            let alpha = 1.2_f32;
            let corpus = random_corpus(node_count, dimensions, seed);
            let refs = refs(&corpus);
            let dist = |a: u32, b: u32| {
                source_inner_product_distance(refs[a as usize], refs[b as usize])
            };
            let assignment =
                assign_shards(&refs, dimensions, shard_count, eps, seed).unwrap();
            // Membership count per global node: the stitch re-prunes exactly the
            // nodes appearing in >= 2 shards.
            let mut membership = vec![0_u32; node_count];
            for shard in &assignment.members {
                for &n in shard {
                    membership[n as usize] += 1;
                }
            }
            let shard_graphs =
                build_shard_graphs(&assignment.members, graph_degree, 64, alpha, seed, &dist);
            let (graph, _stats) =
                stitch_shard_graphs(node_count, &shard_graphs, graph_degree, alpha, &dist);

            let mut checked_multi = 0_usize;
            for node in 0..node_count as u32 {
                if membership[node as usize] < 2 {
                    continue; // passthrough node — exempt (see comment above)
                }
                checked_multi += 1;
                let neighbors = &graph.neighbors[node as usize];
                for j in 0..neighbors.len() {
                    let vj = neighbors[j];
                    let d_node_vj = dist(node, vj);
                    for &vi in neighbors.iter().take(j) {
                        // vi kept before vj => vi must not alpha-dominate vj.
                        let lhs = alpha * dist(vi, vj);
                        prop_assert!(
                            lhs + 1e-4 >= d_node_vj,
                            "alpha-prune violated at node {}: alpha*d({},{})={} < d(node,{})={}",
                            node, vi, vj, lhs, vj, d_node_vj
                        );
                    }
                }
            }
            // `checked_multi` is diagnostic only: some well-separated random
            // corpora legitimately produce no closure overlap even at eps>0,
            // in which case the invariant holds vacuously. Closure duplication
            // is exercised on average by tc038_closure_duplication_scales.
            let _ = checked_multi;
        }
    }

    // FR-077-AC-2: stitching an already-stitched graph is a no-op. Model the
    // stitched graph as a single shard covering all nodes and re-stitch: every
    // node is single-membership, so the passthrough path must reproduce it
    // exactly.
    #[test]
    fn tc038_stitch_idempotence() {
        let dimensions = 8;
        let node_count = 120;
        let graph_degree = 16;
        let corpus = random_corpus(node_count, dimensions, 0xABCD);
        let refs = refs(&corpus);
        let dist = |a: u32, b: u32| source_inner_product_distance(refs[a as usize], refs[b as usize]);

        let (graph, _medoid, _stats) =
            build(&corpus, dimensions, graph_degree, 4, 0.2, 0xABCD);

        // Present the stitched graph as one shard covering every node.
        let single = ShardGraph {
            nodes: (0..node_count as u32).collect(),
            neighbors: graph.neighbors.clone(),
        };
        let (restitched, _stats) =
            stitch_shard_graphs(node_count, std::slice::from_ref(&single), graph_degree, 1.2, &dist);
        assert_eq!(
            restitched.neighbors, graph.neighbors,
            "re-stitching a single-shard graph must be a no-op"
        );
    }

    // Closure overlap actually duplicates nodes when epsilon > 0, and never
    // when epsilon == 0 (each node lands in exactly one shard).
    #[test]
    fn tc038_closure_duplication_scales_with_epsilon() {
        let dimensions = 12;
        let node_count = 300;
        let corpus = random_corpus(node_count, dimensions, 7);

        let (_g0, _m0, stats0) = build(&corpus, dimensions, 16, 6, 0.0, 7);
        assert!(
            (stats0.duplication_factor - 1.0).abs() < 1e-9,
            "epsilon=0 must not duplicate, got {}",
            stats0.duplication_factor
        );

        let (_g1, _m1, stats1) = build(&corpus, dimensions, 16, 6, 0.3, 7);
        assert!(
            stats1.duplication_factor > 1.0,
            "epsilon=0.3 should duplicate some boundary nodes, got {}",
            stats1.duplication_factor
        );
    }

    // resolve_shard_count honors explicit requests and the monolithic default.
    #[test]
    fn resolve_shard_count_behaviour() {
        assert_eq!(resolve_shard_count(10_000, 0), 1, "small corpus auto = monolithic");
        assert_eq!(resolve_shard_count(100_000, 0), 4, "100k auto shards");
        assert_eq!(resolve_shard_count(5_000, 8), 8, "explicit request honored");
        assert_eq!(resolve_shard_count(3, 8), 3, "request clamped to node_count");
        assert_eq!(resolve_shard_count(1_000_000, 0), 16, "auto shard cap");
    }

    // FR-077-CON-1 + CON-3 regression: repair_reachability must connect a fully
    // disconnected component from the medoid WITHOUT any node exceeding
    // graph_degree, even when every reachable source already sits at R (the
    // degenerate saturation the reviewer flagged: the old code appended past R
    // and would have failed staging). Two triangles at R=2, medoid in one.
    #[test]
    fn repair_reachability_bounds_degree_on_disconnected_graph() {
        let graph_degree = 2;
        let node_count = 6;
        let mut graph = VamanaGraph::empty(node_count, graph_degree);
        // Component A {0,1,2} (contains medoid 0), component B {3,4,5}.
        graph.neighbors[0] = vec![1, 2];
        graph.neighbors[1] = vec![0, 2];
        graph.neighbors[2] = vec![0, 1];
        graph.neighbors[3] = vec![4, 5];
        graph.neighbors[4] = vec![3, 5];
        graph.neighbors[5] = vec![3, 4];
        // Every source in component A is already at R=2, so the repair must
        // evict a non-protected neighbor rather than append past R.
        let dist = |a: u32, b: u32| (a as f32 - b as f32).abs();
        let repairs = repair_reachability(&mut graph, 0, graph_degree, &dist)
            .expect("repair must succeed without exceeding R");
        assert!(repairs >= 1, "component B was stranded and must be repaired");

        // CON-1: no node exceeds graph_degree.
        for (node, neighbors) in graph.neighbors.iter().enumerate() {
            assert!(
                neighbors.len() <= graph_degree,
                "node {node} degree {} exceeds R {graph_degree}",
                neighbors.len()
            );
        }
        // CON-3: every node reachable from the medoid.
        let reached = bfs_reachable(&graph, 0);
        assert_eq!(reached.len(), node_count, "all nodes must be reachable post-repair");
    }
}
