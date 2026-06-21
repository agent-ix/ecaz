# Task 120 / 003 Target Candidate Rank Artifacts

- head SHA: `ad99a4db8e3f7a5c8fcf8d50eed9065daada3bd8`
- task bucket: `reviews/task-120/003-target-candidate-rank`
- lane: local validation
- fixture: not applicable; diagnostic/API and CLI unit coverage only
- storage format: not applicable
- rerank mode: local SPIRE scan plan candidate frontier diagnostics
- isolated one-index-per-table vs shared-table surface: not applicable
- timestamp: `2026-06-21T14:54:17Z`

## Artifacts

### `cargo-fmt-check.log`

- command: `cargo fmt --check`
- result: passed
- key lines:
  - `Script done on 2026-06-21 07:49:31-07:00 [COMMAND_EXIT_CODE="0"]`
  - stable rustfmt emitted the repo's existing unstable-option warnings for `imports_granularity` and `group_imports`.

### `cargo-test-ecaz-cli-spire-pipeline.log`

- command: `cargo test -p ecaz-cli spire_pipeline`
- result: passed
- key lines:
  - `running 22 tests`
  - `test commands::bench::spire_pipeline::tests::stage_containment_records_per_stage_truth_retention ... ok`
  - `test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 388 filtered out; finished in 0.00s`
  - `Script done on 2026-06-21 07:50:17-07:00 [COMMAND_EXIT_CODE="0"]`

### `cargo-test-ecaz-target-candidate-rank.log`

- command: `cargo test -p ecaz target_candidate_rank`
- result: passed
- key lines:
  - `Finished test profile [unoptimized + debuginfo] target(s) in 3m 45s`
  - `Running unittests src/lib.rs`
  - `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2227 filtered out; finished in 0.00s`
  - `Script done on 2026-06-21 07:54:04-07:00 [COMMAND_EXIT_CODE="0"]`
