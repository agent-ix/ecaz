# SPIRE Secret Operator Entrypoints

## Scope

Task 30 SPIRE Phase 7 now makes the new conninfo-secret gates discoverable
through the compact operator entrypoint contract.

Code checkpoint: `93513495` (`Expose SPIRE secret gates in operator contract`)

## Changes

- Added `ec_spire_remote_search_libpq_secret_summary` to
  `ec_spire_remote_operator_entrypoint_contract()` as the search conninfo-secret
  gate.
- Added `ec_spire_remote_conninfo_secret_resolution_status` to the operator
  entrypoint contract as the single-secret probe.
- Extended the Phase 7 policy-contract PG18 fixture to assert both new
  entrypoints are reachable through `pg_proc` and expose the intended operator
  use / next action values.
- Updated the Phase 7 task note.

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- `git diff --check`

## Review Focus

- Whether the operator entrypoint contract should include the per-node
  `ec_spire_remote_search_libpq_secret_plan(...)` surface too, or keep the
  compact list at summary/probe level.
