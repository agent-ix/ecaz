# Review Request: SPiRE Scan Heap Relation Guard

## Summary

This slice hardens SPiRE scan heap-relation ownership by replacing manual
`table_close` bookkeeping with an owning guard.

Code checkpoint: `757f2faa1551ff7cf35163dc0e6c9f0a2bf81359`

## Safety Handling

- Added `ResolvedScanHeapRelation`, which distinguishes borrowed heap
  relations from relations opened by the scan helper.
- The guard closes owned heap relations in `Drop`, so both `amrescan` and
  snapshot candidate preparation no longer need manual `heap_relation_owned`
  close paths.
- Made `resolve_scan_heap_relation()` safe and fail-closed on null scan
  descriptors or failed heap relation opens.
- Made `resolve_scan_snapshot()` safe and fail-closed on null scan descriptors.
- Updated callers to pass the guard's relation pointer instead of unpacking a
  raw `(Relation, bool)` pair.

The remaining unsafe blocks are the PostgreSQL pointer reads/calls inside the
resolver and existing heap-slot/source read boundaries.

## Baseline Delta

- Before: 4,748 unsafe baseline entries across 106 files.
- After: 4,738 unsafe baseline entries across 106 files.
- Net: 10 entries removed.

The main affected production file is `src/am/ec_spire/scan/relation.rs`,
which moved from 46 baseline entries to 36.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `make fmt-check`
- `git diff --check HEAD^ HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`

`cargo check` passes with the existing PostgreSQL header warnings and existing
unused SPIRE re-export warning.

## Artifacts

- `artifacts/unsafe-baseline-before.log`
- `artifacts/unsafe-baseline-after.log`
- `artifacts/audit-unsafe.log`
- `artifacts/fmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18.log`

## Review Focus

- Does `ResolvedScanHeapRelation` correctly encode the borrowed-vs-owned heap
  relation lifetime?
- Should any caller still be responsible for manually closing the opened heap
  relation, or is the guard the right boundary?
- Are the null-scan and failed-open error paths appropriate for these AM helper
  functions?
