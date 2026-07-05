# Review Request: Task 41 IVF posting append buffer guard

## Summary

Task 41 buffer-resource slice for IVF posting-list append helpers.

This uses `LockedBufferGuard` in `src/am/ec_ivf/page.rs` for:

- `try_append_ivf_posting_to_block`
- `append_ivf_posting_to_new_block`

It also addresses reviewer feedback from packet `31237` by adding a
module-level constructor map to `src/storage/buffer_guard.rs`.

Code commit: `ffea9dfa`

## Safety Effect

- Moves posting append `UnlockReleaseBuffer` ownership into
  `LockedBufferGuard`.
- Uses `read_main_locked` for the `RBM_ZERO_AND_LOCK` / `P_NEW` path to avoid
  double-locking buffers PostgreSQL returns already locked.
- Keeps existing posting-list pages under exclusive locks while page free space
  and tuple insertion are updated.
- Updates the unsafe comment baseline from `3922` to `3912`.

## Review Focus

- Confirm the existing-block append paths still record free space before
  returning `Ok(None)` or `Ok(Some(...))`.
- Confirm the `P_NEW` path does not double-lock the buffer.
- Confirm WAL state is dropped or finished before the locked buffer guard drops.
- Confirm the new `buffer_guard.rs` module docs accurately describe the guard
  constructor choices.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
