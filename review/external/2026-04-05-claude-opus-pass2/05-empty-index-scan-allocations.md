# 05 — empty-index scan allocates frontier/visited/expanded sets unnecessarily

**Severity:** Low  
**File:** `src/am/scan.rs:236–247`

## Finding

`reset_scan_position` (called from `amrescan`) unconditionally calls `clear_scan_candidate_state`, `reset_scan_visited_state`, and `reset_scan_expanded_state`. Each of these allocates a Rust heap object (`Vec::new()`, `HashSet::new()`) via `Box::into_raw` if the corresponding pointer is null.

For empty-index scans (`metadata.dimensions == 0`), `amgettuple` returns `false` immediately at line 144 without ever touching these structures. The three heap allocations are wasted.

## Impact

Negligible for typical workloads. Could matter for a pathological case of many short-lived scan descriptors on empty indexes (e.g., a planner that opens and closes scans speculatively).

## Suggestion

Defer allocation to first use rather than allocating eagerly during `reset_scan_position`. The `candidate_frontier_mut`, `visited_tids`, and `expanded_source_tids` accessors already handle null pointers gracefully — the reset functions just need to skip allocation when the pointer is null and the index is empty.

Alternatively, gate the allocations behind a `metadata.dimensions != 0` check in `amrescan`.
