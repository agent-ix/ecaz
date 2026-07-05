---
topic: spire-delete-not-found-idempotence
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30920
stage: phase-12.4
status: open
---

# Review Request: SPIRE DELETE Not-Found Idempotence

## Scope

Please review commit `00616750` (`Make SPIRE DELETE not-found idempotent`).

This closes the Phase 12.4 decision for the concurrent DELETE collision policy:
v1 treats DELETE-not-found as successful idempotence, preserving PostgreSQL
zero-row DELETE semantics while still cleaning stale placement rows.

## What Changed

- ADR-069 now documents the v1 DELETE collision policy:
  - absent placement row returns zero affected rows and does not dispatch
    remotely;
  - placement row present but owning heap row absent removes the placement row
    and returns zero affected rows.
- `ec_spire_prepare_coordinator_delete_tuple_payload(...)` now:
  - returns `delete_not_found_noop` when the coordinator placement row is
    already absent;
  - accepts zero local or remote deleted rows as success;
  - preserves the existing hard error if more than one row is deleted;
  - reports local and remote not-found statuses distinctly.
- Placement-row deletion is idempotent; a disappeared placement row now returns
  `placement_deleted = false` instead of raising.
- Added a PG18 fixture covering both missing placement and stale local
  placement cleanup.
- Phase 12.4 tracker marks the DELETE collision decision and implementation
  rows complete.

## Evidence

See `artifacts/manifest.md`.

Validation run against `006167504ecd10b17104f862cc346d94e1211ffd`:

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_prepare_coordinator_delete`

## Review Focus

- Confirm the idempotent not-found behavior matches the ADR-069 v1 policy.
- Confirm the no-placement sentinel row (`node_id = -1`, `served_epoch = 0`,
  `prepared_gid = 'none'`) is acceptable for the helper surface.
- Confirm allowing zero remote deletes as a prepared success is the right v1
  behavior, given coordinator placement cleanup still occurs.
