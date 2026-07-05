# Review Request: Task 41 HNSW heap rerank RAII state

## Summary

Task 41 production follow-up for HNSW grouped heap-f32 rerank state in
`src/am/ec_hnsw/scan.rs`.

This slice replaces the raw grouped heap rerank fields on `TqScanOpaque`
(`Relation`, owned booleans, `Snapshot`, raw slot pointer, source attnum, and
source kind) with one boxed `GroupedHeapRerankState`. The boxed state owns:

- `TupleTableSlotGuard`
- `ResolvedHnswScanSnapshot`
- `ResolvedHnswScanHeapRelation`
- `source::SourceAttribute`

The scan opaque remains zero-initializable by storing only a nullable pointer to
the boxed state.

Code commit: `615fb549`

## Safety Effect

- Removes manual `ExecDropSingleTupleTableSlot` from HNSW grouped heap rerank
  cleanup.
- Removes manual `table_open` / `table_close` from HNSW grouped heap rerank
  relation resolution.
- Moves registered snapshot unregister into `ResolvedHnswScanSnapshot::Drop`.
- Removes grouped heap rerank ownership booleans from the scan opaque.
- Updates the unsafe comment baseline from `4102` to `4099`.

## Review Focus

- Confirm `GroupedHeapRerankState` is dropped by
  `free_grouped_heap_rerank_state` on rescan and endscan paths.
- Confirm `TqScanOpaque` remains safe to allocate with `PgBox::alloc0` because
  it stores only a raw pointer to the Rust-owned state.
- Confirm the state field drop order preserves slot cleanup before snapshot and
  relation cleanup.
- Confirm borrowed heap relations and snapshots are not closed/unregistered.
- Confirm grouped heap source scoring cannot run without configured state.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
