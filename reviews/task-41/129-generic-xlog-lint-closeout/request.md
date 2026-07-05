# Task 41 GenericXLog lint closeout

## Summary

This packet requests review for the final lint gap identified in:

- `reviews/task-41/127-ffi-resource-closeout-live-smoke/feedback/2026-05-18-01-reviewer.md`

Code commit: `33e86790e7f429564bdb616e818084ad32fa2ee7`

Changes:

- Added a `RawApiRule` that confines `pg_sys::GenericXLogStart`,
  `pg_sys::GenericXLogFinish`, and `pg_sys::GenericXLogAbort` to
  `src/storage/wal.rs`.
- Added a negative self-test fixture for a raw `GenericXLogStart` call outside
  the guard module.
- Added an allowed self-test fixture for `GenericXLogFinish` and
  `GenericXLogAbort` inside `src/storage/wal.rs`.

## Safety Effect

`GenericXLogTxn` already provides the RAII wrapper for GenericXLog state. This
slice adds the missing forward-protection so future code cannot bypass that
wrapper without failing `make ffi-lint`.

## Review Focus

- Check that the regex covers the GenericXLog APIs that must stay in
  `src/storage/wal.rs`.
- Check that the fixtures prove both rejection and allowed-wrapper behavior.

## Validation

See `artifacts/manifest.md`.
