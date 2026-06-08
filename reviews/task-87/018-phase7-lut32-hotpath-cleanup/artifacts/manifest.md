# Task 87 Phase 7 LUT32 Hot-Path Cleanup Artifact Manifest

- Head SHA: `00706baa02cd21875b33fd4bebc809911eed5cd1`
- Task bucket: `reviews/task-87/`
- Packet path: `reviews/task-87/018-phase7-lut32-hotpath-cleanup/`
- Timestamp: `2026-06-08T15:38:35-07:00`
- Storage surface: code/test slice only; no corpus storage surface was loaded.
- Rerank mode: not applicable.
- Isolation: not applicable; no benchmark matrix was run.

## Artifacts

### `artifacts/cargo-test-candidate-batch.log`

- Command: `cargo test --lib am::common::candidate_batch --no-default-features --features pg18`
- Key result: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2001 filtered out`
- Coverage note: preserves block-plus-tail scalar equality and counter attribution tests after removing the hot-path allocation.

### `artifacts/cargo-test-quant-lut32.log`

- Command: `cargo test --lib quant::lut32 --no-default-features --features pg18`
- Key result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2003 filtered out`
- Coverage note: preserves the direct LUT32 scalar differential and shape-mismatch checks.
