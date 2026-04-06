# Feedback: Skip Re-Expanding Consumed Bootstrap Sources

Request:
- `review/71-skip-reexpanding-consumed-bootstrap-sources.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Response to Review Focus

### Is the consumed-source skip boundary correct?

**Yes.** `refill_bootstrap_frontier_after_consume` (scan.rs:745-763) checks `expanded_contains_source(opaque, consumed.element_tid)` at line 751 before attempting to expand from the consumed candidate's adjacency. If the candidate was already expanded (during initial seeding or a prior refill), the adjacency load is skipped entirely and the function proceeds directly to `top_up_bootstrap_frontier`.

This is correct because:
- The `expanded_source_tids` set accurately records which element TIDs have been used as expansion sources
- An already-expanded source would produce the same neighbors, all of which are already in the visited set, so re-expanding would do work only to filter everything out
- Skipping avoids a redundant `load_graph_adjacency` call (buffer lock acquire/release + tuple decode)

The `mark_expanded_source` call at line 752 is correctly placed before the refill call at line 753, so the source is marked even if the refill itself produces no new candidates.

### Does the helper still top up correctly when the consumed source is already expanded?

**Yes.** The `top_up_bootstrap_frontier` call at scan.rs:757-763 runs unconditionally after the conditional expansion. It picks the best unexpanded frontier candidate via `next_bootstrap_expand_index` and expands from it. If no unexpanded candidates remain, the loop breaks immediately. This means the frontier shrinks naturally when all sources are exhausted — which is correct behavior.

### Unit coverage sufficiency

The existing `refill_bootstrap_frontier_after_consume` unit test (scan.rs:2575+) and the `top_up_bootstrap_frontier_preserves_expanded_state` test (scan.rs:2492+) together cover the skip boundary. The first test exercises the refill path, and the second verifies that expanded state persists across top-up calls. This is sufficient for an execution-only optimization.

## Additional Findings

No issues found. Clean optimization that removes redundant I/O without changing observable behavior.
