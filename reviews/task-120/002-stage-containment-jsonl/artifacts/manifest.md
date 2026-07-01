head_sha: b3057a4f31efdfb2ed7d354c62c8380a0de241c4
task_bucket: reviews/task-120
packet_path: reviews/task-120/002-stage-containment-jsonl
timestamp: 2026-06-21
lane: local-validation
fixture: none
storage_format: not applicable
rerank_mode: not applicable
surface: SPIRE spire-pipeline stage containment JSONL
isolated_one_index_per_table: not applicable

# Artifacts

- `cargo-fmt-check.log`
  - command: `cargo fmt --package ecaz-cli --check`
  - result: passed with existing stable-rustfmt warnings.
- `cargo-test-ecaz-cli-spire-pipeline.log`
  - command: `cargo test -p ecaz-cli spire_pipeline`
  - result: passed; 22 tests passed, 0 failed.

# Key Result Lines

- `test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 388 filtered out`.
- `Script done on 2026-06-21 07:25:19-07:00 [COMMAND_EXIT_CODE="0"]`.

# Implementation Notes

`--stage-containment-output` requires `--include-recall` and
`--include-query-metrics` so the command has exact truth ids and returned ids.
The suite runner now treats `stage_containment_output` as an expected artifact
for `spire-pipeline` steps.

The output is intentionally explicit about evidence strength. Route/leaf/block
rows use the target block-rank snapshot. Candidate and rerank rows use final
hits as a lower bound until a target candidate-rank snapshot lands.
