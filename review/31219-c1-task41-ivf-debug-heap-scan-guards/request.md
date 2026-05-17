# Review Request: Task 41 IVF debug heap scan guards

## Summary

Task 41 follow-up for IVF debug heap-backed scan helpers in
`src/am/ec_ivf/scan.rs`.

This slice changes `DebugHeapBackedScan` from raw index/heap relation fields to
`IndexRelationGuard` and `HeapRelationGuard`. The debug end helper still ends
the scan and pops/unregisters the active snapshot explicitly; relation closes
are now owned by the guards.

Code commit: `5845b45f`
Merge/base refresh commits: `51bdeae8`, `67b714e4`

## Safety Effect

- Removes manual `index_open` / `index_close` and `table_open` / `table_close`
  pairs from `debug_begin_heap_backed_scan` and `debug_end_heap_backed_scan`.
- Keeps `index_beginscan` fed by raw relation pointers derived from live guards.
- Preserves explicit `index_endscan`, `PopActiveSnapshot`, and
  `UnregisterSnapshot` behavior.
- Updates the merged unsafe comment baseline to `4105`.

## Review Focus

- Confirm `DebugHeapBackedScan` guard fields stay alive until after
  `debug_end_heap_backed_scan` calls `index_endscan`.
- Confirm the scan-null error path still pops/unregisters the pushed snapshot
  and then lets relation guards drop.
- Confirm the cfg-gated guard import avoids normal-build unused-import warnings.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
