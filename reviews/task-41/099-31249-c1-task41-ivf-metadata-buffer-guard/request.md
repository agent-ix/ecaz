# Review Request: Task 41 IVF metadata buffer guard

## Summary

Task 41 buffer-resource slice for IVF metadata pages.

This uses `LockedBufferGuard` in `src/am/ec_ivf/page.rs` for:

- `initialize_metadata_page`
- `read_metadata_page`
- `update_metadata_page`

Code commit: `43eef78a`

## Safety Effect

- Moves metadata-page buffer lock/release ownership into `LockedBufferGuard`.
- Uses `read_main_locked` for the `RBM_ZERO_AND_LOCK` new-page allocation path.
- Removes manual `UnlockReleaseBuffer` from metadata update success and error
  paths.
- Updates the unsafe comment baseline from `3864` to `3852`.

## Review Focus

- Confirm `initialize_metadata_page` uses the already-locked constructor only
  for the `P_NEW` / `RBM_ZERO_AND_LOCK` path.
- Confirm WAL registration still receives the same buffer identity via
  `buffer.buffer()`.
- Confirm `update_metadata_page` drops the `GenericXLogTxn` before the guard on
  metadata decode/update errors.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
