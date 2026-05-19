# Task 35 Packet 070: Spire Coordinator Lifecycle Safety

## Summary

This packet documents unsafe PostgreSQL lifecycle boundaries in
`src/am/ec_spire/coordinator/lifecycle.rs` and removes that file from the
unsafe-comment baseline.

Code commit under review:
- `d5a3e8f329a9680b458fef477158d23b01b5e661` (`Document Spire coordinator lifecycle safety`)

Scope:
- Added safety comments for publish relation lock/unlock by relation OID.
- Added safety comments for resolving an index relation to its heap relation.
- Added a safety comment for reading the backend-local active snapshot.
- Updated `scripts/unsafe_comment_baseline.txt`.

Baseline movement:
- Global unsafe-comment baseline: `2353 -> 2347`
- `src/am/ec_spire/coordinator/lifecycle.rs`: `6 -> 0`

## Validation

- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/unsafe-baseline-report-after.log`: baseline is `2347` entries across `63` files.
- `artifacts/lifecycle-baseline-after.log`: `src/am/ec_spire/coordinator/lifecycle.rs` has `entries: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known unrelated warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

Additional review artifacts:
- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/lifecycle-baseline-before.log`
- `artifacts/unsafe-audit-before-baseline-update.log`
- `artifacts/diff-before-baseline-update.patch`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/final-diff.patch`
