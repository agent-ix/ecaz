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
- Follow-up for reviewer feedback: publish readiness now derives blocked state
  from the per-node publish plan, so stale served-epoch or retention-window
  gaps block the manifest through `remote_epoch_window` even when the remote
  descriptor exists.
- Cross-cutting descriptor feedback is also closed at this head: descriptor
  registration requires advancing generation, and failed/disabled descriptors
  cannot resolve as libpq pipeline-ready.
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

Head SHA: `9866d033`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_node_descriptor`
- `cargo pgrx test pg18 remote_node_desc_failed_blocks_libpq_dispatch`
- `cargo pgrx test pg18 remote_epoch_publish_manifest_stale_descriptor`
- `git diff --check`

Result:

- PG18 `remote_node_descriptor` filter passed:
  - `pg_test_ec_spire_remote_node_descriptor_registration_contract`
  - `pg_test_ec_spire_remote_node_descriptor_contract`
  - `pg_test_ec_spire_remote_node_descriptor_readiness_missing`
  - `pg_test_ec_spire_remote_node_descriptor_catalog_active`
  - `pg_test_ec_spire_remote_node_descriptor_stale_generation_rejected`
- PG18 `remote_node_desc_failed_blocks_libpq_dispatch` filter passed:
  - `pg_test_ec_spire_remote_node_desc_failed_blocks_libpq_dispatch`
- PG18 `remote_epoch_publish_manifest_stale_descriptor` filter passed:
  - `pg_test_ec_spire_remote_epoch_publish_manifest_stale_descriptor`

## Notes

This is a manifest contract surface. It proves the coordinator can expose the
planned manifest entries and final manifest decision, but it does not persist a
distributed manifest or publish it to remote nodes.
