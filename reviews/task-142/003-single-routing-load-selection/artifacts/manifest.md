# Task 142 Packet 003 Artifact Manifest

- head SHA: `0b442f657c751d5fd2bbc7f08837f4777cff1162`
- branch: `task-142-spire-epoch-cache-overhead`
- task bucket: `reviews/task-142/003-single-routing-load-selection`
- timestamp: `2026-07-05T07:58:42Z`
- scope: first Phase 1 coordinator-routing reduction; production selection now shares one loaded routing hierarchy across leaf counting and route selection.
- isolated/shared surface: not applicable; focused unit-level validation only.

## Artifacts

### `cargo-test-core-single-routing-load-r3.log`

- command: `cargo test --lib collect_resolved_scan_plan_selection_loads_routing_hierarchy_once -- --nocapture`
- result: pass
- key line: `test am::ec_spire::scan::tests::collect_resolved_scan_plan_selection_loads_routing_hierarchy_once ... ok`

### `cargo-test-core-production-read-rollup-r2.log`

- command: `cargo test --lib production_read_profile_row_preserves_metric_rollup -- --nocapture`
- result: pass
- key line: `test am::ec_spire::production_executor_state_tests::production_read_profile_row_preserves_metric_rollup ... ok`
