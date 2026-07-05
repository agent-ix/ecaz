# Review Request: Task 142 Packet 003

## Scope

This packet requests review for commit `0b442f657c751d5fd2bbc7f08837f4777cff1162`
(`Share SPIRE production routing selection load`) on branch
`task-142-spire-epoch-cache-overhead`.

The slice removes the first duplicated coordinator routing hierarchy walk from
production scan planning:

- Adds `collect_resolved_scan_plan_selection`, a scan-level helper that loads
  the coordinator routing hierarchy once, counts routable leaves from that
  loaded hierarchy, resolves the scan plan, and selects leaf PIDs through the
  same hierarchy.
- Preserves the existing `leaf_count` and `route_select` profile fields by
  returning separate elapsed counters from the combined helper.
- Switches production read/handoff/profile/threshold paths from
  `count_scan_plan_routable_leaf_pids` + `collect_scan_plan_selected_leaf_pids`
  to the combined helper.
- Updates production-read profiling so `routing_hierarchy_load_count` reports
  one hierarchy load for the shared selection path.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo test --lib collect_resolved_scan_plan_selection_loads_routing_hierarchy_once -- --nocapture`
- `cargo test --lib production_read_profile_row_preserves_metric_rollup -- --nocapture`

Both focused validations passed.

## Notes

This is not the full epoch cache. It is the first coordinator-side removal of
the repeated per-query routing hierarchy load and sets up the remaining Phase 1
work: backend-local epoch-keyed reuse across queries and publish-time cost
statistics for planner callbacks.
