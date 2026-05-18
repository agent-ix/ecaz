---
topic: spire-bulk-load-registration-docs
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30926
stage: phase-12.4
status: open
---

# Review Request: SPIRE Bulk-Load Registration Docs

## Scope

Please review commit `4229539f` (`Clarify SPIRE bulk-load registration docs`).

This responds to P3 feedback from `30924`.

## What Changed

- `docs/SPIRE_DIAGNOSTICS.md` now names the v1 bulk-load primitives:
  `ec_spire_classify_centroid(...)` and
  `ec_spire_register_placement_batch(...)`.
- The diagnostics runbook now clarifies that
  `ec_spire_register_placement_batch(...)` is transactional within the calling
  session: entries from one call become visible together at commit or not at
  all on rollback.
- ADR-069 mirrors the transaction-boundary wording, so the partial visibility
  warning is scoped to committed bulk-load batches/tool runs rather than one
  registration transaction.

## Evidence

See `artifacts/manifest.md`.

Validation run against
`4229539f9f2b595c61e4456872c1cb0dd799dc36`:

- `git diff --check HEAD^ HEAD`

No runtime tests were run; this is a docs-only clarification.

## Review Focus

- Confirm the transaction-boundary wording accurately describes
  `ec_spire_register_placement_batch(...)`.
- Confirm naming the two primitives in the diagnostics runbook resolves the
  `30924` P3 cross-link request.
