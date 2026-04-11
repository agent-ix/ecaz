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

The final A5 insert path now uses that same lock protocol across:

- forward-link append on the new node
- layer-0 and upper-layer backlink mutation
- full-slice pruning for overflowed targets
- bounded stale-snapshot replanning for concurrent full-slice drift

The lock order still needs to stay explicit because retrying a stale full-slice plan must not
reintroduce deadlock risk by mixing page orders or by replanning while holding a data-page
`EXCLUSIVE` lock.

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
7. If a full-slice rewrite sees a stale live layer under the page lock, do not replan while that
   page is locked.
   Record the target for retry, finish the current ordered page pass, then re-enter read-only
   planning for those targets before another ordered write pass begins.
8. Acquire the metadata-page `EXCLUSIVE` lock only after all data-page writes are complete.
   Metadata promotion/repair never overlaps a data-page `EXCLUSIVE` lock.

In short: data pages first, one page at a time, in ascending physical order; metadata last.

## Consequences

### Positive

- Concurrent live inserts share a deterministic data-page lock order.
- Backlink mutation can group multiple tuple rewrites on one page without widening the deadlock
  surface.
- Full-slice retry can happen without holding a data-page `EXCLUSIVE` lock across replan reads.

### Negative

- Backlink targets must be materialized and sorted before the write phase starts.
- A stale full-slice rewrite may need another read-only planning pass before it can be retried.

### Neutral

- First-insert initialization remains on the existing metadata-lock path because it does not yet
  touch multiple data pages.
- This ADR does not change tuple layout or WAL format; it only constrains lock acquisition order.
