---
id: US-008
title: Parallel Index Build
type: user-story
status: DRAFT
priority: P2-high
traces:
  - StR-004
---
# US-008: Parallel Index Build

**As** a platform engineer loading large datasets,
**I want** `CREATE INDEX USING tqhnsw` to parallelize heap scanning and tqvector validation across multiple workers,
**So that** index build time scales with available CPU cores rather than being single-threaded.

## Acceptance Criteria

1. `SET max_parallel_maintenance_workers = 4; CREATE INDEX ... USING tqhnsw ...` uses parallel workers for heap scanning
2. Build completion time with 4 workers is ≤ 60% of serial build time on a 100K-row table
3. The resulting index is identical in structure and recall to a serially-built index on the same data
4. `CREATE INDEX CONCURRENTLY ... USING tqhnsw ...` works correctly with parallel workers
