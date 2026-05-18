# Review Request: Task 41 SPIRE new page buffer guard

## Summary

Task 41 buffer-resource follow-up for SPIRE metadata initialization and new
object-page allocation.

This adds `LockedBufferGuard::read_main_locked` for PostgreSQL read modes that
return an already locked buffer, then migrates the remaining
`RBM_ZERO_AND_LOCK` paths in `src/am/ec_spire/page.rs`:

- `initialize_spire_metadata_block_zero`
- `append_object_tuple_to_new_block`

Code commit: `c01d9d62`

## Safety Effect

- Removes the remaining direct `ReadBufferExtended` / `UnlockReleaseBuffer`
  ownership pairs from `src/am/ec_spire/page.rs`.
- Keeps normal `RBM_NORMAL` locking on `read_main`, while `read_main_locked`
  records the already-locked `RBM_ZERO_AND_LOCK` contract.
- Keeps `GenericXLogTxn` cleanup before buffer unlock/release on error paths.
- Updates the unsafe comment baseline from `4037` to `4026`.

## Review Focus

- Confirm `read_main_locked` is only used with read modes that return locked
  buffers.
- Confirm WAL transaction drop order remains before `LockedBufferGuard` drop.
- Confirm metadata page initialization still handles both first-page allocation
  and existing metadata block rewrite paths.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
