# Task 50 Review Request: Type Info Boundary

## Summary

Code commit: `68a7d617a4dbbfd38526f74c5ff8d61e3ce1574e`

This slice centralizes duplicated PostgreSQL type metadata calls behind
`src/storage/type_info.rs`:

- `getBaseType`
- `format_type_be`
- `pfree` of the formatted type-name allocation

Callers migrated in this packet:

- HNSW source type validation
- DiskANN ambuild indexed-column validation
- SPIRE build tuple layout validation and source-identity type checks

Direct unsafe count moved from packet 134's `1571` to `1563`. The file count
increased from `120` to `121` because this slice adds the new central helper
module that owns the remaining type metadata unsafe blocks.

## Validation

- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - passed with the known pre-existing `src/am/mod.rs` unused import warning
- `make unsafe-block-count`
  - `unsafe_blocks 1563`
  - `files 121`
- `make unsafe-ledger`
- `make unsafe-ledger-check`
  - `ledger covers 1563 current unsafe rows`

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
