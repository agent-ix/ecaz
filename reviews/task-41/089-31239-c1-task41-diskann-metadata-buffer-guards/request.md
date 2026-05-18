# Review Request: Task 41 DiskANN metadata buffer guards

## Summary

Task 41 buffer-resource slice for DiskANN metadata helpers.

This uses `LockedBufferGuard` in `src/am/ec_diskann/insert.rs` for:

- `read_metadata_page`
- `with_locked_metadata_page`

Code commit: `65122af7`

## Safety Effect

- Moves DiskANN metadata-page `UnlockReleaseBuffer` ownership into
  `LockedBufferGuard`.
- Keeps metadata reads under a shared lock and metadata rewrites under an
  exclusive lock.
- Keeps GenericXLog registration on the same buffer while using the guard as
  the buffer-release owner.
- Updates the unsafe comment baseline from `3957` to `3948`.

## Review Focus

- Confirm metadata page pointers do not escape the guard lifetime.
- Confirm `with_locked_metadata_page` still aborts unfinished WAL records on
  callback or decode errors via `GenericXLogTxn`.
- Confirm lock modes match the prior manual implementation.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
