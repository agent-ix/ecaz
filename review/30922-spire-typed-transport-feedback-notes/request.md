---
topic: spire-typed-transport-feedback-notes
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30922
stage: phase-12.2
status: open
---

# Review Request: SPIRE Typed Transport Feedback Notes

## Scope

Please review commit `00d2cc63` (`Clarify SPIRE typed transport feedback
points`).

This is a docs-only response to the accepted reviewer feedback on packets
`30917`, `30918`, and `30919`.

## What Changed

- `plan/design/spire-typed-tuple-transport.md` now states that empty
  projections are a supported `pg_binary_attr_v1` shape.
- The design pins v1 composite scope to named heap-column composite values and
  explicitly defers anonymous computed `record` projections until a future
  projection surface.
- The negotiation section now says transport capability advertisement is
  protocol-level, while per-column binary I/O gaps still fail with
  `unsupported_type_binary_io`.
- The design also records that `tuple_transport_default` is hardcoded for v1,
  not a per-descriptor operator override.
- The Phase 12.2 tracker notes that endpoint-level typed coverage now covers
  the v1 scalar/array/composite/domain/NULL type-class gaps that blocked the
  JSON bridge.

## Evidence

See `artifacts/manifest.md`.

Validation run against `00d2cc6310191981f4823248718dff576a8d7c9d`:

- `git diff --check HEAD^ HEAD`

No code tests were run because this packet changes only design/tracker
Markdown.

## Review Focus

- Confirm the anonymous-composite v1 deferral matches the table-column
  projection contract.
- Confirm the capability-vs-column-type wording handles the 30919 P3 without
  weakening strict typed validation.
- Confirm the tracker note does not overclaim CustomScan receive or full
  negotiation completion.
