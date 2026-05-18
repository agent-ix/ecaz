# Review Request: Task 41 HNSW scan debug buffer guard

## Summary

Task 41 buffer-resource slice for HNSW scan debug graph-page reads.

This uses `LockedBufferGuard` in `src/am/ec_hnsw/scan_debug.rs` for:

- `debug_collect_element_tids_at_level`
- `debug_collect_element_tids_at_or_above_level`
- `debug_collect_element_tid_by_heap_tid`

Code commit: `1c7cdc26`

## Safety Effect

- Moves debug graph-page buffer open/share-lock/release ownership into
  `LockedBufferGuard`.
- Removes manual `ReadBufferExtended`, `LockBuffer`, `BufferGetPage`,
  `BufferGetPageSize`, and `UnlockReleaseBuffer` calls from the three helper
  loops.
- Keeps the guard alive across tuple inspection and `graph::load_exact_graph_element`
  calls so decoded tuple references remain scoped to the locked page.
- Updates the unsafe comment baseline from `3829` to `3817`.

## Review Focus

- Confirm the debug helpers do not retain page or tuple references beyond the
  `LockedBufferGuard` scope.
- Confirm the `pgrx::error!` path after failed buffer open is acceptable for
  these test/`pg_test` helpers.
- Confirm there are no remaining direct PostgreSQL buffer API calls in
  `src/am/ec_hnsw/scan_debug.rs`.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
