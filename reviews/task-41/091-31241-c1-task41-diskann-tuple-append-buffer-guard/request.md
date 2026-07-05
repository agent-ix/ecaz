# Review Request: Task 41 DiskANN tuple append buffer guard

## Summary

Task 41 buffer-resource slice for DiskANN raw tuple append.

This adds `LockedBufferGuard::block_number` and uses `LockedBufferGuard` in
`src/am/ec_diskann/insert.rs` for `append_raw_tuple_payload`.

Code commit: `deec73a8`

## Safety Effect

- Moves tuple append `UnlockReleaseBuffer` ownership into `LockedBufferGuard`.
- Uses `read_main_locked` for the `RBM_ZERO_AND_LOCK` / `P_NEW` path to avoid
  double-locking buffers PostgreSQL returns already locked.
- Keeps existing-page append attempts under exclusive locks.
- Explicitly drops the WAL transaction and buffer guard before recursing to
  append on `P_NEW` when an existing target page has insufficient free space.
- Updates the unsafe comment baseline from `3943` to `3938`.

## Review Focus

- Confirm the `P_NEW` path does not double-lock the buffer.
- Confirm the insufficient-free-space path releases both WAL state and the
  locked buffer before the recursive append attempt.
- Confirm the new `block_number` helper matches the guard-owned locked-buffer
  invariant.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
