# Review Request: Task 41 HNSW insert main page buffer guard

## Summary

Task 41 buffer-resource slice for HNSW insert main page append paths.

This uses `LockedBufferGuard` in `src/am/ec_hnsw/insert.rs` for:

- `append_heap_tuple`
- `append_turbo_hot_cold_tuple`
- `append_pq_fastscan_tuple`

Code commit: `030bec33`

## Safety Effect

- Moves main append buffer open/lock/release ownership into `LockedBufferGuard`.
- Uses `read_main_locked` when the target is `P_NEW` with `RBM_ZERO_AND_LOCK`,
  and `read_main` with an exclusive lock when the target is an existing block.
- Removes direct `ReadBufferExtended`, `LockBuffer`, `BufferGetPageSize`,
  `BufferGetBlockNumber`, and `UnlockReleaseBuffer` use from the three main
  append helpers.
- Preserves the previous fallback behavior: if an existing last page lacks
  space, the WAL transaction is dropped and the buffer guard is dropped before
  calling the fallback `*_to_new_page` allocator.
- Updates the unsafe comment baseline from `3751` to `3733`.

## Review Focus

- Confirm the conditional guard constructor matches PostgreSQL lock semantics
  for `RBM_ZERO_AND_LOCK` versus `RBM_NORMAL`.
- Confirm the existing-page full fallback still releases the lock before
  allocating the fallback page.
- Confirm WAL registration uses the same underlying buffer previously passed
  directly to `GenericXLogRegisterBuffer`.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
