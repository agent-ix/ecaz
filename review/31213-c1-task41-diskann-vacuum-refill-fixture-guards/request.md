# Review Request: Task 41 DiskANN vacuum refill fixture guards

## Summary

Task 41 follow-up for the remaining manual DiskANN tuple-slot pair in
`src/am/ec_diskann/routine.rs`.

This slice converts the `find_vacuum_refill_fixture` PG test helper from raw
resource ownership to shared RAII guards:

- `IndexRelationGuard::access_share` owns the index relation opened for the
  fixture search.
- `HeapRelationGuard::try_access_share` owns the heap relation derived from the
  index relation.
- `TupleTableSlotGuard::single_for_heap` owns the heap tuple slot threaded into
  `VacuumFillPlanner`.

Code commit: `c47ef9bf`

## Safety Effect

- Removes one raw `index_open` / `index_close` pair.
- Removes one raw `table_open` / `table_close` pair.
- Removes one raw heap slot allocation / `ExecDropSingleTupleTableSlot` pair.
- Keeps the slot guard scoped outside the nested fixture-planning closures so
  `VacuumFillPlanner.slot` stays valid through
  `plan_vacuum_fill_candidates_for_target`.
- Updates the unsafe comment baseline from `4135` to `4129`.

## Review Focus

- Confirm guard drop order is safe: slot drops before heap relation, heap
  relation before index relation.
- Confirm the raw pointers passed into `VacuumFillPlanner` do not escape the
  lifetime of their guards.
- Confirm this test helper no longer needs local manual cleanup for early panic
  or `expect` paths.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
