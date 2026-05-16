---
id: US-018
title: Operate SPIRE Across Local NVMe Stores
type: user-story
artifact_type: US
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-005"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-048"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-053"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-060"
    type: "derives_into"
    cardinality: "1:N"
---
# US-018: Operate SPIRE Across Local NVMe Stores

**As** a database operator,
**I want** to configure SPIRE partition stores across local physical NVMe devices while PostgreSQL remains online,
**So that** I can scale read bandwidth and write capacity without turning SPIRE partitions into PostgreSQL table partitions.

## Acceptance Criteria

### US-018-AC-1

The operator can configure a bounded local-store surface with PostgreSQL-managed store relations and optional tablespace placement.

### US-018-AC-2

The operator can inspect store relation IDs, store states, object counts, object bytes, selected PID placement, and per-store scan counters.

### US-018-AC-3

SPIRE can place partition objects across bounded local stores by deterministic PID hash without creating one PostgreSQL relation per PID.

### US-018-AC-4

Queries group selected PIDs by `(node_id, local_store_id)` and prefetch selected store relation blocks before sequential decode and scoring.

### US-018-AC-5

Strict scans fail closed for stale or unavailable required placements. Degraded mode may skip unavailable placements only when explicitly configured and with diagnostics.

### US-018-AC-6

Local multi-store evidence distinguishes PostgreSQL read-ahead/prefetch from true parallel multi-NVMe execution; true parallel store execution requires a later ADR and benchmark packet.
