# Review Request: C1 ADR-032 Element-Cache Arena Slice 1

## Context

ADR-031 Tier 1 remains the best kept warm path on the real `50k` seam:

- `m=8`
- `ef_search=40`
- `p50 ~= 1.48ms`
- `p99 ~= 2.4ms`
- `mean ~= 1.51ms`

Packet `290` tried the next obvious copy-elision seam by pinning candidate buffers across
the approximate pass and exact-scoring from borrowed bytes. That variant validated but
regressed the canonical warm surface and was discarded.

ADR-032 is the next lane because the remaining scan-local graph element cache still pays a
heap/object-management cost on miss:

- `HashMap<ItemPointer, Arc<CachedGraphElement>>`
- `Arc::clone` / `drop` traffic during traversal
- per-entry heap allocation for the `Arc` wrapper itself

Tier 1 already moved the payloads inline, so the next contained step is to remove the
`Arc` ownership model before attempting the broader neighbor-cache half of the ADR.

## Problem

The current element cache keeps `CachedGraphElement` in heap-allocated `Arc`s. That means
every cache miss still allocates an object and every traversal hop still touches atomic
refcounts, even though the element lifetime is already naturally bounded to a single scan.

We do not need the full arena rewrite all at once to test that thesis. The first credible
slice is:

1. keep `CachedGraphElement` layout unchanged
2. move ownership into a scan-lifetime arena/vector
3. replace `Arc<CachedGraphElement>` flow with stable arena indices or borrowed refs
4. leave the neighbor cache alone for now

## Planned Slice

Implement an element-only arena for scan-local graph element caching in `src/am/scan.rs`.

Likely shape:

- `Vec<CachedGraphElement>` as the arena backing store
- `HashMap<ItemPointer, u32>` from TID to arena slot
- helper accessors that hand out `&CachedGraphElement` by slot

Non-goals for this slice:

- no neighbor-cache arena yet
- no page-layout changes
- no new scoring algorithm
- no persisted-format change

## Success Criteria

- the element cache no longer stores `Arc<CachedGraphElement>`
- scan traversal behavior stays unchanged
- `cargo test`, `cargo pgrx test pg17`, and clippy are green
- the packet records whether the canonical warm real-`50k` seam improves enough to keep
  the slice

## Implementation

Implemented the element-only arena in `src/am/scan.rs`:

- replaced `HashMap<ItemPointer, Arc<CachedGraphElement>>` with
  `GraphElementArena { elements: Vec<CachedGraphElement>, index_by_tid: HashMap<_, u32> }`
- threaded arena slot ids through the element-cache lookup/materialization helpers
- left the neighbor cache unchanged on `Arc<GraphNeighbors>` for this slice

No page-layout or scoring changes were included.

## Validation

Green from the arena-build candidate state:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Benchmark Result

Canonical warm real-`50k` seam, release build:

- command:
  `scripts/bench_sql_latency_verified_scratch.sh --prefix tqhnsw_real_50k --m 8 --ef-search 40 --cache-state warm-after-prime3 --warmup-passes 3 --session-mode per-cell --timing-mode cached-plan`
- standing kept ADR-031 Tier 1 baseline: `p50 ~= 1.48ms`, `p99 ~= 2.39-2.42ms`, `mean ~= 1.507-1.510ms`
- run 1: `p50=1.506ms`, `p95=2.113ms`, `p99=2.545ms`, `mean=1.544ms`
- run 2: `p50=1.501ms`, `p95=2.103ms`, `p99=2.496ms`, `mean=1.541ms`

## Outcome

Discarded.

The element-only arena validated cleanly but came back slightly slower than the standing
ADR-031 Tier 1 baseline on two back-to-back canonical warm runs. The regression is small,
but it is consistent enough that this slice does not earn a keep on its own.

Next ADR-032 work should target a larger ownership/copy boundary than the element-only
`Arc` removal, or shift to a different arena seam entirely.
