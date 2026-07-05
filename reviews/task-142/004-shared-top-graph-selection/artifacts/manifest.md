# Task 142 Packet 004 Artifact Manifest

- head SHA: `ba9e7ccb75c878f402985c6563ae45239e1f83ec`
- branch: `task-142-spire-epoch-cache-overhead`
- task bucket: `reviews/task-142/004-shared-top-graph-selection`
- timestamp: `2026-07-05T08:06:13Z`
- scope: coordinator route-selection top-graph reuse; no benchmark matrix run in this slice.
- isolated/shared surface: not applicable; focused unit-level validation only.

## Artifacts

### `cargo-test-core-top-graph-selection-r2.log`

- command: `cargo test --lib collect_resolved_scan_plan_selection_reuses_loaded_top_graph -- --nocapture`
- result: pass
- key line: `test am::ec_spire::scan::tests::collect_resolved_scan_plan_selection_reuses_loaded_top_graph ... ok`

### `cargo-test-core-single-routing-load.log`

- command: `cargo test --lib collect_resolved_scan_plan_selection_loads_routing_hierarchy_once -- --nocapture`
- result: pass
- key line: `test am::ec_spire::scan::tests::collect_resolved_scan_plan_selection_loads_routing_hierarchy_once ... ok`
