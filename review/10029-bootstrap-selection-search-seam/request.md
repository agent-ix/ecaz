# Request: Bootstrap Selection Search Seam

Commit: `1847439`

Summary:
- add a `search::VisibleFrontier::select_next_with_refill(...)` helper that owns bootstrap candidate consumption until result selection succeeds
- move the live bootstrap result-selection path in `src/am/scan.rs` behind that search-owned traversal seam
- keep scan-owned result adjudication, graph refill/top-up callbacks, expanded-source bookkeeping, and the linear fallback unchanged
- keep the old scan-local selection helper only for unit tests

Please review:
- whether `select_next_with_refill(...)` is the right search-owned seam for the current bootstrap consume-until-selected runtime path
- whether the new runtime wiring in `try_select_next_bootstrap_frontier_result(...)` preserves existing refill/top-up ordering and expanded-source semantics
- whether this is the right smallest integration step before extracting more of the remaining bootstrap fill policy itself
