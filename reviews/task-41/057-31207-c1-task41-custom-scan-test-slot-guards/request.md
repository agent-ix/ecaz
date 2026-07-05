# Review Request: Task 41 Custom Scan Test Slot Guards

## Summary

This slice migrates the custom scan payload-slot pg-test from manual heap
relation and tuple slot cleanup to the shared guards, then narrows the
remaining unsafe slot access to documented callsites.

Code commit: `2ac2993400e6618587199363148af04b65869d61`

## Changes

Updated `src/tests/custom_scan.rs`:

- Replaced manual `table_open` / `table_close` with
  `HeapRelationGuard::try_access_share`.
- Replaced manual `MakeSingleTupleTableSlot` /
  `ExecDropSingleTupleTableSlot` with
  `TupleTableSlotGuard::single_for_heap`.
- Removed the broad test-level `unsafe` block and kept narrow documented
  unsafe calls for the test-only payload store, slot attribute reads, and
  datum decodes.
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: `4142`
- After: `4141`

## Review Focus

- Confirm the slot guard is declared after the heap relation guard, so the slot
  drops before the heap relation closes.
- Confirm the documented unsafe slot reads are covered by the live slot guard
  and attributes from the test relation schema.
- Confirm the rewritten test keeps the same payload assertions.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
