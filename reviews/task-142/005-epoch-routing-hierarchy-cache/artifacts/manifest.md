# Task 142 Packet 005 Artifact Manifest

- head SHA: `d616663cdb38728f4f1c00dec8c4c53f24ac6f52`
- branch: `task-142-spire-epoch-cache-overhead`
- task bucket: `reviews/task-142/005-epoch-routing-hierarchy-cache`
- timestamp: `2026-07-05T08:32:37Z`
- scope: backend-local epoch-keyed coordinator routing hierarchy cache.
- isolated/shared surface: not applicable; focused unit-level validation only.

## Artifacts

### `cargo-test-core-routing-cache.log`

- command: `cargo test --lib collect_cached_resolved_scan_plan_selection_reuses_epoch_hierarchy -- --nocapture`
- result: pass
- key line: `test am::ec_spire::scan::tests::collect_cached_resolved_scan_plan_selection_reuses_epoch_hierarchy ... ok`

### `cargo-test-core-production-read-rollup.log`

- command: `cargo test --lib production_read_profile_row_preserves_metric_rollup -- --nocapture`
- result: pass
- key line: `test am::ec_spire::production_executor_state_tests::production_read_profile_row_preserves_metric_rollup ... ok`
