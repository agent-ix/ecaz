# SPIRE Contract Drift Invariants

## Summary

This checkpoint adds PG18 regression coverage for the contract-drift feedback
left on the descriptor-state registry and libpq executor-readiness packets.

Changes:

- Adds a descriptor-state invariant in
  `test_ec_spire_remote_node_descriptor_state_contract`.
- Verifies the SQL CHECK constraint on `ec_spire_remote_node_descriptor`
  includes every catalog-backed state exposed by
  `ec_spire_remote_node_descriptor_state_contract()`.
- Verifies the synthetic `missing` state remains outside the descriptor table
  CHECK constraint.
- Adds a search executor invariant in
  `test_ec_spire_remote_node_descriptor_catalog_active`.
- Compares ready search executor-readiness actions against
  `ec_spire_remote_search_libpq_executor_step_contract()`.
- Adds a manifest executor invariant in
  `test_ec_spire_remote_epoch_manifest_persist_ready`.
- Compares ready manifest executor-readiness actions against
  `ec_spire_remote_epoch_manifest_libpq_executor_step_contract()`.

## Files

- `src/lib.rs`

## Validation

Head SHA: `89652de6`

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_state_contract`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_catalog_active`
- `cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_persist_ready`
- `git diff --check`

Result:

- PG18 descriptor-state contract filter passed:
  - `pg_test_ec_spire_remote_node_descriptor_state_contract`
- PG18 active descriptor catalog filter passed:
  - `pg_test_ec_spire_remote_node_descriptor_catalog_active`
- PG18 remote epoch manifest persistence filter passed:
  - `pg_test_ec_spire_remote_epoch_manifest_persist_ready`

## Notes

This is a test-only feedback follow-up. It does not add the libpq executor
itself; it pins the pre-I/O contracts so future executor work fails loudly if
contract/action strings drift again.
