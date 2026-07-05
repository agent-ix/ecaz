# Review Request: Task 39 Coverage Lane Portability

Task: `plan/tasks/39-test-quality-measurement.md`

Implementation commit: `071479bac0d0ea7b05d6da3f195df88d7f8a2781`

## Scope

This slice fixes two issues found while rerunning the Task 39 coverage lane on the rebased branch:

- `scripts/hardening.sh coverage` now uses `cargo +stable` only when the configured cargo binary actually supports rustup toolchain directives.
- On Homebrew Rust layouts without `llvm-tools-preview`, the coverage lane falls back to Homebrew LLVM's `llvm-cov` and `llvm-profdata` when present.
- `hardening/careful/src/lib.rs` now has a single `am` module; the quant page-format constants were moved into the existing harness module so `cfg(coverage)` builds do not define `am` twice.

## Validation

Packet-local evidence is under `artifacts/`; see `artifacts/manifest.md`.

- `make coverage`: passed and wrote `target/quality/coverage/summary.txt`, `coverage.json`, and `careful-coverage.json`.
- `make coverage-baseline-check`: passed, 40 critical paths present.
- `scripts/check_coverage_delta.sh target/quality/coverage/summary.txt fixtures/quality/coverage-baseline.tsv`: passed with no baseline regressions.
- `cargo test --manifest-path hardening/careful/Cargo.toml --lib`: passed, 157 tests.
- `bash -n scripts/hardening.sh`: passed.
- `git diff --check`: clean.

The fresh delta check preserves the current baseline: `quant/simd.rs` reports `95.18` against baseline `94.59`; all other baseline paths are at or above their recorded floors.

## Remaining Task 39 Gaps

This packet keeps the existing coverage lane runnable and packeted. It does not close the known structural gaps that still require new coverage or mutation work: PG18/pgrx callback instrumentation, AM page codec coverage raises, SPIRE storage/coordinator coverage raises, storage guard coverage/mutation, broader critical-module mutation triage, and scheduled CI burn-in evidence.
