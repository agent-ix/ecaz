---
id: ADR-032
title: "Node-Centric Approximate-First Scan State"
status: PROPOSED
impact: Affects NFR-001, ADR-031
date: 2026-04-13
---
# ADR-032: Node-Centric Approximate-First Scan State

## Context

### ADR-031 proved the remaining lever is algorithmic

ADR-031 delivered the major warm-path win because it reduced expensive exact work:

- cached binary state
- lazy exact scoring
- real `50k` warm `m=8`, `ef_search=40` down to about `p50=1.48ms`, `mean=1.51ms`

That result changed the optimization target. The hot path is no longer asking for tiny
container cleanups first. It is asking for a scan architecture that does less repeated node
work and makes approximate-first traversal cheaper to sustain.

### The weak ADR-032 interpretation was tested and rejected

Packets `291` and `292` explored the narrow interpretation of ADR-032:

- packet `291`: element-cache arena only
- packet `292`: neighbor-cache arena plus inline adjacency fast path

Both validated and both regressed the canonical warm real-`50k` seam. That does **not**
invalidate a larger ADR-032 redesign. It only invalidates the weak version:

- replace one `Arc`/`Vec` ownership boundary at a time
- keep the current split cache layout
- keep the current tid-based lookup pattern
- hope local allocation savings matter on their own

They do not.

### Current scan state is still split across repeated tid lookups

The graph-first path still spreads one logical node's state across multiple scan-local
structures:

- element cache keyed by `element_tid`
- neighbor cache keyed by `neighbortid`
- exact score cache keyed by `element_tid`
- frontier and result bookkeeping carried as tids rather than stable scan-local node ids

That means the hot path keeps rediscovering and rejoining state that belongs together:

1. load or look up the source element
2. load or look up its adjacency separately
3. consult a separate score cache
4. carry tids through frontier logic, then look the node up again later

If ADR-032 is going to matter, it must attack that repeated node-state churn directly.

## Decision

ADR-032 is reframed as a **node-centric, approximate-first scan redesign**, not as an
arena-cleanup ADR.

The goal is to make the scan algorithmically cheaper by co-locating all hot query-local
state for a graph node in one scan-lifetime slot and letting traversal operate on those
slots directly.

## Proposal

Introduce a fused scan-local node cache keyed by `element_tid`.

### Design shape

```text
ScanNodeArena {
    nodes: Vec<CachedGraphNode>,
    index_by_tid: HashMap<ItemPointer, u32>,
}

CachedGraphNode {
    tid: ItemPointer,
    level: u8,
    deleted: bool,
    heaptids: [ItemPointer; 10] + count,
    neighbortid: ItemPointer,
    binary_words: [u64; 24] or wider fallback,
    neighbors: Option<CachedNeighbors>,
    exact_score: NodeExactScoreState,
    traversal: NodeTraversalState,
}

CachedNeighbors {
    tids: inline-or-heap adjacency payload,
    count: usize,
}

NodeExactScoreState =
    Unscored
  | ApproxOnly
  | Exact(f32)

Frontier / result structures:
    store node-slot ids rather than element tids
```

### Core properties

1. **One lookup per node.** Element metadata, binary state, adjacency, and exact-score
   lifecycle live behind one slot id.

2. **Lazy adjacency fill.** A node slot can exist before adjacency is decoded. Source
   expansion populates `neighbors` on first need, then reuses it from the same slot.

3. **Exact-score-once semantics.** Once a node graduates from approximate-only to exact, the
   score is attached to the node slot itself instead of living in a side cache keyed by tid.

4. **Slot-based traversal.** Frontier candidates, visible seeds, binary-prefilter survivors,
   and result materialization can all operate on scan-local slot ids instead of repeated
   tid re-lookups.

5. **Approximate-first traversal support.** Because approximate state and exact-score
   lifecycle live together, the scan can postpone exact work until a node becomes
   competitive, rather than bouncing between caches to recover the same state.

## Why this is different from packets 291 and 292

