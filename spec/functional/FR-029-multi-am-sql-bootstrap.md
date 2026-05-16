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
| Access methods | `ec_hnsw`, `ec_ivf`, `ec_diskann`, `ec_spire` |
| HNSW opclasses | `ecvector_ip_ops`, `tqvector_ip_ops` |
| IVF opclasses | `ecvector_ip_ops`, `tqvector_ip_ops` scoped to `ec_ivf` |
| DiskANN opclasses | `ecvector_diskann_ip_ops`, `tqvector_diskann_ip_ops` |
| SPIRE opclasses | `ecvector_spire_ip_ops`, `tqvector_spire_ip_ops` scoped to `ec_spire` |
| Operators | `<#>` for supported type/query combinations |
| Functions | encode, scoring, casts, AM handlers, diagnostics, and stats surfaces exposed by bootstrap SQL |

## Acceptance Criteria

### FR-029-AC-1

After `CREATE EXTENSION ecaz`, `pg_am` includes `ec_hnsw`, `ec_ivf`, `ec_diskann`, and `ec_spire`.

### FR-029-AC-2

An `ecvector` column can be indexed by all three implemented AMs with the documented opclass.

### FR-029-AC-3

`DROP EXTENSION ecaz CASCADE` removes the extension-owned SQL objects.

### FR-029-AC-4

The `ec_spire` access method registers its AM handler and SPIRE-specific opclasses for local partition-object indexes; distributed remote reads use the `EcSpireDistributedScan` CustomScan path when active remote placements exist.
