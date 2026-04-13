---
id: ADR-032
title: "Arena-Based Scan Element Cache"
status: PROPOSED
impact: Affects NFR-001, ADR-031
date: 2026-04-12
---
# ADR-032: Arena-Based Scan Element Cache

## Context

### Per-element heap allocation on every cache miss

The current scan-local graph element cache (`HashMap<ItemPointer, Arc<CachedGraphElement>>`)
allocates on every cache miss:

1. `Arc::new(CachedGraphElement)` — one heap allocation for the Arc wrapper
2. `Vec<ItemPointer>` for heaptids — up to 10 × 6 bytes
3. `Vec<u64>` for binary words (ADR-031 sidecar) — up to 24 × 8 bytes
4. HashMap entry bookkeeping (hashbrown's per-entry overhead)

Each `Arc::clone`/`drop` during beam search also touches the atomic reference count, adding
cache-line contention on the allocator side.

At `m=8`, `ef_search=40`, a single query traverses ~300-400 elements. On the warm path most
are cache hits, but on cold startup or when the working set exceeds cache capacity, every miss
pays the full allocation chain.

### pgvector's approach

pgvector uses a flat `visited` bitset for deduplication and stack-allocated neighbor arrays
(`ItemPointerData indextids[HNSW_MAX_M * 2]`). There are zero per-element heap allocations
during graph traversal. Scores are computed from pinned buffer pages with no intermediate
copies.

### Current state after ADR-031

ADR-031 (packets 278-285) shifted the dominant warm cost from scoring to element
load/cache overhead. The lazy-scoring architecture means exact scoring is deferred, but the
per-element allocation chain on cache miss remains unchanged. Further warm latency
improvement requires reducing allocation overhead, not scoring cost.

## Proposal

Replace the per-element `Arc<CachedGraphElement>` cache with a scan-lifetime arena
allocator. Elements are bump-allocated into a contiguous memory region and referenced by
index rather than Arc pointer.

### Design shape

```
ScanArena {
    elements: Vec<ArenaElement>,       // flat, pre-sized
    element_index: HashMap<ItemPointer, u32>,  // TID -> arena slot
    scores: HashMap<ItemPointer, f32>, // score cache (unchanged)
}

struct ArenaElement {
    tid: ItemPointer,
    level: u8,
    deleted: bool,
    heaptid_count: u8,
    heaptids: [ItemPointer; 10],       // inline, no Vec
    neighbortid: ItemPointer,
    binary_words: [u64; 24],           // inline, no Vec (ADR-031)
}
```

Key properties:

1. **Zero per-element heap allocation.** Elements are written into pre-allocated arena slots.
   No `Arc`, no `Vec`, no per-element `Box`.

2. **Inline fixed-size fields.** Heaptids (10 × 6B = 60B) and binary words (24 × 8B = 192B)
   are stored inline on the arena element. These are bounded by compile-time constants
   (`HEAPTID_INLINE_CAPACITY`, binary word count derived from dimensions).

3. **Pre-sized arena.** The arena can be sized to `ef_search * expected_expansion_factor` at
   scan start. If it fills, it grows like a `Vec` but without per-element allocation.

4. **No reference counting.** Elements are referenced by `u32` arena index. No atomic
   refcount operations. The arena is dropped in bulk at scan end.

5. **Cache-friendly layout.** Contiguous arena storage means sequential element access
   benefits from hardware prefetch, unlike chasing `Arc` pointers scattered across the heap.

### Neighbor cache

The neighbor cache (`HashMap<ItemPointer, Arc<GraphNeighbors>>`) has the same pattern:
`Arc::new(GraphNeighbors)` with an internal `Vec<ItemPointer>`. This can use a parallel
arena with a fixed-size neighbor array (`[ItemPointer; MAX_M * 2]`).

### Migration path

1. Introduce `ArenaElement` with inline arrays as an internal type in `scan.rs`
2. Replace `Arc<CachedGraphElement>` with arena index in the element cache
3. Replace `Arc<GraphNeighbors>` with arena index in the neighbor cache
4. Update all beam-search and successor functions to use arena references
5. Benchmark warm and cold paths on real 50K corpus

This is a scan-internal refactor — it does not change page layout, tuple encoding, or the
graph read API. External consumers (build, vacuum, insert) continue using the existing
`GraphElement` / `GraphNeighbors` types.

## Expected Impact

### Allocation reduction

Per cache miss, eliminates:
- 1 × `Arc` allocation (~32-48 bytes + allocator overhead)
- 1 × `Vec<ItemPointer>` (60 bytes + allocator overhead)
- 1 × `Vec<u64>` (192 bytes + allocator overhead)
- Atomic refcount operations on every `Arc::clone`/`drop`

### Latency estimate

The allocation chain is not the dominant cost at ~4.7ms (50K warm), but it is the next
addressable seam after scoring cost was reduced by ADR-031. Expected improvement: 5-15% on
warm cache-miss-heavy workloads, potentially more on cold startup where most elements are
misses.

### Interaction with ADR-031 persisted sidecar

With persisted binary sidecars (packet 285), the arena element can read binary words
directly from the page without an intermediate `Vec`. The inline `[u64; 24]` is populated
by copying from the borrowed `TqElementTupleRef` sidecar bytes — a single `memcpy` into
the pre-allocated slot.

## Risks

1. **Fixed-size arrays waste memory for small elements.** An element with 1 heaptid and no
   binary words still occupies the full `ArenaElement` size (~320 bytes). At `ef_search=40`
   with ~400 elements, the arena would be ~128KB — well within L2 cache.

2. **Arena index lifetime.** Arena indices are only valid for the scan lifetime. Misuse
   (stashing an arena index across scans) would be a correctness bug. This is the same
   lifetime constraint as the current raw-pointer scan opaque state.

3. **Larger refactor surface.** Every function that currently takes `Arc<CachedGraphElement>`
   must change to take an arena reference or index. This is a broad scan-internal change.

## Decision Criteria

- Warm 50K latency improvement measurable on verified harness
- Cold startup latency improvement (first-query-after-restart) measurable
- No recall regression on the real corpus summary harness
- Clean cargo test / pgrx test / clippy gates

## Status

PROPOSED and now under active exploration on the dedicated `adr032-arena-scan-cache`
branch.

Tier 1 is complete and kept. Tier 2 was implemented and measured in packet `290`, then
discarded after it regressed the canonical warm real-`50k` ADR-031 seam. That leaves the
arena refactor as the next legitimate path for pushing beyond the current kept Tier 1
surface.

## References

- Packet 278: Introduced `CachedGraphElement` without code payload
- Packet 281: ADR-031 cached binary prefilter, showed allocation chain is the remaining cost
- Packet 285: Persisted binary sidecar, inline arena read path
- pgvector `hnswutils.c`: Zero-allocation graph traversal reference