Packets `291` and `292` only changed ownership shape locally.

This ADR instead changes the **unit of scan state** from:

- separate element object
- separate adjacency object
- separate exact-score cache entry

to:

- one node slot with one lifecycle

That is the minimum redesign that has a plausible path to being algorithmically cheaper
rather than cosmetically cheaper.

## Non-Goals

- no new persisted index format
- no new quantizer or scoring codec
- no change to on-disk tuple layout in this ADR
- no claim that arena allocation alone is the win

ADR-030 remains the larger index-v2 encoding/layout project. ADR-032 is a scan-runtime
redesign for the current persisted format.

That sequencing is intentional:

- ADR-031 is already very promising on the current format
- ADR-032 is the larger scan-runtime redesign that tries to push that promising path further
- ADR-030 is still the bigger persisted encoding/layout redesign and should remain separate
  from this runtime lane

## Initial implementation direction

The first legitimate ADR-032 code slice should:

1. introduce `CachedGraphNode` plus `ScanNodeArena`
2. key the scan-local cache only by `element_tid`
3. move adjacency onto the same node slot as lazy state
4. preserve current scan semantics while converting frontier expansion to node slots

Only after that seam exists should follow-on slices decide whether to:

- collapse the separate exact-score cache fully into node slots
- carry more traversal state per node
- let broader approximate-first ranking operate on slots directly

## Expected Impact

### Primary

- fewer repeated joins between element state, adjacency state, and score state
- fewer redundant tid-based cache lookups on the hot path
- cleaner exact-score-once lifecycle
- better substrate for stronger ADR-031 follow-on filtering

### Secondary

- less `Arc`/`Vec` churn as a side effect
- better locality from co-locating per-node state

The secondary effects are welcome, but they are not the reason for the ADR.

## Risks

1. **Large refactor surface.** This touches scan-local caches, frontier bookkeeping, and
   result materialization together.

2. **Slot lifetime correctness.** Slot ids are scan-lifetime objects. Any leakage across
   rescans would be a correctness bug.

3. **May still not be enough.** If the dominant remaining cost is exact-score count rather
   than node-state churn, ADR-032 may only become valuable when paired with a stronger
   ADR-031 filtering seam.

## Decision Criteria

- canonical warm real-`50k` seam improves beyond the kept ADR-031 Tier 1 baseline
- exact-score pressure or repeated node-state churn is measurably reduced
- no recall regression on the real-corpus summary harness
- clean `cargo test`, `cargo pgrx test pg17`, and clippy gates on kept slices

## Status

PROPOSED and reframed on the dedicated `adr032-arena-scan-cache` branch.

Packets `291` and `292` are now treated as **rejected weak variants** of the ADR, not as the
ADR itself. They showed that local arena/cache substitutions alone do not move the kept
ADR-031 warm path in the right direction.

The first actual slot-based frontier/scheduler cut also did not beat the kept ADR-031 path.
That result matters: it means "replace tids with slots" is not, by itself, enough to justify
ADR-032.

The next viable ADR-032 question is therefore narrower and more demanding:

- can a slot-centric scan state reduce exact-score pressure, source-expansion churn, or other
  expensive work per traversal step
- not merely whether the same work can be routed through a cleaner node-centric bookkeeping
  shape

The next legitimate ADR-032 slice is a fused node-centric scan cache, not another isolated
`Arc`/`Vec` ownership swap.

ADR-032 is being run down before ADR-030 specifically because ADR-031 already has a strong
signal on the current persisted format, so the next rational question is whether a larger
scan-runtime redesign can compound that win before taking on the bigger index-v2 format work.

## References

- packet `281`: ADR-031 cached binary prefilter runtime win
- packet `287`: ADR-031 Tier 1 inline scan cache keep
- packet `291`: rejected element-cache-only arena variant
- packet `292`: rejected neighbor-cache-only arena variant
- `pgvector` `hnswutils.c`: reference point for tight scan-local node handling
