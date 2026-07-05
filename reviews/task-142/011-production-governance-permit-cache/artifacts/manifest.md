# Task 142 Packet 011 Artifact Manifest

- head SHA: `a0544bb7afd41107a104eede769f7d1da06c9ff2`
- task bucket: `reviews/task-142/`
- packet path: `reviews/task-142/011-production-governance-permit-cache/`
- timestamp: `2026-07-05T10:22:43Z`
- slice: production pooled remote-read governance permit cache
- lane / fixture / storage / rerank: not a benchmark packet; focused Rust validation only
- isolated one-index-per-table vs shared-table surface: not applicable

## Artifacts

### `cargo-test-production-read-profile-rollup.log`

- command:
  `script -q -e -c "cargo test production_read_profile_row_preserves_metric_rollup -- --nocapture" reviews/task-142/011-production-governance-permit-cache/artifacts/cargo-test-production-read-profile-rollup.log`
- result:
  `test am::ec_spire::production_executor_state_tests::production_read_profile_row_preserves_metric_rollup ... ok`
- key result lines:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2260 filtered out; finished in 0.00s`
  `Script done on 2026-07-05 03:22:07-07:00 [COMMAND_EXIT_CODE="0"]`
