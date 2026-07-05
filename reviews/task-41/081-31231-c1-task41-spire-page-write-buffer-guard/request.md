# Review Request: Task 41 SPIRE page write buffer guard

## Summary

Task 41 buffer-resource follow-up for SPIRE object page write helpers.

This extends the `LockedBufferGuard` use from 31230 into the `RBM_NORMAL`
exclusive-lock write paths in `src/am/ec_spire/page.rs`:

- `rewrite_object_tuple_same_len`
- `delete_object_tuples_no_compact`
- `try_append_object_tuple_to_block`

Code commit: `8735efcc`

## Safety Effect

- Removes repeated manual `UnlockReleaseBuffer` calls from SPIRE object page
  write paths.
- Keeps WAL abort/finish explicit while the buffer guard owns lock and pin
  cleanup.
- Leaves the `RBM_ZERO_AND_LOCK` new-block path for a later constructor because
  that PostgreSQL read mode returns an already locked buffer.
- Updates the unsafe comment baseline from `4058` to `4037`.

## Review Focus

- Confirm `GenericXLogTxn` drops before `LockedBufferGuard` on error paths and
  successful exits.
- Confirm page pointers registered with `GenericXLogTxn` are not used after the
  buffer guard drops.
- Confirm the new-block path was intentionally left unchanged.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
