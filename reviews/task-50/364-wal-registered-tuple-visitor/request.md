# Task 50 Review Request: WAL Registered Tuple Visitor

## Summary

This slice extends the page tuple contract from read-only locked buffers to
WAL-registered mutable pages.

Changes:

- `src/storage/wal.rs` adds a crate-private `RegisteredBufferPage` token from
  `GenericXLogTxn::register_locked_buffer_full_image_page`.
- The token owns the checked mutable tuple visitor for a WAL-registered page,
  tying mutation to both the `GenericXLogTxn` borrow and the
  `LockedBufferGuard`.
- `src/am/ec_diskann/insert.rs` now uses that token for duplicate-bind,
  backlink-add, and backlink-rewrite tuple mutation paths.
- Removed the DiskANN insert-local raw `page_tuple_location` and
  `with_page_tuple_bytes_mut` helpers.

The direct `unsafe { ... }` count moved from `1210` to `1206`.

Code commit: `3963d2b4f7950ab37474978f0c32c0e729229766`

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed. It still reports the known pre-existing SPIRE DML unused-import
  warning in `src/am/mod.rs`.
- `git diff --check` passed.
- Raw-boundary guard scan produced no matches. The `rg` command exits 1 because
  it found no rows.
- WAL tuple-helper scan confirms the DiskANN insert-local raw page tuple helper
  was removed and the remaining mutable tuple boundary is centralized in
  `src/storage/wal.rs`.
- Unsafe ledger check passed for `1206` current unsafe rows.

Artifacts are recorded in `artifacts/manifest.md`.
