use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct GraphNode {
    pub(super) owner_ordinal: u32,
    pub(super) vec_id: u64,
    pub(super) tombstone: bool,
    pub(super) neighbors: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct GraphSeedSet {
    pub(super) name: String,
    pub(super) vec_ids: Vec<u64>,
}

#[derive(Debug, Serialize, PartialEq)]
pub(super) struct GraphDiagnostic {
    schema: &'static str,
    stitch_definition: &'static str,
    owners: Vec<GraphSummary>,
    aggregate: GraphSummary,
    seed_reachability: Vec<SeedReachability>,
}

#[derive(Debug, Serialize, PartialEq)]
struct GraphSummary {
    owner_ordinal: Option<u32>,
    stored_nodes: usize,
    live_nodes: usize,
    tombstones: usize,
    directed_edges: usize,
    local_edges: usize,
    remote_edges: usize,
    stitch_edges: usize,
    invalid_edges: usize,
    duplicate_edges: usize,
    self_edges: usize,
    strongly_connected_components: usize,
    largest_strongly_connected_component: usize,
    weak_components: usize,
    largest_weak_component: usize,
    in_degree: DegreeSummary,
    out_degree: DegreeSummary,
    articulation_candidate_count: usize,
    articulation_candidate_ids: Vec<String>,
    bridge_candidate_count: usize,
    bridge_candidate_edges: Vec<[String; 2]>,
    adjacency_sha256: String,
}

#[derive(Debug, Default, Serialize, PartialEq)]
struct DegreeSummary {
    zero: usize,
    min: usize,
    p50: usize,
    p95: usize,
    p99: usize,
    max: usize,
}

#[derive(Debug, Serialize, PartialEq)]
struct SeedReachability {
    name: String,
    configured_seeds: usize,
    live_seeds: usize,
    missing_seeds: usize,
    reachable_live_nodes: usize,
    unreachable_live_nodes: usize,
    reachable_fraction: f64,
}

pub(super) fn analyze_graph(nodes: &[GraphNode], seed_sets: &[GraphSeedSet]) -> GraphDiagnostic {
    let owners = nodes
        .iter()
        .map(|node| node.owner_ordinal)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|owner| summarize(nodes, Some(owner)))
        .collect();
    let aggregate = summarize(nodes, None);
    let live = live_graph(nodes, None);
    let seed_reachability = seed_sets
        .iter()
        .map(|seed_set| seed_reachability(&live, seed_set))
        .collect();
    GraphDiagnostic {
        schema: "ec_distann_graph_diagnostic_v1",
        stitch_definition: "persisted cross-owner directed edge",
        owners,
        aggregate,
        seed_reachability,
    }
}

