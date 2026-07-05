# Review Request: SPIRE DML Frontdoor Coordinator Fixture Split

## Summary

Code commit: `260a8b40d65fdaf2c3d836c523739bc2e030b95f`

This checkpoint closes the Phase 12b.2 `tests/dml_frontdoor.rs` row by moving the remaining coordinator update/delete/select tuple-payload fixtures and the update/delete schema-drift guard from `src/tests/mod.rs` into `src/tests/dml_frontdoor.rs`.

The move keeps the existing textual `include!("dml_frontdoor.rs")` boundary, so fixture names and `pg_schema` scope remain unchanged.

## Scope

- Moved 10 coordinator DML fixtures into `src/tests/dml_frontdoor.rs`.
- Verified `src/tests/mod.rs` now only retains `include!("dml_frontdoor.rs")` for DML-frontdoor/coordinator fixture names.
- Marked `tests/dml_frontdoor.rs` complete in `plan/tasks/task30-phase12b-spire-cleanup.md`.

## Validation

- `cargo fmt --check`
- `cargo test --no-default-features --features pg18 test_ec_spire_forward_coordinator_update_tuple_payload_sql -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_prepare_coordinator_delete_tuple_payload_sql -- --nocapture`
- `cargo test --no-default-features --features pg18 test_ec_spire_forward_coordinator_select_tuple_payload_sql -- --nocapture`
- Packet-local location and line-count checks under `artifacts/`.

All focused tests passed. They emitted the pre-existing unused-import warning in `src/am/mod.rs`.
