# Feedback: Bootstrap Frontier Multi-Hop Fill

Request:
- `review/61-bootstrap-frontier-multihop-fill.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Note: Line Numbers Updated

`fill_bootstrap_frontier` now at scan.rs:589. `top_up_bootstrap_frontier` at scan.rs:603. `next_bootstrap_expand_index` at scan.rs:510. Semantics unchanged — beam scheduler now provides expansion ordering.

## Response to Review Focus

### Is bounded multi-hop fill the right next traversal seam?

**Yes.** The `fill_bootstrap_frontier` → `top_up_bootstrap_frontier` loop (scan.rs:589-631) is structurally identical to the inner loop of HNSW greedy search: pick the best unexpanded candidate, expand its neighbors into the frontier, repeat until full or nothing left to expand. The only differences from real traversal are:

1. The frontier cap is `MAX_BOOTSTRAP_FRONTIER_CANDIDATES` (3) instead of `ef_search`
2. Expansion happens at `amrescan` time, not during `amgettuple`
3. Results come from the linear scan, not from frontier consumption

These are parameter and timing differences, not structural ones. The multi-hop fill doesn't introduce hidden assumptions — it's a direct precursor to the traversal loop.

The `expanded_source_tids` set correctly prevents infinite re-expansion: once a candidate is expanded, it won't be selected again by `next_bootstrap_expand_index` (scan.rs:510). The loop terminates either when the frontier is full or all candidates have been expanded.

### Does insertion-order expansion make sense for this stage?

**This is already superseded** — review 63 changes the expansion policy to score-ordered. Insertion-order was a correct intermediate step that preserved existing behavior while introducing the policy seam. The unit test (`fill_bootstrap_frontier_can_expand_beyond_entry_neighbors`, scan.rs:2409) verifies that the multi-hop chain works: entry → child → grandchild. This is the important contract regardless of expansion ordering.

### Test strength

The unit test exercises the key edge: a grandchild candidate discovered two hops from the entry, with correct provenance chain. The pg regression validates that the frontier shape is bounded and provenance is coherent. This is sufficient — the specific graph shapes are parameterized through the test fixture, not hardcoded.

## Additional Findings

No issues found. The multi-hop fill is the right bridge to real traversal.
