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
