---
topic: spire-typed-tuple-endpoint-scalar
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
stage: phase-12.2
status: open
---

# Review Request: SPIRE Typed Tuple Endpoint Scalar

## Scope

Phase 12.2 implementation slice 1 for commit `92e63f1b`
(`Add SPIRE typed tuple payload endpoint`).

This is the endpoint scaffold and scalar JSON-parity fixture requested by the
accepted typed transport design packet `30913`:

- adds `ec_spire_remote_search_tuple_payload_typed(...)` beside the existing
  JSON endpoint;
- reuses the same local heap candidate path as
  `ec_spire_remote_search_tuple_payload(...)`;
- returns the existing candidate identity fields plus typed projection arrays:
  `payload_attnums`, `payload_names`, `payload_type_oids`,
  `payload_typmods`, `payload_collations`, `payload_nulls`,
  `payload_values`, and `payload_formats`;
- emits `tuple_transport = 'pg_binary_attr_v1'` and
  `tuple_transport_status = 'ready'` for supported scalar rows;
- fetches per-column metadata from `pg_attribute` / `pg_type` and calls each
  requested column type's `typsend` function to build `bytea[]` payload values;
- keeps the JSON endpoint and CustomScan receive path unchanged;
- adds a PG18 scalar parity fixture for `bigint` and `text` that compares the
  typed bytes against `int8send(...)` and `textsend(...)`;
- updates the Phase 12.2 tracker to mark the typed endpoint scaffold complete
  while leaving negotiation, executor receive, and non-scalar coverage open.

## Files

- `src/lib.rs`
- `plan/tasks/task30-phase12-spire-production-hardening.md`

## Validation

- `git diff --check HEAD^ HEAD`
  - artifact: `artifacts/git-diff-check.log`
- `cargo fmt --check`
  - artifact: `artifacts/cargo-fmt-check.log`
- `cargo pgrx test pg18 test_ec_spire_typed_tuple_payload_scalar_parity_sql`
  - artifact: `artifacts/cargo-pgrx-test-typed-scalar-parity.log`
- `cargo pgrx test pg18 test_ec_spire_remote_search_tuple_payload`
  - artifact: `artifacts/cargo-pgrx-test-json-tuple-payload-regression.log`

## Review Focus

- Confirm the endpoint scaffold matches the accepted per-attribute
  `pg_binary_attr_v1` shape from packet `30913`.
- Confirm using dynamic SQL around each column's `typsend` function is an
  acceptable first endpoint implementation before moving receive/conversion
  into the CustomScan executor.
- Confirm the scalar fixture proves JSON parity without overclaiming array,
  composite, NULL, domain, negotiation, or executor receive coverage.
