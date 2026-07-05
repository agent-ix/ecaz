# Task 142 Packet 015 Artifact Manifest

- head SHA: `e3c0e8b98c9643288979ebb2ee072217ae66a626`
- task bucket: `reviews/task-142/`
- packet path: `reviews/task-142/015-microsecond-profile-timers/`
- timestamp: `2026-07-05T11:57:59Z`
- slice: production-read profile timer precision
- lane / fixture / storage / rerank: not a benchmark packet; focused core and CLI validation only
- isolated one-index-per-table vs shared-table surface: not applicable

## Code Under Review

Commit `e3c0e8b98` updates the production-read profile timing surface:

- core profile counters use microseconds via `elapsed_micros_u64()`
- `ec_spire_remote_search_production_read_profile()` emits `*_elapsed_us`
  metric rows
- `ecaz bench spire-pipeline` aggregates `*_elapsed_us` rows and falls back
  to historical `*_elapsed_ms` rows
- production-read timeline payload decode is exposed as
  `payload_decode_elapsed_us`

## Artifacts

### `cargo-test-core-profile-rollup.log`

- command:
  `script -q -e -c "cargo test --lib production_read_profile_row_preserves_metric_rollup -- --nocapture" reviews/task-142/015-microsecond-profile-timers/artifacts/cargo-test-core-profile-rollup.log`
- result:
  `test am::ec_spire::production_executor_state_tests::production_read_profile_row_preserves_metric_rollup ... ok`
- key result line:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2260 filtered out; finished in 0.00s`

### `cargo-test-cli-profile-render.log`

- command:
  `script -q -e -c "cargo test -p ecaz-cli spire_pipeline_renders_production_read_profile -- --nocapture" reviews/task-142/015-microsecond-profile-timers/artifacts/cargo-test-cli-profile-render.log`
- result:
  `test commands::bench::spire_pipeline::tests::spire_pipeline_renders_production_read_profile ... ok`
- key result line:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 437 filtered out; finished in 0.00s`

### `cargo-test-cli-timeline-render.log`

- command:
  `script -q -e -c "cargo test -p ecaz-cli spire_pipeline_renders_production_read_timeline -- --nocapture" reviews/task-142/015-microsecond-profile-timers/artifacts/cargo-test-cli-timeline-render.log`
- result:
  `test commands::bench::spire_pipeline::tests::spire_pipeline_renders_production_read_timeline ... ok`
- key result line:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 437 filtered out; finished in 0.00s`

### `git diff --check`

- command:
  `git diff --check`
- result: clean; no output.
