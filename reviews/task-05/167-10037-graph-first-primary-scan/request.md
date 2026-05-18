# Review Request: Graph-First Primary Scan

Commit: `e871676`

Scope:
- `src/am/scan.rs`
- `src/lib.rs`

Summary:
- make the graph/search-owned ordered traversal path the primary `amgettuple` lane once it has
  materialized a result
- keep the linear scan shell only as fallback when graph traversal exhausts before producing any
  ordered result
- update the pg test contract to reflect staged A3 behavior: graph-first ordered results remain
  required, duplicate drains remain required for selected duplicate-backed elements, but scans no
  longer silently continue into a full linear tail after graph-ordered output has started

Please review:
- whether `select_next_scan_result(...)` now narrows linear fallback at the right phase boundary
  without breaking duplicate/result materialization semantics
- whether using `result_state.current().has_element()` is the correct signal for
  "graph traversal already produced ordered output"
- whether the updated pg tests capture the intended A3 contract precisely enough without becoming
  too loose
