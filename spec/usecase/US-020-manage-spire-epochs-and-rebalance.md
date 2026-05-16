---
id: US-020
title: Manage SPIRE Epochs, Updates, and Rebalancing
type: user-story
artifact_type: US
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-005"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-052"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-054"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-060"
    type: "derives_into"
    cardinality: "1:N"
---
# US-020: Manage SPIRE Epochs, Updates, and Rebalancing

**As** a database operator,
**I want** SPIRE to publish, retain, and retire index epochs while handling inserts, deletes, splits, merges, and placement changes,
**So that** queries can run against coherent index state while the system evolves.

## Acceptance Criteria

### US-020-AC-1

SPIRE publishes a new epoch only after required root/control metadata, hierarchy metadata, placement entries, store descriptors, object manifests, and partition-object bytes validate.

### US-020-AC-2

Old epochs remain readable for a configured minimum retention window and can be cleaned after no active query requires them.

### US-020-AC-3

Inserts and deletes are represented by delta objects or replacement partition-object versions before compaction.

### US-020-AC-4

Split, merge, rebalance, and vacuum compaction publish replacement placement and hierarchy metadata through epoch transitions rather than changing state under active queries.

### US-020-AC-5

Operators can inspect active epochs, retired epochs, failed epochs, stale stores/remotes, maintenance plans, and cleanup eligibility through SQL diagnostics.

### US-020-AC-6

A failed or partial epoch publish does not poison the active epoch and is recoverable through diagnostics, retry, or cleanup.
