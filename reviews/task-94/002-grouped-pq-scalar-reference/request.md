---
agent: coder-1
role: coder
model: gpt-5
date: 2026-06-09
seq: 02
---

# Task 94 Phase 2 Review Request — Scalar Grouped-PQ Block Kernel

## Scope

This packet implements the Phase 2 scalar reference for the grouped-PQ /
PqFastScan block-kernel family.

Code checkpoint:

- `360429525 Add grouped-PQ scalar block kernel`

## What Changed

- Added `src/quant/grouped_pq_block/{mod.rs,scalar.rs,neon.rs,sve.rs,avx2.rs}`.
- Added the 32-candidate scalar block scorer and scalar tail scorer.
- Added shape checks for `lut.len() == group_count * 16`, code length
  `>= group_count.div_ceil(2)`, and output/candidate count parity.
- Added NEON/SVE/AVX2 fallback stubs that delegate to scalar and return
  `Isa::Scalar`, matching ADR-076 fallback semantics.
- Added unit tests comparing every scalar block/tail/batch output to
  `grouped_pq_score_f32(...).to_bits()`.

## Validation

Local only:

```text
cargo test grouped_pq_block --lib
```

Result: 5 passed, 0 failed. See
`artifacts/test-grouped-pq-block.log`.

No CI, AWS, or benchmark runs were performed.

## Notes For Reviewer

- The parity fixture includes group counts `7`, `8`, `16`, and `32`.
- LUT values are deterministic non-trivial f32 values rather than zero/power-of-two fixtures.
- Real NEON, SVE2, and AVX2 kernels are not claimed in this packet; those remain
  Phase 3, Phase 4, and Phase 5.
