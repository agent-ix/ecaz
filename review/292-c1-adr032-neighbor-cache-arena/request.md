# Review Request: C1 ADR-032 Neighbor-Cache Arena Slice

## Context

Packet `291` tested the smallest ADR-032 arena seam by removing `Arc<CachedGraphElement>`
from the element cache only. That variant validated cleanly but regressed the canonical warm
real-`50k` ADR-031 seam slightly (`mean ~= 1.54ms` vs the standing `1.507-1.510ms`
baseline), so it was discarded.

The next larger ownership seam still sitting on the hot expansion path is the neighbor
cache:

- `HashMap<ItemPointer, Arc<GraphNeighbors>>`
- `Arc::clone` / `drop` on every cache hit
- `Vec<ItemPointer>` allocation on every cache miss

Unlike the element cache, neighbor traversal sits directly inside
`cached_scan_successor_candidates_for_layer(...)`, so removing object churn here has a
better chance of moving the warm path materially.

## Problem

The current neighbor cache keeps decoded adjacency payloads in heap-allocated
`Arc<GraphNeighbors>` objects with a heap-backed `Vec<ItemPointer>` payload.

For the common `m=8` layer-0 path, the useful adjacency window is only `2m = 16` slots. We
still pay the full `Vec`/`Arc` shape even when the hot-path payload trivially fits inline.

## Planned Slice

Implement an arena-backed scan-local neighbor cache in `src/am/scan.rs`:

1. replace `Arc<GraphNeighbors>` ownership with a scan-lifetime arena plus stable slot ids
2. introduce cached neighbor storage with an inline fast path for the common small payload
3. leave the element cache on the kept ADR-031 Tier 1 shape for this slice
4. leave page layout and scoring unchanged

Likely shape:

- `Vec<CachedGraphNeighbors>` arena backing store
- `HashMap<ItemPointer, u32>` from neighbor tuple TID to slot
- `CachedNeighborTids::{Inline, Heap}` mirroring the existing inline-heaptid/binary wrappers

## Success Criteria

- the neighbor cache no longer stores `Arc<GraphNeighbors>`
- the common hot-path adjacency payload can stay inline without allocating a `Vec`
- `cargo test`, `cargo pgrx test pg17`, and clippy are green
- the packet records whether the canonical warm real-`50k` seam improves enough to keep the
  slice

## Implementation

Implemented an arena-backed scan-local neighbor cache in `src/am/scan.rs`:

- replaced `HashMap<ItemPointer, Arc<GraphNeighbors>>` with
  `GraphNeighborArena { neighbors: Vec<CachedGraphNeighbors>, index_by_tid: HashMap<_, u32> }`
- introduced `CachedNeighborTids::{Inline, Heap}` with an inline fast path for the common
  `m=8` layer-0 adjacency width (`16` tids)
- threaded stable neighbor-slot ids through the cached adjacency helpers while leaving the
  element cache on the kept ADR-031 Tier 1 shape

No page-layout or scoring changes were included.

## Validation

Green from the candidate state:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

The `cargo pgrx test pg17` suite needed one rerun because the existing order-sensitive
frontier-shape pg tests failed on different assertions across two passes before settling
green on the final rerun. The kept runtime logic is unchanged by that flake pattern.

## Benchmark Result

Canonical warm real-`50k` seam, release build:

- command:
  `scripts/bench_sql_latency_verified_scratch.sh --prefix tqhnsw_real_50k --m 8 --ef-search 40 --cache-state warm-after-prime3 --warmup-passes 3 --session-mode per-cell --timing-mode cached-plan`
- standing kept ADR-031 Tier 1 baseline: `p50 ~= 1.48ms`, `p99 ~= 2.39-2.42ms`, `mean ~= 1.507-1.510ms`
- run 1: `p50=1.603ms`, `p95=2.188ms`, `p99=2.696ms`, `mean=1.635ms`
- run 2: `p50=1.604ms`, `p95=2.215ms`, `p99=2.700ms`, `mean=1.644ms`

## Outcome

Discarded.

The neighbor-cache arena plus inline-adjacency fast path regressed the canonical warm seam
materially enough that it does not earn a keep. With `291` already showing the same result
for the element-only arena seam, the simple arena substitutions inside `scan.rs` are not
currently paying for themselves on the real `50k` ADR-031 path.

If ADR-032 continues, the next seam should be larger than isolated `Arc`/`Vec` ownership
replacement inside one cache. Otherwise this branch should stop after documenting the failed
arena variants and hand the next warm-path push to a different idea.