fn summarize(nodes: &[GraphNode], owner: Option<u32>) -> GraphSummary {
    let selected = nodes
        .iter()
        .filter(|node| owner.map_or(true, |owner| node.owner_ordinal == owner))
        .collect::<Vec<_>>();
    let all_live_owners = nodes
        .iter()
        .filter(|node| !node.tombstone)
        .map(|node| (node.vec_id, node.owner_ordinal))
        .collect::<HashMap<_, _>>();
    let selected_live = selected
        .iter()
        .filter(|node| !node.tombstone)
        .map(|node| node.vec_id)
        .collect::<HashSet<_>>();
    let live = live_graph(nodes, owner);
    let mut invalid_edges = 0_usize;
    let mut duplicate_edges = 0_usize;
    let mut self_edges = 0_usize;
    let mut local_edges = 0_usize;
    let mut remote_edges = 0_usize;
    let mut directed_edges = 0_usize;
    let mut in_degree = vec![0_usize; live.ids.len()];
    let mut out_degree = vec![0_usize; live.ids.len()];
    for node in &selected {
        if node.tombstone {
            continue;
        }
        let mut seen = HashSet::new();
        for neighbor in &node.neighbors {
            if !seen.insert(*neighbor) {
                duplicate_edges = duplicate_edges.saturating_add(1);
                continue;
            }
            if *neighbor == node.vec_id {
                self_edges = self_edges.saturating_add(1);
            }
            let Some(neighbor_owner) = all_live_owners.get(neighbor) else {
                invalid_edges = invalid_edges.saturating_add(1);
                continue;
            };
            directed_edges = directed_edges.saturating_add(1);
            if let Some(index) = live.index.get(&node.vec_id) {
                out_degree[*index] = out_degree[*index].saturating_add(1);
            }
            if *neighbor_owner == node.owner_ordinal {
                local_edges = local_edges.saturating_add(1);
            } else {
                remote_edges = remote_edges.saturating_add(1);
            }
        }
    }
    for node in nodes.iter().filter(|node| !node.tombstone) {
        let mut seen = HashSet::new();
        for neighbor in &node.neighbors {
            if seen.insert(*neighbor) && selected_live.contains(neighbor) {
                if let Some(index) = live.index.get(neighbor) {
                    in_degree[*index] = in_degree[*index].saturating_add(1);
                }
            }
        }
    }
    let (scc_count, largest_scc) = strongly_connected_components(&live.adjacency);
    let undirected = undirected_adjacency(&live.adjacency);
    let (weak_count, largest_weak) = weak_components(&undirected);
    let (articulations, bridges) = articulation_and_bridges(&undirected);
    GraphSummary {
        owner_ordinal: owner,
        stored_nodes: selected.len(),
        live_nodes: live.ids.len(),
        tombstones: selected.iter().filter(|node| node.tombstone).count(),
        directed_edges,
        local_edges,
        remote_edges,
        stitch_edges: remote_edges,
        invalid_edges,
        duplicate_edges,
        self_edges,
        strongly_connected_components: scc_count,
        largest_strongly_connected_component: largest_scc,
        weak_components: weak_count,
        largest_weak_component: largest_weak,
        in_degree: degree_summary(&in_degree),
        out_degree: degree_summary(&out_degree),
        articulation_candidate_count: articulations.len(),
        articulation_candidate_ids: articulations
            .iter()
            .take(64)
            .map(|index| format!("{:016x}", live.ids[*index]))
            .collect(),
        bridge_candidate_count: bridges.len(),
        bridge_candidate_edges: bridges
            .iter()
            .take(64)
            .map(|(left, right)| {
                [
                    format!("{:016x}", live.ids[*left]),
                    format!("{:016x}", live.ids[*right]),
                ]
            })
            .collect(),
        adjacency_sha256: adjacency_digest(&selected),
    }
}

struct LiveGraph {
    ids: Vec<u64>,
    index: HashMap<u64, usize>,
    adjacency: Vec<Vec<usize>>,
}

fn live_graph(nodes: &[GraphNode], owner: Option<u32>) -> LiveGraph {
    let selected = nodes
        .iter()
        .filter(|node| {
            !node.tombstone && owner.map_or(true, |owner| node.owner_ordinal == owner)
        })
        .collect::<Vec<_>>();
    let mut ids = selected.iter().map(|node| node.vec_id).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    let index = ids
        .iter()
        .enumerate()
        .map(|(index, vec_id)| (*vec_id, index))
        .collect::<HashMap<_, _>>();
    let by_id = selected
        .into_iter()
        .map(|node| (node.vec_id, node))
        .collect::<BTreeMap<_, _>>();
    let adjacency = ids
        .iter()
        .map(|vec_id| {
            let mut neighbors = by_id
                .get(vec_id)
                .into_iter()
                .flat_map(|node| node.neighbors.iter())
                .filter_map(|neighbor| index.get(neighbor).copied())
                .collect::<Vec<_>>();
            neighbors.sort_unstable();
            neighbors.dedup();
            neighbors
        })
        .collect();
    LiveGraph {
        ids,
        index,
        adjacency,
    }
}

fn strongly_connected_components(adjacency: &[Vec<usize>]) -> (usize, usize) {
    let count = adjacency.len();
    let mut reverse = vec![Vec::new(); count];
    for (source, neighbors) in adjacency.iter().enumerate() {
        for target in neighbors {
            reverse[*target].push(source);
        }
    }
    let mut visited = vec![false; count];
    let mut order = Vec::with_capacity(count);
    for start in 0..count {
        if visited[start] {
            continue;
        }
        let mut stack = vec![(start, false)];
        while let Some((node, exiting)) = stack.pop() {
            if exiting {
                order.push(node);
                continue;
            }
            if visited[node] {
                continue;
            }
            visited[node] = true;
            stack.push((node, true));
            for neighbor in adjacency[node].iter().rev() {
                if !visited[*neighbor] {
                    stack.push((*neighbor, false));
                }
            }
        }
    }
    visited.fill(false);
    let mut components = 0_usize;
    let mut largest = 0_usize;
    while let Some(start) = order.pop() {
        if visited[start] {
            continue;
        }
        components += 1;
        let mut size = 0_usize;
        let mut stack = vec![start];
        visited[start] = true;
        while let Some(node) = stack.pop() {
            size += 1;
            for neighbor in &reverse[node] {
                if !visited[*neighbor] {
                    visited[*neighbor] = true;
                    stack.push(*neighbor);
                }
            }
        }
        largest = largest.max(size);
    }
    (components, largest)
}

