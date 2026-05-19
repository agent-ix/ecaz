# Task 35 Packet 061: Spire Recursive Build Publish Safety

## Summary

This packet documents the unsafe publish boundaries in
`src/am/ec_spire/build/recursive.rs` and removes that file from the
unsafe-comment baseline.

Code commit under review:
- `4d98234251de011f338a2583d08b2c74539093f5` (`Document Spire recursive build publish safety`)

Scope:
- Added safety comments for recursive routing placement writes to the open
  PostgreSQL index relation.
- Added safety comments for recursive routing manifest bundle writes.
- Added safety comments for root control page initialization after manifest
  locator publication.
- Updated `scripts/unsafe_comment_baseline.txt`.

Baseline movement:
- Global unsafe-comment baseline: `2389 -> 2386`
- `src/am/ec_spire/build/recursive.rs`: `3 -> 0`

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - `artifacts/unsafe-audit-after.log`
  - Result: pass.
- `bash scripts/unsafe_baseline_report.sh`
  - `artifacts/unsafe-baseline-report-after.log`
  - Result: `2386` entries across `72` files.
- Per-file baseline check for `src/am/ec_spire/build/recursive.rs`
  - `artifacts/recursive-baseline-after.log`
  - Result: `entries: 0`.
- `git diff --check`
  - `artifacts/git-diff-check.log`
  - Result: pass.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - `artifacts/cargo-check-pg18-bench.log`
  - Result: pass with the known unrelated unused-import warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.

Additional packet artifacts:
- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/recursive-baseline-before.log`
- `artifacts/unsafe-audit-before-baseline-update.log`
- `artifacts/diff-before-baseline-update.patch`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/final-diff.patch`
