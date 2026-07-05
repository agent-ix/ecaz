# Review Request: Task 41 HNSW insert duplicate buffer guard

## Summary

Task 41 buffer-resource slice for HNSW duplicate heap-tid coalescing writes.

This uses `LockedBufferGuard` in `src/am/ec_hnsw/insert.rs` for:

- `coalesce_duplicate_heap_tid`
- `coalesce_duplicate_turbo_hot_heap_tid`
- `coalesce_duplicate_grouped_heap_tid`

Code commit: `7bb55545`

## Safety Effect

- Moves duplicate element buffer open/exclusive-lock/release ownership into
  `LockedBufferGuard`.
- Removes direct `ReadBufferExtended`, `LockBuffer`, `BufferGetPageSize`, and
  `UnlockReleaseBuffer` use from the three duplicate-coalescing helpers.
- Keeps `GenericXLogTxn::finish` before the guard drops on both changed and
  no-op duplicate paths, preserving the previous WAL lifecycle ordering.
- Leaves append paths that switch between existing pages and `P_NEW` unchanged
  for a separate guard slice.
- Updates the unsafe comment baseline from `3797` to `3782`.

## Review Focus

- Confirm WAL registration uses the same underlying buffer previously passed
  directly to `GenericXLogRegisterBuffer`.
- Confirm no page pointer or tuple bytes escape the locked buffer guard scope.
- Confirm the no-op duplicate paths preserve the previous `finish`-then-release
  ordering.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
