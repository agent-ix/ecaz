---
topic: spire-multirow-insert-trigger-fixture
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30928
stage: phase-12.4
status: open
---

# Review Request: SPIRE Multi-Row INSERT Trigger Fixture

## Scope

Please review commit `f72869db` (`Add SPIRE multi-row insert trigger fixture`).

This closes the Phase 12.4 tracker row for proving multi-row coordinator
INSERT trigger dispatch across rows and local-commit remote prepared
resolution.

## What Changed

- Added `test_ec_spire_trigger_multirow_commits_prepares_sql`.
- The fixture creates a two-leaf coordinator index and rewrites the two leaf
  placements to different remote node IDs, so the v1 prepared GID scope
  `(index_oid, node_id, served_epoch, top_xid)` remains unique per row in the
  same transaction.
- The fixture inserts two coordinator rows in one explicit transaction through
  `ec_spire_enable_coordinator_insert(...)`, commits, then verifies:
  - two placement rows were staged with the expected PK bytes, node IDs,
    centroid IDs, served epoch, and source identities;
  - duplicate probes on the remote table affect zero rows, proving both remote
    rows committed;
  - `pg_prepared_xacts` has no remaining SPIRE prepared transactions.
- Added a test-only `tests.ec_spire_test_set_env_var(...)` helper so the
  external loopback backend executing the trigger can resolve the conninfo
  secret within its own process environment.
- Marked the Phase 12.4 multi-row INSERT trigger fixture row complete.

## Evidence

See `artifacts/manifest.md`.

Validation run against
`f72869db9c1b855724263fa187a864e95da1baeb`:

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_trigger_multirow_commits_prepares_sql`

## Review Focus

- Confirm the fixture covers per-row trigger dispatch and commit-time prepared
  transaction resolution for the current v1 GID scope.
- Confirm the two-node setup is the right scope for this tracker row, leaving
  same-node multi-row batching to the wide-fanout/async dispatch work.
- Confirm the test-only env helper is acceptable for external loopback backend
  secret resolution.
