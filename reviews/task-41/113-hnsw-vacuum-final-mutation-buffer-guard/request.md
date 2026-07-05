# Review Request: Task 41 HNSW vacuum final mutation buffer guard

## Summary

Task 41 buffer-resource slice for the remaining HNSW vacuum mutation helpers.

This uses `LockedBufferGuard` in `src/am/ec_hnsw/vacuum.rs` for:

- `apply_repair_plans_on_page`
- `finalize_fully_dead_elements_on_page_with_storage`

Code commit: `dff6021f`

## Safety Effect

- Moves the remaining HNSW vacuum exclusive buffer open/lock/release ownership
  into `LockedBufferGuard`.
- Removes direct `ReadBufferExtended`, `LockBuffer`, `BufferGetPage`,
  `BufferGetPageSize`, and `UnlockReleaseBuffer` use from
  `src/am/ec_hnsw/vacuum.rs`.
- Keeps `GenericXLogTxn::finish` before guard drop on changed paths, and keeps
  no-op paths releasing by guard drop.
- Updates the unsafe comment baseline from `3721` to `3711`.

## Review Focus

- Confirm `apply_repair_plans_on_page` still aborts the generic WAL transaction
  before releasing the buffer when no tuple changes are needed.
- Confirm `finalize_fully_dead_elements_on_page_with_storage` still holds the
  exclusive lock while collecting updates and applying WAL changes.
- Confirm no page pointer or decoded tuple view escapes the guard scope.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
