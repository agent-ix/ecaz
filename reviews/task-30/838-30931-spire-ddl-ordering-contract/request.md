---
topic: spire-ddl-ordering-contract
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30931
stage: phase-12.5
status: open
---

# Review Request: SPIRE DDL Ordering Contract

## Scope

Please review commit `e1c75ee95e83902e395446761e3f80964d7d1865`
(`Document SPIRE DDL ordering contract`).

This docs-only slice closes the Phase 12.5 DDL ordering and DDL-window guard
decision rows:

- ADR-069 now spells out the v1 operator sequence: pause writes and bulk-load
  placement registration, apply DDL to the coordinator, apply matching DDL to
  every remote, refresh affected remote-node descriptors, verify descriptor
  state, then resume writes.
- `docs/SPIRE_DIAGNOSTICS.md` mirrors the operator-facing runbook wording.
- The lightweight guard decision is explicit: v1 does not add a separate
  DDL-window guard GUC/catalog flag; the operational guard is the documented
  pause/apply/refresh/resume sequence, and the planned Phase 12.5 schema-drift
  fingerprint remains the fail-closed safety net for violated ordering.

No runtime behavior changed, and this does not claim the schema-drift
fingerprint implementation has landed.

## Review Focus

- Confirm the ordering sequence is complete enough for v1 operator docs.
- Confirm the no-separate-DDL-guard decision is scoped correctly and does not
  preclude the still-open schema-drift fingerprint work.
- Confirm the tracker rows are not overclaiming implementation beyond this
  docs/decision slice.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
