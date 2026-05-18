# Review Request: Task 41 IVF rerank prefetch buffer guard

## Summary

Task 41 buffer-resource slice for IVF heap rerank prefetch.

This uses `PinnedBufferGuard` in `src/am/ec_ivf/scan.rs` for:

- PG18 `prefetch_heap_rerank_blocks`

Code commit: `0d2727eb`

## Safety Effect

- Moves read-stream returned buffer pin release into `PinnedBufferGuard`.
- Removes the direct manual `ReleaseBuffer` from the IVF scan rerank prefetch
  loop.
- Documents the local adoption invariant for the read-stream returned pin.
- Updates the unsafe comment baseline from `3840` to `3839`.

## Review Focus

- Confirm the read-stream buffer is adopted exactly once and released by the
  guard at the end of each loop iteration.
- Confirm the guard is scoped tightly enough that prefetch does not retain heap
  buffer pins across the full stream.
- Confirm the non-PG18 `PrefetchBuffer` path is unchanged.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
