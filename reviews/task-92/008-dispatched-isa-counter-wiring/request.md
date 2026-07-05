# Task 92 Packet 008: Dispatched ISA Counter Wiring

## Summary

This packet addresses the Packet 006 / Packet 007 reviewer constraint before
any real non-scalar LUT32 kernel lands.

Code checkpoint:
`cbabe7c07e3718961e07ec3e952794cdb013aecd`

Changes:

- `score_lut_no_qjl_4bit_block32` now returns the ISA attributed by the backend
  that scored the block.
- The current AVX2, NEON, and SVE backend files still delegate to the scalar
  implementation and therefore return `Isa::Scalar`. This avoids publishing
  false ISA rows while these files are fallback stubs.
- `score_turboquant_no_qjl_4bit_batch_lut32` stores the returned backend ISA in
  `BatchScoringTiming`.
- `score_turboquant_no_qjl_4bit_batch_for` uses that returned ISA in
  `BlockKernelCounterKey` for kernel-block rows.
- The LUT32 module-level doc now states the counter contract: kernel rows must
  use the returned backend ISA, while scalar tails stay on `Isa::Scalar`.

This is the wiring needed before the first real Graviton 4 SVE2 LUT32 kernel:
when `sve::score_block32_sve` stops delegating and returns `Isa::Sve2`, the
counter row will report `isa=sve2`; scalar tails will still be recorded through
the scalar-tail API.

## Validation

- `cargo test --lib quant::lut32::tests --no-default-features --features pg18`
  - `4 passed; 0 failed`
  - artifact: `artifacts/cargo-test-lut32.log`
- `cargo test --lib am::common::candidate_batch::tests --no-default-features --features pg18`
  - `5 passed; 0 failed`
  - artifact: `artifacts/cargo-test-candidate-batch.log`
- `git diff --check`
  - passed with no output
  - artifact: `artifacts/git-diff-check.log`

## Review Focus

- Confirm fallback backend stubs correctly return `Isa::Scalar` until real ISA
  kernels replace them.
- Confirm the candidate-batch counter key now consumes the backend-returned ISA
  for kernel rows.
- Confirm this closes the Graviton 4/SVE2 counter-attribution precondition for
  future Task 93-98 kernel packets.
