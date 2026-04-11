---
id: ADR-027
title: "Vacuum Graph Repair Lock Ordering"
status: ACCEPTED
impact: Affects FR-010, FR-022, ADR-019
date: 2026-04-11
---
# ADR-027: Vacuum Graph Repair Lock Ordering

## Context

A6 extends `tqhnsw` vacuum from heap-TID stripping and tombstone finalization into graph repair.
Once pass 2 starts rewriting persisted neighbor tuples, one vacuum run may mutate many data pages
that are unrelated to the deleted element's home page.

The first pass-2 checkpoint only unlinked dead-element TIDs from neighbor tuples. The next slices
now add layer-aware replacement fill, but they still preserve the same deadlock-safety boundary by
planning candidates read-only and only filling currently free slots during the ordered page write.

## Decision

Vacuum graph repair follows this write order:

1. Build the delete-set from pass-1 dead-element discovery first.
2. Scan data pages in ascending block order under `BUFFER_LOCK_SHARE`.
3. If a page needs repair, release the share lock, reopen that same page alone, and rewrite it
   under one `BUFFER_LOCK_EXCLUSIVE`.
   That one page-local write window may update multiple logical layer slices of the same
   persisted neighbor tuple; the ordering rule is per physical page, not per logical layer.
4. Hold at most one data-page `EXCLUSIVE` lock at a time.
5. Do not hold a metadata-page `EXCLUSIVE` lock during pass 2.
6. If future replacement search needs replanning, do that read-only work outside any data-page
   `EXCLUSIVE` lock before the next ordered page rewrite.
7. The current replacement slice may only top up `INVALID` slots during the write phase; it does
   not evict live neighbors while holding the page write lock.

In short: ordered page scan, one data-page write lock at a time, no metadata overlap.

## Consequences

### Positive

- Pass-2 unlink repair can touch arbitrary neighbor pages without introducing mixed lock order.
- Layer-aware replacement fill now has a stable concurrency boundary to build on.
- The current share-then-exclusive rewrite shape matches the existing narrow vacuum pass-1 pattern.

### Negative

- Repair may rescan a page under `EXCLUSIVE` before rewriting it.
- A full-index unlink pass is more I/O than a narrowly targeted adjacency-only rewrite.

### Neutral

- This ADR constrains lock acquisition order only. It does not change tuple layout or WAL format.
- Finalization stays a separate page-local pass after graph repair.
