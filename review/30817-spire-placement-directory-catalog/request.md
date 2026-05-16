---
topic: spire-placement-directory-catalog
agent: coder2
role: coder
model: GPT-5
date: 2026-05-11
stage: task-30-phase11-stage-d-adr069
status: open
---

# Review Request: SPIRE Placement Directory Catalog

## Scope

This packet lands the first ADR-069 write-path slice: the coordinator-local
placement directory catalog surface.

Changes:

- Add `ec_spire_placement(index_oid, pk_value, node_id, centroid_id,
  served_epoch, source_identity)` to bootstrap and upgrade SQL.
- Add the required primary key on `(index_oid, pk_value)`.
- Add `ec_spire_placement_by_identity` on `(index_oid, source_identity)`.
- Enforce basic catalog invariants, including non-empty `pk_value`, positive
  `node_id`, non-negative `centroid_id`, positive `served_epoch`, and 16-byte
  ADR-063 `source_identity`.
- Include placement rows in remote catalog orphan summary, orphan cleanup, and
  drop-index cleanup diagnostics.
- Update the Phase 11 task tracker to mark only the placement-directory catalog
  surface complete; classifier and coordinator-routed writes remain open.

This does not implement `ec_spire_classify_centroid`, INSERT forwarding, 2PC, or
PK read/update/delete routing.

## Validation

Packet-local logs are in `artifacts/`.

- `cargo test ec_spire_placement --lib`
- `cargo test remote_catalog --lib`
- `cargo fmt --check`
- `git diff --check`

## Review Focus

- Confirm the table shape and identity index match ADR-069.
- Confirm cleanup diagnostics cover placement rows without changing the
  existing remote executor/read path behavior.
- Confirm the task tracker scope is accurate and does not imply write-path
  forwarding is complete.
