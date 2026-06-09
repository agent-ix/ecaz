# Task 91 Packet 008 Artifact Manifest

- head SHA: `ed5fb20e9706`
- task bucket: `reviews/task-91`
- packet path: `reviews/task-91/008-hnsw-turboquant-scan-quantcodec`
- timestamp: `2026-06-08T22:20:08-07:00`
- lane: HNSW TurboQuant scan scoring through `QuantCodec`
- fixture: focused Rust unit tests
- storage format: HNSW TurboQuant
- rerank mode: exact scan scoring; no heap rerank benchmark run
- table surface: no PostgreSQL benchmark tables were created

## Artifacts

### `artifacts/cargo-test-hnsw-turboquant-codec.log`

- command: `cargo test --lib am::ec_hnsw::scan::tests::hnsw_turboquant_scan_codec_matches_direct_exact_modes --no-default-features --features pg18`
- purpose: focused bit-level parity for the HNSW TurboQuant `QuantCodec`
  adapter across exact, full-LUT, tiled-LUT, and int8 prepared-query variants
- key result lines:
  - `test am::ec_hnsw::scan::tests::hnsw_turboquant_scan_codec_matches_direct_exact_modes ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2022 filtered out; finished in 0.04s`

### `artifacts/cargo-test-hnsw-scan.log`

- command: `cargo test --lib am::ec_hnsw::scan::tests --no-default-features --features pg18`
- purpose: broader HNSW scan unit regression after routing TurboQuant scan
  scoring through `QuantCodec`
- key result lines:
  - `test am::ec_hnsw::scan::tests::hnsw_turboquant_scan_codec_matches_direct_exact_modes ... ok`
  - `test am::ec_hnsw::scan::tests::miri_score_scan_element_result_via_raw_opaque_ptr_updates_stats_delta ... ok`
  - `test result: ok. 75 passed; 0 failed; 0 ignored; 0 measured; 1948 filtered out; finished in 0.07s`

### `artifacts/git-diff-check.log`

- command: `git diff --check`
- purpose: whitespace check for the code and packet diff
- key result lines:
  - `COMMAND_EXIT_CODE="0"`
