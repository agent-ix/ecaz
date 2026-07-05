# Task 35 Review Request: IVF Page Helper Safety

## Summary

Code commit under review: `0d23ebffcc6b699f51c8f190c8788535f04f45cd`

This slice documents generic IVF page helper unsafe boundaries in
`src/am/ec_ivf/page.rs`.

The covered helpers are:

- `page_item_id`
- `with_page_line_tuple_bytes`
- `with_required_page_tuple_bytes`
- `page_line_pointer_count`

The added `SAFETY:` comments cover line-pointer address arithmetic, item-id
lookup after offset validation, tuple byte slice construction after page-bound
checks, required-tuple forwarding through the line helper, and reading
`pd_lower` to compute the line-pointer count.

## Baseline Accounting

- Global unsafe baseline: `2846 -> 2841`
- `src/am/ec_ivf/page.rs`: `25 -> 20`

## Validation

- `bash scripts/check_unsafe_comments.sh` passed with an empty log:
  `artifacts/unsafe-audit-after.log`
- `make unsafe-baseline-report` reports `2841` entries and IVF page at `20`:
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
