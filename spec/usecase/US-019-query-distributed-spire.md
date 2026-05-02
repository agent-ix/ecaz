---
id: US-019
title: Query Distributed SPIRE Across Postgres Instances
type: user-story
artifact_type: US
status: DRAFT
relationships:
  - target: "ix://agent-ix/tqvector/StR-005"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/tqvector/FR-042"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/tqvector/FR-041"
    type: "derives_into"
    cardinality: "1:N"
---
# US-019: Query Distributed SPIRE Across Postgres Instances

**As** a platform engineer,
**I want** a coordinator PostgreSQL instance to query SPIRE partition objects on remote PostgreSQL instances through libpq,
**So that** distributed SPIRE can preserve high write throughput and near-data scoring while returning one merged result stream to the client.

## Acceptance Criteria

### US-019-AC-1

Remote storage nodes are PostgreSQL instances with the SPIRE extension installed and local heap rows or row locators for their owned vector data.

### US-019-AC-2

The coordinator routes selected PIDs to remote nodes through the placement map and uses libpq, preferably pipeline mode, for remote search calls.

### US-019-AC-3

Remote nodes score local partition objects near their heap rows and return compact candidate rows to the coordinator.

### US-019-AC-4

The coordinator merges remote and local candidates by stable `vec_id`, deduplicates boundary replicas, resolves final row delivery, and returns a single ordered result stream.

### US-019-AC-5

Distributed search degrades gracefully by default when configured for degraded recall: unreachable or stale nodes are skipped or downweighted with explicit diagnostics. Strict epoch-matching fail-closed behavior remains available as a consistency mode.

### US-019-AC-6

Replicated partition objects MAY be considered in a future phase to increase read throughput and availability, but distributed v1 assumes one primary node placement per PID.
