# Artifact Manifest: SPIRE Multistore Build Fixture Split

- Head SHA: `f39a2af68aad30ad76e2d6062b9fb60c8a8b1a75`
- Packet/topic: `31015-spire-build-multistore-fixture-split`
- Lane: Phase 12b.2 fixture-sink cleanup
- Fixture: auxiliary-store relcache, multistore, reindex, tqvector populated-build, and two-store scan fixtures
- Storage format: mixed fixture build coverage; no benchmark storage format
- Rerank mode: not applicable
- Surface: existing textual `include!("build.rs")`; no isolated one-index-per-table or shared-table measurement surface
- Measurement claim: none

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Timestamp: 2026-05-13 17:18:03 PDT
- Result: command exit code `0`
- Key line: `Script done on 2026-05-13 17:18:03-07:00 [COMMAND_EXIT_CODE="0"]`
- Note: emitted the repo's stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `cargo-test-multistore-large-fixture.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_multistore_large_fixture_routes_all_stores -- --nocapture`
- Timestamp: 2026-05-13 17:20:31 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_multistore_large_fixture_routes_all_stores ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 42.97s`

### `cargo-test-multistore-reindex-rejected.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_multistore_reindex_rejected -- --nocapture`
- Timestamp: 2026-05-13 17:22:34 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_multistore_reindex_rejected - should panic ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 33.27s`

### `cargo-test-relation-two-store-scan.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_relation_two_store_scan_roundtrip -- --nocapture`
- Timestamp: 2026-05-13 17:24:35 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_relation_two_store_scan_roundtrip ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 31.69s`

### `location-check.log`

- Command: `rg -n 'fn test_ec_spire_aux_store_relcache_disables_autovacuum|fn test_ec_spire_multistore_large_fixture_routes_all_stores|fn test_ec_spire_singlestore_reindex_succeeds|fn test_ec_spire_multistore_reindex_rejected|fn test_ec_spire_tqvector_populated_build_scans_with_heap_rerank|fn test_ec_spire_relation_two_store_scan_roundtrip' src/tests/build.rs src/tests/mod.rs`
- Timestamp: 2026-05-13 17:24:46 PDT
- Result: command exit code `0`
- Key lines:
  - `src/tests/build.rs`
  - `460:    fn test_ec_spire_aux_store_relcache_disables_autovacuum() {`
  - `538:    fn test_ec_spire_multistore_large_fixture_routes_all_stores() {`
  - `630:    fn test_ec_spire_singlestore_reindex_succeeds() {`
  - `668:    fn test_ec_spire_multistore_reindex_rejected() {`
  - `698:    fn test_ec_spire_tqvector_populated_build_scans_with_heap_rerank() {`
  - `730:    fn test_ec_spire_relation_two_store_scan_roundtrip() {`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/build.rs src/lib.rs`
- Timestamp: 2026-05-13 17:24:46 PDT
- Result: command exit code `0`
- Key lines:
  - `36486 src/tests/mod.rs`
  - `758 src/tests/build.rs`
  - `17812 src/lib.rs`
