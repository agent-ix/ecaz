---
id: FR-028
title: ecvector Canonical Row Type
type: FR
status: IMPLEMENTED
object: entity
relationships:
  - target: "ix://agent-ix/ecaz/US-012"
    type: "implements"
    cardinality: "N:1"
---
# FR-028: ecvector Canonical Row Type

## Description

The extension SHALL register `ecvector` as the canonical exact/raw fp32 row type for application tables.

## Behavior

1. `ecvector(dim)` SHALL enforce dimensionality through typmod.
2. Typmod-less `ecvector` SHALL be accepted where index metadata or caller context owns dimensional consistency.
3. Text and binary I/O SHALL round-trip finite fp32 vectors.
4. Casts between `real[]`, `bytea`, and `ecvector` SHALL preserve fp32 payloads according to the registered cast functions.
5. `encode_to_ecvector(real[], integer, bigint)` SHALL accept only the canonical quantizer defaults `(4, 42)` on current main and SHALL reject other bit/seed pairs with a clear error.
6. Non-finite values SHALL be rejected.

## Properties

The `ecvector` SQL type is registered in `sql/bootstrap.sql` with the following attributes.

| Field | Type | Description |
|---|---|---|
| INTERNALLENGTH | variable | Varlena (variable-length, TOASTable) representation. |
| INPUT | `ecvector_in(cstring, oid, integer)` | Text input; receives the typmod for dimensionality enforcement. |
| OUTPUT | `ecvector_out(ecvector) → cstring` | Text output. |
| RECEIVE | `ecvector_recv(internal, oid, integer)` | Binary receive; receives the typmod. |
| SEND | `ecvector_send(ecvector) → bytea` | Binary send. |
| TYPMOD_IN | `ecvector_typmod_in(cstring[]) → integer` | Parses the `ecvector(dim)` typmod modifier. |
| STORAGE | external | TOAST storage strategy (uncompressed, out-of-line allowed). |
| dim (typmod) | integer | Declared dimensionality constraint; capped at `ECVECTOR_MAX_DIM` (65535). A typmod of `-1` means unconstrained. |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-028-AC-1 | `CREATE EXTENSION ecaz` registers `ecvector` with typmod input, text I/O, binary I/O, and casts | Test |
| FR-028-AC-2 | Wrong-dimensionality inserts into `ecvector(N)` raise ERROR | Test |
| FR-028-AC-3 | `real[]`/`bytea` round-trips through `ecvector` preserve the fp32 payload | Test |
| FR-028-AC-4 | `encode_to_ecvector` accepts only canonical `(4, 42)` quantizer defaults; other pairs raise ERROR | Test |

### FR-028-AC-1

`CREATE EXTENSION ecaz` registers `ecvector`, its typmod input function, text I/O, binary I/O, and casts.

### FR-028-AC-2

Inserting a vector with the wrong dimensionality into `ecvector(N)` raises ERROR.

### FR-028-AC-3

`real[] -> ecvector -> real[]` and `bytea -> ecvector -> bytea` preserve the fp32 payload.

### FR-028-AC-4

`encode_to_ecvector(input, 4, 42)` returns a storable `ecvector`; non-canonical bit/seed pairs raise ERROR.

## Dependencies

- **Upstream**: US-012 (implements)
- **Downstream**: none identified
