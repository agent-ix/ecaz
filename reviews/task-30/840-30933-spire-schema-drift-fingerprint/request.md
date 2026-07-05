---
topic: spire-schema-drift-fingerprint
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30933
stage: phase-12.5
status: open
---

# Review Request: SPIRE Schema Drift Fingerprint

## Scope

Please review commit `369c50d1c57641f6f7b8a9a8bd4656623d3ffdbd`
(`Add SPIRE insert schema drift guard`).

This slice closes the remaining Phase 12.5 schema-drift rows:

- Adds `coordinator_insert_shape_fingerprint` to
  `ec_spire_remote_node_descriptor`, bootstrap SQL, and upgrade SQL.
- Adds SQL helpers for hashing the coordinator heap column shape from a table
  or SPIRE index.
- Binds the fingerprint during
  `ec_spire_register_remote_node_descriptor`.
- Validates the descriptor-bound fingerprint before coordinator-routed INSERT
  opens remote libpq work, failing closed with a schema-drift message and the
  pause/apply/refresh/retry remediation hint.
- Extends descriptor contract rows so the new field is operator-visible.
- Adds `test_ec_spire_schema_drift_fails_before_dispatch_sql`, which alters
  only the coordinator table, attempts INSERT, and verifies no remote row and
  no SPIRE prepared transaction are left behind.
- Updates the Phase 12.5 tracker rows for fingerprinting, descriptor binding,
  pre-dispatch comparison, and the coordinator-only DDL fixture.

This does not implement automatic DDL propagation. The operator contract is
still the v1 pause writes, apply matching DDL on coordinator and remotes,
refresh descriptors, then resume writes sequence from packet `30931`.

## Review Focus

- Confirm the fingerprint inputs are the right v1 "column shape" boundary:
  physical column order, name, type OID, typmod, collation, and NOT NULL flag.
- Confirm the guard runs early enough that coordinator-only DDL cannot open a
  remote transaction or leave an orphaned prepared xact.
- Confirm binding the fingerprint through descriptor registration is the right
  refresh point for the documented DDL workflow.
- Confirm the migration default/backfill behavior is acceptable for upgraded
  descriptors.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_schema_drift_fails_before_dispatch_sql`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_contract`

Key results:

- schema-drift fixture: `1 passed; 0 failed; 1686 filtered out`
- descriptor contract fixture: `1 passed; 0 failed; 1686 filtered out`
