# Review Request: Task 41 pinned read-stream buffer guard

## Summary

Task 41 buffer-resource slice for pin-only read-stream buffers.

This adds `PinnedBufferGuard` in `src/storage/buffer_guard.rs` and uses it for
small PG18 read-stream loops in:

- `src/am/ec_spire/scan/relation.rs`
- `src/am/ec_ivf/page.rs`

Code commit: `3645a329`

## Safety Effect

- Adds a pin-only sibling to `LockedBufferGuard` for buffers returned already
  pinned by PostgreSQL APIs such as `read_stream_next_buffer`.
- Moves selected `ReleaseBuffer` ownership into `PinnedBufferGuard`.
- Keeps IVF visitor calls inside a block so the pin is released before stream
  error handling calls `read_stream_end`.
- Updates the unsafe comment baseline from `4002` to `3999`.

## Review Focus

- Confirm `PinnedBufferGuard::from_pinned` is used only after callers filter
  `InvalidBuffer`.
- Confirm IVF error paths release the pinned buffer before ending the read
  stream.
- Confirm the guard’s `block_number` helper does not outlive the pinned buffer
  contract.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
