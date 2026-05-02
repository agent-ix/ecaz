---
id: US-020
title: Manage SPIRE Epochs, Updates, and Rebalancing
type: user-story
artifact_type: US
status: DRAFT
relationships:
  - target: "ix://agent-ix/tqvector/StR-005"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/tqvector/FR-041"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/tqvector/FR-043"
    type: "derives_into"
    cardinality: "1:N"
---
# US-020: Manage SPIRE Epochs, Updates, and Rebalancing

**As** a database operator,
**I want** SPIRE to publish, retain, and retire index epochs while handling inserts, deletes, splits, merges, and placement changes,
**So that** queries can run against coherent index state while the system evolves.

## Acceptance Criteria

### US-020-AC-1

SPIRE can publish a new epoch only after the required root metadata, hierarchy metadata, placement metadata, and partition objects are present.

### US-020-AC-2

Old epochs remain readable for a configured minimum retention window and can be cleaned after no active query requires them.

### US-020-AC-3

Inserts and deletes can be represented by either live deltas or replacement partition-object versions before compaction.

### US-020-AC-4

Split, merge, and rebalance operations publish new placement/hierarchy metadata through an epoch transition rather than silently changing the state under active queries.

### US-020-AC-5

Operators can inspect active epoch, retained epochs, pending epoch publication, stale nodes/stores, and cleanup eligibility through SQL diagnostics.
