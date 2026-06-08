# Task 87 Packet 016 Artifact Manifest

Head SHA: `4cf65356e35084f3ec867230aeb7b9123114e416`

Task bucket: `reviews/task-87/`

Packet path: `reviews/task-87/016-phase7-lut32-kernel-slice/`

Timestamp: 2026-06-08

## Artifacts

### `cargo-test-quant-lut32.log`

- Lane: Phase 7 quant-layer 32-candidate LUT scorer.
- Fixture: Rust unit tests.
- Command:
  `cargo test --lib quant::lut32 --no-default-features --features pg18`
- Result: passed.
- Key result line:
  `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2002 filtered out`

### `cargo-test-candidate-batch.log`

- Lane: Phase 7 shared CandidateBatch hook.
- Fixture: Rust unit tests.
- Command:
  `cargo test --lib am::common::candidate_batch --no-default-features --features pg18`
- Result: passed.
- Key result line:
  `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2001 filtered out`

## Scope Notes

This packet is a Phase 7 implementation slice only. It does not claim the final
Phase 7 gates:

- no scoring-share counters yet;
- no real-corpus rerun yet;
- no HNSW per-frontier batch-width distribution yet;
- no final status flip.
