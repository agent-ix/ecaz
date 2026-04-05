---
id: ADR-008
title: "Bootstrap non-empty tqhnsw scans with forward linear page iteration"
status: DECIDED
impact: HIGH for FR-009, FR-010
date: 2026-04-04
---
# ADR-008: Bootstrap non-empty tqhnsw scans with forward linear page iteration

## Context

The access method already had:

- scan descriptor allocation and teardown
- `amrescan` validation for a single non-NULL `real[]` ORDER BY query
- scan-owned storage for the copied query payload
- `amgettuple` lifecycle gating
- a safe empty-index fast path

What it still lacked was any non-empty tuple production.

The full intended search path is graph-based HNSW traversal with query-dependent scoring and
ordered candidate management. Implementing that in one step would cross several boundaries at
once:

- graph navigation
- candidate/result heap state
- distance ordering
- tuple visibility and recheck semantics
- planner-facing execution behavior

That is too large for the current staged implementation approach.

## Decision

The first non-empty `amgettuple` implementation SHALL use a temporary forward-only linear scan of
data pages.

Specifically:

- `amrescan` resets a scan-local page/offset cursor.
- `amgettuple` supports only forward scan direction in this stage.
- Each call walks data pages from the saved cursor, decodes element tuples, skips deleted or
  heap-TID-empty tuples, and returns the first heap TID from the next matching element tuple.
- When no more tuples remain, the scan is marked exhausted and later calls return `false`.

This is an execution scaffold, not the final HNSW search algorithm.

## Consequences

### Benefits

- Non-empty scans now produce real heap TIDs through the AM callback surface.
- The implementation stays narrow and testable.
- Cursor ownership stays inside scan opaque state, which matches the existing query-payload
  lifetime boundary.

### Tradeoffs

- Results are not distance-ordered.
- Neighbor links and entry-point metadata are ignored.
- Duplicate vectors currently return only the first stored heap TID per element tuple in this
  bootstrap path.
- Backward scan direction remains unsupported.

## Follow-Up

Later scan slices should replace this temporary linear walk with staged HNSW execution:

1. define candidate/result state in scan opaque
2. score candidates against the stored raw query payload
3. traverse the graph from the persisted entry point
4. return ordered tuples with explicit recheck/visibility semantics
5. revisit planner enablement only after the execution contract is credible
