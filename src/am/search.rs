use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashSet};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeamCandidate<NodeId> {
    pub node: NodeId,
    pub score: f32,
    pub source: Option<NodeId>,
}

impl<NodeId> BeamCandidate<NodeId> {
    pub fn new(node: NodeId, score: f32) -> Self {
        Self {
            node,
            score,
            source: None,
        }
    }

    pub fn with_source(node: NodeId, score: f32, source: NodeId) -> Self {
        Self {
            node,
            score,
            source: Some(source),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeamTrace<NodeId> {
    pub discovered: Vec<BeamCandidate<NodeId>>,
    pub expanded: Vec<BeamCandidate<NodeId>>,
    pub frontier: Vec<BeamCandidate<NodeId>>,
}

#[derive(Debug, Clone)]
pub struct BeamSearch<NodeId> {
    ef_search: usize,
    frontier: BinaryHeap<Reverse<QueuedCandidate<NodeId>>>,
    visited: HashSet<NodeId>,
    discovery_order: Vec<BeamCandidate<NodeId>>,
    sequence: u64,
}

impl<NodeId> BeamSearch<NodeId>
where
    NodeId: Copy + Eq + Hash,
{
    pub fn new(ef_search: usize) -> Self {
        Self {
            ef_search,
            frontier: BinaryHeap::new(),
            visited: HashSet::new(),
            discovery_order: Vec::new(),
            sequence: 0,
        }
    }

    pub fn seed(&mut self, candidate: BeamCandidate<NodeId>) -> bool {
        self.push_candidate(candidate)
    }

    pub fn visited_count(&self) -> usize {
        self.visited.len()
    }

    pub fn discovered(&self) -> &[BeamCandidate<NodeId>] {
        &self.discovery_order
    }

    pub fn pop_best(&mut self) -> Option<BeamCandidate<NodeId>> {
        self.frontier.pop().map(|Reverse(queued)| queued.candidate)
    }

    pub fn run<NeighborFn, NeighborIter>(&mut self, mut neighbors: NeighborFn) -> BeamTrace<NodeId>
    where
        NeighborFn: FnMut(&BeamCandidate<NodeId>) -> NeighborIter,
        NeighborIter: IntoIterator<Item = BeamCandidate<NodeId>>,
    {
        let mut expanded = Vec::new();
        while expanded.len() < self.ef_search {
            let Some(candidate) = self.pop_best() else {
                break;
            };

            expanded.push(candidate);
            for neighbor in neighbors(&candidate) {
                self.push_candidate(neighbor);
            }
        }

        BeamTrace {
            discovered: self.discovery_order.clone(),
            expanded,
            frontier: self.snapshot_frontier(),
        }
    }

    fn push_candidate(&mut self, candidate: BeamCandidate<NodeId>) -> bool {
        if !self.visited.insert(candidate.node) {
            return false;
        }

        self.discovery_order.push(candidate);
        self.frontier
            .push(Reverse(QueuedCandidate::new(candidate, self.sequence)));
        self.sequence += 1;
        true
    }

    fn snapshot_frontier(&self) -> Vec<BeamCandidate<NodeId>> {
        let mut frontier = self
            .frontier
            .clone()
            .into_vec()
            .into_iter()
            .map(|Reverse(queued)| queued.candidate)
            .collect::<Vec<_>>();
        frontier.sort_by(candidate_order);
        frontier
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct QueuedCandidate<NodeId> {
    candidate: BeamCandidate<NodeId>,
    sequence: u64,
}

impl<NodeId> QueuedCandidate<NodeId> {
    fn new(candidate: BeamCandidate<NodeId>, sequence: u64) -> Self {
        Self {
            candidate,
            sequence,
        }
    }
}

impl<NodeId: PartialEq> Eq for QueuedCandidate<NodeId> {}

impl<NodeId: PartialEq> Ord for QueuedCandidate<NodeId> {
    fn cmp(&self, other: &Self) -> Ordering {
        candidate_order(&self.candidate, &other.candidate)
            .then_with(|| self.sequence.cmp(&other.sequence))
    }
}

impl<NodeId: PartialEq> PartialOrd for QueuedCandidate<NodeId> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn candidate_order<NodeId>(
    left: &BeamCandidate<NodeId>,
    right: &BeamCandidate<NodeId>,
) -> Ordering {
    left.score.total_cmp(&right.score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn mock_graph() -> (HashMap<u64, Vec<u64>>, HashMap<u64, f32>) {
        let mut edges = HashMap::new();
        edges.insert(1, vec![2, 3, 4]);
        edges.insert(2, vec![4, 5]);
        edges.insert(3, vec![5, 6]);
        edges.insert(4, vec![6]);
        edges.insert(5, vec![7]);
        edges.insert(6, vec![7]);
        edges.insert(7, Vec::new());

        let mut scores = HashMap::new();
        scores.insert(1, 0.9);
        scores.insert(2, 0.7);
        scores.insert(3, 0.2);
        scores.insert(4, 0.5);
        scores.insert(5, 0.4);
        scores.insert(6, 0.1);
        scores.insert(7, 0.05);

        (edges, scores)
    }

    #[test]
    fn beam_search_expands_best_first_from_seeds() {
        let (edges, scores) = mock_graph();
        let mut search = BeamSearch::new(4);
        search.seed(BeamCandidate::new(1, scores[&1]));

        let trace = search.run(|candidate| {
            edges[&candidate.node]
                .iter()
                .copied()
                .map(|node| BeamCandidate::with_source(node, scores[&node], candidate.node))
                .collect::<Vec<_>>()
        });

        assert_eq!(
            trace
                .expanded
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![1, 3, 6, 7],
            "expansion should follow the lowest-score frontier nodes first"
        );
        assert_eq!(
            trace
                .discovered
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 6, 7],
            "discovery should keep unique nodes only, even when multiple parents reach the same node"
        );
        assert_eq!(
            trace
                .frontier
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![5, 4, 2],
            "remaining frontier should stay sorted from best to worst score"
        );
    }

    #[test]
    fn beam_search_deduplicates_self_loops_and_parallel_edges() {
        let mut edges = HashMap::new();
        edges.insert(10, vec![10, 11, 11, 12]);
        edges.insert(11, vec![12, 13]);
        edges.insert(12, vec![13]);
        edges.insert(13, Vec::new());

        let mut scores = HashMap::new();
        scores.insert(10, 0.6);
        scores.insert(11, 0.3);
        scores.insert(12, 0.2);
        scores.insert(13, 0.1);

        let mut search = BeamSearch::new(8);
        search.seed(BeamCandidate::new(10, scores[&10]));

        let trace = search.run(|candidate| {
            edges[&candidate.node]
                .iter()
                .copied()
                .map(|node| BeamCandidate::with_source(node, scores[&node], candidate.node))
                .collect::<Vec<_>>()
        });

        assert_eq!(
            trace
                .discovered
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![10, 11, 12, 13],
            "duplicate edges and self-loops should not create duplicate discoveries"
        );
        assert_eq!(
            search.visited_count(),
            4,
            "visited tracking should only count unique nodes"
        );
    }

    #[test]
    fn beam_search_respects_expansion_budget() {
        let mut edges = HashMap::new();
        edges.insert(1, vec![2, 3]);
        edges.insert(2, vec![4]);
        edges.insert(3, vec![5]);
        edges.insert(4, Vec::new());
        edges.insert(5, Vec::new());

        let mut scores = HashMap::new();
        scores.insert(1, 0.9);
        scores.insert(2, 0.8);
        scores.insert(3, 0.1);
        scores.insert(4, 0.2);
        scores.insert(5, 0.05);

        let mut search = BeamSearch::new(2);
        search.seed(BeamCandidate::new(1, scores[&1]));

        let trace = search.run(|candidate| {
            edges[&candidate.node]
                .iter()
                .copied()
                .map(|node| BeamCandidate::with_source(node, scores[&node], candidate.node))
                .collect::<Vec<_>>()
        });

        assert_eq!(
            trace
                .expanded
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![1, 3],
            "ef_search should cap the number of expanded nodes"
        );
        assert_eq!(
            trace
                .frontier
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![5, 2],
            "the remaining frontier should preserve the unexpanded best candidates discovered before the budget expired"
        );
    }
}
