# Review Request: NaN Gamma Validation

Scope:
- `src/am/build.rs` — `build_heap_tuple`
- `src/am/insert.rs` — `tqhnsw_aminsert`

## Problem

Neither the build nor insert path validates that `gamma` is a finite float. If a `tqvector` datum
contains `NaN` as its gamma value:

- **Duplicate detection**: `gamma.to_bits()` comparison works (NaN.to_bits() == NaN.to_bits()), so
  NaN duplicates coalesce correctly.
- **Build graph construction**: `score_code_inner_product` returns NaN, which propagates through
  hnsw_rs distance calculations. NaN comparisons in the priority queue produce undefined ordering.
- **Search scoring**: NaN gamma produces NaN scores, making the node unreachable by ordered
  traversal (NaN fails all comparisons) but still occupying graph slots.

## Impact

Affects **graph traversal** if NaN-gamma nodes are present. A NaN-gamma node at a high HNSW level
could break greedy descent (graph.rs:328 — `next.score >= current.score` is false when either is
NaN, so descent would stop prematurely at the NaN node).

This is an edge case — gamma comes from the quantizer and should always be finite. But defensive
validation is cheap.

## Suggested Fix

Add a finite check in `build_heap_tuple` and the `tqhnsw_aminsert` path:

```rust
if !gamma.is_finite() {
    pgrx::error!("tqhnsw does not support non-finite gamma values");
}
```

Alternatively, validate at the tqvector type level (during encode/pack) so it's caught earlier.
