---
id: US-006
title: Async I/O Accelerated Scan
type: user-story
status: DRAFT
priority: P1-critical
traces:
  - StR-004
---
# US-006: Async I/O Accelerated Scan

**As** a platform engineer running tqvector on PostgreSQL 18 with network-attached storage,
**I want** HNSW index scans to use async I/O prefetch for both graph traversal and linear scan phases,
**So that** cold-cache query latency is reduced by prefetching pages the scan will need before it blocks on them.

## Acceptance Criteria

1. On PG18 with `io_method=worker` or `io_method=io_uring`, HNSW scan uses the `read_stream` API — not single-page `ReadBufferExtended` calls
2. During bootstrap graph traversal, neighbor element pages are prefetched via a graph-mode `ReadStream` before being scored
3. During linear scan fallback, consecutive index pages are prefetched via a sequential-mode `ReadStream`
4. On PG17, the scan falls back to the existing synchronous `ReadBufferExtended` path with no behavior change
5. Cold-cache query latency at `effective_io_concurrency=16` is measurably lower than at `effective_io_concurrency=0`
