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

#[derive(Debug, Clone, PartialEq)]
pub struct VisibleFrontier<NodeId> {
    candidates: Vec<BeamCandidate<NodeId>>,
}

impl<NodeId> VisibleFrontier<NodeId> {
    pub const fn empty() -> Self {
        Self {
            candidates: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn clear(&mut self) {
        self.candidates.clear();
    }

    pub fn push(&mut self, candidate: impl Into<BeamCandidate<NodeId>>) {
        self.candidates.push(candidate.into());
    }

    pub fn extend<I, C>(&mut self, candidates: I)
    where
        I: IntoIterator<Item = C>,
        C: Into<BeamCandidate<NodeId>>,
    {
        self.candidates
            .extend(candidates.into_iter().map(Into::into));
    }
}

impl<NodeId> Default for VisibleFrontier<NodeId> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<NodeId> VisibleFrontier<NodeId>
where
    NodeId: Copy,
{
    pub fn iter(&self) -> impl Iterator<Item = BeamCandidate<NodeId>> + '_ {
        self.candidates.iter().copied()
    }

    pub fn slot(&self, index: usize) -> Option<BeamCandidate<NodeId>> {
        self.candidates.get(index).copied()
    }
}

impl<NodeId> VisibleFrontier<NodeId>
where
    NodeId: Copy + Eq + Hash,
{
    pub fn contains_node(&self, node: NodeId) -> bool {
        self.candidates
            .iter()
            .any(|candidate| candidate.node == node)
    }

    pub fn best_candidate(
        &self,
        expansion: &mut BeamSearch<NodeId>,
    ) -> Option<BeamCandidate<NodeId>> {
        if let Some(candidate) = expansion.peek_best_matching(|node| self.contains_node(node)) {
            return Some(candidate);
        }

        // The scheduler tracks queued traversal sources, not every still-visible result
        // candidate, so a fully drained scheduler must fall back to the visible Vec itself.
        self.best_candidate_by_score()
    }

    pub fn consume_best(
        &mut self,
        expansion: &mut BeamSearch<NodeId>,
    ) -> Option<BeamCandidate<NodeId>> {
        if let Some(candidate) = expansion.take_best_matching(|node| self.contains_node(node)) {
            return self.remove_node(candidate.node);
        }

        let head = self.best_candidate_by_score()?;
        self.remove_node(head.node)
    }

    fn best_candidate_by_score(&self) -> Option<BeamCandidate<NodeId>> {
        self.candidates
            .iter()
            .min_by(|left, right| left.score.total_cmp(&right.score))
            .copied()
    }

    fn remove_node(&mut self, node: NodeId) -> Option<BeamCandidate<NodeId>> {
        let index = self
            .candidates
            .iter()
            .position(|candidate| candidate.node == node)?;
        Some(self.candidates.remove(index))
    }
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

    pub fn seed_many<I>(&mut self, candidates: I) -> usize
    where
        I: IntoIterator<Item = BeamCandidate<NodeId>>,
    {
        candidates
            .into_iter()
            .filter(|candidate| self.push_candidate(*candidate))
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.frontier.is_empty()
    }

    pub fn frontier_len(&self) -> usize {
        self.frontier.len()
    }

    pub fn visited_count(&self) -> usize {
        self.visited.len()
    }

    pub fn discovered(&self) -> &[BeamCandidate<NodeId>] {
        &self.discovery_order
    }

    pub fn peek_best(&self) -> Option<BeamCandidate<NodeId>> {
        self.frontier.peek().map(|Reverse(queued)| queued.candidate)
    }

    pub fn peek_best_matching<MatchFn>(
        &mut self,
        mut matches: MatchFn,
    ) -> Option<BeamCandidate<NodeId>>
    where
        MatchFn: FnMut(NodeId) -> bool,
    {
        loop {
            let best = self.peek_best()?;
            if matches(best.node) {
                return Some(best);
            }

            self.forget_queued(best.node);
        }
    }

    pub fn take_best_matching<MatchFn>(
        &mut self,
        mut matches: MatchFn,
    ) -> Option<BeamCandidate<NodeId>>
    where
        MatchFn: FnMut(NodeId) -> bool,
    {
        loop {
            let best = self.peek_best()?;
            let removed = self.forget_queued(best.node)?;
            if matches(removed.node) {
                return Some(removed);
            }
        }
    }

    pub fn forget_queued(&mut self, node: NodeId) -> Option<BeamCandidate<NodeId>> {
        let mut removed = None;
        let retained = self
            .frontier
            .drain()
            .filter_map(|Reverse(queued)| {
                if queued.candidate.node == node {
                    removed = Some(queued.candidate);
                    None
                } else {
                    Some(Reverse(queued))
                }
            })
            .collect::<Vec<_>>();
        self.frontier = retained.into();

        if let Some(candidate) = removed {
            self.visited.remove(&node);
            self.discovery_order
                .retain(|discovered| discovered.node != node);
            return Some(candidate);
        }

        None
    }

    pub fn pop_best(&mut self) -> Option<BeamCandidate<NodeId>> {
        self.frontier.pop().map(|Reverse(queued)| queued.candidate)
    }

    pub fn expand_one<NeighborFn, NeighborIter>(
        &mut self,
        mut neighbors: NeighborFn,
    ) -> Option<BeamCandidate<NodeId>>
    where
        NeighborFn: FnMut(&BeamCandidate<NodeId>) -> NeighborIter,
        NeighborIter: IntoIterator<Item = BeamCandidate<NodeId>>,
    {
        let candidate = self.pop_best()?;
        for neighbor in neighbors(&candidate) {
            self.push_candidate(neighbor);
        }
        Some(candidate)
    }

    pub fn run<NeighborFn, NeighborIter>(&mut self, mut neighbors: NeighborFn) -> BeamTrace<NodeId>
    where
        NeighborFn: FnMut(&BeamCandidate<NodeId>) -> NeighborIter,
        NeighborIter: IntoIterator<Item = BeamCandidate<NodeId>>,
    {
        let mut expanded = Vec::new();
        while expanded.len() < self.ef_search {
            let Some(candidate) = self.expand_one(|candidate| neighbors(candidate)) else {
                break;
            };

            expanded.push(candidate);
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

    #[test]
    fn beam_search_seed_many_accepts_unique_candidates_only() {
        let mut search = BeamSearch::new(4);
        let seeded = search.seed_many([
            BeamCandidate::new(1_u64, 0.9),
            BeamCandidate::new(3_u64, 0.2),
            BeamCandidate::new(1_u64, 0.1),
            BeamCandidate::new(2_u64, 0.7),
        ]);

        assert_eq!(
            seeded, 3,
            "seed_many should count only newly accepted nodes"
        );
        assert_eq!(
            search
                .discovered()
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![1, 3, 2],
            "discovery order should preserve the first accepted instance of each node"
        );
        assert_eq!(
            search
                .snapshot_frontier()
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![3, 2, 1],
            "frontier snapshot should stay sorted best-first after multi-seed initialization"
        );
    }

    #[test]
    fn beam_search_expand_one_matches_incremental_best_first_progress() {
        let (edges, scores) = mock_graph();
        let mut search = BeamSearch::new(4);
        search.seed(BeamCandidate::new(1, scores[&1]));

        assert!(
            !search.is_empty(),
            "seeded search should expose non-empty frontier"
        );
        assert_eq!(
            search.peek_best().map(|candidate| candidate.node),
            Some(1),
            "peek_best should expose the current best frontier candidate without consuming it"
        );

        let first = search
            .expand_one(|candidate| {
                edges[&candidate.node]
                    .iter()
                    .copied()
                    .map(|node| BeamCandidate::with_source(node, scores[&node], candidate.node))
                    .collect::<Vec<_>>()
            })
            .expect("first expansion should consume the entry seed");
        assert_eq!(first.node, 1);
        assert_eq!(
            search.peek_best().map(|candidate| candidate.node),
            Some(3),
            "after expanding the seed, the next-best discovered neighbor should become the frontier head"
        );
        assert_eq!(
            search
                .snapshot_frontier()
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![3, 4, 2],
            "incremental expansion should keep the frontier sorted best-first"
        );

        let second = search
            .expand_one(|candidate| {
                edges[&candidate.node]
                    .iter()
                    .copied()
                    .map(|node| BeamCandidate::with_source(node, scores[&node], candidate.node))
                    .collect::<Vec<_>>()
            })
            .expect("second expansion should consume the current best frontier node");
        assert_eq!(second.node, 3);
        assert_eq!(
            search
                .snapshot_frontier()
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![6, 5, 4, 2],
            "second incremental expansion should discover successors and preserve best-first order"
        );
    }

    #[test]
    fn beam_search_run_matches_repeated_expand_one() {
        let (edges, scores) = mock_graph();

        let mut incremental = BeamSearch::new(4);
        incremental.seed(BeamCandidate::new(1, scores[&1]));
        let mut expanded = Vec::new();
        while expanded.len() < 4 {
            let Some(candidate) = incremental.expand_one(|candidate| {
                edges[&candidate.node]
                    .iter()
                    .copied()
                    .map(|node| BeamCandidate::with_source(node, scores[&node], candidate.node))
                    .collect::<Vec<_>>()
            }) else {
                break;
            };
            expanded.push(candidate);
        }

        let mut batch = BeamSearch::new(4);
        batch.seed(BeamCandidate::new(1, scores[&1]));
        let trace = batch.run(|candidate| {
            edges[&candidate.node]
                .iter()
                .copied()
                .map(|node| BeamCandidate::with_source(node, scores[&node], candidate.node))
                .collect::<Vec<_>>()
        });

        assert_eq!(
            expanded, trace.expanded,
            "run() should remain equivalent to repeated expand_one() steps"
        );
        assert_eq!(
            incremental.snapshot_frontier(),
            trace.frontier,
            "incremental stepping should leave the same remaining frontier as run()"
        );
        assert_eq!(
            incremental.discovered(),
            trace.discovered.as_slice(),
            "incremental stepping should preserve the same discovery order as run()"
        );
    }

    #[test]
    fn beam_search_forget_queued_removes_frontier_node_and_allows_reseed() {
        let mut search = BeamSearch::new(4);
        search.seed_many([
            BeamCandidate::new(1_u64, 0.9),
            BeamCandidate::new(3_u64, 0.2),
            BeamCandidate::new(2_u64, 0.7),
        ]);

        let removed = search
            .forget_queued(3)
            .expect("queued node should be removable from the frontier");
        assert_eq!(removed, BeamCandidate::new(3_u64, 0.2));
        assert_eq!(
            search
                .snapshot_frontier()
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![2, 1],
            "forgetting a queued node should remove it from the best-first frontier ordering"
        );
        assert_eq!(
            search
                .discovered()
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![1, 2],
            "forgetting a queued node should remove it from scheduler discovery state"
        );
        assert_eq!(
            search.visited_count(),
            2,
            "forgetting a queued node should release its visited slot"
        );

        assert!(
            search.seed(BeamCandidate::new(3_u64, 0.1)),
            "forgotten queued nodes should be seedable again"
        );
        assert_eq!(
            search.peek_best().map(|candidate| candidate.node),
            Some(3),
            "reseeding a forgotten node should restore it to the frontier with its new score"
        );
    }

    #[test]
    fn beam_search_forget_queued_ignores_expanded_or_unknown_nodes() {
        let (edges, scores) = mock_graph();
        let mut search = BeamSearch::new(4);
        search.seed(BeamCandidate::new(1, scores[&1]));

        let expanded = search
            .expand_one(|candidate| {
                edges[&candidate.node]
                    .iter()
                    .copied()
                    .map(|node| BeamCandidate::with_source(node, scores[&node], candidate.node))
                    .collect::<Vec<_>>()
            })
            .expect("seed should expand");
        assert_eq!(expanded.node, 1);

        assert_eq!(
            search.forget_queued(1),
            None,
            "expanded nodes are no longer queued and should not be forgotten by the queued-node API"
        );
        assert_eq!(
            search.forget_queued(99),
            None,
            "unknown nodes should leave scheduler state untouched"
        );
        assert_eq!(
            search
                .snapshot_frontier()
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![3, 4, 2],
            "non-removals should preserve existing frontier order"
        );
        assert_eq!(
            search.visited_count(),
            4,
            "non-removals should preserve visited accounting"
        );
    }

    #[test]
    fn beam_search_peek_best_matching_skips_stale_leaders() {
        let mut search = BeamSearch::new(4);
        search.seed_many([
            BeamCandidate::new(1_u64, 0.1),
            BeamCandidate::new(2_u64, 0.2),
            BeamCandidate::new(3_u64, 0.3),
        ]);

        let best = search.peek_best_matching(|node| node != 1);
        assert_eq!(
            best,
            Some(BeamCandidate::new(2_u64, 0.2)),
            "stale unmatched leaders should be dropped until a live candidate remains"
        );
        assert_eq!(
            search.snapshot_frontier(),
            vec![
                BeamCandidate::new(2_u64, 0.2),
                BeamCandidate::new(3_u64, 0.3)
            ],
            "dropping stale leaders should also prune them from the queued frontier"
        );
    }

    #[test]
    fn beam_search_peek_best_matching_returns_none_after_dropping_fully_stale_frontier() {
        let mut search = BeamSearch::new(4);
        search.seed_many([
            BeamCandidate::new(1_u64, 0.1),
            BeamCandidate::new(2_u64, 0.2),
        ]);

        assert_eq!(
            search.peek_best_matching(|_| false),
            None,
            "a fully stale frontier should drain away and report no best candidate"
        );
        assert!(
            search.is_empty(),
            "draining a fully stale frontier should leave the scheduler empty"
        );
    }

    #[test]
    fn beam_search_take_best_matching_consumes_first_live_candidate() {
        let mut search = BeamSearch::new(4);
        search.seed_many([
            BeamCandidate::new(1_u64, 0.1),
            BeamCandidate::new(2_u64, 0.2),
            BeamCandidate::new(3_u64, 0.3),
        ]);

        let taken = search.take_best_matching(|node| node != 1);
        assert_eq!(
            taken,
            Some(BeamCandidate::new(2_u64, 0.2)),
            "take_best_matching should drop stale leaders and consume the first live candidate"
        );
        assert_eq!(
            search.snapshot_frontier(),
            vec![BeamCandidate::new(3_u64, 0.3)],
            "take_best_matching should remove both stale leaders and the consumed live candidate"
        );
        assert_eq!(
            search.discovered(),
            &[BeamCandidate::new(3_u64, 0.3)],
            "consumed candidates should leave scheduler discovery state"
        );
    }

    #[test]
    fn visible_frontier_best_candidate_prefers_live_scheduler_node() {
        let mut visible = VisibleFrontier::default();
        visible.extend([
            BeamCandidate::new(1_u64, 0.1),
            BeamCandidate::new(2_u64, 0.2),
        ]);

        let mut expansion = BeamSearch::new(4);
        expansion.seed_many([
            BeamCandidate::new(9_u64, 0.05),
            BeamCandidate::new(2_u64, 0.2),
            BeamCandidate::new(1_u64, 0.1),
        ]);

        assert_eq!(
            visible.best_candidate(&mut expansion),
            Some(BeamCandidate::new(1_u64, 0.1)),
            "best_candidate should drop stale scheduler leaders until the current live visible node remains"
        );
        assert_eq!(
            expansion.snapshot_frontier(),
            vec![
                BeamCandidate::new(1_u64, 0.1),
                BeamCandidate::new(2_u64, 0.2)
            ],
            "best_candidate should prune stale scheduler nodes while preserving live queued order"
        );
    }

    #[test]
    fn visible_frontier_best_candidate_falls_back_after_scheduler_drains() {
        let mut visible = VisibleFrontier::default();
        visible.extend([
            BeamCandidate::new(1_u64, 0.1),
            BeamCandidate::new(2_u64, 0.2),
        ]);

        let mut expansion = BeamSearch::new(4);
        expansion.seed_many([
            BeamCandidate::new(9_u64, 0.05),
            BeamCandidate::new(8_u64, 0.07),
        ]);

        assert_eq!(
            visible.best_candidate(&mut expansion),
            Some(BeamCandidate::new(1_u64, 0.1)),
            "best_candidate should fall back to Vec score order once the scheduler drains away as fully stale"
        );
        assert!(
            expansion.is_empty(),
            "dropping a fully stale scheduler should leave the queued expansion state empty"
        );
    }

    #[test]
    fn visible_frontier_consume_best_prefers_live_scheduler_node() {
        let mut visible = VisibleFrontier::default();
        visible.extend([
            BeamCandidate::new(1_u64, 0.1),
            BeamCandidate::new(2_u64, 0.2),
        ]);

        let mut expansion = BeamSearch::new(4);
        expansion.seed_many([
            BeamCandidate::new(9_u64, 0.05),
            BeamCandidate::new(2_u64, 0.2),
            BeamCandidate::new(1_u64, 0.1),
        ]);

        let consumed = visible.consume_best(&mut expansion);
        assert_eq!(
            consumed,
            Some(BeamCandidate::new(1_u64, 0.1)),
            "consume_best should drop stale scheduler leaders and consume the first live visible node"
        );
        assert_eq!(
            visible
                .iter()
                .map(|candidate| candidate.node)
                .collect::<Vec<_>>(),
            vec![2],
            "consume_best should remove the consumed visible node from the compacted frontier"
        );
        assert_eq!(
            expansion.snapshot_frontier(),
            vec![BeamCandidate::new(2_u64, 0.2)],
            "consume_best should keep the scheduler aligned with the remaining live visible frontier"
        );
    }
}
