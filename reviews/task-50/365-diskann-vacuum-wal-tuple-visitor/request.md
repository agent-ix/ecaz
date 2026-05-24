# Task 50 Review Request: DiskANN Vacuum WAL Tuple Visitor

## Summary

This slice continues the P4 page tuple / line-pointer contract rollout by
applying the WAL registered-page tuple visitor from packet 364 to DiskANN vacuum
rewrite paths.

Changes:

- `src/am/ec_diskann/routine.rs` now uses
  `GenericXLogTxn::register_locked_buffer_full_image_page` and
  `RegisteredBufferPage::visit_tuple_bytes_mut` for vacuum rewrite tuple
  mutation.
- The pg_test-only raw tuple rewrite helper uses the same token.
- Removed the DiskANN vacuum-local raw `vacuum_page_tuple_location` and
  `with_vacuum_page_tuple_bytes_mut` helpers.

The direct `unsafe { ... }` count moved from `1206` to `1204`.

Code commit: `99cb9ef41b81e24dc4ba5be4272ed20aefbfb962`

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed. It still reports the known pre-existing SPIRE DML unused-import
  warning in `src/am/mod.rs`.
- `git diff --check` passed.
- Raw-boundary guard scan produced no matches. The `rg` command exits 1 because
  it found no rows.
- DiskANN vacuum WAL helper scan confirms the local raw vacuum tuple helpers
  were removed and remaining mutable tuple pointer work is centralized in
  `src/storage/wal.rs`.
- Unsafe ledger check passed for `1204` current unsafe rows.

Artifacts are recorded in `artifacts/manifest.md`.
