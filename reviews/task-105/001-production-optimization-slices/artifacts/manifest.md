# Manifest — Task 105 packet 001: production optimization slices

- Task bucket / packet: `reviews/task-105/001-production-optimization-slices/`
- Branch `task-105-full-sweep`; slices per ADR-077 §4/§6 (ACCEPTED) +
  the packet 009 test-contract disposition.

## Changes

1. `src/quant/isa.rs` — **aarch64 dispatch flip**: `select_highest_isa`
   prefers Neon over Sve/Sve2 (code comment cites the Task 99 G4
   measurement). Cap semantics unchanged (ceiling, never a raise);
   selection tests updated: `select_highest_isa_prefers_neon_on_graviton4`,
   `select_highest_isa_uses_sve_tiers_only_without_neon` (synthetic),
   capped-selection expectations re-anchored to the NEON preference.
   x86 behavior unchanged (avx2 first).
2. `src/am/ec_ivf/options.rs` — `ec_ivf.scratch_soa_batch_decode`
   default false → **true** (ADR-077 §4 decision; description updated;
   `cfg(test)` getter stub stays false for unit-test determinism —
   tests that want batch behavior pass it explicitly, unchanged).
3. `src/quant/rabitq32/mod.rs` — the two strict bit-equality-vs-
   production assertions in `partial_dispatch_matches_anchor_and_
   production_batch` and `simd_block32_is_bit_equal_with_production_
   batch` now grade under the family envelope (`assert_close_simd`,
   ≤4 ULP or 1e-5), with comments citing the Sapphire Rapids 1-ULP
   codegen finding (packet 009). Anchor assertions untouched.

## Validation (local Intel desktop)

- Focused tests green (logs packeted): `quant::isa` 8 (incl. the new
  preference tests), `rabitq32` 6, `candidate_batch` 19; clippy clean
  (`-D warnings`).
- Earlier in-session: `lut32` 11, `qjl32_` 11 also green post-flip.
- Full `cargo test --lib -- --skip pg_test_`: 11–12 failures that
  **reproduce identically on clean main** (parallelism/environment
  isolation in build_parallel/scan/spire test groups, never selected
  together by the project's focused-filter practice) — zero new
  failures from these slices. Recorded as a pre-existing hygiene
  observation, not addressed here.
- Backend installed: release, sha256 `f8d64f66…`, probe `release`.
- **Default-engagement smoke** (`smoke-ivf-default-on-latency.log`):
  `bench latency` on `t99_ivf_tq_100k` in a default session (no GUC,
  no flag) → `surface=ivf quant=turboquant isa=avx2` kernel rows,
  1,432,663 kernel candidates, `scalar_candidates=0` on batch rows
  (~234 ns/c, the lut32 band). The default flip reaches production
  dispatch.
- The aarch64 flip is locally compile-validated only (x86 host); its
  behavioral evidence is the Task 99 G4 NEON-capped run (which IS the
  post-flip behavior at 100k) plus the Phase 2 G4 day-one smoke +
  100k confirmation column.
- Timestamp: 2026-06-12 (PDT)
