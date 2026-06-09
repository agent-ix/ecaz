---
task: 92
packet: 003-counter-surface-phase2
agent: coder
date: 2026-06-09
---

# Task 92 Phase 2: Block Kernel Counter Surface

## Summary

This checkpoint starts the Phase 2 counter-surface implementation by adding the
extension-side `(surface, quant_kind, isa)` counter identity and preserving the
Task 87 SQL compatibility view.

Code commit:

- `6e5dc127ab6fc5ad5c2072a8034b4eaddd6eb2ce`
  `Add block kernel counter surface`

Changes:

- Adds `src/quant/isa.rs` with explicit `Isa::{Scalar, Neon, Sve, Sve2, Avx2}`
  and stable labels.
- Adds stable `QuantCodecKind` labels used by counter rows.
- Replaces the Task 87 AM-only backing counters with block-kernel counters keyed
  by `(CandidateBatchScoringSurface, QuantCodecKind, Isa)`.
- Adds `ec_block_kernel_scoring_reset()` and
  `ec_block_kernel_scoring_snapshot()`.
- Keeps `ec_task87_candidate_batch_scoring_reset()` as a compatibility wrapper.
- Keeps `ec_task87_candidate_batch_scoring_snapshot()` as the old AM-only view,
  aggregated from the new counter rows.
- Records Task 87 LUT32 work as `(quant_kind=turboquant, isa=scalar)` for now,
  with whole 32-candidate blocks in `kernel_*` and scalar tails in `scalar_*`.

Scope intentionally left for the next Phase 2 slice:

- CLI parser/output migration to `[block-kernel-counters]`.
- SQL-level PG18 smoke output.
- Broader per-candidate off-path scalar instrumentation outside the existing
  LUT32 batch scorer.

## Validation

See `artifacts/manifest.md` for artifact metadata.

- `git diff --check`: passed with no output.
- `cargo test --lib am::common::candidate_batch::tests --no-default-features --features pg18`:
  `4 passed; 0 failed`.

## Review Focus

- Confirm `Isa::Sve2` as an explicit enum variant satisfies the Graviton 4
  counter identity requirement.
- Confirm the Task 87 compatibility snapshot should remain AM-only and
  aggregate from the new block-kernel rows.
- Confirm current LUT32 rows should report `isa=scalar` until Phase 3 lands
  runtime ISA dispatch/backfill modules.
- Confirm scalar tails are attributed to `isa=scalar` through `scalar_*` fields
  in the new snapshot.
