# Task 87 Phase 7 Bench Counter Capture Artifact Manifest

- Head SHA: `52d1b251f6bbe4f17115baf119340ff539fab6de`
- Task bucket: `reviews/task-87/`
- Packet path: `reviews/task-87/019-phase7-bench-counter-capture/`
- Timestamp: `2026-06-08T15:47:09-07:00`
- Storage surface: code/test slice only; no corpus storage surface was loaded.
- Rerank mode: not applicable.
- Isolation: not applicable; no benchmark matrix was run.

## Artifacts

### `artifacts/cargo-test-ecaz-cli-bench-suite.log`

- Command: `cargo test -p ecaz-cli bench::suite --no-default-features`
- Key result: `test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 366 filtered out`
- Coverage note: includes suite expansion coverage for the `task87_candidate_batch_counters` pass-through on latency and spire-pipeline steps.

### `artifacts/cargo-test-candidate-batch.log`

- Command: `cargo test --lib am::common::candidate_batch --no-default-features --features pg18`
- Key result: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2001 filtered out`
- Coverage note: preserves CandidateBatch block-plus-tail equality and counter attribution coverage.

### `artifacts/cargo-test-quant-lut32.log`

- Command: `cargo test --lib quant::lut32 --no-default-features --features pg18`
- Key result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2003 filtered out`
- Coverage note: preserves LUT32 scalar differential and shape-mismatch checks.
