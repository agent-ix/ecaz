# Task 91 Packet 011: DiskANN Build Binding Rename

## Summary

This packet renames the DiskANN build-time storage adapter:

- `DiskannBuildCodec` -> `DiskannBuildBinding`

The rename updates `src/am/ec_diskann/quantizer.rs`, `insert.rs`, and `ambuild.rs`. Behavior is unchanged: the binding still owns build/insert storage-format preparation, payload encoding, search-code discriminator metadata, binary-sidecar availability, and grouped-PQ model exposure.

## Scope Notes

- Code commit: `e79f41ac6 Rename DiskANN build codec binding`
- This closes the remaining Task 91 §6 adapter-name cleanup called out by reviewer feedback on Packet 007.
- This packet is intentionally naming-only; Packet 010 handled DiskANN prefilter scoring through `QuantCodec`.
- This packet does not land DiskANN TurboQuant search-code support.
- No ISA or Graviton target behavior changes are introduced.

## Validation

See `artifacts/manifest.md`.

Commands run:

- `cargo test --lib am::ec_diskann::quantizer::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_diskann::routine::tests --no-default-features --features pg18`
- `git diff --check`

Key results:

- DiskANN quantizer tests: 6 passed, 0 failed.
- DiskANN routine tests: 24 passed, 0 failed.
- `git diff --check`: passed.

## Review Ask

Please review whether `DiskannBuildBinding` now clearly names the DiskANN storage-binding role while leaving scoring responsibilities under `QuantCodec`.
