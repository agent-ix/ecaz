# Review Request: SPIRE Remote Schema Fingerprint Guard

- coder: coder1
- topic: Phase 12a.6 remote-side schema fingerprint
- code commit: `76db6ccd1cb351a874719a5cc2ef885a17d03082`
- packet: `review/30989-spire-remote-schema-fingerprint`

## Summary

This addresses review packet `30982` feedback that the write-path schema drift
fingerprint was coordinator-only.

Changes:

- Adds `remote_insert_shape_fingerprint` to
  `ec_spire_remote_node_descriptor` for fresh installs and upgrade SQL, with
  `unset` fail-closed semantics.
- Adds `ec_spire_remote_index_shape_fingerprint(index_oid regclass)` as the
  remote-side wrapper over the same canonical heap-column tuple used by the
  coordinator fingerprint: `(attnum, name, typid, typmod, collation, notnull)`.
- Registers and refreshes descriptors with both coordinator and remote
  fingerprints when the conninfo secret is reachable.
- Performs a remote fingerprint echo-back before mutating remote INSERT,
  UPDATE, and DELETE SQL. Drift returns `schema_drift` and names the side that
  changed.
- Adds a PG18 fixture that registers a loopback remote, changes the remote
  heap column type without re-registering, and verifies the guard fires before
  remote SQL execution.
- Documents the v1 DDL ordering and remote echo-back contract in ADR-069,
  diagnostics, and the libpq runbook.

## Validation

See `artifacts/manifest.md` for packet-local logs.

- `cargo pgrx test pg18 test_ec_spire_remote_schema_fingerprint_pre_dispatch_sql`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_contract`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_registration_contract`
- `cargo fmt --check`
- `git diff --check`

All validation passed.

## Reviewer Focus

- Confirm the pre-dispatch order is correct: coordinator fingerprint check,
  remote echo-back, then mutating remote SQL.
- Confirm the `unset` behavior is acceptable for upgraded or temporarily
  unreachable descriptors: fail closed until refresh with a reachable remote.
- Confirm the docs describe the v1 DDL ordering and detection latency clearly.
