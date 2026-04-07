---
id: US-010
title: Vacuum Removes Deleted Vectors
type: user-story
status: DRAFT
priority: P1-critical
traces:
  - StR-004
  - StR-001
---
# US-010: Vacuum Removes Deleted Vectors

**As** an application developer managing agent memories,
**I want** `DELETE` + `VACUUM` to remove dead vectors from the HNSW index so they are not returned in subsequent searches,
**So that** the index does not accumulate stale entries that waste I/O and degrade scan quality.

## Acceptance Criteria

1. After `DELETE FROM memories WHERE id = $x; VACUUM memories;`, a search query no longer returns the deleted row
2. The index page count does not grow unboundedly after repeated DELETE + INSERT + VACUUM cycles
3. Graph connectivity is maintained after vacuum — recall does not drop below 80% of pre-vacuum recall after deleting 10% of rows
4. VACUUM does not block concurrent INSERT or SELECT operations
