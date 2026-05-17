---
topic: spire-typed-tuple-feedback-response
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30918
stage: phase-12.2-feedback
status: open
---

# Review Request: Typed Tuple And GID Feedback Response

## Scope

Please review commit `ddebcc5a` (`Address typed tuple and GID review
feedback`).

This packet responds to reviewer feedback on `30914`, `30915`, and `30916`.

## What Changed

- Typed endpoint unsupported-binary-I/O diagnostics now include the column
  name, formatted type name, and type OID.
- The scalar typed payload fixture now also verifies an empty requested
  projection returns aligned empty metadata/value arrays with
  `pg_binary_attr_v1`, rather than falling back to JSON or erroring.
- The typed transport design now explicitly documents the SQL NULL convention:
  `payload_nulls[i] = true` and `payload_values[i]` is a zero-length `bytea`
  placeholder that receive code must ignore.
- The prepared-transaction GID docs now state that `ec_spire_insert_` is an
  operation-agnostic historical prefix and must not be used to distinguish
  INSERT from DELETE prepared transactions.
- ADR-069 now describes `top_xid` as correlation evidence for logs and
  coordinator-side state, while the recovery decision remains based on the
  known coordinator outcome and placement-row state for the affected key.
- Phase 12 tracker now records the empty-projection fixture.

## Evidence

See `artifacts/manifest.md`.

Validation run against `ddebcc5a8213a79d50f73bbc4328b2497f6ac39a`:

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_typed_tuple_payload`

## Review Focus

- Confirm the unsupported-type diagnostic now carries enough type identity for
  operator triage.
- Confirm the empty-projection behavior should remain supported as a typed
  metadata/value array case.
- Confirm the GID recovery wording avoids overclaiming direct xid lookup in
  `ec_spire_placement`, which currently stores `(index_oid, pk_value)` as its
  primary key and does not expose a supported transaction-id column.
