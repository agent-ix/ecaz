# Feedback: Stale Scheduler Node Cleanup

Request:
- `review/85-stale-scheduler-node-cleanup.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-06

## Response to Review Focus

### Is purging stale scheduler nodes during head recomputation and consume the right defensive behavior?

**Yes.** `scheduler_best_frontier_node` (scan.rs:430-438) implements a purge loop: peek the beam's best, check if it maps to a visible frontier candidate, and if not `forget_queued` it and try the next. This is called from both `current_candidate_frontier_head_tid` (scan.rs:704) and `consume_candidate_frontier_head` (scan.rs:721).

This is the right fix for the dual-structure phase because:
1. The beam scheduler can accumulate nodes that were consumed from the Vec (via `remove_node`) without a corresponding `forget_queued` in some edge paths
2. Without the purge loop, the beam would repeatedly return the same stale node, causing `take_candidate_frontier_node` to return `None` and the scan to stall
3. The loop is bounded — each iteration removes exactly one stale node from the beam, and the beam is bounded by the frontier size

The `forget_queued` call at line 447 is O(n) (drains and rebuilds the heap), but at `MAX_BOOTSTRAP_FRONTIER_CANDIDATES = 3` this is trivial. At larger ef_search values this could matter, but the stale-node case should be rare once seeding and consumption paths are fully synchronized.

### Can any remaining scheduler-first path still get stuck behind an unmappable node?

**No.** Both entry points into scheduler authority go through `scheduler_best_frontier_node`:
- `current_candidate_frontier_head_tid` → `scheduler_best_frontier_node` (scan.rs:707)
- `consume_candidate_frontier_head` → `scheduler_best_frontier_node` (scan.rs:722)

Both purge stale nodes before returning. The only other scheduler mutation points are:
- `seed_discovered_candidates` (scan.rs:648) — adds nodes that also enter the Vec simultaneously, so they start live
- `forget_queued` in `consume_candidate_frontier_head` (scan.rs:724) — removes the consumed node from the beam after Vec removal

No path can get stuck. The unit test (`current_candidate_frontier_head_tid_drops_stale_scheduler_nodes`, scan.rs:1374) directly validates this: seed one stale node (in beam but not Vec) and one live node, verify the stale one is dropped and the live one is selected.

### Should the next slice reduce visible frontier ownership rather than add more synchronization?

**Yes.** The purge loop is correct defensive code for the dual-structure phase, but it's a symptom of split authority. The right direction is to reduce the surface area where drift can happen — either by making the beam the sole authority (eliminating the Vec as data store) or by encapsulating the dual structure behind a single API that maintains the invariant. The subsequent reviews (86-95) appear to take exactly this path.

## Additional Findings

No issues found. Clean defensive fix for an inherent dual-structure hazard.
