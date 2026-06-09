# Task 91 Packet 008: HNSW TurboQuant Scan QuantCodec

## Summary

This packet starts Task 91 Phase 4 by routing HNSW's TurboQuant scan exact
scorer through a local `QuantCodec` adapter:

- adds `HnswTurboQuantScanCodec`;
- adds `HnswTurboQuantPreparedQuery` covering the existing scan-owned prepared
  states:
  - exact `PreparedQuery`;
  - full LUT no-QJL 4-bit;
  - tiled LUT no-QJL 4-bit;
  - int8 approximation no-QJL 4-bit;
- changes `score_scan_element_result` to select the existing prepared state and
  call `QuantCodec::score_ip_candidate`;
- preserves the score polarity, stats accounting, scan-owned prepared-query
  lifetime, and existing opaque layout.

This is intentionally a TurboQuant-only HNSW slice. PqFastScan grouped search,
PqFastScan binary traversal, and RaBitQ traversal scoring still have direct
scorer paths and remain open for later Task 91 Phase 4 slices.

## Code

- `ed5fb20e9706` - `Route HNSW TurboQuant scan scoring through QuantCodec`

## Validation

Artifacts are packet-local under `artifacts/`:

- `artifacts/cargo-test-hnsw-turboquant-codec.log`
  - command: `cargo test --lib am::ec_hnsw::scan::tests::hnsw_turboquant_scan_codec_matches_direct_exact_modes --no-default-features --features pg18`
  - result: 1 passed; 0 failed
- `artifacts/cargo-test-hnsw-scan.log`
  - command: `cargo test --lib am::ec_hnsw::scan::tests --no-default-features --features pg18`
  - result: 75 passed; 0 failed
- `artifacts/git-diff-check.log`
  - command: `git diff --check`
  - result: passed

## Review Notes

The focused adapter test asserts `QuantCodec` metadata and bit-level score
parity against the direct quantizer methods for all four TurboQuant prepared
query variants. This makes the migration explicit without changing HNSW's
stored scan state yet.
