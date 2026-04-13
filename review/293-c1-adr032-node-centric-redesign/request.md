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

## Attempted First Cut: Fused Element-Local Neighbor/Score State

I tried the lightest plausible first cut before moving all the way to slot-based traversal:

- remove the separate scan-local neighbor cache
- remove the separate exact-score cache
- attach lazy neighbors and exact-score-once state directly to `CachedGraphElement`
- keep the rest of traversal keyed by tids

This was a legitimate step beyond packets `291` and `292`, but it still stopped short of the
actual ADR-032 target because frontier and result bookkeeping continued to carry tids rather
than stable scan-local node ids.

## Measurement

Canonical warm real-`50k`, `m=8`, `ef_search=40`, `warmup-passes=3`, `session-mode=per-cell`,
`timing-mode=cached-plan`:

- standing ADR-031 Tier 1 baseline: `mean ~= 1.507-1.510ms`
- first fused-node cut run 1: `p50=1.560ms`, `p99=2.525ms`, `mean=1.588ms`
- first fused-node cut run 2: `p50=1.564ms`, `p99=2.720ms`, `mean=1.602ms`

Validation for the attempted cut was green before benchmarking:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Conclusion

This first cut is another rejected weak variant.

It does co-locate more node-local state than packets `291` and `292`, but it still leaves the
scan algorithm operating on tids rather than stable node slots. That means it does not remove
enough repeated lookup/join work to justify the extra interior mutability and object shape.

The runtime code for this cut was discarded after the two confirmation runs above.

## Updated Next Slice

The next legitimate ADR-032 implementation must move past "fused cache entry" and into the
actual redesign seam:

1. introduce stable scan-local node slots
2. let frontier/result bookkeeping carry slot ids instead of tids
3. make source expansion and exact-score lifecycle operate through those slots
4. only then judge whether ADR-032 is reducing repeated node-state churn in a way that can beat
   the kept ADR-031 baseline

## Success Criteria

- ADR-032 is explicitly documented as a larger scan-runtime redesign, not a cleanup ADR
- the next code slice targets the fused node cache, not another isolated `Arc`/`Vec` swap
- the packet is kept live as the current draft while that larger slice is implemented and
  measured
