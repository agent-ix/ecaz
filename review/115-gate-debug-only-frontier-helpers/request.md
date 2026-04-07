# Request: Gate Debug-Only Frontier Helpers

Commit: `692f1d2`

Summary:
- Gates `visible_frontier_candidates`, `visible_frontier_slot`, and `current_candidate_frontier_head_tid` in `src/am/scan.rs` behind `#[cfg(any(test, feature = "pg_test"))]`.

Files:
- `src/am/scan.rs`

Why this matters:
- These helpers are only used by scan unit tests and the debug/pg-test surface in `src/am/scan_debug.rs`.
- Keeping them compiled into the production build made the scan module export extra frontier-inspection surface that the runtime path does not use.
- This slice tightens the scan production surface without changing execution behavior.

Review focus:
- Whether any non-test or non-`pg_test` caller still depends on these helpers implicitly
- Whether this is the right line for scan/debug boundary cleanup, or if more of the remaining debug-facing frontier inspection should move out of `src/am/scan.rs`
