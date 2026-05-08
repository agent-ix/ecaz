# SPIRE Remote Upgrade Catalog Tables

## Scope

Task 30 SPIRE Phase 7 now backfills the remote catalog tables in the
`0.1.0` to `0.1.1` extension upgrade script.

Code checkpoint: `9f9fed14` (`Add SPIRE remote tables to upgrade script`)

## Changes

- Added these tables to `ecaz--0.1.0--0.1.1.sql`:
  - `ec_spire_remote_node_descriptor`
  - `ec_spire_remote_epoch_manifest`
  - `ec_spire_remote_epoch_manifest_entry`
- Kept the definitions aligned with `sql/bootstrap.sql`, including descriptor
  state checks and the manifest-entry foreign key with `ON DELETE CASCADE`.
- Updated `ec_spire_remote_catalog_lifecycle_contract()` so the extension
  upgrade row reports `supported_after_upgrade_script`.
- Extended the PG18 policy-contract test to assert that upgrade status.

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

- Whether the upgrade-script table definitions should stay manually duplicated
  from `sql/bootstrap.sql`, or whether this repo needs a generation/check
  mechanism for bootstrap/upgrade parity.
- Whether remote catalog functions also need explicit upgrade-script SQL in a
  later slice, beyond table backfill.
