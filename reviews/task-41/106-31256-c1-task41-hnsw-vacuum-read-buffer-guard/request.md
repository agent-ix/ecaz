# Review Request: Task 41 HNSW vacuum read buffer guard

## Summary

Task 41 buffer-resource slice for HNSW vacuum read-only planning passes.

This uses `LockedBufferGuard` in `src/am/ec_hnsw/vacuum.rs` for:

- `run_bulkdelete_with_adapter` share-lock pass-one planning
- `collect_repair_requests`
- `unlink_deleted_graph_connections` share-lock pass-two planning
- `top_up_repair_replacements_from_linear_scan`

Code commit: `c412a2a4`

## Safety Effect

- Moves read-only vacuum buffer open/share-lock/release ownership into
  `LockedBufferGuard`.
- Removes direct `ReadBufferExtended`, `LockBuffer`, `BufferGetPage`,
  `BufferGetPageSize`, and `UnlockReleaseBuffer` use from the converted
  read-only planning loops.
- Preserves the old ordering where share locks are released before reopening
  the same block for exclusive rewrite.
- Leaves exclusive/WAL mutation paths unchanged for a separate write-buffer
  guard slice.
- Updates the unsafe comment baseline from `3817` to `3797`.

## Review Focus

- Confirm the scoped blocks release share locks before any exclusive reopen on
  the same block.
- Confirm page pointers passed to planning helpers do not outlive the
  `LockedBufferGuard` scope.
- Confirm the remaining raw buffer API sites in `vacuum.rs` are exclusive
  mutation/WAL paths or explicit exclusive reopen points, not missed read-only
  planning loops.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
