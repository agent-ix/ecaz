# Review Request: Task 41 HNSW Scan-State Debug Relation Guards

## Summary

This slice migrates another HNSW pg-test debug helper group from manual
`index_open` / `index_close` pairs to `IndexRelationGuard`.

Code commit: `97be339e8afe0dd25cfc9ec82f200e92a9368566`

## Changes

Updated these helpers in `src/am/ec_hnsw/scan_debug.rs`:

- `debug_scan_uses_grouped_storage`
- `debug_gettuple_exhaustion_state`
- `debug_gettuple_current_result_state`
- `debug_gettuple_orderby_score`
- `debug_gettuple_orderby_score_lifecycle`
- `debug_rescan_entry_candidate_state`
- `debug_rescan_successor_candidate_state`
- `debug_rescan_candidate_frontier`
- `debug_gettuple_consumes_bootstrap_candidate`

Each helper now owns the index relation through `IndexRelationGuard` and uses
the guard's raw pointer for the existing scan calls. Explicit scan cleanup
remains in place before the function returns, so scans still end before the
relation guard drops.

## Baseline

- Before: `4220`
- After: `4202`

## Review Focus

- Confirm each removed manual `index_close` is covered by the relation guard.
- Confirm each `IndexScanEnd` remains before the guard drops.
- Confirm `debug_scan_uses_grouped_storage` still holds the relation while
  reading metadata and deriving `GraphStorageDescriptor`.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
