# Task 35 Packet 087: Common Explain Hook Safety

## Summary

This slice documents the remaining unsafe boundaries in `src/am/common/explain.rs` and removes the file from `scripts/unsafe_comment_baseline.txt`.

The comments cover:

- PG18 EXPLAIN extension id lookup and extension state access
- per-node PlanState and IndexScanState relcache inspection
- `get_am_name` C string ownership and `pfree`
- EXPLAIN group/property emission calls
- EXPLAIN option and per-node hook C-boundary guards
- global PG18 hook registration

## Code Under Review

- Code commit: `b4b825f8ef659218472c4caf78a6cbcc07182534`
- Files changed:
  - `src/am/common/explain.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Unsafe Baseline Movement

- Global baseline: `1692 -> 1677`
- Baseline files: `48 -> 47`
- `src/am/common/explain.rs`: `15 -> 0`

Evidence:

- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/common-explain-baseline-before.log`
- `artifacts/unsafe-baseline-report-after.log`
- `artifacts/common-explain-baseline-after.log`

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline` passed and wrote `1677` entries after formatting.
- `bash scripts/check_unsafe_comments.sh` passed.
- `bash scripts/unsafe_baseline_report.sh` reported `1677` entries across `47` files.
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
- `artifacts/common-explain-baseline-after.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/final-diff.patch`

## Reviewer Notes

This is comment-only in Rust source plus baseline removal. No hook behavior changed.
