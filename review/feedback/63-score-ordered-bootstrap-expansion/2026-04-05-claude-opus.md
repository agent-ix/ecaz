# Feedback: Score-Ordered Bootstrap Expansion

Request:
- `review/63-score-ordered-bootstrap-expansion.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Response to Review Focus

### Is score-ordered expansion the right bootstrap policy?

**Yes.** Score-ordered expansion means the frontier grows by exploring the most promising candidates first — this is the fundamental HNSW greedy-descent heuristic. The `ScoreOrder` policy (scan.rs:480-491) picks the lowest-scoring unexpanded candidate (lowest `<#>` value = most similar to query), breaking ties by slot index.

This creates the right behavior: during `amrescan` seeding, the entry candidate is expanded first (as the only candidate), then among its neighbors the best-scored one is expanded next, and so on. This means the bootstrap frontier is populated with the best local neighborhood of the entry point, ordered by query similarity.

The policy does not create misleading traversal semantics — it's exactly what a real frontier heap would do. The difference is just the container (Vec with linear scan vs. BinaryHeap) and the cap (3 vs. ef_search). When the transition to real traversal happens, `ScoreOrder` becomes the natural `BinaryHeap::pop()`.

### Tie-breaking stability

The tie-breaking (`left_index.cmp(right_index)` at scan.rs:489) is deterministic and consistent with the frontier-head selection in `recompute_candidate_frontier_head` (scan.rs:540). Both prefer the lower index on tie. This ensures stable, reproducible expansion ordering. For a real heap, ties would be broken by insertion order or element TID, but the current behavior is equivalent for the bounded frontier sizes involved.

### Clean path toward real traversal?

**Yes.** The `BootstrapExpandPolicy::ScoreOrder` is not a temporary special case — it's the actual HNSW expansion policy expressed through the current framework. The transition to real traversal will:
1. Replace `Vec` with `BinaryHeap` (score ordering becomes implicit)
2. Replace `MAX_BOOTSTRAP_FRONTIER_CANDIDATES` with `ef_search`
3. Move expansion from `amrescan` to `amgettuple`

None of these changes require modifying the expansion policy itself.

## Additional Findings

No issues found. Score-ordered expansion is the right convergence toward HNSW semantics.
