# Task 35 Packet 073: Spire Remote Scan Output Safety

## Summary

This packet documents unsafe production scan-output boundaries in
`src/am/ec_spire/coordinator/remote_candidates/scan_output.rs` and removes that
file from the unsafe-comment baseline.

Code commit under review:
- `928eca8fba13619041821101a0e366d023d2bf26` (`Document Spire remote scan output safety`)

Scope:
- Added safety comments for production executor, degraded-skip, handoff, heap-resolution, tuple-payload, read-profile, and operator-diagnostics wrappers.
- Added safety comments for SPIRE root-control, epoch-manifest, relation-object-store, reloptions, local-heap, and libpq dispatch planning reads.
- Updated `scripts/unsafe_comment_baseline.txt`.

Baseline movement:
- Global unsafe-comment baseline: `2333 -> 2309`
- `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs`: `24 -> 0`

## Validation

- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/unsafe-baseline-report-after.log`: baseline is `2309` entries across `60` files.
- `artifacts/scan-output-baseline-after.log`: `src/am/ec_spire/coordinator/remote_candidates/scan_output.rs` has `entries: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known unrelated warnings in `src/am/common/parallel.rs` and `src/am/mod.rs`.

Additional review artifacts:
- `artifacts/unsafe-baseline-report-before.log`
- `artifacts/scan-output-baseline-before.log`
- `artifacts/unsafe-audit-before.log`
- `artifacts/diff-before-baseline-update.patch`
- `artifacts/unsafe-baseline-update.log`
- `artifacts/cargo-fmt.log`
- `artifacts/unsafe-baseline-update-after-fmt.log`
- `artifacts/final-diff.patch`
