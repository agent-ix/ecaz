# Review Request: Task 41 DiskANN test-helper relation guards

## Summary

Task 41 follow-up for raw relation ownership in DiskANN PG test helpers in
`src/am/ec_diskann/routine.rs`.

This slice migrates several remaining manual test-helper relation lifetimes to
the shared guards:

- `index_metadata`
- `index_materialized_chain`
- `test_ec_diskann_session_list_size_override_changes_scan_width`
- `debug_vacuum_stats`
- `debug_vacuum_remove_heap_tids`
- the two vacuum-rewrite fixture callsites that call `apply_tuple_rewrites`

Code commit: `4675e65f`

## Safety Effect

- Removes manual `index_open` / `index_close` pairs from metadata/materialize
  helpers, session override inspection, vacuum stats, and rewrite fixture
  callsites.
- Removes the optional manual `table_open` / `table_close` pair from
  `debug_vacuum_remove_heap_tids`.
- Keeps raw relation pointers local to the existing PostgreSQL APIs while
  guard values own the actual close operations.
- Updates the unsafe comment baseline from `4129` to `4114`.

## Review Focus

- Confirm `IndexVacuumInfo.index` and `IndexVacuumInfo.heaprel` do not outlive
  the relation guards in the debug vacuum helpers.
- Confirm the `ShareUpdateExclusiveLock` and `RowExclusiveLock` modes are
  preserved after moving to `IndexRelationGuard::open`.
- Confirm the optional heap relation in `debug_vacuum_remove_heap_tids` preserves
  the prior null-heap behavior for `InvalidOid`.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
