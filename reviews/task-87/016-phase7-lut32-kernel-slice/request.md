# Task 87 Packet 016: Phase 7 32-Candidate LUT Kernel Slice

## Summary

This packet asks for review of the first Task 87 Phase 7 code slice after the
packet 015 reopen. It lands a safe Rust 32-candidate blocked LUT scorer under
`src/quant/` and routes the existing shared TurboQuant no-QJL 4-bit
`CandidateBatch` entry point through it for batches of at least 32 candidates.

This is not final Phase 7 closeout. It does not yet add scoring-share counters,
real-corpus Phase 7 measurements, HNSW batch-width measurement, or the final
packet 016/017 closeout matrix requested by reviewer feedback.

Current code checkpoint:

- `4cf65356e35084f3ec867230aeb7b9123114e416` - `Add 32-candidate LUT scorer for Task 87`

## Code Changes

- `src/quant/lut32.rs`
  - new quant-layer 32-candidate blocked LUT scorer for TurboQuant no-QJL
    4-bit MSE codes;
  - scalar tail handling for residual candidates after full 32-candidate
    blocks;
  - shape validation for LUT length, output count, and code length.
- `src/quant/mod.rs`
  - exposes the crate-local `lut32` module.
- `src/quant/prod.rs`
  - adds a crate-local helper to extract no-QJL 4-bit MSE code bytes from an
    AM payload.
- `src/am/common/candidate_batch.rs`
  - validates metadata once, then routes batches with at least 32 candidates
    through `quant::lut32`;
  - retains the existing scalar LUT path for smaller batches.

No new `unsafe` was added.

## Validation

Packet-local logs are under `artifacts/`.

- `cargo test --lib quant::lut32 --no-default-features --features pg18`
  - `2 passed; 0 failed`
- `cargo test --lib am::common::candidate_batch --no-default-features --features pg18`
  - `3 passed; 0 failed`

## Review Focus

- Confirm the kernel lives in the quant layer and not in an AM module.
- Confirm the 32-candidate blocked scorer is recall-equivalent to the scalar
  LUT scorer for full blocks and scalar tails.
- Confirm routing it from the shared `score_turboquant_no_qjl_4bit_batch`
  entry point is the right hook for SPIRE and IVF.
- Confirm no `unsafe` documentation is needed because this slice uses safe Rust.
