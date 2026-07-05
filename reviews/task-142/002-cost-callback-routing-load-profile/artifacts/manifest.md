# Task 142 Packet 002 Artifact Manifest

- head SHA: `ddf73ba1ae94c3832d1baca180fd6fe86731b429`
- branch: `task-142-spire-epoch-cache-overhead`
- task bucket: `reviews/task-142/002-cost-callback-routing-load-profile`
- timestamp: `2026-07-05T07:22:25Z`
- scope: Phase 0 SPIRE epoch-cache instrumentation; no benchmark matrix run in this slice.
- isolated/shared surface: not applicable; focused unit-level validation only.

## Artifacts

### `cargo-test-cli-explain-profile-r2.log`

- command: `cargo test -p ecaz-cli explain_sql_uses_spire_profile_gucs_and_cost_snapshot -- --nocapture`
- result: pass
- key line: `test commands::bench::suite::tests::explain_sql_uses_spire_profile_gucs_and_cost_snapshot ... ok`

### `cargo-test-cli-production-read-profile.log`

- command: `cargo test -p ecaz-cli spire_pipeline_renders_production_read_profile -- --nocapture`
- result: pass
- key line: `test commands::bench::spire_pipeline::tests::spire_pipeline_renders_production_read_profile ... ok`

### `cargo-test-core-production-read-rollup.log`

- command: `cargo test --lib production_read_profile_row_preserves_metric_rollup -- --nocapture`
- result: pass
- key line: `test am::ec_spire::production_executor_state_tests::production_read_profile_row_preserves_metric_rollup ... ok`
