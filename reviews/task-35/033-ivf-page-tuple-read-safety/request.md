# Task 35 Review Request: IVF Page Tuple Read Safety

## Summary

Code commit under review: `2c9b449b5781711e44465c9f89f7a557d2a52e37`

This slice documents IVF page tuple-read unsafe boundaries in
`src/am/ec_ivf/page.rs`.

The covered helpers are:

- `read_page_tuple`
- `find_next_tuple_with_tag`

This also tightens one existing exclusive-rewrite `SAFETY:` placement so the
unsafe-comment checker sees the note within its three-line window.

The added `SAFETY:` comments cover share-locked tuple reads, required tuple
byte validation before decoding, block-count bounded forward scans, per-block
share locks, and tuple-tag inspection through the validated page helper.

## Baseline Accounting

- Global unsafe baseline: `2852 -> 2846`
- `src/am/ec_ivf/page.rs`: `31 -> 25`

## Validation

- `bash scripts/check_unsafe_comments.sh` passed with an empty log:
  `artifacts/unsafe-audit-after.log`
- `make unsafe-baseline-report` reports `2846` entries and IVF page at `25`:
  `artifacts/unsafe-baseline-report-after.log`
- `cargo fmt --all` ran; known unrelated format churn was restored before
  final validation: `artifacts/cargo-fmt.log`
- `git diff --check` passed with an empty log:
  `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed with the existing unrelated warnings in `src/am/common/parallel.rs`
  and `src/am/mod.rs`: `artifacts/cargo-check-pg18-bench.log`

## Artifacts

See `artifacts/manifest.md` for command lines, timestamps, and packet-local
evidence files.