fn undirected_adjacency(adjacency: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut undirected = vec![BTreeSet::new(); adjacency.len()];
    for (source, neighbors) in adjacency.iter().enumerate() {
        for target in neighbors {
            if source != *target {
                undirected[source].insert(*target);
                undirected[*target].insert(source);
            }
        }
    }
    undirected
        .into_iter()
        .map(|neighbors| neighbors.into_iter().collect())
        .collect()
}

fn weak_components(adjacency: &[Vec<usize>]) -> (usize, usize) {
    let mut visited = vec![false; adjacency.len()];
    let mut components = 0_usize;
    let mut largest = 0_usize;
    for start in 0..adjacency.len() {
        if visited[start] {
            continue;
        }
        components += 1;
        let mut size = 0_usize;
        let mut stack = vec![start];
        visited[start] = true;
        while let Some(node) = stack.pop() {
            size += 1;
            for neighbor in &adjacency[node] {
                if !visited[*neighbor] {
                    visited[*neighbor] = true;
                    stack.push(*neighbor);
                }
            }
        }
        largest = largest.max(size);
    }
    (components, largest)
}

fn articulation_and_bridges(adjacency: &[Vec<usize>]) -> (BTreeSet<usize>, Vec<(usize, usize)>) {
    let count = adjacency.len();
    let missing = usize::MAX;
    let mut discovered = vec![missing; count];
    let mut low = vec![0_usize; count];
    let mut parent = vec![missing; count];
    let mut children = vec![0_usize; count];
    let mut clock = 0_usize;
    let mut articulations = BTreeSet::new();
    let mut bridges = Vec::new();
    for root in 0..count {
        if discovered[root] != missing {
            continue;
        }
        discovered[root] = clock;
        low[root] = clock;
        clock += 1;
        let mut stack = vec![(root, 0_usize)];
        while let Some((node, next)) = stack.last_mut() {
            if *next < adjacency[*node].len() {
                let neighbor = adjacency[*node][*next];
                *next += 1;
                if discovered[neighbor] == missing {
                    parent[neighbor] = *node;
                    children[*node] += 1;
                    discovered[neighbor] = clock;
                    low[neighbor] = clock;
                    clock += 1;
                    stack.push((neighbor, 0));
                } else if neighbor != parent[*node] {
                    low[*node] = low[*node].min(discovered[neighbor]);
                }
                continue;
            }
            let finished = *node;
            stack.pop();
            let ancestor = parent[finished];
            if ancestor == missing {
                if children[finished] > 1 {
                    articulations.insert(finished);
                }
                continue;
            }
            low[ancestor] = low[ancestor].min(low[finished]);
            if low[finished] > discovered[ancestor] {
                bridges.push((ancestor.min(finished), ancestor.max(finished)));
            }
            if parent[ancestor] != missing && low[finished] >= discovered[ancestor] {
                articulations.insert(ancestor);
            }
        }
    }
    bridges.sort_unstable();
    bridges.dedup();
    (articulations, bridges)
}

fn seed_reachability(graph: &LiveGraph, seed_set: &GraphSeedSet) -> SeedReachability {
    let seeds = seed_set
        .vec_ids
        .iter()
        .filter_map(|vec_id| graph.index.get(vec_id).copied())
        .collect::<BTreeSet<_>>();
    let mut visited = vec![false; graph.ids.len()];
    let mut queue = VecDeque::new();
    for seed in &seeds {
        visited[*seed] = true;
        queue.push_back(*seed);
    }
    while let Some(node) = queue.pop_front() {
        for neighbor in &graph.adjacency[node] {
            if !visited[*neighbor] {
                visited[*neighbor] = true;
                queue.push_back(*neighbor);
            }
        }
    }
    let reachable = visited.iter().filter(|value| **value).count();
    SeedReachability {
        name: seed_set.name.clone(),
        configured_seeds: seed_set.vec_ids.len(),
        live_seeds: seeds.len(),
        missing_seeds: seed_set.vec_ids.len().saturating_sub(seeds.len()),
        reachable_live_nodes: reachable,
        unreachable_live_nodes: graph.ids.len().saturating_sub(reachable),
        reachable_fraction: if graph.ids.is_empty() {
            1.0
        } else {
            reachable as f64 / graph.ids.len() as f64
        },
    }
}

