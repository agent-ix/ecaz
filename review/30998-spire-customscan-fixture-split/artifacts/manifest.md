# Artifacts: 30998 SPIRE CustomScan Fixture Split

- Code commit: `a212e3adf16ad7f7c5ac19fd7edd9c024edfb12d`
- Packet/topic: `30998-spire-customscan-fixture-split`
- Lane / fixture / storage format / rerank mode: Phase 12b.2 PG18 fixture layout split; CustomScan tuple payload fixture uses rabitq and existing rerank defaults
- Isolated one-index-per-table or shared-table surfaces: not applicable to the layout change; the focused pg_test creates its existing coordinator and loopback remote fixture tables

## Validation

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30998-spire-customscan-fixture-split/artifacts/cargo-fmt-check.log`
- Timestamp: 2026-05-13 15:26 PDT
- Key result lines:
  - command exited `0`
  - rustfmt emitted the repository's existing stable-channel warnings for unstable `imports_granularity` / `group_imports` options

### `cargo-test-customscan-fixture.log`

- Command: `script -q -e -c "cargo test --no-default-features --features pg18 test_ec_spire_customscan_returns_loopback_remote_tuple_payload -- --nocapture" review/30998-spire-customscan-fixture-split/artifacts/cargo-test-customscan-fixture.log`
- Timestamp: 2026-05-13 15:27 PDT
- Key result lines:
  - `test tests::pg_test_ec_spire_customscan_returns_loopback_remote_tuple_payload ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out`
  - pre-existing unused-import warning in `src/am/mod.rs`

### `fixture-location-check.log`

- Command: `script -q -e -c "rg -n 'include!\\(\\\"custom_scan.rs\\\"\\)|fn test_ec_spire_customscan_|fn test_ec_spire_custom_scan_index_eligibility_remote|fn test_ec_spire_remote_search_local_heap_resolution_plan' src/tests && wc -l src/tests/mod.rs src/tests/custom_scan.rs src/lib.rs" review/30998-spire-customscan-fixture-split/artifacts/fixture-location-check.log`
- Timestamp: 2026-05-13 15:29 PDT
- Key result lines:
  - `src/tests/mod.rs:14861:    include!("custom_scan.rs");`
  - `src/tests/custom_scan.rs:46:    fn test_ec_spire_customscan_returns_loopback_remote_tuple_payload() {`
  - `src/tests/mod.rs:14865:    fn test_ec_spire_remote_search_local_heap_resolution_plan() {`
  - `47932 src/tests/mod.rs`
  - `796 src/tests/custom_scan.rs`
