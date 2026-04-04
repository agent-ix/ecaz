---
id: US-005
title: VACUUM Cleans Up Deleted Vectors
type: user-story
status: APPROVED
traces:
  - StR-001
---
# US-005: VACUUM Cleans Up Deleted Vectors

**As** a platform engineer,
**I want** VACUUM to properly clean up deleted vectors from the HNSW index,
**So that** the index does not grow unboundedly and deleted rows do not appear in search results.

## Acceptance Criteria

1. After DELETE + VACUUM, the deleted vector no longer appears in search results
2. The HNSW graph is repaired — neighbors of deleted nodes get replacement connections
3. No crash or corruption occurs if VACUUM runs concurrently with INSERT or SELECT
