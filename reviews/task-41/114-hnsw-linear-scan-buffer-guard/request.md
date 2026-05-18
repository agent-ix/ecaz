# Review Request: Task 41 HNSW linear scan buffer guard

## Summary

Task 41 buffer-resource slice for the HNSW linear fallback scan path.

This uses `LockedBufferGuard` in `src/am/ec_hnsw/scan.rs` for:

- PG18 `read_stream_next_buffer` linear scan buffers
- non-PG18 `ReadBufferExtended` linear scan buffers
- `select_linear_scan_result_from_buffer`

Code commit: `ac36a382`

## Safety Effect

- Moves HNSW linear fallback scan buffer lock/release ownership into
  `LockedBufferGuard`.
- Removes the caller-side raw `UnlockReleaseBuffer` calls from this path.
- Replaces selector-local raw `LockBuffer`, `BufferGetPage`, and
  `BufferGetPageSize` calls with guard-owned accessors.
- Keeps scoring outside the buffer lock by copying the selected tuple payload
  into owned Rust data and dropping the guard before `score_scan_element_result`.
- Updates the unsafe comment baseline from `3711` to `3705`.

## Review Focus

- Confirm every selected-result early return releases through guard drop before
  scoring.
- Confirm no page pointer or decoded page-backed borrow escapes the guard scope.
- Confirm the remaining raw `LockBuffer` matches in `src/am/ec_hnsw/scan.rs`
  are the separate graph-prefetch path, not the linear fallback path.
- Confirm this packet is only invariant #3 buffer ownership work and does not
  overlap the separate invariant #2 memory-context lifetime slice.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
