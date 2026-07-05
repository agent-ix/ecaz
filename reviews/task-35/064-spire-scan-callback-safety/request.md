# Task 35 Packet 064: Spire Scan Callback Safety

## Summary

This packet documents the unsafe PostgreSQL scan callback boundaries in
`src/am/ec_spire/scan/callbacks.rs` and removes that file from the
unsafe-comment baseline.

Code commit under review:
- `c393e728b6c144eddc9dc7081edb948feded9fd0` (`Document Spire scan callback safety`)

Scope:
- Added safety comments for `ambeginscan` scan descriptor allocation and opaque
  state installation.
- Added safety comments for `amrescan` scan/orderby validation before descriptor
  and opaque-state access.
- Added safety comments for `amgettuple` scan result field writes after
  descriptor, direction, opaque, and rescan checks.
- Added safety comments for `amendscan` opaque-state cleanup.
- Updated `scripts/unsafe_comment_baseline.txt`.

Baseline movement:
- Global unsafe-comment baseline: `2381 -> 2377`
- `src/am/ec_spire/scan/callbacks.rs`: `4 -> 0`

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - `artifacts/unsafe-audit-after.log`
  - Result: pass.
- `bash scripts/unsafe_baseline_report.sh`
  - `artifacts/unsafe-baseline-report-after.log`
  - Result: `2377` entries across `69` files.
- Per-file baseline check for `src/am/ec_spire/scan/callbacks.rs`
  - `artifacts/scan-callbacks-baseline-after.log`
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
- `artifacts/scan-callbacks-baseline-before.log`
- `artifacts/unsafe-audit-before-baseline-update.log`
- `artifacts/diff-before-baseline-update.patch`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/final-diff.patch`
