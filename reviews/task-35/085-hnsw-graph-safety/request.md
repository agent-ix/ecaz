# Task 35 Packet 085: HNSW Graph Safety

## Summary

This slice documents the remaining unsafe boundaries in `src/am/ec_hnsw/graph.rs` and removes the file from `scripts/unsafe_comment_baseline.txt`.

The comments cover:

- grouped codebook tuple chain reads
- graph element and neighbor adjacency wrappers
- layer-0 and upper-layer successor expansion closures
- raw page tuple byte access behind locked buffer guards

## Code Under Review

- Code commit: `43b5c726b12caee3e0ca72c90cb30b70d52ee17b`
- Files changed:
  - `src/am/ec_hnsw/graph.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Unsafe Baseline Movement

- Global baseline: `1760 -> 1725`
- Baseline files: `50 -> 49`
- `src/am/ec_hnsw/graph.rs`: `35 -> 0`

Evidence:

- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/hnsw-graph-baseline-before.log`
- `artifacts/unsafe-baseline-report-after.log`
- `artifacts/hnsw-graph-baseline-after.log`

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline` passed and wrote `1725` entries after formatting.
- `bash scripts/check_unsafe_comments.sh` passed.
- `bash scripts/unsafe_baseline_report.sh` reported `1725` entries across `49` files.
- `git diff --check` passed.
- `cargo fmt --all` completed. It emitted the existing stable-rustfmt warnings about unstable import grouping options.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known unused-import warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

Artifacts:

- `artifacts/unsafe-audit-before.log`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/diff-before-format.patch`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/unsafe-audit-after.log`
- `artifacts/unsafe-baseline-report-after.log`
- `artifacts/hnsw-graph-baseline-after.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/final-diff.patch`

## Reviewer Notes

This is comment-only in Rust source plus baseline removal. The only expression reshaping changes convert inline `|...| unsafe { ... }` closures into block closures with the `SAFETY:` comment immediately before the unsafe block.
