# Feedback: Forget Consumed Frontier Nodes

Request:
- `review/80-forget-consumed-frontier-nodes.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Answers to Review Questions

### Does `forget_queued` have the right semantics for queued-vs-expanded nodes?

**Yes.** Verified in `search.rs:98-122`:
- `forget_queued(node)` drains the `BinaryHeap`, filters out the target, rebuilds the heap, and also removes the node from `visited` and `discovery_order`.
- If the node isn't in the frontier (e.g., already expanded via `expand_one` / `pop_best`), it returns `None` and leaves state untouched.
- After forgetting, the node can be re-seeded (verified by unit test at search.rs:557-564).

The queued-vs-expanded distinction is correct: `expand_one` calls `pop_best` which removes the node from the frontier. After that, `forget_queued` won't find it. Only still-queued nodes can be forgotten. This matches the scan-side need: `consume_candidate_frontier_head` (scan.rs:656) calls `forget_queued` right after removing the consumed node from the frontier Vec — the node is still queued in the beam scheduler because the scan popped it from the Vec, not from the beam scheduler.

### Does removing consumed nodes create rescan/refill edge cases?

**No.** On rescan, `reset_scan_position` → `reset_bootstrap_expansion_state` creates a fresh `BeamSearch`, so any forgotten/consumed state is wiped. On refill after consume, `refill_bootstrap_frontier_after_consume` (scan.rs:736-754) expands from the consumed source and tops up — both paths add new candidates via `seed_discovered_candidates` which properly seeds into the beam scheduler. The forgotten node is gone from the scheduler but its discovered neighbors are fresh entries. No edge case.

### Is the alignment test strong enough?

Verified: the unit test `beam_search_forget_queued_removes_frontier_node_and_allows_reseed` (search.rs:520-566) checks:
- Removal from frontier, visited, and discovery order
- Re-seeding after forget works
- The `ignores_expanded_or_unknown_nodes` test (search.rs:568-609) verifies that already-expanded and unknown nodes are no-ops.

The scan-side alignment is tested through the existing consume/refill pg regressions which exercise `consume_and_refill_bootstrap_frontier` which calls `consume_candidate_frontier_head` which calls `forget_queued`. Coverage is sufficient.

## Additional Findings

**Performance note (reiterated from review 73):** `forget_queued` is O(n) on the heap size because it drains and rebuilds. At `MAX_BOOTSTRAP_FRONTIER_CANDIDATES = 3` this is trivial. At `ef_search = 40-1000` this could matter if called frequently during a single `amgettuple` expansion loop. When the frontier Vec is eliminated and the beam scheduler becomes the sole container, `forget_queued` should be reconsidered — either switched to lazy deletion or eliminated entirely if consumption can go through `pop_best` directly.
