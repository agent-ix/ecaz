# Review Request: Task 41 HNSW shared read buffer guard

## Summary

Task 41 buffer-resource slice for HNSW shared read helpers.

This extends `LockedBufferGuard` with a constructor for buffers that PostgreSQL
returns already pinned, then uses the guard in `src/am/ec_hnsw/shared.rs` for:

- `count_element_tuples`
- `highest_level_live_entry_candidate`
- `read_metadata_page`
- `read_data_page`

Code commit: `390b18b4`

## Safety Effect

- Moves selected HNSW shared read-path `UnlockReleaseBuffer` ownership into
  `LockedBufferGuard`.
- Adds `LockedBufferGuard::lock_pinned` for PG18 read-stream buffers that need
  a lock before page inspection.
- Keeps existing shared-lock semantics for page reads and read-stream tuple
  counting.
- Updates the unsafe comment baseline from `3983` to `3966`.

## Review Focus

- Confirm `lock_pinned` is used only after PostgreSQL hands the caller a pinned
  buffer.
- Confirm the PG18 read-stream loop still releases each buffer before the next
  `read_stream_next_buffer` call and before `read_stream_end`.
- Confirm no HNSW page pointer escapes the `LockedBufferGuard` lifetime.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
