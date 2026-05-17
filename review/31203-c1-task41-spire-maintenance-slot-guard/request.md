# Review Request: Task 41 SPIRE Maintenance Slot Guard

## Summary

This slice deletes the local `SpireHeapSlotGuard` and uses the shared
`TupleTableSlotGuard::single_for_heap` for SPIRE coordinator maintenance heap
slots.

Code commit: `b619d27686de098646d6ce5eadcb0ea96f0a2637`

## Changes

- Removed `SpireHeapSlotGuard` from
  `src/am/ec_spire/coordinator/lifecycle.rs`.
- Updated
  `build_relation_selected_scheduled_maintenance_input` in
  `src/am/ec_spire/coordinator/maintenance.rs` to use
  `crate::storage::slot_guard::TupleTableSlotGuard::single_for_heap`.
- Adjusted the shared slot guard drop comment so it correctly covers both
  constructors (`create` and `single_for_heap`).
- Updated `scripts/unsafe_comment_baseline.txt`.

## Baseline

- Before: `4174`
- After: `4171`

## Review Focus

- Confirm `single_for_heap` matches the removed local allocation:
  `MakeSingleTupleTableSlot((*relation).rd_att, table_slot_callbacks(relation))`.
- Confirm the shared guard lifetime still covers the scheduled split
  replacement heap-source build call.
- Confirm the error path preserves the prior message for slot allocation
  failure.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
