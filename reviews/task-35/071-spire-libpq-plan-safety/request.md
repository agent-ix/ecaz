# Task 35 Packet 071: Spire Libpq Plan Safety

## Summary

This packet documents unsafe PostgreSQL relation forwarding and `rd_id` reads in
`src/am/ec_spire/coordinator/remote_candidates/libpq_plan.rs` and removes that
file from the unsafe-comment baseline.

Code commit under review:
- `a06bad85845953dadd24f723ff84c9470d8a3676` (`Document Spire libpq plan safety`)

Scope:
- Added safety comments for libpq request, connection, and dispatch planning wrappers.
- Added safety comments for copying `rd_id` from live index relations into OID-only descriptor lookups.
- Hoisted one inline `rd_id` unsafe read into an `index_oid` local so the boundary is explicit.
- Updated `scripts/unsafe_comment_baseline.txt`.

Baseline movement:
- Global unsafe-comment baseline: `2347 -> 2340`
- `src/am/ec_spire/coordinator/remote_candidates/libpq_plan.rs`: `7 -> 0`

## Validation

- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/unsafe-baseline-report-after.log`: baseline is `2340` entries across `62` files.
- `artifacts/libpq-plan-baseline-after.log`: `src/am/ec_spire/coordinator/remote_candidates/libpq_plan.rs` has `entries: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known unrelated warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

Additional review artifacts:
- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/libpq-plan-baseline-before.log`
- `artifacts/unsafe-audit-before.log`
- `artifacts/diff-before-baseline-update.patch`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/final-diff.patch`
