# Task 142 Packet 013 Artifact Manifest

- head SHA: `11a1f80f2d4b95967c830fc5a9f1dd9be8c84bf6`
- task bucket: `reviews/task-142/`
- packet path: `reviews/task-142/013-production-manifest-cache-profile/`
- timestamp: `2026-07-05T10:59:59Z`
- slice: production active-epoch manifest cache profile counters
- lane / fixture / storage / rerank: not a benchmark packet; focused Rust and CLI validation only
- isolated one-index-per-table vs shared-table surface: not applicable

## Artifacts

### `cargo-test-production-read-profile-rollup.log`

- command:
  `script -q -e -c "cargo test production_read_profile_row_preserves_metric_rollup -- --nocapture" reviews/task-142/013-production-manifest-cache-profile/artifacts/cargo-test-production-read-profile-rollup.log`
- result:
  `test am::ec_spire::production_executor_state_tests::production_read_profile_row_preserves_metric_rollup ... ok`
- key result line:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2260 filtered out; finished in 0.00s`
- command exit:
  `Script done on 2026-07-05 03:57:59-07:00 [COMMAND_EXIT_CODE="0"]`

### `cargo-test-cli-production-profile-render.log`

- command:
  `script -q -e -c "cargo test -p ecaz-cli spire_pipeline_renders_production_read_profile -- --nocapture" reviews/task-142/013-production-manifest-cache-profile/artifacts/cargo-test-cli-production-profile-render.log`
- result:
  `test commands::bench::spire_pipeline::tests::spire_pipeline_renders_production_read_profile ... ok`
- key result line:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 437 filtered out; finished in 0.00s`
- command exit:
  `Script done on 2026-07-05 03:59:04-07:00 [COMMAND_EXIT_CODE="0"]`
