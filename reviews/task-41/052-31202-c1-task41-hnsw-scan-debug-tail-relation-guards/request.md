# Review Request: Task 41 HNSW Scan Debug Tail Relation Guards

## Summary

This slice removes the remaining direct `index_open` / `index_close` uses from
`src/am/ec_hnsw/scan_debug.rs` by moving the tail debug helpers to
`IndexRelationGuard`.

Code commit: `ff06488e12716901c8b2fff6083f33e975403d7b`

## Changes

Updated these helpers:

- `debug_gettuple_current_result_lifecycle`
- `debug_gettuple_current_result_neighbors`
- `debug_gettuple_current_result_heap_progress`
- `debug_gettuple_backward_after_rescan`
- `debug_gettuple_rescan_after_exhaustion`
- `debug_gettuple_rescan_after_partial`
- `debug_entry_point_neighbor_tids`

The normal scan helpers still call `ec_hnsw_amendscan` and `IndexScanEnd`
before returning. The backward-scan error-path helper keeps its AM behavior
unchanged, but relation ownership now sits in an RAII guard so pgrx unwind
paths close the relation.

## Baseline

- Before: `4188`
- After: `4174`

## Review Focus

- Confirm all remaining direct `index_open` / `index_close` uses are gone from
  `src/am/ec_hnsw/scan_debug.rs`.
- Confirm the raw relation pointer remains scoped under a live
  `IndexRelationGuard`.
- Confirm scan cleanup ordering is preserved for the helpers that explicitly
  end scans.
- Confirm the backward-scan helper intentionally keeps its error-path AM scan
  behavior unchanged while improving relation ownership.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
