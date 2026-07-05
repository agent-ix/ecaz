# Task 35 Packet 065: HNSW Module Wrapper Safety

## Summary

This packet documents the unsafe delegation wrappers in `src/am/ec_hnsw/mod.rs`
and removes that file from the unsafe-comment baseline.

Code commit under review:
- `0a776802d52f1eedc340a8393dad628f89e9ae50` (`Document HNSW module wrapper safety`)

Scope:
- Added safety comments for HNSW cost, admin, and planner-integration snapshot
  wrapper calls that forward live index relations.
- Added a safety comment for the EXPLAIN counter wrapper that forwards
  PostgreSQL's live `IndexScanState` pointer.
- Updated `scripts/unsafe_comment_baseline.txt`.

Baseline movement:
- Global unsafe-comment baseline: `2377 -> 2373`
- `src/am/ec_hnsw/mod.rs`: `4 -> 0`

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - `artifacts/unsafe-audit-after.log`
  - Result: pass.
- `bash scripts/unsafe_baseline_report.sh`
  - `artifacts/unsafe-baseline-report-after.log`
  - Result: `2373` entries across `68` files.
- Per-file baseline check for `src/am/ec_hnsw/mod.rs`
  - `artifacts/hnsw-mod-baseline-after.log`
  - Result: `entries: 0`.
- `git diff --check`
  - `artifacts/git-diff-check.log`
  - Result: pass.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - `artifacts/cargo-check-pg18-bench.log`
  - Result: pass with the known unrelated unused-import warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.

Additional packet artifacts:
- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/hnsw-mod-baseline-before.log`
- `artifacts/unsafe-audit-before-baseline-update.log`
- `artifacts/diff-before-baseline-update.patch`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/final-diff.patch`
