# Task 35 Review Request: IVF Page Tuple Rewrite Safety

## Summary

Code commit under review: `0ccaebf94d58163bb717233590850b0a675e8624`

This slice documents the IVF directory and posting tuple rewrite unsafe
boundaries in `src/am/ec_ivf/page.rs`.

The covered helpers are:

- `rewrite_ivf_list_directory`
- `update_ivf_list_directory`
- `rewrite_ivf_posting`

The added `SAFETY:` comments cover exclusive buffer acquisition, generic WAL
transaction startup, full-page-image registration, item-id lookup after
line-pointer validation, required tuple-byte validation, fixed-size in-place
tuple copies, and WAL transaction finish points.

## Baseline Accounting

- Global unsafe baseline: `2885 -> 2866`
- `src/am/ec_ivf/page.rs`: `64 -> 45`

## Validation

- `bash scripts/check_unsafe_comments.sh` passed with an empty log:
  `artifacts/unsafe-audit-after.log`
- `make unsafe-baseline-report` reports `2866` entries and IVF page at `45`:
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
