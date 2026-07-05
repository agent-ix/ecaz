# Task 35 Packet 089: SPIRE Test Safety

## Summary

This slice documents and consolidates the remaining SPIRE test-only unsafe boundaries in:

- `src/am/ec_spire/custom_scan/tests.rs`
- `src/am/ec_spire/dml_frontdoor/tests.rs`

The comments cover:

- read-only inspection of the static custom-scan method table
- parser-node stack fixtures cast to PostgreSQL `Expr` base pointers for classifier tests

## Code Under Review

- Code commit: `fda113978c63520f4a90dccdc44b86039995880d`
- Files changed:
  - `src/am/ec_spire/custom_scan/tests.rs`
  - `src/am/ec_spire/dml_frontdoor/tests.rs`
  - `scripts/unsafe_comment_baseline.txt`

## Unsafe Baseline Movement

- Global baseline: `1653 -> 1637`
- Baseline files: `46 -> 44`
- SPIRE test files: `16 -> 0`

Evidence:

- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/spire-test-baseline-before.log`
- `artifacts/unsafe-baseline-report-after.log`
- `artifacts/spire-test-baseline-after.log`

## Validation

- `bash scripts/check_unsafe_comments.sh --update-baseline` passed and wrote `1637` entries after formatting.
- `bash scripts/check_unsafe_comments.sh` passed.
- `bash scripts/unsafe_baseline_report.sh` reported `1637` entries across `44` files.
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
- `artifacts/spire-test-baseline-after.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/final-diff.patch`

## Reviewer Notes

This is test-only source cleanup plus baseline removal. The custom-scan method-table test now takes one shared reference to the static table and performs safe field assertions from that reference.
