# Review Request: Task 41 IVF heap rerank RAII state

## Summary

Task 41 production follow-up for IVF heap-f32 rerank state in
`src/am/ec_ivf/scan.rs`.

This slice replaces the raw heap rerank fields on `EcIvfScanOpaque`
(`Relation`, owned booleans, `Snapshot`, raw slot pointer, and attnum) with one
boxed `IvfHeapRerankState`. The boxed state owns:

- `TupleTableSlotGuard`
- `ResolvedIvfScanSnapshot`
- `ResolvedIvfScanHeapRelation`

The scan opaque remains zero-initializable by storing only a nullable pointer to
the boxed state.

Code commit: `bd07d424`

## Safety Effect

- Removes manual `ExecDropSingleTupleTableSlot` from IVF heap rerank cleanup.
- Removes manual `table_open` / `table_close` from IVF heap rerank relation
  resolution.
- Moves registered snapshot unregister into `ResolvedIvfScanSnapshot::Drop`.
- Removes the heap rerank ownership booleans from the scan opaque.
- Updates the unsafe comment baseline from `4105` to `4102`.

## Review Focus

- Confirm `IvfHeapRerankState` is dropped by `free_heap_rerank_state` on rescan
  and endscan paths.
- Confirm `EcIvfScanOpaque` remains safe to allocate with `PgBox::alloc0`
  because it stores only a raw pointer to the Rust-owned state.
- Confirm the state field drop order preserves slot cleanup before snapshot and
  relation cleanup.
- Confirm borrowed heap relations and snapshots are not closed/unregistered.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
