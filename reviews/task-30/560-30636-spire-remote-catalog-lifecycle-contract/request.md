# SPIRE Remote Catalog Lifecycle Contract

## Scope

Task 30 SPIRE Phase 7 now has a queryable contract for remote catalog lifecycle
behavior across restore, DROP INDEX, physical backup, and extension upgrade.

Code checkpoint: `2c695235` (`Add SPIRE remote catalog lifecycle contract`)

## Changes

- Added `ec_spire_remote_catalog_lifecycle_contract()`.
- The contract records four lifecycle events:
  - `pg_dump_restore`
  - `drop_index`
  - `basebackup_wal_replay`
  - `extension_upgrade_0_1_0_to_0_1_1`
- The rows spell out OID stability expectations, catalog risk, operator action,
  cleanup surface, migration surface, status, and recommendation.
- Extended `test_ec_spire_remote_phase7_policy_contracts` to assert the four
  rows and key statuses/surfaces.

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

- Whether the logical restore row is explicit enough that operators know remote
  descriptors must be re-registered after OIDs are reassigned.
- Whether `drop_index` should remain a manual orphan-cleanup contract for this
  phase, or whether automatic event-trigger cleanup must land before libpq
  executor work.
- Whether the extension-upgrade row is the right checklist for the next
  migration-script slice.
