# Task 35 Packet 043: Spire Custom Scan Plan Private Safety

## Code Under Review

- Commit: `5c0bf9e3d1226dbb0c786b715f6ef5d66389832f`
- Scope: `src/am/ec_spire/custom_scan/plan_private.rs` and
  `scripts/unsafe_comment_baseline.txt`

## Summary

This slice documents the unsafe boundaries in Spire custom-scan plan-private
metadata handling. It covers planner expression inspection, CustomPath and
CustomScan private metadata reads, DML plan-private list construction, copied
plan-private roundtrip test parsing, counted column-list parsing, PK-column
offset derivation, and PostgreSQL String node decoding.

Key safety boundaries documented:

- PostgreSQL NodeTag checks before Const/Param casts
- float4 array Const datum decoding without taking PostgreSQL ownership
- CustomScanState plan pointer access
- CustomPath/CustomScan private list offset reads for mode and index OID
- planner-owned plan-private List construction using copied PostgreSQL strings
- supported OidList/List private metadata node formats
- counted DML column-list bounds checks before list reads
- PostgreSQL String node and C string decoding
- derived projected-column and PK-column offsets from validated counts

## Baseline Accounting

- Global unsafe-comment baseline: `2650 -> 2627`
- `src/am/ec_spire/custom_scan/plan_private.rs`: `23 -> 0`

## Validation

- `artifacts/unsafe-baseline-report-before.log`: before-count report showing
  `2650` global entries and `23 src/am/ec_spire/custom_scan/plan_private.rs`.
- `artifacts/spire-plan-private-baseline-before.log`: pre-slice plan-private
  baseline entry list.
- `artifacts/unsafe-baseline-update.log` and
  `artifacts/unsafe-baseline-update-after-fmt.log`: regenerated baseline logs,
  ending at `2627` entries.
- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh`
  completed with exit code 0 and no diagnostic output.
- `artifacts/unsafe-baseline-report-after.log`: after-count report showing
  `2627` global entries and no remaining plan-private entry.
- `artifacts/spire-plan-private-baseline-after.log`: after-count output showing
  `entries: 0`.
- `artifacts/git-diff-check.log`: `git diff --check` completed with exit code
  0 and no diagnostic output.
- `artifacts/cargo-fmt.log`: `cargo fmt --all`.
- `artifacts/cargo-check-pg18-bench.log`:
  `cargo check --all-targets --no-default-features --features pg18,bench`
  completed successfully with the known unrelated warnings in
  `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `artifacts/final-diff.patch`: final review diff for the slice.
