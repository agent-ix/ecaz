# Review Request: Task 41 DiskANN scan-state RAII

## Summary

Task 41 production follow-up for DiskANN scan heap/snapshot cleanup.

This slice replaces the scan path's `(Relation, owned)` and `(Snapshot, owned)`
tuples plus explicit `release_owned_scan_heap_state` call with two scoped
wrappers in `src/am/ec_diskann/scan_state.rs`:

- `ResolvedScanHeapRelation`
- `ResolvedScanSnapshot`

`ec_diskann_amrescan` now keeps those wrappers alive through heap rerank and
passes raw pointers via `as_ptr()` only where PostgreSQL APIs require them.

Code commit: `8d7f527f`

## Safety Effect

- Removes manual heap relation release from the DiskANN scan rerank path.
- Removes the explicit scan-state release call from `ec_diskann_amrescan`.
- Moves owned snapshot unregistering into `ResolvedScanSnapshot::Drop`.
- Keeps borrowed scan heap relations and borrowed snapshots borrowed.
- Updates the unsafe comment baseline from `4102` to `4099`.

## Review Focus

- Confirm `ResolvedScanHeapRelation` lives across slot creation, heap block
  prefetch, and exact heap rerank.
- Confirm `ResolvedScanSnapshot` lives across all `exact_heap_rerank_distance`
  calls and unregisters only snapshots registered by `resolve_scan_snapshot`.
- Confirm removing `release_owned_scan_heap_state` preserves the prior cleanup
  order on normal and `pgrx::error!` unwind paths.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
