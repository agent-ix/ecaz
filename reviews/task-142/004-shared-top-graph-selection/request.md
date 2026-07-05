# Review Request: Task 142 Packet 004

## Scope

This packet requests review for commit `ba9e7ccb75c878f402985c6563ae45239e1f83ec`
(`Reuse loaded SPIRE top graph in route selection`) on branch
`task-142-spire-epoch-cache-overhead`.

The slice removes the remaining duplicate coordinator top-graph manifest walk
inside route selection:

- Extends `SpireLoadedRoutingHierarchy` with an optional loaded top-graph object.
- Loads the coordinator top-graph object during the existing coordinator routing
  hierarchy pass.
- Removes `load_snapshot_coordinator_top_graph_object`; top-graph-enabled route
  selection now uses the already loaded top graph from the hierarchy.
- Adds a top-graph-enabled regression test for `collect_resolved_scan_plan_selection`.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo test --lib collect_resolved_scan_plan_selection_reuses_loaded_top_graph -- --nocapture`
- `cargo test --lib collect_resolved_scan_plan_selection_loads_routing_hierarchy_once -- --nocapture`

Both focused validations passed.

## Notes

This completes the within-query sharing for the coordinator route-selection
path: leaf count, route selection, and top-graph routing now use one coordinator
hierarchy load. The remaining Task 142 work is cross-query epoch-keyed caching,
planner cost callbacks reading publish-time stats, remote/session snapshot
reuse, and release A/B evidence.
