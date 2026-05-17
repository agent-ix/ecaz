# Review Request: Task 41 DiskANN vacuum heap relation guard

## Summary

Task 41 production follow-up for DiskANN vacuum heap relation resolution in
`src/am/ec_diskann/routine.rs`.

This slice replaces the `(Relation, owned: bool)` convention and
`release_owned_vacuum_heap_relation` helper with
`ResolvedVacuumHeapRelation`, which stores an `Option<HeapRelationGuard>` for
the owned case and `None` for the borrowed case.

Code commit: `709e8bfd`

## Safety Effect

- Removes the production manual `table_open` / `table_close` pair used when
  `IndexVacuumInfo.heaprel` is absent.
- Deletes the ownership boolean and local release helper.
- Keeps borrowed heap relations borrowed while making owned heap relations close
  through `HeapRelationGuard::Drop`.
- Leaves `src/am/ec_diskann/routine.rs` without direct `index_open`,
  `table_open`, `index_close`, `table_close`, or the targeted manual
  `ExecDropSingleTupleTableSlot(slot)` pattern.
- Updates the unsafe comment baseline from `4105` to `4102`.

## Review Focus

- Confirm `ResolvedVacuumHeapRelation::owned` derives the raw pointer before
  moving the guard and keeps the guard alive across `fill_vacuum_neighbor_slots`.
- Confirm the borrowed path does not close `IndexVacuumInfo.heaprel`.
- Confirm error paths through `fill_vacuum_neighbor_slots` drop the owned heap
  guard before leaving the vacuum pass.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
