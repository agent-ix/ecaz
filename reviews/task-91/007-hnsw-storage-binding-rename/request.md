# Task 91 Packet 007: HNSW Storage Binding Rename

## Summary

This packet addresses the naming part of Task 91 acceptance criterion 2:

- renames `src/am/ec_hnsw/codec.rs` to
  `src/am/ec_hnsw/storage_binding.rs`;
- renames `HnswStorageCodec` to `HnswStorageBinding`;
- renames the graph descriptor accessor from `codec()` to
  `storage_binding()`;
- keeps the storage-binding responsibilities unchanged: reloption mapping,
  metadata identity, tuple-fit checks, and page-format selection.

No quant scoring path is migrated by this slice. It only removes the HNSW
storage adapter's misleading "codec" name so `codec` can mean the common
`QuantCodec` trait in Task 91 follow-up work.

## Code

- `cab0cc580f3c` - `Rename HNSW storage codec binding`

## Validation

Artifacts are packet-local under `artifacts/`:

- `artifacts/cargo-test-hnsw-storage-binding.log`
  - command: `cargo test --lib am::ec_hnsw::storage_binding::tests --no-default-features --features pg18`
  - result: 4 passed; 0 failed
- `artifacts/cargo-test-hnsw-graph.log`
  - command: `cargo test --lib am::ec_hnsw::graph::tests --no-default-features --features pg18`
  - result: 18 passed; 0 failed
- `artifacts/git-diff-check.log`
  - command: `git diff --check`
  - result: passed

## Review Notes

This is intentionally narrow. HNSW scoring migration remains open for Task 91
Phase 4: TurboQuant exact modes, gamma fallback, PqFastScan, and RaBitQ still
need to route through `QuantCodec`.
