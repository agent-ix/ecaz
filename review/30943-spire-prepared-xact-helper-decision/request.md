---
topic: spire-prepared-xact-helper-decision
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30943
stage: phase-12.4
status: open
---

# Review Request: SPIRE Prepared Xact Helper Decision

## Scope

Please review commit `a3e8258bc0be5e1aa92b2b880b937c9a9a3897cf`
(`Defer automated SPIRE prepared-xact recovery`).

This docs/tracker slice closes the Phase 12.4 row to consider
`ec_spire_recover_orphaned_prepared_xacts(node_id)`:

- ADR-069 now explicitly defers the helper for v1 because remote
  `pg_prepared_xacts` does not include the affected primary key or coordinator
  transaction outcome needed by the documented commit/rollback rule.
- `docs/SPIRE_DIAGNOSTICS.md` tells operators to use the manual
  placement-directory recovery runbook until SPIRE records durable
  prepared-transaction intent metadata.
- The Phase 12 tracker marks the helper decision complete as deferred and
  marks the `max_prepared_transactions` readiness parent complete now that all
  of its child evidence rows are complete.

## Review Focus

- Confirm deferring the helper is the right v1 safety decision.
- Confirm the docs make clear that automated bulk resolution from the remote
  side alone is unsafe.
- Confirm the tracker update does not overclaim implementation beyond the
  documented/manual recovery boundary.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `rg -n 'recover_orphaned_prepared_xacts|Decision: defer|max_prepared_transactions readiness' ...`

No PG tests were run because this slice changes only docs and the tracker.
