# SPIRE Remote Epoch Manifest Plan

## Summary

This checkpoint adds the SQL-visible distributed epoch manifest planning surface
for Phase 7 without writing or publishing a remote manifest.

Changes:

- Adds `ec_spire_remote_epoch_manifest_plan(...)`.
- Adds `ec_spire_remote_epoch_manifest_summary(...)`.
- Composes existing publish plan/gate state into manifest-entry actions:
  `include_remote_node` or `block_manifest`.
- Reports the final manifest decision for local-only, blocked distributed, and
  distributed-ready publish states.
- Keeps this as pre-publish contract work; no remote manifest persistence or
  libpq I/O is introduced.
- Updates the Phase 7 task note with the manifest planning surface.

## Files

- `src/am/ec_spire/root/snapshots.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `442325fb`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_node_cap_summary`
- `cargo pgrx test pg18 remote_epoch_publish_plan_missing`
- `cargo pgrx test pg18 remote_node_descriptor_catalog_active`
- `git diff --check`

Result:

- PG18 `remote_node_cap_summary` filter passed:
  - `pg_test_ec_spire_remote_node_cap_summary_local`
  - `pg_test_ec_spire_remote_node_cap_summary_missing`
- PG18 `remote_epoch_publish_plan_missing` filter passed:
  - `pg_test_ec_spire_remote_epoch_publish_plan_missing`
- PG18 `remote_node_descriptor_catalog_active` filter passed:
  - `pg_test_ec_spire_remote_node_descriptor_catalog_active`

## Notes

This is a manifest contract surface. It proves the coordinator can expose the
planned manifest entries and final manifest decision, but it does not persist a
distributed manifest or publish it to remote nodes.
