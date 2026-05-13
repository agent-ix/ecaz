# Artifacts: 31001 SPIRE DML Frontdoor Fixture Split

- Code commit: `438c0fbfdc3c9f5c03c2242def4187d250e62556`
- Packet/topic: `31001-spire-dml-frontdoor-fixture-split`
- Lane / fixture / storage format / rerank mode: Phase 12b.2 PG18 fixture layout split; focused DML hook-status fixture has no storage/rerank behavior
- Isolated one-index-per-table or shared-table surfaces: not applicable to the layout change; focused pg_test reads hook status only

## Validation

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/31001-spire-dml-frontdoor-fixture-split/artifacts/cargo-fmt-check.log`
- Timestamp: 2026-05-13 15:44 PDT
- Key result lines:
  - command exited `0`
  - rustfmt emitted the repository's existing stable-channel warnings for unstable `imports_granularity` / `group_imports` options

### `cargo-test-dml-frontdoor-fixture.log`

- Command: `script -q -e -c "cargo test --no-default-features --features pg18 test_ec_spire_dml_frontdoor_hook_status_installed_pass_through -- --nocapture" review/31001-spire-dml-frontdoor-fixture-split/artifacts/cargo-test-dml-frontdoor-fixture.log`
- Timestamp: 2026-05-13 15:44 PDT
- Key result lines:
  - `test tests::pg_test_ec_spire_dml_frontdoor_hook_status_installed_pass_through ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out`
  - pre-existing unused-import warning in `src/am/mod.rs`

### `fixture-location-check.log`

- Command: `script -q -e -c "rg -n 'include!\\(\\\"dml_frontdoor.rs\\\"\\)|fn test_ec_spire_dml_frontdoor_hook_status_installed_pass_through|fn test_ec_spire_dml_customscan_remote_delete_sql|fn test_ec_spire_reaper_resolves_lost_prepare_ack_fixture|fn test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send' src/tests && wc -l src/tests/mod.rs src/tests/dml_frontdoor.rs src/tests/insert.rs src/tests/remote_search.rs src/tests/custom_scan.rs src/lib.rs" review/31001-spire-dml-frontdoor-fixture-split/artifacts/fixture-location-check.log`
- Timestamp: 2026-05-13 15:47 PDT
- Key result lines:
  - `src/tests/mod.rs:8625:    include!("dml_frontdoor.rs");`
  - `src/tests/dml_frontdoor.rs:2:    fn test_ec_spire_dml_frontdoor_hook_status_installed_pass_through() {`
  - `src/tests/dml_frontdoor.rs:864:    fn test_ec_spire_dml_customscan_remote_delete_sql() {`
  - `src/tests/mod.rs:8629:    fn test_ec_spire_reaper_resolves_lost_prepare_ack_fixture() {`
  - `src/tests/mod.rs:8945:    fn test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send() {`
  - `42537 src/tests/mod.rs`
  - `999 src/tests/dml_frontdoor.rs`