fn degree_summary(values: &[usize]) -> DegreeSummary {
    if values.is_empty() {
        return DegreeSummary::default();
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let percentile = |numerator: usize, denominator: usize| {
        let rank = sorted
            .len()
            .saturating_mul(numerator)
            .div_ceil(denominator)
            .saturating_sub(1)
            .min(sorted.len() - 1);
        sorted[rank]
    };
    DegreeSummary {
        zero: sorted.iter().filter(|value| **value == 0).count(),
        min: sorted[0],
        p50: percentile(50, 100),
        p95: percentile(95, 100),
        p99: percentile(99, 100),
        max: *sorted.last().expect("non-empty degree list"),
    }
}

fn adjacency_digest(nodes: &[&GraphNode]) -> String {
    let mut ordered = nodes.to_vec();
    ordered.sort_unstable_by_key(|node| (node.owner_ordinal, node.vec_id));
    let mut hasher = Sha256::new();
    for node in ordered {
        hasher.update(node.owner_ordinal.to_le_bytes());
        hasher.update(node.vec_id.to_le_bytes());
        hasher.update([u8::from(node.tombstone)]);
        let mut neighbors = node.neighbors.clone();
        neighbors.sort_unstable();
        hasher.update((neighbors.len() as u64).to_le_bytes());
        for neighbor in neighbors {
            hasher.update(neighbor.to_le_bytes());
        }
    }
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::{analyze_graph, GraphNode, GraphSeedSet};

    fn node(owner: u32, vec_id: u64, neighbors: &[u64]) -> GraphNode {
        GraphNode {
            owner_ordinal: owner,
            vec_id,
            tombstone: false,
            neighbors: neighbors.to_vec(),
        }
    }

    #[test]
    fn graph_diagnostic_is_deterministic_and_classifies_structure() {
        let nodes = vec![
            node(0, 1, &[2, 2, 1, 99]),
            node(0, 2, &[1, 3]),
            node(1, 3, &[4]),
            node(1, 4, &[3]),
            node(1, 5, &[]),
        ];
        let seeds = vec![GraphSeedSet {
            name: "persisted_head".to_owned(),
            vec_ids: vec![1, 88],
        }];
        let diagnostic = analyze_graph(&nodes, &seeds);
        assert_eq!(diagnostic.aggregate.live_nodes, 5);
        assert_eq!(diagnostic.aggregate.invalid_edges, 1);
        assert_eq!(diagnostic.aggregate.duplicate_edges, 1);
        assert_eq!(diagnostic.aggregate.self_edges, 1);
        assert_eq!(diagnostic.aggregate.remote_edges, 1);
        assert_eq!(diagnostic.aggregate.weak_components, 2);
        assert_eq!(diagnostic.aggregate.strongly_connected_components, 3);
        assert_eq!(diagnostic.aggregate.articulation_candidate_count, 2);
        assert_eq!(diagnostic.aggregate.bridge_candidate_count, 3);
        assert_eq!(diagnostic.seed_reachability[0].live_seeds, 1);
        assert_eq!(diagnostic.seed_reachability[0].missing_seeds, 1);
        assert_eq!(diagnostic.seed_reachability[0].reachable_live_nodes, 4);

        let mut reordered = nodes.clone();
        reordered.reverse();
        for node in &mut reordered {
            node.neighbors.reverse();
        }
        let reordered = analyze_graph(&reordered, &seeds);
        assert_eq!(
            diagnostic.aggregate.adjacency_sha256,
            reordered.aggregate.adjacency_sha256
        );
    }

    #[test]
    fn graph_diagnostic_handles_deep_chain_without_recursion() {
        let nodes = (0..20_000_u64)
            .map(|vec_id| {
                node(
                    0,
                    vec_id,
                    (vec_id + 1 < 20_000).then_some(vec_id + 1).as_slice(),
                )
            })
            .collect::<Vec<_>>();
        let diagnostic = analyze_graph(&nodes, &[]);
        assert_eq!(diagnostic.aggregate.live_nodes, 20_000);
        assert_eq!(diagnostic.aggregate.weak_components, 1);
        assert_eq!(diagnostic.aggregate.strongly_connected_components, 20_000);
        assert_eq!(diagnostic.aggregate.bridge_candidate_count, 19_999);
        assert_eq!(diagnostic.aggregate.articulation_candidate_count, 19_998);
    }
}
