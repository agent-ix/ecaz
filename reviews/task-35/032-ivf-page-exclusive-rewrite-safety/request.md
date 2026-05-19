# Task 35 Review Request: IVF Page Exclusive Rewrite Safety

## Summary

Code commit under review: `b7a35c922df572398a73a3b984e7fc2ca4165b7c`

This slice documents the IVF posting exclusive-buffer rewrite unsafe boundaries
in `src/am/ec_ivf/page.rs`.

The covered helper is `rewrite_ivf_postings_from_exclusive_buffer`.

The added `SAFETY:` comments cover generic WAL transaction startup, full-page
image registration for the exclusive-locked buffer, validated per-line tuple
exposure and fixed-size in-place rewrites, compact and non-compact delete
offset handling, WAL transaction finish, and post-rewrite free-space accounting.

## Baseline Accounting

- Global unsafe baseline: `2859 -> 2852`
- `src/am/ec_ivf/page.rs`: `38 -> 31`

## Validation

- `bash scripts/check_unsafe_comments.sh` passed with an empty log:
  `artifacts/unsafe-audit-after.log`
- `make unsafe-baseline-report` reports `2852` entries and IVF page at `31`:
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
