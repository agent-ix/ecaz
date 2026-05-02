---
id: FR-038
title: SPIRE Partition Object Storage and Placement
type: functional-requirement
artifact_type: FR
status: DRAFT
object_type: process
relationships:
  - target: "ix://agent-ix/tqvector/US-017"
    type: "implements"
    cardinality: "N:1"
---
# FR-038: SPIRE Partition Object Storage and Placement

## Requirement

`ec_spire` SHALL store SPIRE hierarchy state as PID-addressed partition objects with explicit placement metadata so local single-store, local multi-NVMe, and future multi-machine deployments use the same logical routing model.

## Terminology

- **SPIRE partition:** an index-internal cluster object addressed by PID. It is not a PostgreSQL table partition.
- **PID:** durable SPIRE partition object identifier.
- **Partition object:** an immutable or versioned object containing either internal routing metadata and child PIDs or leaf vector assignment/posting rows.
- **Partition store:** a bounded physical container for many partition objects. A local deployment may place stores in different tablespaces backed by different NVMe devices.
- **Epoch:** a published SPIRE index version that identifies a compatible root graph, hierarchy, placement map, and partition object set.

## Behavior

1. `ec_spire` SHALL keep SPIRE partition selection inside the SPIRE access method or coordinator; PostgreSQL planner partition pruning SHALL NOT choose SPIRE PIDs.
2. Root/control metadata SHALL record the active epoch, hierarchy metadata, root graph metadata, and PID placement map.
3. Internal partition objects SHALL record level, PID, parent PID where applicable, routing metadata, and child PIDs.
4. Leaf partition objects SHALL record level, PID, parent PID where applicable, and assignment/posting rows.
5. Assignment/posting rows SHALL include stable `vec_id`, local heap TID or row locator, PID, encoded scoring payload, and flags for primary assignment, boundary replica, tombstone, or delta state where applicable.
6. The first local implementation MAY map all PIDs to one partition store, but the on-disk metadata SHALL preserve the `pid -> local_store_id -> object location` abstraction.
7. Local multi-NVMe placement SHALL map PIDs across a bounded set of local partition stores, normally by `hash(pid) % local_store_count`.
8. Multi-machine placement SHALL extend the map to `pid -> node_id -> local_store_id -> object location` and SHALL require stable `vec_id` values suitable for remote candidate merge.
9. Partition objects SHALL be versioned directly by epoch or referenced by an epoch manifest so a query reads a consistent object set.
10. Old epochs SHALL remain readable until in-flight queries using them can finish or fail with an explicit stale-epoch error.
11. Diagnostics SHALL expose read-only SQL functions or views for partition counts, placement map state, per-store object bytes, assignment cardinality, active epoch, and stale/unavailable placement entries.

## Data Schema

```mermaid
erDiagram
    SPIRE_ROOT {
        oid index_oid
        bigint active_epoch
        oid heap_relid
        text consistency_mode
        int root_graph_pid
    }
    EPOCH_MANIFEST {
        bigint epoch
        text state
        timestamptz published_at
        timestamptz retain_until
    }
    PLACEMENT_ENTRY {
        bigint epoch
        bigint pid
        int node_id
        int local_store_id
        text object_locator
        text state
    }
    PARTITION_OBJECT {
        bigint pid
        bigint object_version
        int level
        bigint parent_pid
        text kind
        bytea payload
    }
    ASSIGNMENT_ROW {
        bigint pid
        bytea vec_id
        tid heap_tid
        bytea encoded_payload
        int flags
    }

    SPIRE_ROOT ||--o{ EPOCH_MANIFEST : publishes
    EPOCH_MANIFEST ||--o{ PLACEMENT_ENTRY : maps
    PLACEMENT_ENTRY ||--|| PARTITION_OBJECT : locates
    PARTITION_OBJECT ||--o{ ASSIGNMENT_ROW : contains
```

`object_locator` is intentionally abstract in this requirement. Phase 0 SHALL
compare table/relation-backed layouts first and may choose another AM-owned
layout only if measurement or PostgreSQL mechanics show tables are the wrong
hot-path container.

## Architecture

```mermaid
flowchart TD
    Planner["PostgreSQL planner\nchooses ec_spire path"]
    Root["SPIRE root/control metadata\nactive epoch, root graph, placement map"]
    Router["SPIRE router\nquery vector -> selected PIDs"]
    Stores["Partition stores\nPostgres-managed where viable"]
    Leaf["Leaf partition objects\nassignment/posting rows"]
    Exec["PostgreSQL executor\nheap visibility and result rows"]

    Planner --> Root
    Root --> Router
    Router --> Stores
    Stores --> Leaf
    Leaf --> Exec
```

## Acceptance Criteria

### FR-038-AC-1

A single-level `ec_spire` build persists leaf partition objects with one logical assignment row per indexed vector.

### FR-038-AC-2

Boundary replication can add multiple assignment rows for the same `vec_id` across different PIDs without changing the persisted row schema.

### FR-038-AC-3

Admin diagnostics can report PID count, leaf assignment cardinality, active epoch, and placement distribution without allowing user DML against SPIRE partition objects.

### FR-038-AC-4

The local multi-store path can place partition objects across at least two store relations or equivalent bounded containers without creating one PostgreSQL relation per PID.

### FR-038-AC-5

A query that requests an unavailable or stale epoch fails explicitly rather than silently mixing partition objects from incompatible epochs.
