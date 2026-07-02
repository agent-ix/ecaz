# Task 120 / 005 Target Candidate Rank Output Artifacts

- head SHA: `9178f4bc04743018b2302859780a5e21b763b3ce`
- task bucket: `reviews/task-120/005-target-candidate-rank-output`
- lane: local validation
- fixture: not applicable; CLI/suite output-surface coverage only
- storage format: not applicable
- rerank mode: SPIRE target candidate-rank diagnostic output
- isolated one-index-per-table vs shared-table surface: not applicable
- timestamp: `2026-06-21T15:12:29Z`

## Artifacts

### `cargo-fmt-check.log`

- command: `cargo fmt --check`
- result: passed
- key line: `Script done on 2026-06-21 08:10:56-07:00 [COMMAND_EXIT_CODE="0"]`
- note: stable rustfmt emitted the repo's existing unstable-option warnings.

### `cargo-test-ecaz-cli-spire-pipeline.log`

- command: `cargo test -p ecaz-cli spire_pipeline`
- result: passed
- key lines:
  - `running 22 tests`
  - `test commands::bench::suite::tests::expands_spire_pipeline_with_production_profile ... ok`
  - `test commands::bench::spire_pipeline::tests::spire_pipeline_rejects_invalid_limits ... ok`
  - `test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 388 filtered out; finished in 0.00s`
  - `Script done on 2026-06-21 08:11:11-07:00 [COMMAND_EXIT_CODE="0"]`

### `ecaz-spire-pipeline-help.log`

- command: `cargo run -p ecaz-cli -- bench spire-pipeline --help`
- result: passed
- key lines:
  - `--target-candidate-rank-output <TARGET_CANDIDATE_RANK_OUTPUT>`
  - `This records retained approximate candidate rank and rerank-prefix membership for each exact truth row.`
  - `Script done on 2026-06-21 08:12:18-07:00 [COMMAND_EXIT_CODE="0"]`
