# Task 50 Review Request: Query Analysis Boundary

## Summary

Code commit: `6358c019c673a3361bdd81955b1ba5a5cd3a7d8f`

This slice centralizes the DML diagnostic SQL parser/analyzer path behind
`src/storage/query.rs`:

- `pg_parse_query`
- parser/analyzer `List` length checks
- `list_nth`
- `pg_analyze_and_rewrite_fixedparams`

The SPIRE DML diagnostic SQL functions in `src/lib.rs` now call the safe
`storage::query::analyze_single_query` helper instead of carrying parser/list
unsafe at each call site.

Direct unsafe count moved from packet 136's `1556` to `1548`. The file count
increased from `122` to `123` because this slice adds the new central query
helper module that owns the remaining parser/list unsafe block.

## Validation

- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - passed with the known pre-existing `src/am/mod.rs` unused import warning
- `make unsafe-block-count`
  - `unsafe_blocks 1548`
  - `files 123`
- `make unsafe-ledger`
- `make unsafe-ledger-check`
  - `ledger covers 1548 current unsafe rows`

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
