# Task 87 Phase 7 Scoring Counters Artifact Manifest

- Head SHA: `76df28d44e64a8d951d923700654991240193c4d`
- Task bucket: `reviews/task-87/`
- Packet path: `reviews/task-87/017-phase7-scoring-counters/`
- Timestamp: `2026-06-08T15:33:59-07:00`
- Storage surface: code/test slice only; no corpus storage surface was loaded.
- Rerank mode: not applicable.
- Isolation: not applicable; no benchmark matrix was run.

## Artifacts

### `artifacts/cargo-test-candidate-batch.log`

- Command: `cargo test --lib am::common::candidate_batch --no-default-features --features pg18`
- Key result: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2001 filtered out`
- Coverage note: includes `turboquant_lut_batch_records_surface_counters`, which verifies IVF attribution, 39-candidate LUT32 counter attribution, and reset behavior.

### `artifacts/cargo-test-quant-lut32.log`

- Command: `cargo test --lib quant::lut32 --no-default-features --features pg18`
- Key result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2003 filtered out`
- Coverage note: preserves the scalar differential and shape-mismatch checks from packet 016.
