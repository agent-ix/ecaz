# SPIRE Remote Catalog Index Cleanup

## Scope

Task 30 SPIRE Phase 7 now has an exact coordinator-OID cleanup function for
remote catalog rows. This is the narrow target a future DROP INDEX lifecycle
hook can call without sweeping unrelated orphaned rows.

Code checkpoint: `609c2ff6` (`Add SPIRE remote catalog index cleanup`)

## Changes

- Added `ec_spire_remote_catalog_index_cleanup(index_oid)`.
- The function removes remote manifest headers for the exact coordinator OID,
  relies on manifest-entry FK cascade, then removes remote descriptors for that
  same OID.
- The returned counts include descriptor rows, manifest rows, and manifest
  entry rows counted before deletion.
- Updated `ec_spire_remote_catalog_lifecycle_contract()` so the `drop_index`
  lifecycle row names both exact-index cleanup and orphan-sweep cleanup.
- Added PG18 coverage with a synthetic coordinator OID and materialized cleanup
  result to avoid invoking the destructive function more than once.
- Updated the Phase 7 task note.

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_catalog_index_cleanup`
- `cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts`
- `git diff --check`

## Review Focus

- Whether exact cleanup should validate that the OID no longer resolves to a
  live `ec_spire` index, or whether it should remain a low-level hook target
  that trusts the caller.
- Whether the eventual DROP INDEX automation should call this exact helper, then
  leave broad orphan cleanup as an operator sweep.
