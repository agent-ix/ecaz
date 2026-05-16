# Artifacts: 31000 SPIRE Insert Fixture Split

- Code commit: `6f682516c7c2fe33e020763800a5a11b26a80065`
- Packet/topic: `31000-spire-insert-fixture-split`
- Lane / fixture / storage format / rerank mode: Phase 12b.2 PG18 fixture layout split; focused insert-plan fixture uses existing ec_spire defaults
- Isolated one-index-per-table or shared-table surfaces: not applicable to the layout change; focused pg_test creates its existing local table/index fixture

## Validation

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/31000-spire-insert-fixture-split/artifacts/cargo-fmt-check.log`
- Timestamp: 2026-05-13 15:38 PDT
- Key result lines:
  - command exited `0`
  - rustfmt emitted the repository's existing stable-channel warnings for unstable `imports_granularity` / `group_imports` options

### `cargo-test-insert-fixture.log`

- Command: `script -q -e -c "cargo test --no-default-features --features pg18 test_ec_spire_plan_coordinator_insert_sql -- --nocapture" review/31000-spire-insert-fixture-split/artifacts/cargo-test-insert-fixture.log`
- Timestamp: 2026-05-13 15:39 PDT
- Key result lines:
  - `test tests::pg_test_ec_spire_plan_coordinator_insert_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out`
  - pre-existing unused-import warning in `src/am/mod.rs`

### `fixture-location-check.log`

- Command: `script -q -e -c "rg -n 'include!\\(\\\"insert.rs\\\"\\)|fn test_ec_spire_plan_coordinator_insert_sql|fn test_ec_spire_insert_trigger_payload_type_roundtrip_sql|fn test_ec_spire_schema_drift_fails_before_dispatch_sql|fn test_ec_spire_insert_after_build_delta_epoch' src/tests && wc -l src/tests/mod.rs src/tests/insert.rs src/tests/remote_search.rs src/tests/custom_scan.rs src/lib.rs" review/31000-spire-insert-fixture-split/artifacts/fixture-location-check.log`
- Timestamp: 2026-05-13 15:41 PDT
- Key result lines:
  - `src/tests/mod.rs:4009:    include!("insert.rs");`
  - `src/tests/insert.rs:2:    fn test_ec_spire_plan_coordinator_insert_sql() {`
  - `src/tests/insert.rs:1507:    fn test_ec_spire_insert_trigger_payload_type_roundtrip_sql() {`
  - `src/tests/mod.rs:4012:    fn test_ec_spire_schema_drift_fails_before_dispatch_sql() {`
  - `src/tests/mod.rs:19885:    fn test_ec_spire_insert_after_build_delta_epoch() {`
  - `43534 src/tests/mod.rs`
  - `1766 src/tests/insert.rs`
