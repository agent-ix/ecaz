# Review Request: Task 41 HNSW insert duplicate scan buffer guard

## Summary

Task 41 buffer-resource slice for HNSW duplicate-finder read scans.

This uses `LockedBufferGuard` in `src/am/ec_hnsw/insert.rs` for:

- `find_duplicate_element_tid`
- `find_duplicate_turbo_hot_element_tid`
- `find_duplicate_grouped_element_tid`

Code commit: `250bda40`

## Safety Effect

- Moves duplicate scan buffer open/share-lock/release ownership into
  `LockedBufferGuard`.
- Removes direct `ReadBufferExtended`, `LockBuffer`, `BufferGetPage`,
  `BufferGetPageSize`, and `UnlockReleaseBuffer` use from the three duplicate
  finder loops.
- Converts early duplicate-match returns to rely on guard drop for unlock and
  release, eliminating manual early-return cleanup.
- Leaves append/retry insert paths unchanged for separate guard slices.
- Updates the unsafe comment baseline from `3782` to `3767`.

## Review Focus

- Confirm early returns release the scan buffer through guard drop.
- Confirm tuple bytes and decoded element values do not borrow page memory past
  the guard scope.
- Confirm nested rerank payload loads in the TurboQuant/PqFastScan duplicate
  scans preserve the previous lock ordering.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
