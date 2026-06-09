# Task 92 Packet 012: SPIRE Off-Path Scalar Counters

## Summary

This packet fixes a Task 92 Phase 2 attribution gap for the Task 87 LUT32
comparison path:

- exposes a narrow crate-local `record_block_scalar_score_for(...)` helper;
- records SPIRE no-QJL 4-bit TurboQuant scalar fallback nanos under
  `(surface=spire, quant=turboquant, isa=scalar)` when candidate-batch routing
  does not take the LUT32 kernel path;
- keeps the existing scalar score loop and score values unchanged;
- serializes the candidate-batch unit tests that mutate global scoring
  counters, making the counter validation deterministic under Rust's default
  parallel test runner.

This is a code-level off-path attribution fix, not the full Task 92 closeout
benchmark calibration. The remaining closeout still needs workload evidence
comparing kernel-on and kernel-off totals against the Task 87 LUT32 baseline.

## Code

- `b5cf53f28900` - `Record SPIRE scalar offpath counters`

## Validation

Artifacts are packet-local under `artifacts/`:

- `artifacts/cargo-test-candidate-batch.log`
  - command: `cargo test --lib am::common::candidate_batch::tests --no-default-features --features pg18`
  - result: 5 passed; 0 failed
- `artifacts/cargo-test-spire-quantizer.log`
  - command: `cargo test --lib am::ec_spire::quantizer::tests --no-default-features --features pg18`
  - result: 18 passed; 0 failed
- `artifacts/git-diff-check.log`
  - command: `git diff --check`
  - result: passed

## Review Notes

The Graviton 4/SVE2 counter path remains unchanged by this packet. This slice
only makes the scalar comparison row visible for SPIRE TurboQuant when the
kernel path is bypassed, so later Graviton 4 evidence can compare
`isa=sve2` kernel rows against `isa=scalar` off-path rows.
