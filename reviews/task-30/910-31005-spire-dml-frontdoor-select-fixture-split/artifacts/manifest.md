# Artifact Manifest: SPIRE DML Frontdoor Select Fixture Split

- Head SHA: `6ad29ba45a365d1c8161c3c5df036957f6a7db03`
- Packet/topic: `31005-spire-dml-frontdoor-select-fixture-split`
- Lane: Phase 12b.2 fixture-sink cleanup
- Fixture: DML frontdoor PK-select/custom-scan plan and replacement-decision SQL fixtures
- Storage format: not applicable; structural fixture move
- Rerank mode: not applicable
- Surface: existing textual `include!("dml_frontdoor.rs")`; no isolated one-index-per-table or shared-table measurement surface
- Measurement claim: none

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Timestamp: 2026-05-13 16:07:20 PDT
- Result: command exit code `0`
- Key line: `Script done on 2026-05-13 16:07:20-07:00 [COMMAND_EXIT_CODE="0"]`
- Note: emitted the repo's stable-rustfmt warnings for unstable `imports_granularity` and `group_imports`.

### `cargo-test-coordinator-dml-frontdoor-plan.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_coordinator_dml_frontdoor_plan_sql -- --nocapture`
- Timestamp: 2026-05-13 16:09:37 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_coordinator_dml_frontdoor_plan_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 39.46s`

### `cargo-test-dml-frontdoor-replacement-decision.log`

- Command: `cargo test --no-default-features --features pg18 test_ec_spire_dml_frontdoor_replacement_decision_sql -- --nocapture`
- Timestamp: 2026-05-13 16:11:32 PDT
- Result: command exit code `0`
- Key lines:
  - `test tests::pg_test_ec_spire_dml_frontdoor_replacement_decision_sql ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 30.50s`

### `location-check.log`

- Command: `rg -n 'fn test_ec_spire_dml_frontdoor_pk_select_customscan_local_sql|fn test_ec_spire_custom_scan_dml_plan_private_copyobject_sql|fn test_ec_spire_forward_coordinator_select_rejects_multirow_sql|fn test_ec_spire_coordinator_dml_frontdoor_plan_sql|fn test_ec_spire_dml_frontdoor_replacement_decision_sql' src/tests/dml_frontdoor.rs src/tests/mod.rs`
- Timestamp: 2026-05-13 16:13:55 PDT
- Result: command exit code `0`
- Key lines:
  - `src/tests/dml_frontdoor.rs`
  - `1365:    fn test_ec_spire_dml_frontdoor_pk_select_customscan_local_sql() {`
  - `1437:    fn test_ec_spire_custom_scan_dml_plan_private_copyobject_sql() {`
  - `1447:    fn test_ec_spire_forward_coordinator_select_rejects_multirow_sql() {`
  - `1496:    fn test_ec_spire_coordinator_dml_frontdoor_plan_sql() {`
  - `1536:    fn test_ec_spire_dml_frontdoor_replacement_decision_sql() {`

### `line-counts.log`

- Command: `wc -l src/tests/mod.rs src/tests/dml_frontdoor.rs src/lib.rs`
- Timestamp: 2026-05-13 16:13:55 PDT
- Result: command exit code `0`
- Key lines:
  - `40623 src/tests/mod.rs`
  - `1669 src/tests/dml_frontdoor.rs`
  - `17812 src/lib.rs`
