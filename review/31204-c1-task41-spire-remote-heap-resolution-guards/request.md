# Review Request: Task 41 SPIRE Remote Heap Resolution Guards

## Summary

This slice migrates SPIRE coordinator remote heap resolution from local manual
heap relation and tuple slot cleanup to the shared `HeapRelationGuard` and
`TupleTableSlotGuard::single_for_heap`.

Code commit: `279a3311f366ebd43014381237b5269f267fa6c5`

## Changes

- Removed the local `remote_search_heap_slot` helper from
  `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`.
- Replaced the raw `table_open` / `table_close` pair in
  `remote_search_heap_candidate_rows_from_compact_candidates` with
  `HeapRelationGuard::try_access_share`.
- Replaced the raw `MakeSingleTupleTableSlot` /
  `ExecDropSingleTupleTableSlot` ownership path with
  `TupleTableSlotGuard::single_for_heap`.
- Kept the existing heap relation pointer flow by deriving raw pointers from
  the live guards at each callsite.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: `4171`
- After: `4165`

## Review Focus

- Confirm the slot guard drops before the heap relation guard because the slot
  is declared after the relation guard.
- Confirm early returns after opening the heap relation now drop through the
  heap relation guard instead of needing manual `table_close`.
- Confirm the raw heap relation pointer passed into indexed-attribute and
  heap-row loading calls is only derived from a live `HeapRelationGuard`.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
