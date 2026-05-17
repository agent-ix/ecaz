# Review Request: SPIRE DML Frontdoor Select Fixture Split

## Summary

Code commit: `6ad29ba45a365d1c8161c3c5df036957f6a7db03`

This checkpoint continues Phase 12b.2 fixture-sink cleanup by moving the earlier DML frontdoor PK-select/custom-scan plan fixtures and the replacement-decision SQL fixture from `src/tests/mod.rs` into `src/tests/dml_frontdoor.rs`.

The move preserves the existing textual `include!("dml_frontdoor.rs")` module shape, so fixture names and `pg_schema` scope remain unchanged.

## Scope

- Moved these fixtures into `src/tests/dml_frontdoor.rs`:
  - `test_ec_spire_dml_frontdoor_pk_select_customscan_local_sql`
  - `test_ec_spire_custom_scan_dml_plan_private_copyobject_sql`
  - `test_ec_spire_forward_coordinator_select_rejects_multirow_sql`
  - `test_ec_spire_coordinator_dml_frontdoor_plan_sql`
  - `test_ec_spire_dml_frontdoor_replacement_decision_sql`
- Updated `plan/tasks/task30-phase12b-spire-cleanup.md` to record packet `31005`.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_coordinator_dml_frontdoor_plan_sql -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_dml_frontdoor_replacement_decision_sql -- --nocapture`
- Location and line-count artifacts are packet-local under `artifacts/`.

The focused tests passed. They emitted the pre-existing unused-import warning in `src/am/mod.rs`.

## Remaining 12b.2 Work

`tests/dml_frontdoor.rs` remains open in the tracker because broader coordinator update/delete/select tuple-payload SQL fixtures still live in `src/tests/mod.rs`.
