# Task 35 Packet 060: Spire Custom Scan Tuple Payload Safety

## Summary

This packet documents the remaining unsafe boundaries in
`src/am/ec_spire/custom_scan/tuple_payload.rs` and removes that file from the
unsafe-comment baseline.

Code commit under review:
- `227fa5991aa3e249be818f919429306c2a01ce6b` (`Document Spire custom scan tuple payload safety`)

Scope:
- Added safety comments for the custom scan state/scan slot payload writer.
- Added safety comments for JSON payload tuple-slot storage and the pg_test JSON
  slot helper.
- Added safety comments for PostgreSQL text input and binary receive conversion
  calls used by tuple payload decoding.
- Added safety comments for typed payload tuple-slot storage.
- Updated `scripts/unsafe_comment_baseline.txt`.

Baseline movement:
- Global unsafe-comment baseline: `2395 -> 2389`
- `src/am/ec_spire/custom_scan/tuple_payload.rs`: `6 -> 0`

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - `artifacts/unsafe-audit-after.log`
  - Result: pass.
- `bash scripts/unsafe_baseline_report.sh`
  - `artifacts/unsafe-baseline-report-after.log`
  - Result: `2389` entries across `73` files.
- Per-file baseline check for `src/am/ec_spire/custom_scan/tuple_payload.rs`
  - `artifacts/tuple-payload-baseline-after.log`
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
- `artifacts/tuple-payload-baseline-before.log`
- `artifacts/unsafe-audit-before-baseline-update.log`
- `artifacts/diff-before-baseline-update.patch`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/final-diff.patch`
