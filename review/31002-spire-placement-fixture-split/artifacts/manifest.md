# Artifacts: 31002 SPIRE Placement Fixture Split

- Code commit: `7a66582830745d69c0d18a5f317239306934a4f9`
- Packet/topic: `31002-spire-placement-fixture-split`
- Lane / fixture / storage format / rerank mode: Phase 12b.2 PG18 fixture layout split; focused placement catalog fixture has no storage/rerank behavior
- Isolated one-index-per-table or shared-table surfaces: not applicable to the layout change; focused pg_test reads catalog/index presence only

## Validation

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/31002-spire-placement-fixture-split/artifacts/cargo-fmt-check.log`
- Timestamp: 2026-05-13 15:49 PDT
- Key result lines:
  - command exited `0`
  - rustfmt emitted the repository's existing stable-channel warnings for unstable `imports_granularity` / `group_imports` options

### `cargo-test-placement-fixture.log`

- Command: `script -q -e -c "cargo test --no-default-features --features pg18 test_ec_spire_placement_directory_catalog_sql -- --nocapture" review/31002-spire-placement-fixture-split/artifacts/cargo-test-placement-fixture.log`
- Timestamp: 2026-05-13 15:49 PDT
- Key result lines:
  - `test tests::pg_test_ec_spire_placement_directory_catalog_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out`
  - pre-existing unused-import warning in `src/am/mod.rs`

### `fixture-location-check.log`

- Command: `script -q -e -c "rg -n 'include!\\(\\\"placement.rs\\\"\\)|fn test_ec_spire_placement_directory_catalog_sql|fn test_ec_spire_selected_pid_placement_snapshot_sql|fn test_ec_spire_scan_placement_snapshot_sql|fn test_pg18_ec_spire_placement_write_contention_distinct_pk_dml' src/tests && wc -l src/tests/mod.rs src/tests/placement.rs src/tests/dml_frontdoor.rs src/tests/insert.rs src/tests/remote_search.rs src/tests/custom_scan.rs src/lib.rs" review/31002-spire-placement-fixture-split/artifacts/fixture-location-check.log`
- Timestamp: 2026-05-13 15:52 PDT
- Key result lines:
  - `src/tests/mod.rs:2764:    include!("placement.rs");`
  - `src/tests/placement.rs:2:    fn test_ec_spire_placement_directory_catalog_sql() {`
  - `src/tests/placement.rs:343:    fn test_ec_spire_selected_pid_placement_snapshot_sql() {`
  - `src/tests/mod.rs:2767:    fn test_ec_spire_scan_placement_snapshot_sql() {`
  - `src/tests/mod.rs:18730:    fn test_pg18_ec_spire_placement_write_contention_distinct_pk_dml() {`
  - `42134 src/tests/mod.rs`
  - `405 src/tests/placement.rs`
