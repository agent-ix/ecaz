# Review Request: Task 41 DiskANN backlink slot guards

## Summary

Task 41 follow-up for DiskANN tuple-slot resources in
`src/am/ec_diskann/routine.rs`.

This slice migrates two more manual heap tuple slot lifetimes to
`TupleTableSlotGuard::single_for_heap`:

- `plan_backlink_mutations`
- `fill_vacuum_neighbor_slots`

The old code allocated a raw `TupleTableSlot *`, threaded it through heap
source-vector reads, then manually called `ExecDropSingleTupleTableSlot` after
an inner result closure. The new code keeps the slot guard alive across those
same calls and passes `slot.as_ptr()` only to APIs that still require the raw
Postgres pointer.

Code commit: `65a19540`

## Safety Effect

- Removes the normal-path and error-path reliance on paired manual slot drops
  in both functions.
- Keeps backlink planning source-vector fetches covered by the guard lifetime.
- Keeps `VacuumFillPlanner.slot` covered by the live guard for the duration of
  `plan_vacuum_fill_candidates_for_target`.
- Updates the unsafe comment baseline from `4139` to `4135`.

## Review Focus

- Confirm the slot guard in `plan_backlink_mutations` remains live across both
  `fetch_heap_source_vector` call sites.
- Confirm `VacuumFillPlanner.slot` does not escape the `fill_vacuum_neighbor_slots`
  scope and remains covered by the guard while candidates are planned.
- Confirm removing the explicit `ExecDropSingleTupleTableSlot` calls preserves
  early-return and error-path cleanup semantics.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
