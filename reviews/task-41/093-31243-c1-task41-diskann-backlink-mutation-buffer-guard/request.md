# Review Request: Task 41 DiskANN backlink mutation buffer guard

## Summary

Task 41 buffer-resource slice for DiskANN backlink rewrite mutations.

This uses `LockedBufferGuard` in `src/am/ec_diskann/insert.rs` for
`apply_backlink_mutations`.

Code commit: `99729ff7`

## Safety Effect

- Moves backlink rewrite target page `UnlockReleaseBuffer` ownership into
  `LockedBufferGuard`.
- Keeps rewrite target pages under exclusive locks while tuple mutations are
  decoded and applied.
- Keeps retry collection behavior unchanged while moving buffer release to RAII.
- Updates the unsafe comment baseline from `3933` to `3928`.

## Review Focus

- Confirm `retries` accumulation and final de-duplication are unchanged.
- Confirm changed/no-change/error WAL paths still finish or abort before the
  locked buffer guard drops.
- Confirm tuple pointers remain scoped to the locked buffer lifetime.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
