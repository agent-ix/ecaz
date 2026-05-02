---
id: US-017
title: Operate a Local SPIRE Index Lifecycle
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
  - target: "ix://agent-ix/tqvector/FR-040"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/tqvector/FR-041"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/tqvector/FR-043"
    type: "derives_into"
    cardinality: "1:N"
---
# US-017: Operate a Local SPIRE Index Lifecycle

**As** a platform engineer,
**I want** to build, query, update, vacuum, reindex, and inspect a local SPIRE index,
**So that** I can prove the partition-object foundation before adding local multi-NVMe and multi-instance placement.

## Acceptance Criteria

### US-017-AC-1

`ec_spire` can build a single-level IVF-compatible foundation using PID-addressed partition objects and logical `(vec_id, pid)` assignment rows.

### US-017-AC-2

SPIRE partition selection happens inside the SPIRE index from the query vector, root graph or centroid router, hierarchy metadata, and centroids; it does not rely on PostgreSQL declarative table partition pruning.

### US-017-AC-3

Local build publishes an initial epoch that records compatible root metadata, hierarchy metadata, placement metadata, and partition objects.

### US-017-AC-4

Local insert/delete/update paths either mutate a live delta layer or publish a replacement epoch, according to the configured consistency mode.

### US-017-AC-5

Vacuum and cleanup can retire dead rows, compact partition objects, and remove old epochs according to a configured retention policy.
