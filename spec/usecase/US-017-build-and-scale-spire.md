---
id: US-017
title: Build and Scale SPIRE Indexes
type: user-story
artifact_type: US
status: DRAFT
relationships:
  - target: "ix://agent-ix/tqvector/StR-005"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/tqvector/FR-038"
    type: "derives_into"
    cardinality: "1:N"
---
# US-017: Build and Scale SPIRE Indexes

**As** a platform engineer,
**I want** to build a SPIRE index whose partition objects can be placed across local NVMe devices and later across machines,
**So that** I can scale ANN search beyond a single monolithic PostgreSQL index file while preserving query-vector routing and boundary-replica deduplication.

## Acceptance Criteria

### US-017-AC-1

`ec_spire` can build a single-level IVF-compatible foundation using PID-addressed partition objects and logical `(vec_id, pid)` assignment rows.

### US-017-AC-2

SPIRE partition selection happens inside the SPIRE index/coordinator from the query vector, root graph, hierarchy metadata, and centroids; it does not rely on PostgreSQL declarative table partition pruning.

### US-017-AC-3

The storage model can place partition objects across bounded local partition stores, each eligible to live in a different tablespace backed by a physical NVMe device.

### US-017-AC-4

The placement model can later extend from `pid -> local_store_id` to `pid -> node_id -> local_store_id` without changing the logical assignment row shape.

### US-017-AC-5

Queries run against a published SPIRE epoch or manifest so root metadata, hierarchy metadata, placement metadata, and partition objects are mutually compatible for the duration of the search.
