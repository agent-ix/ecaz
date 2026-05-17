---
topic: spire-schema-drift-scope-feedback
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30935
stage: phase-12.5
status: open
---

# Review Request: SPIRE Schema Drift Scope Feedback

## Scope

Please review commit `65a58a08f0bf2716bf908acf45e68f3ee34776e0`
(`Document schema drift guard scope`).

This docs/tracker follow-up addresses the P2/P3 feedback from packet `30933`:

- ADR-069 now states the migration behavior:
  existing descriptors are backfilled when their coordinator index still
  exists; any descriptor left with `unset` fails closed for coordinator-routed
  INSERT until it is registered again.
- ADR-069 now names the canonical fingerprint inputs and clarifies the digest
  is a deterministic drift token, not a security boundary.
- ADR-069 and `docs/SPIRE_DIAGNOSTICS.md` now explicitly state the landed
  guard is INSERT-scoped.
- Phase 12.5 now tracks the follow-up to extend descriptor-bound schema-drift
  coverage to coordinator-routed UPDATE/DELETE payload paths, or document why
  INSERT-only remains the accepted v1 boundary.

## Review Focus

- Confirm the migration/backfill semantics are explicit enough for operators.
- Confirm tracking UPDATE/DELETE schema-drift coverage as a follow-up satisfies
  the `30933` scope feedback without overclaiming current implementation.
- Confirm the canonical fingerprint wording matches the implementation.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
