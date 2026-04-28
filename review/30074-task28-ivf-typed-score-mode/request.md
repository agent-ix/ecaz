# Task 28 A4 Typed Score Mode Dispatch

This packet records completion of A4 from
`plan/tasks/28-ivf-competitive-substrate.md`.

## Change

Commit `cdbbb26` replaces the IVF TurboQuant LUT fast-path string comparison
with a typed enum match:

- Added `ExactScoreMode` in `src/quant/prod.rs`.
- Added `ProdQuantizer::exact_score_mode()`.
- Kept `exact_score_mode_name()` as a display/name helper implemented from the
  typed enum.
- Changed `src/am/ec_ivf/quantizer.rs` to match
  `ExactScoreMode::MseNoQjl4Bit` instead of comparing
  `exact_score_mode_name()` to a string literal.

`src/am/ec_ivf/quantizer.rs` no longer contains the score-mode name literal.

## Validation

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
- `cargo test --lib quantizer_1536_4bit_disables_unused_qjl_and_lut_state --no-default-features --features pg18`
- `cargo test --lib quant::prod::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `git diff --check`

## Disposition

A4 is complete. This is a code-quality fix only; it does not change storage,
planner behavior, or measured latency/recall. No measurement artifacts are
needed beyond this packet.
