---
id: US-018
title: Operate SPIRE Across Local NVMe Stores
type: user-story
artifact_type: US
status: DRAFT
relationships:
  - target: "ix://agent-ix/tqvector/StR-005"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/tqvector/FR-039"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/tqvector/FR-041"
    type: "derives_into"
    cardinality: "1:N"
---
# US-018: Operate SPIRE Across Local NVMe Stores

**As** a database operator,
**I want** to configure SPIRE partition stores across local physical NVMe devices while PostgreSQL remains online,
**So that** I can scale read bandwidth and write capacity without turning SPIRE partitions into PostgreSQL table partitions.

## Acceptance Criteria

### US-018-AC-1

The operator can configure local SPIRE stores through PostgreSQL-managed SQL state, preferably tablespaces and extension-owned catalog/config tables or functions.

### US-018-AC-2

The operator can add, disable, drain, and inspect local stores live, subject to documented locking and epoch-publish boundaries.

### US-018-AC-3

SPIRE can place partition objects across bounded local stores by PID hash or an equivalent placement policy without creating one PostgreSQL relation per PID.

### US-018-AC-4

Queries group selected PIDs by local store and can execute store-local reads concurrently where PostgreSQL execution mechanics allow it.

### US-018-AC-5

Store failure behavior degrades gracefully by default when the index is configured for degraded recall: unavailable stores are skipped or downweighted with explicit diagnostics. Strict fail-closed behavior remains available as a consistency mode.

### US-018-AC-6

Replicated partition objects MAY be considered in a future phase for read throughput and availability, but local multi-store v1 assumes one primary placement per PID.
