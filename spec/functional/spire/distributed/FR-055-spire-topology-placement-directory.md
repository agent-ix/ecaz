---
id: FR-055
title: SPIRE Distributed Topology and Placement Directory
type: functional-requirement
artifact_type: FR
status: APPROVED
object: data_schema
relationships:
  - target: "ix://agent-ix/ecaz/FR-048"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-055: SPIRE Distributed Topology and Placement Directory

## Requirement

Distributed SPIRE SHALL use a coordinator PostgreSQL instance for routing
metadata and one or more remote PostgreSQL shard nodes for row storage and
near-data SPIRE scoring. The coordinator SHALL maintain placement-directory
state for writes and PK-keyed reads, not for vector-read candidate routing.

## Topology

```mermaid
flowchart TD
    App["application SQL"]
    Coord["coordinator logical relation\nrouting index + descriptors"]
    Dir["ec_spire_placement\nPK -> node"]
    R1["remote node 1\nshard table + local ec_spire index"]
    R2["remote node 2\nshard table + local ec_spire index"]
    R3["remote node 3\nshard table + local ec_spire index"]

    App --> Coord
    Coord --> Dir
    Coord --> R1
    Coord --> R2
    Coord --> R3
```

## Coordinator Role

The coordinator SHALL host:

1. the logical relation used by applications;
2. routing centroids, placement metadata, remote descriptors, and epoch
   readiness state;
3. the `ec_spire_placement` table for coordinator-routed INSERT, UPDATE,
   DELETE, bulk placement registration, and PK-keyed reads;
4. optional local shard rows when `node_id = 0` placements are configured.

The coordinator SHALL NOT mirror every remote-origin row merely to make
distributed vector reads work.

## Remote Role

Each remote node SHALL host:

1. a shard table with the same relevant column shape as the coordinator logical
   relation;
2. a local `ec_spire` index over shard rows;
3. a remote descriptor/announce surface reporting endpoint identity, served
   epoch, extension version, tuple transport capability, and schema
   fingerprint state.

## Placement Directory Schema

`ec_spire_placement` SHALL contain:

| Column | Type | Rule |
| --- | --- | --- |
| `index_oid` | `oid` | coordinator SPIRE index |
| `pk_value` | `bytea` | canonical primary-key encoding; v1 bigint uses PostgreSQL `int8send` bytes |
| `node_id` | `integer` | `0` for coordinator-local, positive for remotes |
| `centroid_id` | `bigint` | active-epoch routing leaf identity, opaque across retraining |
| `served_epoch` | `bigint` | positive remote/coordinator epoch |
| `source_identity` | `bytea` | exact 16-byte stable identity payload |

Primary key SHALL be `(index_oid, pk_value)`. A secondary identity index SHALL
support lookup by `(index_oid, source_identity)`.

## Acceptance Criteria

### FR-055-AC-1

The topology distinguishes coordinator routing metadata from remote shard row
storage and local SPIRE scoring.

### FR-055-AC-2

The placement directory is defined as the write-routing and PK-read source of
truth, not as a read-path materialization catalog.

### FR-055-AC-3

The v1 schema states that non-vector non-PK scatter-gather reads and automatic
DDL propagation are out of scope.
