# Feedback: Frontier Head Lifecycle

Request:
- `review/48-frontier-head-lifecycle.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Note: Review Partially Superseded

The "two-slot" frontier described in this review is now a `Vec<ScanCandidate>` + `BeamSearch` scheduler. The lifecycle questions are addressed below against the current code.

## Answers to Review Questions

### Is it correct for the frontier head to remain unchanged through partial linear-scan progress?

**Partially superseded.** The current `amgettuple` (scan.rs:149-167) now consumes frontier candidates *before* falling through to the linear scan. `materialize_next_bootstrap_frontier_result` (scan.rs:770-784) consumes the head and materializes it as a result. So the frontier *does* mutate during scan progress — it's no longer inert decoration. The linear scan is now the fallback path after the frontier is exhausted.

The lifecycle is still correct: exhaustion clears candidate state (scan.rs:933-937), and rescan resets everything via `reset_scan_position` (scan.rs:244-257).

### Is full-exhaustion frontier clearing the right boundary?

**Yes, for the current stage.** Full exhaustion (scan.rs:757, 847) clears candidate state, active candidate, and result state together. This is a clean "scan is done" boundary. The frontier doesn't need to survive exhaustion because there's no use case for re-reading it after the scan reports no more tuples.

For future ordered traversal, this remains correct: once the traversal has exhausted all reachable candidates, the frontier is empty by definition. Rescan re-seeds everything from scratch via `initialize_scan_entry_candidate`, so no state needs to carry over.

### Missing invariants around result state and frontier state clearing together?

No gaps found. Both exhaustion points (scan.rs:756-760 and scan.rs:846-850) clear in the same order: `clear_scan_candidate_state` → `clear_active_scan_candidate` → `clear_scan_result_state`. The `reset_scan_position` path for rescan also clears both. No path clears one without the other.

## Additional Findings

No issues found. The lifecycle is well-defined and the debug helper captures the right transition points.
