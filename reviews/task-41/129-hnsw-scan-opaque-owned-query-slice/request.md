# Review Request: Task 41 Invariant #2 HNSW scan opaque-owned query slice

Code commit: `def32aafdbfba9f80b2447f514b3b558d05ede49`

## Summary

This Phase C slice tightens `src/am/ec_hnsw/scan.rs` so the palloc-backed scan
query slice is exposed through `TqScanOpaque`.

The previous free helper `scan_query_values` is gone. Grouped heap rerank now
uses `opaque.query_values()`, and the only raw slice over `query_values` is
inside the owning scan opaque method. The remaining `from_raw_parts` entry in
this file is a page tuple view and belongs to the later buffer/page Phase D.

## Scope

- Changed `src/am/ec_hnsw/scan.rs` only.
- Preserved the existing `palloc` and `pfree` points for `query_values`.
- Did not change buffer/page views, graph traversal, slot ownership, callback
  control flow, or scan result ordering.

## Validation

- `cargo fmt --all --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1 HEAD`

The PG18 cargo check completed successfully with the pre-existing unused import
warning in `src/am/mod.rs`. No pgrx runtime tests were run for this local
lifetime-shape refactor.

## Artifacts

- `artifacts/fmt-check.log`
- `artifacts/cargo-check-pg18.log`
- `artifacts/git-diff-check.log`
- `artifacts/code-diff-stat.log`
- `artifacts/hnsw-scan-query-slice-inventory.log`
- `artifacts/manifest.md`

## Reviewer Focus

- Confirm the HNSW query slice is now created only by `TqScanOpaque`.
- Confirm the grouped heap-rerank score path still only borrows the query
  slice expression-locally for scoring.
- Confirm the remaining page tuple slice should be handled in Phase D, not this
  palloc scan-state slice.
