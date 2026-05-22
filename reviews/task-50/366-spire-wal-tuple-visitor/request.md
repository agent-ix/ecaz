# Task 50 Review Request: SPIRE WAL Tuple Visitor

## Summary

This slice applies the shared WAL registered-page tuple visitor to the SPIRE
same-length object rewrite path.

Changes:

- `src/am/ec_spire/page.rs` now uses
  `GenericXLogTxn::register_locked_buffer_full_image_page` and
  `RegisteredBufferPage::visit_tuple_bytes_mut` in
  `rewrite_object_tuple_same_len`.
- Removed the SPIRE-local `with_wal_object_tuple_mut` helper and its raw page
  tuple pointer handling.

The direct `unsafe { ... }` count moved from `1204` to `1202`.

Code commit: `0c4e9f528f98066400e0f04a04d603cb16372a01`

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed. It still reports the known pre-existing SPIRE DML unused-import
  warning in `src/am/mod.rs`.
- `git diff --check` passed.
- Raw-boundary guard scan produced no matches. The `rg` command exits 1 because
  it found no rows.
- SPIRE WAL helper scan confirms the SPIRE-local mutable object tuple helper was
  removed and mutable tuple pointer work is centralized in `src/storage/wal.rs`.
- Unsafe ledger check passed for `1202` current unsafe rows.

Artifacts are recorded in `artifacts/manifest.md`.
