# Review Request: Task 41 IVF tuple read buffer guard

## Summary

Task 41 buffer-resource slice for IVF tuple reads.

This uses `LockedBufferGuard` in `src/am/ec_ivf/page.rs` for:

- `read_page_tuple`
- `find_next_tuple_with_tag`

Code commit: `3f6e28b2`

## Safety Effect

- Moves shared buffer lock/release ownership for IVF tuple decode helpers into
  `LockedBufferGuard`.
- Removes manual `UnlockReleaseBuffer` cleanup across early validation errors
  in `read_page_tuple`.
- Removes manual unlock/release from the per-block scan loop in
  `find_next_tuple_with_tag`.
- Updates the unsafe comment baseline from `3877` to `3864`.

## Review Focus

- Confirm tuple byte slices do not escape the locked buffer lifetime.
- Confirm early error paths in `read_page_tuple` still release the buffer by
  dropping the guard.
- Confirm `find_next_tuple_with_tag` still releases each scanned block before
  moving to the next block.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
