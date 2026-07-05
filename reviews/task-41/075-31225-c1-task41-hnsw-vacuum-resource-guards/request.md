# Review Request: Task 41 HNSW vacuum resource guards

## Summary

Task 41 follow-up for HNSW vacuum heap resources in
`src/am/ec_hnsw/vacuum.rs`.

This slice replaces:

- `VacuumHeapSourceScorer`'s raw tuple slot and manual drop with
  `TupleTableSlotGuard`.
- `debug_vacuum_remove_heap_tids` raw `index_open` / `table_open` and manual
  close block with `IndexRelationGuard` and optional `HeapRelationGuard`.

Code commit: `383ce416`

## Safety Effect

- Removes manual `ExecDropSingleTupleTableSlot` from production HNSW vacuum
  source scoring.
- Removes raw debug vacuum `index_open` / `index_close` ownership.
- Removes raw debug vacuum `table_open` / `table_close` ownership.
- Keeps debug relation guard imports behind the same `test` / `pg_test` cfg as
  the helper.
- Updates the unsafe comment baseline from `4091` to `4086`.

## Review Focus

- Confirm `VacuumHeapSourceScorer` still clears the slot between heap source
  fetches while leaving slot destruction to the guard.
- Confirm debug vacuum relation guards stay live until after
  `ec_hnsw_amvacuumcleanup` copies the result.
- Confirm the explicit drop order preserves heap relation close before index
  relation close.
- Confirm optional heap relation handling still preserves the previous
  `InvalidOid` behavior.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
