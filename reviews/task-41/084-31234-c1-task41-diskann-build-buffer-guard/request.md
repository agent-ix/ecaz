# Review Request: Task 41 DiskANN build buffer guard

## Summary

Task 41 buffer-resource slice for DiskANN build persistence.

This reuses `LockedBufferGuard` in `src/am/ec_diskann/ambuild.rs` for:

- metadata page initialization,
- metadata page overwrite,
- data page allocation during build persistence.

Code commit: `3541f808`

## Safety Effect

- Moves DiskANN build `ReadBufferExtended` / `LockBuffer` /
  `UnlockReleaseBuffer` ownership into `LockedBufferGuard`.
- Reuses `read_main_locked` for `RBM_ZERO_AND_LOCK` metadata and data page
  allocation.
- Keeps `GenericXLogTxn` local to each write helper and finished before the
  buffer guard drops.
- Updates the unsafe comment baseline from `4015` to `4002`.

## Review Focus

- Confirm the metadata helper receives a live `LockedBufferGuard` and does not
  retain the buffer handle beyond the guard lifetime.
- Confirm `write_data_pages` still initializes each newly allocated page before
  adding tuples.
- Confirm WAL transaction finish/drop order remains before buffer unlock.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
