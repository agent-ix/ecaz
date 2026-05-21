# Task 50 Review Request: SPIRE Custom Scan DML Expression Boundary

## Summary

Code commit: `9e0bb9d7b25b968293df540adfa70b458ef22abb`

This slice continues the SPIRE custom-scan P11 cleanup:

- reused the shared custom-scan list helpers for `custom_exprs` reads in DML
  execution helpers
- centralized expression-list presence, length, and nth-expression handling in
  small local helpers
- removed direct list length/nth unsafe from the DML LIMIT, PK expression, and
  update value-expression paths

Direct unsafe count moved from packet 138's `1544` to `1542`.

## Validation

- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - passed with the known pre-existing `src/am/mod.rs` unused import warning
- `make unsafe-block-count`
  - `unsafe_blocks 1542`
  - `files 123`
- `make unsafe-ledger`
- `make unsafe-ledger-check`
  - `ledger covers 1542 current unsafe rows`

## Artifacts

- `artifacts/code-stat.log`
- `artifacts/code-diff.patch`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/src-unsafe-block-count-after.log`
- `artifacts/count-summary.md`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
