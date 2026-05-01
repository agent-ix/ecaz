---
id: FR-029
title: Multi-AM SQL Bootstrap Contract
type: functional-requirement
artifact_type: FR
status: IMPLEMENTED
object_type: configuration
relationships:
  - target: "ix://agent-ix/tqvector/US-012"
    type: "implements"
    cardinality: "N:1"
---
# FR-029: Multi-AM SQL Bootstrap Contract

## Requirement

`CREATE EXTENSION ecaz` SHALL register all implemented SQL types, functions, operators, access methods, and operator classes required by the current multi-AM surface.

## Required SQL Surface

| Object class | Required objects |
| --- | --- |
| Types | `ecvector`, `tqvector` |
| Access methods | `ec_hnsw`, `ec_ivf`, `ec_diskann` |
| HNSW opclasses | `ecvector_ip_ops`, `tqvector_ip_ops` |
| IVF opclasses | `ecvector_ip_ops`, `tqvector_ip_ops` scoped to `ec_ivf` |
| DiskANN opclasses | `ecvector_diskann_ip_ops`, `tqvector_diskann_ip_ops` |
| Operators | `<#>` for supported type/query combinations |
| Functions | encode, scoring, casts, AM handlers, diagnostics, and stats surfaces exposed by bootstrap SQL |

## Acceptance Criteria

### FR-029-AC-1

After `CREATE EXTENSION ecaz`, `pg_am` includes `ec_hnsw`, `ec_ivf`, and `ec_diskann`.

### FR-029-AC-2

An `ecvector` column can be indexed by all three implemented AMs with the documented opclass.

### FR-029-AC-3

`DROP EXTENSION ecaz CASCADE` removes the extension-owned SQL objects.
