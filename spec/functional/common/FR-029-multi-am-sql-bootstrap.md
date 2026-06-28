---
id: FR-029
title: Multi-AM SQL Bootstrap Contract
type: FR
status: IMPLEMENTED
object: configuration
relationships:
  - target: "ix://agent-ix/ecaz/US-012"
    type: "implements"
    cardinality: "N:1"
---
# FR-029: Multi-AM SQL Bootstrap Contract

## Description

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

## Configuration

`CREATE EXTENSION ecaz` creates the following access methods and operator classes (defined in `sql/bootstrap.sql`). Each is fixed at extension-creation time.

| Name | Scope | Type | Default | Description |
|---|---|---|---|---|
| ec_hnsw | creation | access_method | created | HNSW index access method (`HANDLER ec_hnsw_handler`). |
| ec_diskann | creation | access_method | created | DiskANN index access method (`HANDLER ec_diskann_handler`). |
| ec_ivf | creation | access_method | created | IVF index access method (`HANDLER ec_ivf_handler`). |
| ec_spire | creation | access_method | created | SPIRE index access method (`HANDLER ec_spire_handler`). |
| tqvector_ip_ops | creation | opclass | created | Default `tqvector` opclass for `ec_hnsw` and (separately) `ec_ivf`. |
| ecvector_ip_ops | creation | opclass | created | Default `ecvector` opclass for `ec_hnsw` and (separately) `ec_ivf`. |
| tqvector_diskann_ip_ops | creation | opclass | created | Default `tqvector` opclass for `ec_diskann`. |
| ecvector_diskann_ip_ops | creation | opclass | created | Default `ecvector` opclass for `ec_diskann`. |
| tqvector_spire_ip_ops | creation | opclass | created | Default `tqvector` opclass for `ec_spire`. |
| ecvector_spire_ip_ops | creation | opclass | created | Default `ecvector` opclass for `ec_spire`. |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-029-AC-1 | After `CREATE EXTENSION ecaz`, `pg_am` includes `ec_hnsw`, `ec_ivf`, `ec_diskann`, and `ec_spire` | Test |
| FR-029-AC-2 | An `ecvector` column can be indexed by all three implemented AMs with the documented opclass | Test |
| FR-029-AC-3 | `DROP EXTENSION ecaz CASCADE` removes the extension-owned SQL objects | Test |
| FR-029-AC-4 | `ec_spire` registers its AM handler and SPIRE opclasses; distributed remote reads use `EcSpireDistributedScan` when active remote placements exist | Test |

### FR-029-AC-1

After `CREATE EXTENSION ecaz`, `pg_am` includes `ec_hnsw`, `ec_ivf`, `ec_diskann`, and `ec_spire`.

### FR-029-AC-2

An `ecvector` column can be indexed by all three implemented AMs with the documented opclass.

### FR-029-AC-3

`DROP EXTENSION ecaz CASCADE` removes the extension-owned SQL objects.

### FR-029-AC-4

The `ec_spire` access method registers its AM handler and SPIRE-specific opclasses for local partition-object indexes; distributed remote reads use the `EcSpireDistributedScan` CustomScan path when active remote placements exist.

## Dependencies

- **Upstream**: US-012 (implements)
- **Downstream**: none identified
