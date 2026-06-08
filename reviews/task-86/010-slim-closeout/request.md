# Task 86 Packet 010: Slim Closeout

## Summary

Task 86 now ships SPIRE TurboQuant LUT routing only.

The TQ+ IVF work is deliberately deferred to Task 89 because it introduces an
operator-visible storage format and reloption surface that needs broader
validation before landing. The measurement work is preserved in git history and
referenced by `plan/tasks/89-turboquant-tqplus-cross-am-validation.md`.

## What Ships

- `src/am/ec_spire/quantizer/mod.rs`
- `src/am/ec_spire/quantizer/tests.rs`

The code routes SPIRE TurboQuant no-QJL 4-bit scoring through
`score_ip_from_parts_lut_no_qjl_4bit`, matching the existing LUT scorer surface
used by other AMs.

## Evidence

- `reviews/task-86/001-turbovec-tq-analysis/`: source-grounded TurboVec
  TurboQuant analysis.
- `reviews/task-86/002-tqplus-prototype/`: TQ+ synthetic probe, retained as
  investigation evidence only.
- `reviews/task-86/003-calibration-renorm-isolation/`: renormalization
  isolation probe, retained as investigation evidence only.
- `reviews/task-86/004-byte-lut-kernel/`: byte LUT probe, shelved.
- `reviews/task-86/005-spire-tq-lut/`: initial SPIRE LUT routing packet.
- `reviews/task-86/006-options-report/`: option report and transferability
  analysis.
- `reviews/task-86/007-spire-suite/`: synthetic SPIRE suite evidence.
- `reviews/task-86/008-spire-real-spread/`: real10k/50k/100k SPIRE
  baseline-vs-change suite; recall and storage unchanged, latency improved.
- `reviews/task-86/009-spire-lut-parity/`: parity test covering the LUT path
  with `gamma > 0`.

## Deferred Work

TQ+ does not ship in this branch. The deferred TQ+ starting points are preserved
in history and referenced by Task 89, especially commits `e0ae9fe7d`,
`16f1e6104`, and `c7e85e8ac`.

## Validation

- Source diff check: `git diff --name-only origin/main...HEAD -- src` shows
  only `src/am/ec_spire/quantizer/mod.rs` and
  `src/am/ec_spire/quantizer/tests.rs`.
- Storage-format check: no Task 86 diff in `src/am/ec_ivf/` or
  `src/quant/prod.rs`; no new `StorageFormat::TurboQuantTqPlus` or
  `turboquant_tqplus` code lands. See `artifacts/storage-format-diff-check.log`
  and `artifacts/storage-format-enum-check.log`.
- `cargo check -p ecaz --lib --no-default-features --features pg18`: see
  `artifacts/cargo-check-pg18.log`.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`:
  see `artifacts/cargo-clippy-pg18.log`.
- `cargo test --manifest-path hardening/careful/Cargo.toml --lib`: see
  `artifacts/hardening-careful-lib.log`.

## Review Focus

- Confirm Task 86 source changes are SPIRE-only.
- Confirm TQ+ is absent from the landing diff and explicitly deferred to Task
  89.
- Confirm packet 008 remains the accepted benchmark evidence for the shipped
  slice.
