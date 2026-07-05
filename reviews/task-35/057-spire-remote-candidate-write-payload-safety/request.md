# Task 35 Packet 057: Spire Remote Candidate Write Payload Safety

## Summary

This packet documents the unsafe boundaries in `src/am/ec_spire/coordinator/remote_candidates/write_payload.rs` and removes that file from the unsafe-comment baseline.

Code commit under review:

- `b84e32b82064dc35921a3e1deaf341c41d0d6d6b` (`Document Spire remote write payload safety`)

Scope:

- Added safety comments for coordinator dispatch planning and batch/prepare wrappers used by remote insert, update, delete, and select tuple payload flows.
- Updated `scripts/unsafe_comment_baseline.txt`.

Baseline movement:

- Global unsafe-comment baseline: `2422 -> 2413`
- `src/am/ec_spire/coordinator/remote_candidates/write_payload.rs`: `9 -> 0`

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - Log: `artifacts/unsafe-audit-after.log`
- `bash scripts/unsafe_baseline_report.sh`
  - Log: `artifacts/unsafe-baseline-report-after.log`
- `awk ... scripts/unsafe_comment_baseline.txt`
  - Log: `artifacts/write-payload-baseline-after.log`
- `git diff --check`
  - Log: `artifacts/git-diff-check.log`
- `cargo fmt --all`
  - Log: `artifacts/cargo-fmt.log`
  - Note: emitted the repository's existing stable-rustfmt warnings for unstable rustfmt options.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Log: `artifacts/cargo-check-pg18-bench.log`
  - Result: passed with the known unrelated warnings for unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` and unused SPIRE re-exports in `src/am/mod.rs`.

## Artifacts

- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/write-payload-baseline-before.log`
- `artifacts/unsafe-audit-before-baseline-update.log`
- `artifacts/diff-before-baseline-update.patch`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/unsafe-audit-after.log`
- `artifacts/unsafe-baseline-report-after.log`
- `artifacts/write-payload-baseline-after.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/final-diff.patch`
