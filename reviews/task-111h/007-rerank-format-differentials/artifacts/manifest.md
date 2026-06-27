# Task 111h / 007 Artifacts Manifest

Head SHA: `bd72a9cc18b39c2d5176a4673d4d1bc492f1eeae`

Task bucket: `reviews/task-111h/007-rerank-format-differentials/`

Timestamp: `2026-06-20T05:06:04Z`

Scope: rerank format source/scalar/batch/persisted-payload differential tests.

Formats covered: f32 and f16 through existing tests; RaBitQ-4, RaBitQ-8, and
TurboQuant through the new batch differential test.

Suite surface: no benchmark suite. These are correctness/static validation logs,
not latency/recall/storage measurement evidence. No benchmark tables were
created; no isolated one-index-per-table or shared-table benchmark surfaces were
used.

## Artifacts

- `cargo-test-rerank-batch-differential.log`
  - Command: `script -q -c "cargo test --no-default-features --features pg18 compact_payload_codecs_batch_paths_match_scalar_scores --lib" reviews/task-111h/007-rerank-format-differentials/artifacts/cargo-test-rerank-batch-differential.log`
  - Key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2199 filtered out`

- `cargo-check-pg18.log`
  - Command: `script -q -c "cargo check --no-default-features --features pg18" reviews/task-111h/007-rerank-format-differentials/artifacts/cargo-check-pg18.log`
  - Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 7.30s`
