# SPIRE Manifest Result Source Contract

## Scope

This packet publishes the result-source contract for the manifest publication
result summary.

Code checkpoint: `a721ffd7` (`Publish SPIRE manifest result source contract`)

## Changes

- Adds `ec_spire_remote_epoch_manifest_publication_result_contract()`.
- Documents the four result-source states:
  - `not_required`
  - `pending_libpq_executor`
  - `remote_manifest_validation_result`
  - `blocked`
- Extends the Phase 7 policy-contract PG18 test to assert the contract row
  count and the pending-libpq validator.
- Updates the Phase 7 task note with the result-source contract surface.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- `git diff --check`

## Notes

This is a contract-only surface for the already-added result summary; it does
not change publication planning or executor readiness behavior.
