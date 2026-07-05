# Task 35 Packet 072: Spire Remote Dispatch Safety

## Summary

This packet documents unsafe planner forwarding and PostgreSQL dynamic-symbol
reads in `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs`, removing
that file from the unsafe-comment baseline.

Code commit under review:
- `749bbf0d87f3d02e6b424b59b7a734f18cccfb5a` (`Document Spire remote dispatch safety`)

Scope:
- Added safety comments for dispatch summary and executor-budget planner wrappers.
- Added safety comments for `dlsym` lookups of PostgreSQL backend symbols.
- Added safety comments for reading sig-atomic PostgreSQL flags and invoking the resolved timeout indicator.
- Updated `scripts/unsafe_comment_baseline.txt`.

Baseline movement:
- Global unsafe-comment baseline: `2340 -> 2333`
- `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs`: `7 -> 0`

## Validation

- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/unsafe-baseline-report-after.log`: baseline is `2333` entries across `61` files.
- `artifacts/dispatch-baseline-after.log`: `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs` has `entries: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known unrelated warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

Additional review artifacts:
- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/dispatch-baseline-before.log`
- `artifacts/unsafe-audit-before.log`
- `artifacts/diff-before-baseline-update.patch`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/unsafe-baseline-update-after-final-comment.log`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/final-diff.patch`
