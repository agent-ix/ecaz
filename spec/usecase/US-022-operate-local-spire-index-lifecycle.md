---
id: US-022
title: Operate a Local SPIRE Index Lifecycle
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
  - target: "ix://agent-ix/ecaz/FR-052"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-053"
    type: "derives_into"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-054"
    type: "derives_into"
    cardinality: "1:N"
---
# US-022: Operate a Local SPIRE Index Lifecycle

**As** a platform engineer,
**I want** to build, query, update, vacuum, maintain, and inspect a local SPIRE index,
**So that** I can operate the partition-object foundation before and alongside distributed deployments.

## Acceptance Criteria

### US-022-AC-1

`ec_spire` builds PID-addressed partition objects with logical vector identity,
assignment rows, root/control metadata, and an active epoch.

### US-022-AC-2

Local searches route query vectors through SPIRE-owned root, top-graph, and
hierarchy metadata; PostgreSQL declarative partition pruning does not choose
SPIRE PIDs.

### US-022-AC-3

Local searches use the eager bounded scan contract: `amrescan` prepares a
ranked cursor and `amgettuple` drains it forward only.

### US-022-AC-4

Insert, delete, vacuum, split, merge, and cleanup work publish deltas or
replacement epochs rather than mutating published objects in place.

### US-022-AC-5

Operators can inspect local health, epoch, placement, leaf, delta, routing,
maintenance, cleanup, and cost state through SPIRE diagnostics.

### US-022-AC-6

Local lifecycle evidence records the fixture, store layout, active epoch,
object counts, route counts, cleanup state, and any deferred capacity claims.
