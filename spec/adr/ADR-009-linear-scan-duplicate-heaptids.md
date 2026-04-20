---
id: ADR-009
title: "Emit all duplicate heap TIDs during linear scan bootstrap"
status: DECIDED
impact: HIGH for FR-010
date: 2026-04-05
---
# ADR-009: Emit all duplicate heap TIDs during linear scan bootstrap

## Context

ADR-008 introduced a temporary forward linear scan for non-empty `ec_hnsw` indexes. That first
slice returned only the first heap TID from each live element tuple.

That shortcut is not just unordered relative to the eventual HNSW search. It also drops visible
tuples whenever multiple heap rows coalesce into one element tuple for the same encoded vector.

The build path and `aminsert` path already preserve those duplicate heap TIDs inline in the
element tuple. The bootstrap scan must not silently suppress them.

## Decision

The temporary linear scan SHALL emit every heap TID stored in each live element tuple before it
advances to the next element tuple.

Specifically:

- scan opaque state keeps an in-memory cursor over the currently decoded element tuple's heap TIDs
- `amgettuple` drains that pending heap-TID list across repeated calls before reading another
  index tuple
- `amrescan` resets both the page/offset cursor and any pending duplicate heap-TID progress

This remains a linear execution scaffold. It still ignores graph links and does not provide
distance-ordered results.

## Consequences

### Benefits

- The bootstrap scan no longer omits heap rows hidden behind duplicate-coalesced element tuples.
- Duplicate handling stays inside scan-local memory; no page-layout change is required.
- Repeated rescans restore deterministic iteration from the start of the temporary linear walk.

### Tradeoffs

- Results remain unordered relative to query distance.
- Scan opaque state now carries a small fixed-capacity heap-TID buffer tied to the current inline
  duplicate representation.

## Follow-Up

Later graph-search slices should replace this pending duplicate buffer with ordered candidate/result
state while preserving the guarantee that every visible heap TID behind a returned element can be
produced.
