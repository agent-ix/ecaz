# Review Request: C1 ADR-032 Node-Centric Redesign Reframe

## Context

ADR-031 is already very promising on the current persisted format, so ADR-032 is being
reframed as the larger runtime redesign lane that tries to compound that win without taking
on ADR-030's index-v2 encoding/layout work yet.

Packets `291` and `292` are now explicitly treated as rejected weak variants:

- `291`: element-cache arena only
- `292`: neighbor-cache arena only

Both validated and both regressed the canonical warm real-`50k` seam. That means ADR-032 is
not "replace one cache object at a time." It needs to be a broader scan-architecture change.

## Reframed Goal

Make the graph-first scan algorithmically cheaper by turning scan-local hot state into one
node-centric lifecycle instead of three independent tid-keyed caches.

Current split state:

- element cache keyed by `element_tid`
- neighbor cache keyed by `neighbortid`
- exact score cache keyed by `element_tid`

Target state:

- one fused `CachedGraphNode` slot keyed by `element_tid`
- lazy adjacency fill on that same slot
- exact-score lifecycle attached to that same slot
- frontier/result bookkeeping carried as slot ids, not repeated tids

## Planned First Real Slice

The next ADR-032 implementation slice should:

1. introduce `ScanNodeArena`
2. introduce `CachedGraphNode { element, neighbors: Option<...>, exact_score_state, ... }`
3. remove the separate scan-local neighbor cache
4. switch source expansion to one node-slot lookup plus lazy adjacency fill
5. preserve current semantics before attempting deeper frontier/result slot conversion

## Success Criteria

- ADR-032 is explicitly documented as a larger scan-runtime redesign, not a cleanup ADR
- the next code slice targets the fused node cache, not another isolated `Arc`/`Vec` swap
- the packet is kept live as the current draft while that larger slice is implemented and
  measured
