# Review Request: Task 41 HNSW insert source slot guard

## Summary

Task 41 production follow-up for HNSW live insert source scoring in
`src/am/ec_hnsw/insert.rs`.

This slice replaces `InsertHeapSourceScorer`'s raw tuple slot pointer and manual
`Drop` implementation with the shared `TupleTableSlotGuard`. The scorer still
uses the existing heap relation and `SnapshotSelfData`, but slot allocation and
drop are now owned by the guard.

Code commit: `0217674a`

## Safety Effect

- Removes the scorer-local manual `ExecDropSingleTupleTableSlot` cleanup.
- Replaces the insert source scorer's raw slot allocation with
  `TupleTableSlotGuard::single_for_heap`.
- Gives `TupleTableSlotGuard` a `Debug` derive so existing debug derivations on
  insert search state remain valid.
- Updates the unsafe comment baseline from `4099` to `4097`.

## Review Focus

- Confirm `InsertHeapSourceScorer` still clears the slot between heap source
  fetches without owning slot destruction manually.
- Confirm replacing `source::allocate_heap_slot` with
  `TupleTableSlotGuard::single_for_heap` is behaviorally equivalent for this
  heap relation.
- Confirm the new `Debug` derive on `TupleTableSlotGuard` does not expose any
  additional behavior beyond formatting the raw pointer value.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
