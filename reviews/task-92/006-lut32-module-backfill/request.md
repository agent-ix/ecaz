---
task: 92
packet: 006-lut32-module-backfill
agent: coder
date: 2026-06-09
---

# Task 92 Phase 3: LUT32 Module Layout Backfill

## Summary

This checkpoint backfills the Task 87 LUT32 scorer into the ADR-076 module
layout while preserving bit-exact scalar behavior.

Code commit:

- `a489a71c9078b4893ec1dbd797ecd336a7804f9a`
  `Backfill LUT32 module layout`

Changes:

- Moves `src/quant/lut32.rs` to `src/quant/lut32/mod.rs`.
- Adds `src/quant/lut32/scalar.rs` with the current bit-exact scalar block and
  scalar-tail implementations.
- Adds `src/quant/lut32/{neon,sve,avx2}.rs` as safe scalar fallbacks. These
  files establish the ADR-076 layout without claiming ISA-specific speedups.
- Routes `score_lut_no_qjl_4bit_block32` through the staged runtime ISA helper;
  all non-scalar branches currently delegate to the scalar reference.
- Adds explicit `to_bits()` parity tests for:
  - batches under block width;
  - one full 32-candidate block;
  - one block plus scalar tail.

## Validation

See `artifacts/manifest.md` for artifact metadata.

- `git diff --check`: passed with no output.
- `cargo test --lib quant::lut32::tests --no-default-features --features pg18`:
  `4 passed; 0 failed`.
- `cargo test --lib am::common::candidate_batch::tests --no-default-features --features pg18`:
  `4 passed; 0 failed`.

## Review Focus

- Confirm the LUT32 backfill matches ADR-076 layout expectations.
- Confirm scalar fallback stubs are acceptable until Tasks 93-98 land real ISA
  kernels.
- Confirm the added `to_bits()` tests satisfy the Task 92 F3 backfill
  requirement for both `>=32` and `<32` fixtures.
