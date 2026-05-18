---
topic: spire-review-followups-2
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30944
stage: phase-12
status: open
---

# Review Request: SPIRE Review Follow-Ups 2

## Scope

Please review commit `4549ed6ac1b5d087c5a4673e088ca1c3b2377af3`
(`Address SPIRE review follow-ups`).

This small feedback-response slice handles accepted P3 notes from recent
Phase 12 reviews:

- Packet `30942` asked for an explicit code comment naming the v1 bigint-only
  PK buffer constraint. `SpireDmlFrontdoorPrimitiveInvocation.pk_value` now
  documents that `[u8; 8]` is tied to ADR-069 v1 bigint PKs and must widen if
  UUID or composite PKs are admitted later.
- Packet `30943` asked to pin automated prepared-xact recovery intent metadata
  in ADR-069's future-ADR list. ADR-069 now lists the future helper and names
  the required durable GID, PK, and outcome metadata.

## Review Focus

- Confirm the comment is accurate and does not imply support beyond v1 bigint
  PK DML.
- Confirm the future-ADR note captures the metadata needed before
  `ec_spire_recover_orphaned_prepared_xacts(node_id)` can be safe.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `rg -n 'ADR-069 v1 DML supports bigint PKs only|Automated orphaned prepared-transaction recovery helper' ...`

No PG tests were run because this slice changes one Rust comment and one ADR
future-scope bullet only.
