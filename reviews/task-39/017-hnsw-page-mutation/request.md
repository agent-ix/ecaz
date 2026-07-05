# Task 39 Review Request: HNSW Page Mutation

Code checkpoint: `59c81555121c12c8c72daba4299587bf4253dd04`

## Summary

This packet closes the HNSW page-codec mutation gap for `src/am/ec_hnsw/page.rs` in the supported pgrx-free careful lane.

Changes:

- Added `src/am/ec_hnsw/page.rs` to the careful-backed `make mutants` mapping.
- Pinned HNSW metadata, tuple, and payload flag layout constants with exact byte assertions.
- Added current-format metadata builder tests for zero-dimension and zero-bit codec gating.
- Added metadata page/content boundary and current-to-legacy fallback tests.
- Added element, grouped-hot, turbo-hot, and neighbor tuple length/count boundary tests.
- Added typed `DataPage` and `DataPageChain` update tests for all HNSW tuple wrappers.
- Replaced derived HNSW metadata byte constants and binary-sidecar flag expression with literals to remove equivalent arithmetic/shift mutants while keeping the existing wire layout.

## Evidence

- Focused tests: `artifacts/careful-hnsw-page-tests.log`
  - 42 passed, 0 failed.
- Initial mutation run: `artifacts/page.rs.mutants/mutants.out/outcomes.json`
  - 477 mutants tested, 119 missed, 283 caught, 75 unviable.
- Intermediate mutation run: `artifacts/rerun-2/page.rs.mutants/mutants.out/outcomes.json`
  - 444 mutants tested, 1 missed, 369 caught, 74 unviable.
- Final mutation run: `artifacts/final/page.rs.mutants/mutants.out/outcomes.json`
  - 444 mutants tested, 0 missed, 370 caught, 74 unviable.
- Production compile check:
  - `artifacts/cargo-check-pg18-bench.log`
- Whitespace check:
  - `artifacts/git-diff-check.log`

## Review Notes

Please focus on whether the literal byte-size constants are preferable to the previous derived expressions for mutation gating, and whether the added layout/boundary assertions pin the intended HNSW wire layout without overfitting implementation details.
