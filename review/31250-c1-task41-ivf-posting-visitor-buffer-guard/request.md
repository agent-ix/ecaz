# Review Request: Task 41 IVF posting visitor buffer guard

## Summary

Task 41 buffer-resource slice for IVF posting visitor helpers.

This uses `LockedBufferGuard` in `src/am/ec_ivf/page.rs` for:

- PG18 read-stream posting visitors
- non-PG18 posting visitor fallbacks
- `visit_ivf_postings_from_buffer`
- `visit_all_ivf_postings_from_buffer`
- `visit_all_ivf_posting_refs_from_buffer`
- `rewrite_ivf_postings_from_exclusive_buffer`

Code commit: `a00c3e57`

## Safety Effect

- Moves caller-pinned read-stream buffers directly into `LockedBufferGuard`
  with `lock_pinned`.
- Passes guard references through posting visitor helpers instead of raw
  `pg_sys::Buffer` values.
- Removes manual shared lock/unlock from the posting visitor helpers.
- Moves the final raw `BufferGetPageSize` in IVF posting rewrite into the
  guard API.
- Leaves `src/am/ec_ivf/page.rs` with no direct matches for
  `ReadBufferExtended`, `LockBuffer`, `UnlockReleaseBuffer`, `ReleaseBuffer`,
  `BufferGetPageSize`, `BufferGetPage`, or `PinnedBufferGuard`.
- Updates the unsafe comment baseline from `3852` to `3843`.

## Review Focus

- Confirm PG18 read-stream buffers are no longer also wrapped in
  `PinnedBufferGuard`, avoiding double release while preserving pin release on
  guard drop.
- Confirm posting tuple refs cannot escape the locked guard lifetime.
- Confirm non-PG18 visitor fallbacks now acquire the same share lock through
  `LockedBufferGuard::read_main`.
- Confirm `rewrite_ivf_postings_from_exclusive_buffer` still registers the
  same buffer with generic WAL via `buffer.buffer()`.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
