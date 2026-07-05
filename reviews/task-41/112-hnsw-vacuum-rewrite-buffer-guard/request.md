# Review Request: Task 41 HNSW vacuum rewrite buffer guard

## Summary

Task 41 buffer-resource slice for HNSW vacuum page rewrite helpers.

This uses `LockedBufferGuard` in `src/am/ec_hnsw/vacuum.rs` for:

- `rewrite_page_pass1`
- `rewrite_page_pass2`

Code commit: `503fb36a`

## Safety Effect

- Moves exclusive reopen/lock/release ownership for vacuum rewrite passes into
  `LockedBufferGuard`.
- Passes the locked guard by value into the private rewrite helpers, so both
  no-op and WAL-update paths release by guard drop.
- Removes direct `ReadBufferExtended`, `LockBuffer`, `BufferGetPage`,
  `BufferGetPageSize`, and `UnlockReleaseBuffer` use from the pass-one and
  pass-two rewrite helpers.
- Keeps `GenericXLogTxn::finish` before the guard drops on update paths, and
  preserves no-op return behavior by dropping the guard at function exit.
- Updates the unsafe comment baseline from `3733` to `3721`.

## Review Focus

- Confirm both rewrite helpers retain the same lock lifetime across planning
  and WAL update.
- Confirm WAL registration uses the same underlying buffer previously passed
  directly to `GenericXLogRegisterBuffer`.
- Confirm no page pointer or tuple view escapes the guard scope.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
