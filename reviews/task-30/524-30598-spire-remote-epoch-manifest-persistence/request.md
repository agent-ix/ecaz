# SPIRE Remote Epoch Manifest Persistence

## Summary

This checkpoint adds the first durable distributed epoch manifest persistence
surface for Phase 7.

Changes:

- Adds extension-owned table `ec_spire_remote_epoch_manifest` for persisted
  distributed manifest headers keyed by `(coordinator_index_oid, active_epoch)`.
- Adds extension-owned table `ec_spire_remote_epoch_manifest_entry` for included
  remote-node entries keyed by `(coordinator_index_oid, active_epoch, node_id)`.
- Adds `ec_spire_persist_remote_epoch_manifest(...)`, which persists only when
  `ec_spire_remote_epoch_manifest_summary(...)` reports
  `emit_distributed_epoch_manifest`.
- Fails closed for blocked or local-only manifest decisions instead of writing a
  partial manifest.
- Adds readback surfaces:
  `ec_spire_remote_epoch_manifest_catalog(...)` and
  `ec_spire_remote_epoch_manifest_entry_catalog(...)`.
- Updates the Phase 7 task note with the durable manifest writer/readback
  boundary.

## Files

- `sql/bootstrap.sql`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `68af0638`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_epoch_manifest_persist`
- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `git diff --check`

Result:

- PG18 `remote_epoch_manifest_persist` filter passed:
  - `pg_test_ec_spire_remote_epoch_manifest_persist_ready`
  - `pg_test_ec_spire_remote_epoch_manifest_persist_blocked`
- The ready test proves a distributed-ready manifest writes one header and one
  included remote-node entry, then reads both back through the catalog functions.
- The blocked test proves missing-descriptor/blocked manifest decisions fail
  closed before persistence.

## Notes

This is still not remote publication or libpq transport execution. It persists
the coordinator-side manifest contract once the existing readiness chain proves
all remote nodes can serve the epoch.
