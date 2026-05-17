# Review Request: Task 41 HNSW source build scan guards

## Summary

Task 41 production follow-up for the HNSW serial build scan used when
`build_source_column` is configured.

This slice replaces manual heap scan source ingestion cleanup in
`src/am/ec_hnsw/build.rs` with RAII guards:

- `TupleTableSlotGuard` owns the scan slot.
- `ActiveSnapshotGuard` owns `RegisterSnapshot` / `PushActiveSnapshot` cleanup.
- New `HeapScanGuard` owns `heap_beginscan` / `heap_endscan`.

Code commit: `4ee57350`

## Safety Effect

- Removes the manual `ExecDropSingleTupleTableSlot` cleanup from the source
  build scan.
- Removes the manual `PopActiveSnapshot` / `UnregisterSnapshot` cleanup from the
  source build scan.
- Removes the manual `heap_endscan` cleanup from the source build scan.
- Removes the special null-scan cleanup branch because already-created guards
  now clean up before the error is raised.
- Updates the unsafe comment baseline from `4097` to `4091`.

## Review Focus

- Confirm `HeapScanGuard::begin` owns exactly the `heap_beginscan` /
  `heap_endscan` pair and receives the same flags as before.
- Confirm guard declaration order keeps heap scan cleanup before active snapshot
  and slot cleanup.
- Confirm `slot_ptr` is only used while the slot guard is live.
- Confirm the null-scan error path still releases any already-created snapshot
  and slot through guard drops.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
