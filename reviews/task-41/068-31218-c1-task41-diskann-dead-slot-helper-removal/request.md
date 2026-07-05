# Review Request: Task 41 DiskANN dead slot helper removal

## Summary

Task 41 cleanup after the DiskANN slot-guard migrations.

`scan_state::allocate_heap_slot` in `src/am/ec_diskann/scan_state.rs` is now
unused because all DiskANN callsites were migrated to
`TupleTableSlotGuard::single_for_heap`. This slice deletes the dead helper and
its remaining `MakeSingleTupleTableSlot` unsafe site.

Code commit: `b2d449f7`

## Safety Effect

- Removes the last DiskANN-local raw heap slot allocation helper.
- Leaves DiskANN heap slot allocation centralized through
  `TupleTableSlotGuard`.
- Updates the unsafe comment baseline from `4099` to `4098`.

## Review Focus

- Confirm there are no remaining references to `scan_state::allocate_heap_slot`.
- Confirm the deleted helper had no behavior other than raw slot allocation now
  covered by `TupleTableSlotGuard::single_for_heap`.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
