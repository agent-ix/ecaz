# Review Request: Task 41 IVF build buffer guard

## Summary

Task 41 buffer-resource slice for IVF build data-page flush.

This uses `LockedBufferGuard` in `src/am/ec_ivf/build.rs` for:

- `write_data_pages`

Code commit: `967867ca`

## Safety Effect

- Moves build-time data-page buffer ownership into `LockedBufferGuard`.
- Uses `read_main_locked` for the `P_NEW` / `RBM_ZERO_AND_LOCK` allocation
  path.
- Removes manual `UnlockReleaseBuffer` from the build data-page flush loop.
- Leaves `src/am/ec_ivf/build.rs` with no direct matches for
  `ReadBufferExtended`, `LockBuffer`, `UnlockReleaseBuffer`, `ReleaseBuffer`,
  `BufferGetPageSize`, or `BufferGetPage`.
- Updates the unsafe comment baseline from `3843` to `3840`.

## Review Focus

- Confirm `RBM_ZERO_AND_LOCK` still uses the already-locked constructor.
- Confirm WAL registration receives the same buffer identity via
  `buffer.buffer()`.
- Confirm the guard lifetime still covers `PageInit`, tuple insertion, and WAL
  finish for each staged data page.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
