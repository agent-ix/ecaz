# Task 87 Phase 7 LUT32 Hot-Path Cleanup

## Scope

This packet addresses reviewer notes on packet 016 before Phase 7 real-corpus measurement.

Code checkpoint:

- `00706baa02cd21875b33fd4bebc809911eed5cd1` - `Avoid allocation in Task 87 LUT32 batch path`

What changed:

- Removed the per-flush `Vec<&[u8]>` allocation from the `CandidateBatch` LUT32 route.
- The batch helper now uses a fixed 32-entry reference scratch for full blocks and scores scalar tails directly through the LUT32 scalar helper.
- Metadata validation is folded into the scoring branch instead of running as a separate pre-pass over every candidate.
- `src/quant/lut32.rs` now exposes crate-local validation plus block/scalar helpers so the shared batch route can avoid rebuilding an owned code-ref vector.

## Validation

Packet-local logs:

- `artifacts/cargo-test-candidate-batch.log`
  - `cargo test --lib am::common::candidate_batch --no-default-features --features pg18`
  - Result: 4 passed; 0 failed.
- `artifacts/cargo-test-quant-lut32.log`
  - `cargo test --lib quant::lut32 --no-default-features --features pg18`
  - Result: 2 passed; 0 failed.

## Review Notes

This is not the final Phase 7 closeout. It is a pre-measurement cleanup so the upcoming scoring-share counters do not include avoidable heap allocation in the kernel path.
