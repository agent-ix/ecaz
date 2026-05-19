# Task 35 Packet 068: Spire Remote Candidate Pipeline Safety

## Summary

This packet documents unsafe coordinator pipeline calls in
`src/am/ec_spire/coordinator/remote_candidates/pipeline.rs` and removes that
file from the unsafe-comment baseline.

Code commit under review:
- `ab0c8a3d668644dbe0c31df23029b244d3204217` (`Document Spire remote candidate pipeline safety`)

Scope:
- Added safety comments for readiness planning and coordinator gate execution.
- Consolidated the relation `rd_id` read into one documented `index_oid` read.
- Added safety comments for heap-resolution summary and local heap-resolution
  plan forwarding.
- Updated `scripts/unsafe_comment_baseline.txt`.

Baseline movement:
- Global unsafe-comment baseline: `2364 -> 2359`
- `src/am/ec_spire/coordinator/remote_candidates/pipeline.rs`: `5 -> 0`

## Validation

- `bash scripts/check_unsafe_comments.sh`
  - `artifacts/unsafe-audit-after.log`
  - Result: pass.
- `bash scripts/unsafe_baseline_report.sh`
  - `artifacts/unsafe-baseline-report-after.log`
  - Result: `2359` entries across `65` files.
- Per-file baseline check for `src/am/ec_spire/coordinator/remote_candidates/pipeline.rs`
  - `artifacts/pipeline-baseline-after.log`
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
- `artifacts/pipeline-baseline-before.log`
- `artifacts/unsafe-audit-before-baseline-update.log`
- `artifacts/diff-before-baseline-update.patch`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/final-diff.patch`
