# Review Request: C1 Scan Cache Code Payload Elision

## Context

The smaller zero-copy and allocation probes are mostly exhausted:

- packet `262` kept a narrow graph tuple copy-boundary fix for a small real win
- packets `263`, `276`, and `277` were all valid follow-up probes and were
  discarded

The remaining structural seam is larger than another small-`Vec` tweak. Warm
C1 still reloads graph elements into a scan-local cache that stores full owned
encoded `code` payloads, even though steady-state ordered scan mostly needs:

- element liveness / heap-tid presence
- neighbor-link metadata
- the element score, which is already cached separately

That means each graph-element cache miss still copies the full compressed code
payload into `GraphElement.code`, even when the code bytes are only needed once
to compute the first score.

## Problem

`cached_graph_element(...)` in `src/am/scan.rs` currently loads
`graph::GraphElement`, which owns:

- `heaptids: Vec<ItemPointer>`
- `code: Vec<u8>`

The code payload is materially larger than the heap-tid list on the real
`1536`-dim, `4-bit` corpus. After the first score is computed, steady-state
scan-local traversal does not need to keep that payload around in the graph
cache. Keeping it means extra per-element allocation/copy work on every graph
cache miss.

## Implementation

Draft.

Target the scan-local graph cache only:

1. add a borrowed element-tuple decode view so scan can score directly from the
   pinned page slice
2. change the scan-local graph-element cache to store only the metadata it
   needs after that first score
3. populate the score cache from the borrowed decode path on cache miss, so the
   encoded code bytes never need to live in the cached scan-local element

This keeps the change local to scan steady-state behavior without rewriting the
broader `graph::GraphElement` surface used by build / vacuum / debug code.

## Exit Criteria

- the packet records whether removing scan-local cached code payload ownership
  improves the verified warm real-corpus `10K`, `m=8`, `ef_search=40`,
  `warm-after-prime3`, `per-cell`, `cached-plan` seam
- the packet records whether the code checkpoint was kept or discarded
