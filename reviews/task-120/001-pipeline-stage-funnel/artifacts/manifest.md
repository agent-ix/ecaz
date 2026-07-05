head_sha: 9ade66f9ea0ff163fdb92908e9b827e878e4e279
task_bucket: reviews/task-120
packet_path: reviews/task-120/001-pipeline-stage-funnel
timestamp: 2026-06-21
lane: local-validation
fixture: none
storage_format: not applicable
rerank_mode: not applicable
surface: SPIRE spire-pipeline funnel JSONL stage diagnostics
isolated_one_index_per_table: not applicable

# Artifacts

- `cargo-fmt-check.log`
  - command: `cargo fmt --package ecaz-cli --check`
  - result: passed with existing stable-rustfmt warnings.
- `cargo-test-ecaz-cli-spire-pipeline.log`
  - command: `cargo test -p ecaz-cli spire_pipeline`
  - result: passed; 21 tests passed, 0 failed.

# Key Result Lines

- `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 388 filtered out`.
- `Script done on 2026-06-21 07:13:26-07:00 [COMMAND_EXIT_CODE="0"]`.

# Implementation Notes

`LocalPipelineRow` now decodes the existing SQL-visible `recommendation`
column, and `FunnelRecord` serializes every local pipeline row into
`pipeline_stages`. Existing flat funnel fields are unchanged.
