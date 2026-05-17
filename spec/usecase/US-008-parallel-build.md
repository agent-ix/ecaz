---
id: US-008
title: Parallel Index Build
type: user-story
artifact_type: US
status: DRAFT
priority: P2-high
relationships:
  - target: "ix://agent-ix/ecaz/StR-004"
    type: "derives_from"
    cardinality: "N:1"
---
# US-008: Parallel Index Build

**As** a platform engineer loading large datasets,
**I want** `CREATE INDEX USING ec_hnsw` to parallelize heap scanning and tqvector validation across multiple workers,
**So that** index build time scales with available CPU cores rather than being single-threaded.

## Acceptance Criteria

### US-008-AC-1

`SET max_parallel_maintenance_workers = 4; CREATE INDEX ... USING ec_hnsw ...`
uses parallel workers for heap scanning.

### US-008-AC-2

Build completion time with 4 workers is no more than 60% of serial build time
on a 100K-row table.

### US-008-AC-3

The resulting index is identical in structure and recall to a serially-built
index on the same data.

### US-008-AC-4

`CREATE INDEX CONCURRENTLY ... USING ec_hnsw ...` works correctly with parallel
workers.
