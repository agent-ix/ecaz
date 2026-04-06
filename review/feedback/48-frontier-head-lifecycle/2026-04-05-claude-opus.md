# Feedback: Frontier Head Lifecycle

Request:
- `review/48-frontier-head-lifecycle.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Answers to Review Questions

### Is it correct for the frontier head to remain unchanged through partial linear-scan progress?

**Yes.** The bootstrap linear scan walks pages sequentially via `next_block_number`/`next_offset_number` — it does not consume candidates from the frontier. The frontier is seeded at `amrescan` time as structural groundwork for ordered traversal. During linear scan progress, the frontier is inert decoration. The head remaining stable through partial progress correctly reflects this: nothing in the linear scan path mutates the frontier.

Verified: the linear scan path (`next_linear_scan_heap_tid`, scan.rs:756+) only clears frontier state on full exhaustion, not during page iteration.

### Is full-exhaustion frontier clearing the right boundary?

**Yes, for the current stage.** Full exhaustion (scan.rs:757, 847) clears candidate state, active candidate, and result state together. This is a clean "scan is done" boundary. The frontier doesn't need to survive exhaustion because there's no use case for re-reading it after the scan reports no more tuples.

For future ordered traversal, this remains correct: once the traversal has exhausted all reachable candidates, the frontier is empty by definition. Rescan re-seeds everything from scratch via `initialize_scan_entry_candidate`, so no state needs to carry over.

### Missing invariants around result state and frontier state clearing together?

No gaps found. Both exhaustion points (scan.rs:756-760 and scan.rs:846-850) clear in the same order: `clear_scan_candidate_state` → `clear_active_scan_candidate` → `clear_scan_result_state`. The `reset_scan_position` path for rescan also clears both. No path clears one without the other.

## Additional Findings

No issues found. The lifecycle is well-defined and the debug helper captures the right transition points.
