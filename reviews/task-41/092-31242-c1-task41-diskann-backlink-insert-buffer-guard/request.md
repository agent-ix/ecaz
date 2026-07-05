# Review Request: Task 41 DiskANN backlink insert buffer guard

## Summary

Task 41 buffer-resource slice for opportunistic DiskANN backlink insertion.

This uses `LockedBufferGuard` in `src/am/ec_diskann/insert.rs` for
`add_backlinks_if_free`.

Code commit: `a31e7bc0`

## Safety Effect

- Moves backlink target page `UnlockReleaseBuffer` ownership into
  `LockedBufferGuard`.
- Keeps backlink target pages under exclusive locks while tuple updates are
  decoded and applied.
- Keeps GenericXLog cleanup behavior unchanged: finish on changed pages, drop
  unfinished transactions on no-change and error paths.
- Updates the unsafe comment baseline from `3938` to `3933`.

## Review Focus

- Confirm changed/no-change/error WAL paths still finish or abort before the
  locked buffer guard drops.
- Confirm `changed` accumulation is unchanged after moving buffer release into
  RAII.
- Confirm tuple pointers remain scoped to the locked buffer lifetime.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
