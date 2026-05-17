---
id: US-010
title: Vacuum Removes Deleted Vectors
type: user-story
artifact_type: US
status: DRAFT
priority: P1-critical
relationships:
  - target: "ix://agent-ix/ecaz/StR-004"
    type: "derives_from"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/StR-001"
    type: "derives_from"
    cardinality: "N:1"
---
# US-010: Vacuum Removes Deleted Vectors

**As** an application developer managing agent memories,
**I want** `DELETE` + `VACUUM` to remove dead vectors from the HNSW index so they are not returned in subsequent searches,
**So that** the index does not accumulate stale entries that waste I/O and degrade scan quality.

## Acceptance Criteria

### US-010-AC-1

After `DELETE FROM memories WHERE id = $x; VACUUM memories;`, a search query
no longer returns the deleted row.

### US-010-AC-2

The index page count does not grow unboundedly after repeated DELETE + INSERT +
VACUUM cycles.

### US-010-AC-3

Graph connectivity is maintained after vacuum: recall does not drop below 80%
of pre-vacuum recall after deleting 10% of rows.

### US-010-AC-4

VACUUM does not block concurrent INSERT or SELECT operations.
