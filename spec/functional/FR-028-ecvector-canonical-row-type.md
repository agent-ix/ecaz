---
id: FR-028
title: ecvector Canonical Row Type
type: functional-requirement
artifact_type: FR
status: IMPLEMENTED
object_type: entity
relationships:
  - target: "ix://agent-ix/tqvector/US-012"
    type: "implements"
    cardinality: "N:1"
---
# FR-028: ecvector Canonical Row Type

## Requirement

The extension SHALL register `ecvector` as the canonical exact/raw fp32 row type for application tables.

## Behavior

1. `ecvector(dim)` SHALL enforce dimensionality through typmod.
2. Typmod-less `ecvector` SHALL be accepted where index metadata or caller context owns dimensional consistency.
3. Text and binary I/O SHALL round-trip finite fp32 vectors.
4. Casts between `real[]`, `bytea`, and `ecvector` SHALL preserve fp32 payloads according to the registered cast functions.
5. `encode_to_ecvector(real[], integer, bigint)` SHALL accept only the canonical quantizer defaults `(4, 42)` on current main and SHALL reject other bit/seed pairs with a clear error.
6. Non-finite values SHALL be rejected.

## Acceptance Criteria

### FR-028-AC-1

`CREATE EXTENSION ecaz` registers `ecvector`, its typmod input function, text I/O, binary I/O, and casts.

### FR-028-AC-2

Inserting a vector with the wrong dimensionality into `ecvector(N)` raises ERROR.

### FR-028-AC-3

`real[] -> ecvector -> real[]` and `bytea -> ecvector -> bytea` preserve the fp32 payload.

### FR-028-AC-4

`encode_to_ecvector(input, 4, 42)` returns a storable `ecvector`; non-canonical bit/seed pairs raise ERROR.
