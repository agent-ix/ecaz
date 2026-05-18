---
topic: spire-dml-pk-byte-buffers
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30942
stage: phase-12.3
status: open
---

# Review Request: SPIRE DML PK Byte Buffers

## Scope

Please review commit `7c5eac3032f4b1be4d0235bd110d8c45f740e9f7`
(`Use fixed SPIRE DML PK byte buffers`).

This slice closes the Phase 12.3 PK-byte allocation cleanup item:

- Changes DML frontdoor bigint PK byte helpers from allocating `Vec<u8>` to
  returning fixed `[u8; 8]` buffers.
- Carries `[u8; 8]` through primitive plans, runtime parameter evaluation,
  invocation metadata, and `EcSpireDistributedScan` executor state.
- Keeps public SQL/SPI `bytea` boundaries as `Vec<u8>` conversions where pgrx
  needs PostgreSQL datum ownership.
- Updates the Phase 12 tracker with the focused PG18 validation evidence.

## Review Focus

- Confirm the fixed-size buffer is appropriate for the current Phase 12 bigint
  PK-only DML contract.
- Confirm the remaining `to_vec()` calls are constrained to PostgreSQL `bytea`
  conversion boundaries.
- Confirm removing the old empty-vector checks does not drop a reachable error
  path now that the internal representation is always exactly eight bytes.

## Validation

Artifacts are packet-local under `artifacts/` and described in
`artifacts/manifest.md`.

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send`
- `cargo pgrx test pg18 test_ec_spire_dml_frontdoor_primitive_plan_from_decision`

Key results:

- `test tests::pg_test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send ... ok`
- `test tests::pg_test_ec_spire_dml_frontdoor_primitive_plan_from_decision ... ok`
