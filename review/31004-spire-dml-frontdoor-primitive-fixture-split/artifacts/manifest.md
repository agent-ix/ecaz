# Artifacts: 31004 SPIRE DML Frontdoor Primitive Fixture Split

- Code commit: `04bf4917ce296eb675b5e0a126e6e61029d542ae`
- Packet/topic: `31004-spire-dml-frontdoor-primitive-fixture-split`
- Lane / fixture / storage format / rerank mode: Phase 12b.2 PG18 fixture layout split; focused DML primitive-plan fixture uses existing ec_spire defaults
- Isolated one-index-per-table or shared-table surfaces: not applicable to the layout change; focused pg_test creates its existing DML primitive-plan fixtures

## Validation

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/31004-spire-dml-frontdoor-primitive-fixture-split/artifacts/cargo-fmt-check.log`
- Timestamp: 2026-05-13 16:00 PDT
- Key result lines:
  - command exited `0`
  - rustfmt emitted the repository's existing stable-channel warnings for unstable `imports_granularity` / `group_imports` options

### `cargo-test-dml-frontdoor-primitive-fixture.log`

- Command: `script -q -e -c "cargo test --no-default-features --features pg18 test_ec_spire_dml_frontdoor_primitive_plan_sql -- --nocapture" review/31004-spire-dml-frontdoor-primitive-fixture-split/artifacts/cargo-test-dml-frontdoor-primitive-fixture.log`
- Timestamp: 2026-05-13 16:00 PDT
- Key result lines:
  - `test tests::pg_test_ec_spire_dml_frontdoor_primitive_plan_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out`
  - pre-existing unused-import warning in `src/am/mod.rs`

### `fixture-location-check.log`

- Command: `script -q -e -c "rg -n 'include!\\(\\\"dml_frontdoor.rs\\\"\\)|fn test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send|fn test_ec_spire_dml_frontdoor_primitive_plan_sql|fn test_ec_spire_dml_frontdoor_replacement_decision_sql' src/tests && wc -l src/tests/mod.rs src/tests/dml_frontdoor.rs src/tests/scan.rs src/tests/placement.rs src/tests/insert.rs src/tests/remote_search.rs src/tests/custom_scan.rs src/lib.rs" review/31004-spire-dml-frontdoor-primitive-fixture-split/artifacts/fixture-location-check.log`
- Timestamp: 2026-05-13 16:02 PDT
- Key result lines:
  - `src/tests/mod.rs:7384:    include!("dml_frontdoor.rs");`
  - `src/tests/dml_frontdoor.rs:1001:    fn test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send() {`
  - `src/tests/dml_frontdoor.rs:1318:    fn test_ec_spire_dml_frontdoor_primitive_plan_sql() {`
  - `src/tests/mod.rs:7704:    fn test_ec_spire_dml_frontdoor_replacement_decision_sql() {`
  - `40931 src/tests/mod.rs`
  - `1364 src/tests/dml_frontdoor.rs`
