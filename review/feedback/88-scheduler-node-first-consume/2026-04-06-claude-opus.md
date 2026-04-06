# Feedback: Scheduler-Node-First Consume

Request:
- `review/88-scheduler-node-first-consume.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-06

## Response to Review Focus

### Is node-first consume the right next reduction in Vec-index coupling?

**Yes.** `consume_candidate_frontier_head` (scan.rs:721-730) now works in TID space:
1. `scheduler_best_frontier_node(opaque)` → returns an element TID (not a Vec index)
2. `take_candidate_frontier_node(opaque, node)` → removes by TID from the Vec via `remove_node` (scan.rs:714-718)
3. `forget_queued(consumed.element_tid)` → removes from beam

The Vec fallback (lines 728-729) also works in TID space: `current_candidate_frontier_head_tid` returns a TID, then `take_candidate_frontier_node` removes by that TID.

Neither path converts a scheduler-chosen node back into a Vec index as an intermediate step. The only index-aware code is inside `VisibleCandidateFrontier::remove_node` (scan.rs:406-411), which does the `position()` + `Vec::remove()` internally. This is the right encapsulation — the index is a private container detail, not an externally visible concept.

### Do any remaining runtime paths still promote scheduler nodes back into Vec-index logic?

**No.** The consume path is fully TID-driven. The only remaining Vec-index operations are:
- `remove_node` internal to `VisibleCandidateFrontier` — encapsulated
- `candidate_slot(opaque, index)` (scan.rs:425) — used by debug helpers for positional slot access, but this is a read-only debug concern, not a runtime path

No runtime scan path converts TID → index → operation.

### Should the next slice target the remaining `find_candidate_frontier_index` dependency?

**Already done.** `find_candidate_frontier_index` no longer exists in the codebase (confirmed via grep). Its functionality was absorbed into `VisibleCandidateFrontier::remove_node` and `VisibleCandidateFrontierRef::contains_node`. The shared helper that normalized TID to index across multiple paths is gone — review 90 completed this.

## Additional Findings

No issues found. Clean reduction of Vec-index coupling in the hot consume path.
