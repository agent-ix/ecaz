---
topic: spire-typed-tuple-domain-composite
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30917
stage: phase-12.2
status: open
---

# Review Request: SPIRE Typed Tuple Domain/Composite Fixture

## Scope

Please review commit `cb40c9d3` (`Cover typed tuple domain and composite
payloads`).

This is a focused Phase 12.2 coverage slice. It extends the typed tuple
endpoint fixture matrix without changing the production CustomScan receive path.

## What Changed

- Added `test_ec_spire_typed_tuple_payload_domain_composite_sql`.
- The fixture creates:
  - a text domain `ec_spire_typed_label_domain`;
  - a named composite `ec_spire_typed_pair`;
  - an isolated table/index pair with domain, composite, bigint, and ecvector
    columns.
- The fixture asserts `ec_spire_remote_search_tuple_payload_typed(...)`
  returns:
  - domain and named composite `payload_type_oids`;
  - column-aligned `payload_attnums`, `payload_names`, `payload_nulls`, and
    `payload_formats`;
  - domain value bytes matching the base text binary send representation;
  - named composite bytes matching `record_send(...)`.
- Updated the Phase 12 tracker to mark the domain/composite endpoint fixture
  class complete.

## Evidence

See `artifacts/manifest.md`.

Validation run against `cb40c9d30ef500f3c7d810bb66366182b5322a3b`:

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_typed_tuple_payload_domain_composite_sql`
- `cargo pgrx test pg18 test_ec_spire_typed_tuple_payload`

The last command passed all three typed endpoint fixtures:

- scalar JSON-parity;
- NULL and `text[]`;
- domain and named composite.

## Review Focus

- Confirm the domain oracle should compare bytes to the base type binary send
  representation while preserving the domain type OID in metadata.
- Confirm `record_send(ROW(...)::named_composite)` is an acceptable oracle for
  named composite payload bytes.
- Confirm this packet should only close endpoint fixture coverage for domain
  and composite values. Negotiation, CustomScan typed binary receive, tuple
  throughput measurement, and production JSON retirement remain open Phase 12.2
  work.
