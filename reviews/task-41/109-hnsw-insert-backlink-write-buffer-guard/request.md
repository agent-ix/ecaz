# Review Request: Task 41 HNSW insert backlink write buffer guard

## Summary

Task 41 buffer-resource slice for HNSW backlink write mutation.

This uses `LockedBufferGuard` in `src/am/ec_hnsw/insert.rs` for:

- `add_backlinks_on_page`

Code commit: `a09293da`

## Safety Effect

- Moves backlink neighbor buffer open/exclusive-lock/release ownership into
  `LockedBufferGuard`.
- Removes direct `ReadBufferExtended`, `LockBuffer`, `BufferGetPageSize`, and
  `UnlockReleaseBuffer` use from the backlink write helper.
- Keeps `GenericXLogTxn::finish` or explicit abort/drop before the buffer guard
  drops, preserving the previous WAL lifecycle ordering.
- Updates the unsafe comment baseline from `3767` to `3763`.

## Review Focus

- Confirm WAL registration uses the same underlying buffer previously passed
  directly to `GenericXLogRegisterBuffer`.
- Confirm tuple bytes decoded from the WAL-registered page do not outlive the
  locked buffer guard scope.
- Confirm the unchanged retry behavior still returns after WAL finish/abort and
  then lets the buffer guard release the lock.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
