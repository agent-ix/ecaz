# Task 35 Packet 076: Spire Coordinator Debug Safety

## Summary

This packet documents unsafe pg_test/debug helper boundaries in
`src/am/ec_spire/coordinator/debug.rs` and removes that file from the
unsafe-comment baseline.

Code commit under review:
- `31e940a76911c9218dcef7b001b23578c3736758` (`Document Spire coordinator debug safety`)

Scope:
- Added safety comments for guarded debug relation object-store openings and readbacks.
- Added safety comments for root/control reads and initialization in debug round trips.
- Added safety comments for manifest writes, placement-entry writes, same-length tuple rewrites, and manifest loads used by debug rewrite helpers.
- Added safety comments for copying relation OIDs/tablespace OIDs from guarded relations.
- Updated `scripts/unsafe_comment_baseline.txt`.

Baseline movement:
- Global unsafe-comment baseline: `2176 -> 2138`
- `src/am/ec_spire/coordinator/debug.rs`: `38 -> 0`

## Validation

- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/unsafe-baseline-report-after.log`: baseline is `2138` entries across `57` files.
- `artifacts/debug-baseline-after.log`: `src/am/ec_spire/coordinator/debug.rs` has `entries: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known unrelated warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

Additional review artifacts:
- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/debug-baseline-before.log`
- `artifacts/unsafe-audit-before.log`
- `artifacts/diff-before-baseline-update.patch`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/final-diff.patch`
