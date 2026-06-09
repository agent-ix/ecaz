---
task: 92
packet: 005-runtime-isa-helper
agent: coder
date: 2026-06-09
---

# Task 92 Phase 3: Runtime ISA Selection Helper

## Summary

This checkpoint begins Phase 3 by growing `src/quant/isa.rs` from labels into a
runtime ISA selection helper.

Code commit:

- `8c9ee08b69c32dec0e94959a901ef2ab6651d164`
  `Add runtime ISA selection helper`

Changes:

- Adds `HostIsaFeatures`, `Aarch64Features`, and `X86Features`.
- Adds runtime feature detection through `is_aarch64_feature_detected!` and
  `is_x86_feature_detected!`, behind target-arch cfg gates.
- Adds `select_highest_isa(...)` and staged `current_isa()`.
- Adds mocked feature-set tests for:
  - Graviton 4 shape: `sve2 && sve` -> `Isa::Sve2`;
  - base SVE shape: `sve && !sve2` -> `Isa::Sve`;
  - neither SVE nor SVE2 -> `Neon` or `Scalar`;
  - x86 AVX2 -> `Avx2`.

`current_isa()` and concrete detection helpers are staged for the LUT32
dispatch backfill and are intentionally not wired into scoring yet.

## Validation

See `artifacts/manifest.md` for artifact metadata.

- `git diff --check`: passed with no output.
- `cargo test --lib quant::isa::tests --no-default-features --features pg18`:
  `4 passed; 0 failed`.

## Review Focus

- Confirm `sve2` detection is explicit enough for the Graviton 4 requirement.
- Confirm mocked feature-set selection tests satisfy reviewer F1b before LUT32
  dispatch uses the helper.
- Confirm `current_isa()` should stay staged until the LUT32 module-layout
  backfill wires dispatch through it.
