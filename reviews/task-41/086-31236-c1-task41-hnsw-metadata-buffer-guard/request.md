# Review Request: Task 41 HNSW metadata buffer guard

## Summary

Task 41 buffer-resource slice for HNSW metadata page helpers.

This uses `LockedBufferGuard` in `src/am/ec_hnsw/shared.rs` for:

- `initialize_metadata_page`
- `update_metadata_page`
- `with_locked_metadata_page`

Code commit: `3840544f`

## Safety Effect

- Moves selected metadata-page `UnlockReleaseBuffer` ownership into
  `LockedBufferGuard`.
- Keeps the existing lock modes: new metadata pages use
  `RBM_ZERO_AND_LOCK`, and existing metadata pages use exclusive locks.
- Keeps the WAL full-image registration flow unchanged while using the guard
  as the only buffer-release owner.
- Updates the unsafe comment baseline from `3999` to `3983`.

## Review Focus

- Confirm the `RBM_ZERO_AND_LOCK` path is not double-locked.
- Confirm each WAL registration still receives the same buffer that owns the
  metadata page being initialized or rewritten.
- Confirm no metadata page pointer escapes the locked-buffer guard lifetime.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
