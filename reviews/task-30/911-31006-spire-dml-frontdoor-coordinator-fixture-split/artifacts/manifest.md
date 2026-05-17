# Artifact Manifest: SPIRE DML Frontdoor Coordinator Fixture Split

- Head SHA: `260a8b40d65fdaf2c3d836c523739bc2e030b95f`
- Packet/topic: `31006-spire-dml-frontdoor-coordinator-fixture-split`
- Lane: Phase 12b.2 fixture-sink cleanup
- Fixture: DML coordinator update/delete/select tuple-payload and update/delete schema-drift fixtures
- Storage format: not applicable; structural fixture move
- Rerank mode: not applicable
- Surface: existing textual `include!("dml_frontdoor.rs")`; no isolated one-index-per-table or shared-table measurement surface
- Measurement claim: none

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Timestamp: 2026-05-13 16:14:50 PDT
- Result: command exit code `0`
- Key line: `Script done on 2026-05-13 16:14:50-07:00 [COMMAND_EXIT_CODE="0"]`
- Note: emitted the repo's stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `cargo-test-forward-coordinator-update-tuple-payload.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_forward_coordinator_update_tuple_payload_sql -- --nocapture`
- Timestamp: 2026-05-13 16:17:15 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_forward_coordinator_update_tuple_payload_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 40.31s`

### `cargo-test-prepare-coordinator-delete-tuple-payload.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_prepare_coordinator_delete_tuple_payload_sql -- --nocapture`
- Timestamp: 2026-05-13 16:19:13 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_prepare_coordinator_delete_tuple_payload_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 30.24s`

### `cargo-test-forward-coordinator-select-tuple-payload.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_forward_coordinator_select_tuple_payload_sql -- --nocapture`
- Timestamp: 2026-05-13 16:21:08 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_forward_coordinator_select_tuple_payload_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 30.28s`

### `location-check.log`

- Command: `rg -n 'fn test_ec_spire_update_delete_schema_drift_guard_sql|fn test_ec_spire_forward_coordinator_update_tuple_payload_sql|fn test_ec_spire_forward_coordinator_update_local_sql|fn test_ec_spire_forward_coordinator_update_missing_placement_sql|fn test_ec_spire_update_rejects_embedding_column_sql|fn test_ec_spire_prepare_coordinator_delete_tuple_payload_sql|fn test_ec_spire_prepare_coordinator_delete_local_sql|fn test_ec_spire_prepare_coordinator_delete_idempotent_sql|fn test_ec_spire_forward_coordinator_select_tuple_payload_sql|fn test_ec_spire_forward_coordinator_select_local_sql' src/tests/dml_frontdoor.rs src/tests/mod.rs`
- Timestamp: 2026-05-13 16:21:17 PDT
- Result: command exit code `0`
- Key lines:
  - `src/tests/dml_frontdoor.rs`
  - `1672:    fn test_ec_spire_update_delete_schema_drift_guard_sql() {`
  - `1830:    fn test_ec_spire_forward_coordinator_update_tuple_payload_sql() {`
  - `2100:    fn test_ec_spire_prepare_coordinator_delete_tuple_payload_sql() {`
  - `2401:    fn test_ec_spire_forward_coordinator_select_tuple_payload_sql() {`
  - `2503:    fn test_ec_spire_forward_coordinator_select_local_sql() {`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/dml_frontdoor.rs src/lib.rs`
- Timestamp: 2026-05-13 16:21:17 PDT
- Result: command exit code `0`
- Key lines:
  - `39730 src/tests/mod.rs`
  - `2562 src/tests/dml_frontdoor.rs`
  - `17812 src/lib.rs`
