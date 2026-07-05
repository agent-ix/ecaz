# Task 142 Packet 012 Artifact Manifest

- head SHA: `c6cc05ca7c84078a60c6e9d2026237f49a0ee0e8`
- task bucket: `reviews/task-142/`
- packet path: `reviews/task-142/012-production-pool-reuse-profile/`
- timestamp: `2026-07-05T10:41:51Z`
- slice: production connection-pool reuse profile counters
- lane / fixture / storage / rerank: not a benchmark packet; focused Rust and CLI validation only
- isolated one-index-per-table vs shared-table surface: not applicable

## Artifacts

### `cargo-test-production-read-profile-rollup.log`

- command:
  `script -q -e -c "cargo test production_read_profile_row_preserves_metric_rollup -- --nocapture" reviews/task-142/012-production-pool-reuse-profile/artifacts/cargo-test-production-read-profile-rollup.log`
- result:
  `test am::ec_spire::production_executor_state_tests::production_read_profile_row_preserves_metric_rollup ... ok`
- key result line:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2260 filtered out; finished in 0.00s`
- command exit:
  `Script done on 2026-07-05 03:39:48-07:00 [COMMAND_EXIT_CODE="0"]`

### `cargo-test-cli-production-profile-render.log`

- command:
  `script -q -e -c "cargo test -p ecaz-cli spire_pipeline_renders_production_read_profile -- --nocapture" reviews/task-142/012-production-pool-reuse-profile/artifacts/cargo-test-cli-production-profile-render.log`
- result:
  `test commands::bench::spire_pipeline::tests::spire_pipeline_renders_production_read_profile ... ok`
- key result line:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 437 filtered out; finished in 0.00s`
- command exit:
  `Script done on 2026-07-05 03:41:12-07:00 [COMMAND_EXIT_CODE="0"]`
