---
topic: spire-typed-tuple-transport-design
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
stage: phase-12.2
status: open
---

# Review Request: SPIRE Typed Tuple Transport Design

## Scope

Docs-only Phase 12.2 design checkpoint for commit `680b3d05`
(`Design SPIRE typed tuple transport`).

This packet intentionally stops before executor or endpoint code changes. It
pins the protocol direction for the P1/P3 JSON retirement work:

- chooses per-attribute PostgreSQL binary I/O over one binary composite/record
  payload;
- defines the proposed typed endpoint shape beside
  `ec_spire_remote_search_tuple_payload(...)`;
- defines coordinator receive behavior, metadata validation, and fail-closed
  malformed-typed-payload handling;
- defines descriptor/identity negotiation fields and strict/degraded fallback
  rules;
- records a one-minor-version JSON fallback window and concrete removal
  criteria;
- updates the Phase 12 tracker to mark the typed protocol design row complete.

## Files

- `plan/design/spire-typed-tuple-transport.md`
- `plan/tasks/task30-phase12-spire-production-hardening.md`

## Validation

- `git diff --check HEAD^ HEAD`
  - artifact: `artifacts/git-diff-check.log`

No tests were run; this is a design/tracker-only checkpoint.

## Review Focus

- Confirm per-attribute `typsend`/receive framing is the right first typed
  transport, versus binary composite/record.
- Confirm the negotiation and JSON fallback/removal criteria are concrete
  enough to prevent the JSON path from lingering indefinitely.
- Confirm the implementation slice order is safe before endpoint/executor code
  changes begin.
