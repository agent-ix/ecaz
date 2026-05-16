---
id: US-019
title: Query Distributed SPIRE Across Postgres Instances
type: user-story
artifact_type: US
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-005"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-055"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-056"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-057"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-058"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/NFR-014"
    type: "derives_into"
    cardinality: "1:N"
---
# US-019: Query Distributed SPIRE Across Postgres Instances

**As** a platform engineer,
**I want** a coordinator PostgreSQL instance to execute vector-ordered reads across remote PostgreSQL shard nodes through `EcSpireDistributedScan`,
**So that** distributed SPIRE can keep row storage near each shard while returning one ordered tuple stream to the client.

## Acceptance Criteria

### US-019-AC-1

Remote storage nodes are PostgreSQL instances with the `ecaz` extension installed, shard-local heap rows, and local SPIRE indexes.

### US-019-AC-2

The coordinator uses CustomScan execution to load routing metadata, select PIDs, group them by remote node, and dispatch typed remote search requests through the production executor.

### US-019-AC-3

Remote nodes score local partition objects, resolve origin-node heap visibility, and return validated candidate envelopes plus typed tuple payloads.

### US-019-AC-4

The coordinator merges local and remote candidates by stable vector identity, deduplicates boundary replicas, and returns virtual tuple payloads through CustomScan rather than coordinator heap mirror rows.

### US-019-AC-5

Strict distributed search fails closed for stale, unavailable, overloaded, identity-incompatible, or typed-transport-incompatible remote work. Degraded search may skip remote work only when configured and explicitly reported.

### US-019-AC-6

Distributed read isolation is documented as v1 read-committed remote-statement behavior rather than coordinator-snapshot repeatability across shards.
