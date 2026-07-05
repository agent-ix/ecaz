# Review Request: Task 41 Invariant #2 HNSW scan debug query slice

Code commit: `71c8bc25f790e366c3957c76981207b5bc02e7e4`

## Summary

This Phase C debug-surface slice removes the direct `query_values` raw slice
from `src/am/ec_hnsw/scan_debug.rs`.

`TqScanOpaque` now exposes a test/debug-only `query_values_or_empty()` owner
method, and the scan debug helper copies through that method. The remaining
`from_raw_parts` hits in `scan_debug.rs` are page tuple views for Phase D.

## Scope

- Changed `src/am/ec_hnsw/scan.rs` and `src/am/ec_hnsw/scan_debug.rs`.
- Preserved existing query allocation/free behavior.
- Did not change debug result semantics, page tuple reads, graph traversal, or
  production scan callback behavior.

## Validation

- `cargo fmt --all --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1 HEAD`

The PG18 cargo check completed successfully with the pre-existing unused import
warning in `src/am/mod.rs`. No pgrx runtime tests were run for this local
debug-surface lifetime refactor.

## Artifacts

- `artifacts/fmt-check.log`
- `artifacts/cargo-check-pg18.log`
- `artifacts/git-diff-check.log`
- `artifacts/code-diff-stat.log`
- `artifacts/hnsw-scan-debug-query-slice-inventory.log`
- `artifacts/manifest.md`

## Reviewer Focus

- Confirm scan debug query copying now goes through the owning `TqScanOpaque`.
- Confirm `query_values_or_empty()` is test/debug-only and preserves the prior
  empty-query debug behavior.
- Confirm the remaining scan debug raw slices are page tuple views for Phase D.
