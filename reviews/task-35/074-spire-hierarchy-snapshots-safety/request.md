# Task 35 Packet 074: Spire Hierarchy Snapshot Safety

## Summary

This packet documents unsafe PostgreSQL and SPIRE object-store boundaries in
`src/am/ec_spire/coordinator/hierarchy_snapshots.rs` and removes that file from
the unsafe-comment baseline.

Code commit under review:
- `3ba25cd2de8cd6bcd6a6f57a551ef0ec10ee3519` (`Document Spire hierarchy snapshot safety`)

Scope:
- Added safety comments for root/control page reads, active manifest tuple reads, and relation reloption reads.
- Added safety comments for relation-backed object-store openings and active object tuple locator reads.
- Added safety comments for remote/local candidate wrappers, coordinator gate/result summaries, and heap-resolution paths.
- Added safety comments for heap relation resolution, active snapshot lookup, indexed-vector attribute resolution, and heap source-vector loading.
- Updated `scripts/unsafe_comment_baseline.txt`.

Baseline movement:
- Global unsafe-comment baseline: `2309 -> 2238`
- `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`: `71 -> 0`

## Validation

- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/unsafe-baseline-report-after.log`: baseline is `2238` entries across `59` files.
- `artifacts/hierarchy-baseline-after.log`: `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` has `entries: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known unrelated warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

Additional review artifacts:
- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/hierarchy-baseline-before.log`
- `artifacts/unsafe-audit-before.log`
- `artifacts/diff-before-baseline-update.patch`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/final-diff.patch`
