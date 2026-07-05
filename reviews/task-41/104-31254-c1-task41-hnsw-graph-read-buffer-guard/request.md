# Review Request: Task 41 HNSW graph read buffer guard

## Summary

Task 41 buffer-resource slice for HNSW graph tuple reads.

This uses `LockedBufferGuard` in `src/am/ec_hnsw/graph.rs` for:

- `read_page_tuple`

Code commit: `d4954030`

## Safety Effect

- Moves ordinary graph tuple buffer open/share-lock/release ownership into
  `LockedBufferGuard`.
- Removes manual `UnlockReleaseBuffer` cleanup from the out-of-range,
  unused-slot, invalid-bounds, and success paths.
- Leaves the PG18 prefetched-buffer path unchanged; it still works over a
  caller-pinned buffer and should be handled as a separate ownership slice.
- Updates the unsafe comment baseline from `3836` to `3829`.

## Review Focus

- Confirm decoded graph tuple refs do not escape the locked buffer lifetime.
- Confirm the pgrx error paths still release by unwinding through the guard.
- Confirm the unchanged `read_page_tuple_from_buffer` path remains scoped to
  caller-owned prefetched buffers.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
