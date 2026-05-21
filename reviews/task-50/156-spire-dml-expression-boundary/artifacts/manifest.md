# Task 50 Packet 156 Artifact Manifest

- head SHA: `7ac915eff4fcb8a319e26f936f665e9f84d146ff`
- task bucket: `reviews/task-50/156-spire-dml-expression-boundary`
- timestamp: `2026-05-20T21:15:38-07:00`
- lane: soundness audit unsafe-boundary burndown
- fixture: Rust compile/static validation
- storage format: n/a
- rerank mode: n/a
- isolated one-index-per-table or shared-table surface: n/a

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/156-spire-dml-expression-boundary/artifacts/cargo-check-pg18-bench.log`
  - result: passed
  - key lines: `Finished dev profile ...`; existing warning remains in `src/am/mod.rs` for unused DML frontdoor imports.
- `git-diff-check.log`
  - command: `script -q -c "git diff --check" reviews/task-50/156-spire-dml-expression-boundary/artifacts/git-diff-check.log`
  - result: passed with no output.
- `unsafe-block-count.log`
  - command: `script -q -c "make unsafe-block-count" reviews/task-50/156-spire-dml-expression-boundary/artifacts/unsafe-block-count.log`
  - result: passed
  - key lines: `src/am/ec_spire/dml_frontdoor/mod.rs` now reports `60`; `src/am/ec_spire/dml_frontdoor/tests.rs` reports `9`.
