# SPIRE Remote Descriptor Catalog

## Summary

This checkpoint adds the first durable coordinator-side remote node descriptor
catalog and wires it into the existing Phase 7 readiness surfaces.

Changes:

- Adds extension-owned table `ec_spire_remote_node_descriptor`, keyed by
  `(coordinator_index_oid, node_id)`.
- Adds `ec_spire_register_remote_node_descriptor(...)`, a validated upsert
  surface for that catalog.
- Validates the coordinator OID as an `ec_spire` index before mutating the
  descriptor catalog.
- Stores only `conninfo_secret_name` as the connection reference; raw conninfo
  remains outside the catalog.
- Extends `ec_spire_remote_node_snapshot(...)` to consume active, draining,
  disabled, and failed descriptor rows from the catalog.
- Lets registered active/draining remote nodes advance from
  `requires_remote_node_descriptor` to the next pre-libpq blocker.
- Extends capability and publish-gate status so active descriptors with a valid
  served/retained epoch window and matching extension version report `ready`.
- Adds PG18 coverage proving an active registered descriptor makes target
  readiness advance to `requires_libpq_transport`.
- Extends that coverage through execution-plan and libpq-request envelope
  summaries so registered descriptors stay blocked on transport, not descriptor
  registration.
- Follow-up for reviewer feedback: descriptor upserts now fail closed unless
  `descriptor_generation` advances the existing catalog row.
- Follow-up for reviewer feedback: failed/disabled descriptors are normalized
  to the read-blocked descriptor status before request/execution/libpq summary
  rollups consume them.
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

Head SHA: `9866d033`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_node_descriptor`
- `cargo pgrx test pg18 remote_node_desc_failed_blocks_libpq_dispatch`
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
  - Confirms a failed descriptor remains visible as descriptor state
    `failed`, but read readiness, connection planning, and dispatch planning
    all block with `requires_remote_node_descriptor`.
- `cargo fmt` was run, then the known unrelated rustfmt churn was restored in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `git diff --check` passed.

## Notes

This is descriptor persistence and readiness integration, not libpq execution.
Registered remote targets now reach the `requires_libpq_transport` gate, which
is the next coordinator transport blocker.
