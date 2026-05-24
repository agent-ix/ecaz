# Task 50 Review Request: SPIRE Custom Scan Current Relation Boundary

## Summary

Code commit: `1c7e8974e73deda8c360c6c0b3526c4088834636`

This slice adds a shared SPIRE custom-scan helper for checked
`CustomScanState.ss.ss_currentRelation` access and routes DML/begin-exec
callers through it:

- tuple payload column discovery
- tuple payload input descriptor setup
- DML PK column lookup

Direct unsafe count moved from packet 139's `1542` to `1541`.

## Validation

- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - passed with the known pre-existing `src/am/mod.rs` unused import warning
- `make unsafe-block-count`
  - `unsafe_blocks 1541`
  - `files 123`
- `make unsafe-ledger`
- `make unsafe-ledger-check`
  - `ledger covers 1541 current unsafe rows`

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
