---
topic: spire-update-delete-schema-drift
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30936
stage: phase-12.5
status: open
---

# Review Request: SPIRE UPDATE/DELETE Schema Drift

## Scope

Please review commit `ebbc9dba7578b42db4b24ae8663b6938f738fe90`
(`Extend schema drift guard to update delete`).

This slice closes the Phase 12.5 follow-up from packet `30935`:

- Generalizes the descriptor-bound schema-drift guard from INSERT wording to
  coordinator write wording.
- Runs the same pre-dispatch fingerprint comparison for remote UPDATE and
  DELETE payload paths.
- Keeps the guard before remote SQL construction, remote conninfo resolution,
  remote libpq transaction setup, and DELETE placement-directory removal.
- Adds `test_ec_spire_update_delete_schema_drift_guard_sql`, which alters only
  the coordinator table and proves:
  - remote UPDATE fails with schema drift and leaves the remote row unchanged;
  - remote DELETE fails with schema drift and leaves the remote row present;
  - no SPIRE prepared transaction is left behind;
  - DELETE does not remove the placement row before the guard fires.
- Updates ADR-069, diagnostics, and the Phase 12.5 tracker to describe the
  guard as covering INSERT, UPDATE, and DELETE remote write paths.

## Review Focus

- Confirm the UPDATE and DELETE guard placement is early enough to avoid remote
  mutation, prepared xacts, and placement-directory loss.
- Confirm reusing the descriptor-bound INSERT shape fingerprint is the right v1
  schema-shape contract for UPDATE and DELETE payload paths.
- Confirm the tracker no longer overstates an open UPDATE/DELETE schema-drift
  gap.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_update_delete_schema_drift_guard_sql`

Key result: `1 passed; 0 failed; 1687 filtered out`.
