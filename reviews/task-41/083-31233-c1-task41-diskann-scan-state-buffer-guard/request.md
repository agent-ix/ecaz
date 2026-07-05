# Review Request: Task 41 DiskANN scan-state buffer guard

## Summary

Task 41 buffer-resource slice for DiskANN scan-state materialization.

This reuses `LockedBufferGuard` in
`src/am/ec_diskann/scan_state.rs::materialize_chain_from_index` for:

- metadata page reads,
- data page chain materialization reads.

Code commit: `a9edc720`

## Safety Effect

- Moves DiskANN scan-state `ReadBufferExtended` / `LockBuffer` /
  `UnlockReleaseBuffer` ownership into `LockedBufferGuard`.
- Keeps buffer lifetimes block-scoped so decoded metadata and copied tuple
  bytes outlive the buffer guard, not borrowed page memory.
- Updates the unsafe comment baseline from `4026` to `4015`.

## Review Focus

- Confirm `metadata_result` and `page_result` blocks drop the guard before the
  function proceeds past the decoded/copy-owned data.
- Confirm the slice copies tuple bytes before the page buffer can drop.
- Confirm scan-state materialization still rejects invalid metadata and data
  page structures with the same errors.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
