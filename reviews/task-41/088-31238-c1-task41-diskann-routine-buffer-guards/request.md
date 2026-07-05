# Review Request: Task 41 DiskANN routine buffer guards

## Summary

Task 41 buffer-resource slice for DiskANN routine vacuum rewrite helpers.

This uses the shared buffer guards in `src/am/ec_diskann/routine.rs` for:

- `apply_tuple_rewrites`
- `write_raw_tuple_bytes`
- `prefetch_heap_rerank_blocks` on PG18

Code commit: `4b7c1402`

## Safety Effect

- Moves selected DiskANN vacuum rewrite `UnlockReleaseBuffer` ownership into
  `LockedBufferGuard`.
- Reuses `PinnedBufferGuard` for the PG18 heap-rerank prefetch read-stream
  loop, replacing manual `ReleaseBuffer`.
- Keeps existing exclusive-lock semantics for tuple rewrite pages.
- Keeps the existing GenericXLog transaction wrapper as the WAL cleanup owner.
- Updates the unsafe comment baseline from `3966` to `3957`.

## Review Focus

- Confirm `apply_tuple_rewrites` still drops the locked page before moving to
  the next rewrite block or returning `RetryReplan`.
- Confirm rewrite error paths abort unfinished GenericXLog transactions and
  release the locked buffer by RAII.
- Confirm `PinnedBufferGuard` is appropriate for PG18 prefetch buffers where no
  page inspection or lock is needed.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
