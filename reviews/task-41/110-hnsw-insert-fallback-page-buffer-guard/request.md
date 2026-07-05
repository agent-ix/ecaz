# Review Request: Task 41 HNSW insert fallback page buffer guard

## Summary

Task 41 buffer-resource slice for HNSW insert fallback `P_NEW` page appends.

This uses `LockedBufferGuard::read_main_locked` in `src/am/ec_hnsw/insert.rs`
for:

- `append_heap_tuple_to_new_page`
- `append_turbo_hot_cold_tuple_to_new_page`
- `append_pq_fastscan_tuple_to_new_page`

Code commit: `14b80280`

## Safety Effect

- Moves fallback new-page buffer ownership into `LockedBufferGuard` for
  `RBM_ZERO_AND_LOCK` pages that PostgreSQL returns already locked.
- Removes direct `ReadBufferExtended`, `BufferGetPageSize`,
  `BufferGetBlockNumber`, and `UnlockReleaseBuffer` use from the three fallback
  append helpers.
- Keeps `GenericXLogTxn::finish` before the guard drops, preserving the previous
  WAL lifecycle ordering.
- Leaves the main append paths that choose between an existing page and `P_NEW`
  unchanged for a separate guard slice.
- Updates the unsafe comment baseline from `3763` to `3751`.

## Review Focus

- Confirm `RBM_ZERO_AND_LOCK` is correctly represented by
  `LockedBufferGuard::read_main_locked`.
- Confirm WAL registration uses the same underlying buffer previously passed
  directly to `GenericXLogRegisterBuffer`.
- Confirm no page pointer escapes the guard scope after `PageInit` and tuple
  insertion.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
