---
topic: spire-read-isolation-runbook
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30939
stage: phase-12.6
status: open
---

# Review Request: SPIRE Read Isolation Runbook

## Scope

Please review commit `79b7d5a43a39274795aebbb4cd0f4ecce95b5dfc`
(`Document SPIRE read isolation limitation`).

This is a narrow follow-up to reviewer feedback on packets `30937` and `30938`:

- Adds an operator-facing `docs/SPIRE_DIAGNOSTICS.md` note that distributed
  table reads provide read-committed semantics only, even when the surrounding
  coordinator transaction is `REPEATABLE READ` or `SERIALIZABLE`.
- States the practical application contract: callers that need PostgreSQL's
  normal repeatable/serializable guarantees for distributed tables need
  application-level locking or must accept the v1 limitation.
- Adds a one-line rationale comment to the insert descriptor race test helper
  explaining why the SPIRE prepared-xact GID prefix `LIKE` query is safe.

## Review Focus

- Confirm the diagnostics wording is operator-facing and does not overstate
  distributed-read guarantees.
- Confirm the helper comment addresses the GID-prefix concern from `30938`
  without adding unnecessary test complexity.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`

No pgrx test was run for this docs/comment-only follow-up.
