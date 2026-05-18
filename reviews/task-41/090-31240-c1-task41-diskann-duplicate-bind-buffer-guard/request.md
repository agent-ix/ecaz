# Review Request: Task 41 DiskANN duplicate-bind buffer guard

## Summary

Task 41 buffer-resource slice for DiskANN duplicate-bind page patching.

This uses `LockedBufferGuard` in `src/am/ec_diskann/insert.rs` for
`apply_duplicate_bind_patches`.

Code commit: `9e0c3b75`

## Safety Effect

- Moves duplicate-bind target page `UnlockReleaseBuffer` ownership into
  `LockedBufferGuard`.
- Keeps target pages under exclusive locks while patches are decoded and
  applied.
- Keeps GenericXLog transaction cleanup separate: unfinished transactions still
  abort via `GenericXLogTxn` when an error path drops `wal_txn`.
- Updates the unsafe comment baseline from `3948` to `3943`.

## Review Focus

- Confirm `wal_txn` is dropped or finished before the locked buffer guard is
  dropped on all page-result paths.
- Confirm `RetryReplan` still returns after the current page scope exits and
  releases the locked buffer.
- Confirm page tuple pointers remain scoped to the locked buffer lifetime.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
