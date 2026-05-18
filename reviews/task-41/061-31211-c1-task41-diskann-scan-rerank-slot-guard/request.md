# Review Request: Task 41 DiskANN Scan Rerank Slot Guard

## Summary

This slice migrates the main DiskANN scan heap-rerank tuple slot from manual
allocation/release through `scan_state::release_owned_scan_heap_state` to
`TupleTableSlotGuard::single_for_heap`.

Code commit: `021c425ec906601c5129c400d4ac96369d769711`

## Changes

Updated `src/am/ec_diskann/routine.rs` and
`src/am/ec_diskann/scan_state.rs`:

- `ec_diskann_amrescan` now owns the heap rerank slot with
  `TupleTableSlotGuard::single_for_heap`.
- The slot guard is scoped around `vamana_scan_with`, so it drops before
  `release_owned_scan_heap_state` can unregister a snapshot or close an owned
  heap relation.
- `release_owned_scan_heap_state` no longer takes or drops a tuple slot.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: `4140`
- After: `4139`

## Review Focus

- Confirm the slot guard lifetime covers every `exact_heap_rerank_distance`
  call during the scan.
- Confirm the slot guard drops before `release_owned_scan_heap_state` releases
  the snapshot or owned heap relation.
- Confirm the rerank error is preserved after moving `RefCell` extraction out
  of the slot scope.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
