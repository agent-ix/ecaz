# SPIRE Remote Descriptor Catalog

## Summary

This checkpoint adds the first durable coordinator-side remote node descriptor
catalog and wires it into the existing Phase 7 readiness surfaces.

Changes:

- Adds extension-owned table `ec_spire_remote_node_descriptor`, keyed by
  `(coordinator_index_oid, node_id)`.
- Stores only `conninfo_secret_name` as the connection reference; raw conninfo
  remains outside the catalog.
- Extends `ec_spire_remote_node_snapshot(...)` to consume active, draining,
  disabled, and failed descriptor rows from the catalog.
- Lets registered active/draining remote nodes advance from
  `requires_remote_node_descriptor` to the next pre-libpq blocker.
- Extends capability and publish-gate status so active descriptors with a valid
  served/retained epoch window and matching extension version report `ready`.
- Adds PG18 coverage proving an active catalog descriptor makes target
  readiness advance to `requires_libpq_transport`.
- Updates the Phase 7 task note with the durable catalog surface.

## Files

- `sql/bootstrap.sql`
- `src/am/ec_spire/mod.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/snapshots.rs`
- `src/am/ec_spire/root/types.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `3381ab75`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_node_descriptor`
- `git diff --check`

Result:

- PG18 `remote_node_descriptor` filter passed:
  - `pg_test_ec_spire_remote_node_descriptor_registration_contract`
  - `pg_test_ec_spire_remote_node_descriptor_contract`
  - `pg_test_ec_spire_remote_node_descriptor_readiness_missing`
  - `pg_test_ec_spire_remote_node_descriptor_catalog_active`
- `cargo fmt --check` was run after formatting touched files and restoring
  unrelated known rustfmt churn. It still reports only the pre-existing
  unrelated differences in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`

## Notes

This is descriptor persistence and readiness integration, not libpq execution.
Registered remote targets now reach the `requires_libpq_transport` gate, which
is the next coordinator transport blocker.
