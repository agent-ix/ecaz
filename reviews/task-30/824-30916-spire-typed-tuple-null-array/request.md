---
topic: spire-typed-tuple-null-array
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
stage: phase-12.2
status: open
---

# Review Request: SPIRE Typed Tuple NULL And Array

## Scope

Phase 12.2 typed endpoint coverage checkpoint for commit `9b0cb4f4`
(`Cover typed tuple NULL and array payloads`).

This packet extends the typed endpoint fixture coverage added in packet `30915`:

- adds a PG18 fixture for out-of-band SQL NULL handling in
  `ec_spire_remote_search_tuple_payload_typed(...)`;
- adds a `text[]` fixture that verifies typed payload bytes match PostgreSQL's
  own `array_send(...)` output;
- verifies column metadata for the mixed projection
  `id bigint`, `title text`, and `tags text[]`;
- verifies NULL columns carry `payload_nulls = true` and an empty bytea payload
  placeholder while non-NULL columns keep binary bytes;
- updates the Phase 12.2 tracker to mark NULL and array typed endpoint fixture
  coverage complete, while leaving composite, domain, negotiation, executor
  receive, and JSON retirement work open.

## Files

- `src/lib.rs`
- `plan/tasks/task30-phase12-spire-production-hardening.md`

## Validation

- `git diff --check HEAD^ HEAD`
  - artifact: `artifacts/git-diff-check.log`
- `cargo fmt --check`
  - artifact: `artifacts/cargo-fmt-check.log`
- `cargo pgrx test pg18 test_ec_spire_typed_tuple_payload_null_array_sql`
  - artifact: `artifacts/cargo-pgrx-test-typed-null-array.log`
- `cargo pgrx test pg18 test_ec_spire_typed_tuple_payload`
  - artifact: `artifacts/cargo-pgrx-test-typed-tuple-payload.log`

## Review Focus

- Confirm the NULL representation matches the accepted design: nullness is
  out-of-band through `payload_nulls`, and payload bytes are ignored for NULL
  attributes.
- Confirm `array_send(...)` is a suitable byte-level oracle for this first
  `text[]` endpoint fixture.
- Confirm this packet does not overclaim remaining Phase 12.2 coverage:
  composite, domain, negotiation, CustomScan receive, and JSON retirement remain
  open.
