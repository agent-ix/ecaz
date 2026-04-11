---
id: ADR-026
title: "Live Insert Backlink Lock Ordering"
status: ACCEPTED
impact: Affects FR-016, ADR-019
date: 2026-04-11
---
# ADR-026: Live Insert Backlink Lock Ordering

## Context

A5 extends `tqhnsw` live insert from disconnected append to graph mutation. Once insert starts
rewriting existing neighbor tuples, a single insert can touch multiple data pages in addition to
the metadata page. Without an explicit lock protocol, concurrent inserts could deadlock by taking
page locks in different orders or by mixing metadata and data-page write locks.

This checkpoint keeps the write surface intentionally narrow:

- the new node gets forward links before it is visible as an entry-point promotion
- backlinks are written only into selected layer-0 neighbors
- backlinks are added only when a free layer-0 slot already exists
- overflow pruning and upper-layer backlink mutation are deferred

Even with that narrow scope, the lock order needs to be durable because the same protocol will be
reused by later overflow and upper-layer work.

## Decision

Live insert follows this write order:

1. Traverse with read-only access first.
   Candidate discovery, greedy descent, and layer-0 selection run before any existing neighbor
   tuple is rewritten.
2. Append the new node under one data-page `EXCLUSIVE` buffer lock.
   The append path may allocate a fresh page, but it holds only that single data-page write lock.
3. Release the append page lock before any backlink work starts.
4. Collect backlink target neighbor tuples and sort them by physical `(block_number, offset_number)`.
5. Rewrite existing neighbor tuples one data page at a time in ascending block order.
   When multiple target tuples live on the same page, they are updated under one buffer
   `EXCLUSIVE` lock and one GenericXLog transaction for that page.
6. Within a page, update neighbor tuples in ascending offset order.
7. Acquire the metadata-page `EXCLUSIVE` lock only after all data-page writes are complete.
   Metadata promotion/repair never overlaps a data-page `EXCLUSIVE` lock.

In short: data pages first, one page at a time, in ascending physical order; metadata last.

## Consequences

### Positive

- Concurrent live inserts share a deterministic data-page lock order.
- The current backlink slice can group multiple tuple rewrites on one page without widening the
  deadlock surface.
- Later overflow-pruning work can reuse the same page-ordering rule.

### Negative

- Backlink targets must be materialized and sorted before the write phase starts.
- The current checkpoint skips layer-0 backlinks when the target slice is already full; that keeps
  the lock surface narrow but leaves pruning work for a later slice.

### Neutral

- First-insert initialization remains on the existing metadata-lock path because it does not yet
  touch multiple data pages.
- This ADR does not change tuple layout or WAL format; it only constrains lock acquisition order.
