# Review Request: Task 41 SPIRE relation-store prefetch buffer guard

## Summary

Task 41 invariant #3 slice for the PG18 SPIRE relation-object prefetch path.

This replaces raw `ReleaseBuffer` in
`src/am/ec_spire/storage/relation_store.rs` with `PinnedBufferGuard`.

Code commit: `8627b8e2`

## Safety Effect

- Adopts each PG18 read-stream buffer returned by
  `read_stream_next_buffer` with `PinnedBufferGuard::from_pinned`.
- Removes the raw `ReleaseBuffer` call from the SPIRE relation-store prefetch
  loop.
- Keeps the path prefetch-only: the pinned buffer guard is dropped immediately
  at the end of each loop iteration.
- Updates the unsafe comment baseline line map after the local insertion; the
  entry count remains `3701`.

## Review Focus

- Confirm the guard owns the same buffer pin that the old raw
  `ReleaseBuffer` call released.
- Confirm early pgrx ERROR paths release the pin through guard drop.
- Confirm this packet does not touch invariant #2 datum/source lifetime work.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
