# Task 50 Review Request: Relation Descriptor Metadata Boundary

## Summary

Code commit: `173aa7c26564f0317c4a3665d94bb0da21f5f6df`

This slice centralizes more relation descriptor field reads behind
`src/storage/relation.rs` helpers:

- copied tuple descriptor reads for HNSW source metadata, DiskANN build
  validation, SPIRE build tuple layout, and SPIRE local-store relation planning
- borrowed tuple descriptor reads for SPIRE custom-scan DML payload metadata
- reloptions pointer reads for HNSW, DiskANN, and SPIRE option decoding

Direct unsafe count moved from the packet-133 baseline of `1575` to `1571`.
The net reduction is lower than the caller count because the remaining direct
relation descriptor dereferences are now concentrated in the shared storage
boundary.

## Validation

- `git diff --check`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - passed with the known pre-existing `src/am/mod.rs` unused import warning
- `make unsafe-block-count`
  - `unsafe_blocks 1571`
  - `files 120`
- `make unsafe-ledger`
- `make unsafe-ledger-check`
  - `ledger covers 1571 current unsafe rows`

`cargo fmt --all -- --check` was also tried, but it fails on unrelated
pre-existing formatting diffs outside this slice; no formatting changes were
applied.

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
