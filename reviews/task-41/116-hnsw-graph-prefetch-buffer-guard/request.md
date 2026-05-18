# Review Request: Task 41 HNSW graph prefetch buffer guard

## Summary

Task 41 buffer-resource slice for the PG18 HNSW graph-prefetch path.

This adds `PinnedBufferLockGuard` in `src/storage/buffer_guard.rs` and uses it
for prefetched graph buffers in:

- `src/am/ec_hnsw/scan.rs`
- `src/am/ec_hnsw/graph.rs`

Code commit: `d626d009`

## Safety Effect

- Changes the graph prefetch map from raw `pg_sys::Buffer` pins to
  `PinnedBufferGuard` values.
- Adds a lock-only guard for temporarily locking a map-owned pinned buffer.
  It unlocks on drop without releasing the pin.
- Removes raw `ReleaseBuffer` and raw `LockBuffer` ownership from the HNSW
  graph prefetch path.
- Replaces PG18 graph tuple reads from prefetched buffers with guard accessors
  instead of raw `BufferGetPage` / `BufferGetPageSize`.
- Updates the unsafe comment baseline from `3705` to `3701`.

## Review Focus

- Confirm `PinnedBufferLockGuard` cannot outlive the borrowed
  `PinnedBufferGuard` pin.
- Confirm the lock-only guard unlocks without releasing the map-owned pin,
  avoiding both leaks and double release.
- Confirm prefetched pins release through `PinnedBufferGuard` drop on all
  returns and pgrx ERROR unwind paths.
- Confirm `src/am/ec_hnsw/scan.rs` and `src/am/ec_hnsw/graph.rs` have no raw
  buffer lock/release/page-access calls left for this path.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
