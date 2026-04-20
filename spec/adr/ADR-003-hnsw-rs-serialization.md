---
id: ADR-003
title: "Walk hnsw_rs graph and write to Postgres pages"
status: SUPERSEDED
superseded_by: ADR-042
impact: HIGH for FR-007 (page layout), FR-008 (build)
date: 2026-04-03
---
# ADR-003: Walk hnsw_rs graph and write to Postgres pages

## Context

`hnsw_rs` serializes via `file_dump()` to flat files. PostgreSQL needs 8KB buffer-managed pages.

## Investigation Results

### Graph topology extraction is supported

The `flatten` module provides exactly what we need:

```rust
pub struct FlatPoint {
    origin_id: DataId,           // our external ID
    p_id: PointId,               // (layer: u8, index: i32)
    neighbours: Vec<Neighbour>,  // sorted by distance
}
```

Additionally, `Point::get_neighborhood_id()` returns `Vec<Vec<Neighbour>>` — per-layer neighbor lists. Combined with `PointIndexation::get_layer_iterator()`, we can walk the entire graph.

### Extraction algorithm

```
for layer in 0..max_layer:
    for point in hnsw.get_point_indexation().get_layer_iterator(layer):
        let origin_id = point.get_origin_id()
        let neighbors_per_layer = point.get_neighborhood_id()
        // → write TqElementTuple (code bytes from heap scan cache)
        // → write TqNeighborTuple (neighbor TIDs from neighbors_per_layer)
```

## Decision

**Option A confirmed**: Walk `hnsw_rs` internals after build, write to Postgres pages.

### Implementation plan

1. During heap scan: build a `HashMap<origin_id, (heap_tid, tqvector_bytes)>`
2. Insert into `hnsw_rs::Hnsw` using f32 vectors (available from heap) with origin_id
3. After build completes: iterate all points via `get_point_indexation()`
4. For each point: look up heap_tid and tqvector_bytes from the HashMap
5. Allocate TqElementTuple + TqNeighborTuple on Postgres pages via GenericXLog
6. Record TID mappings (origin_id → index page TID) for cross-referencing neighbor pointers
7. Second pass: fix up neighbor TID pointers (convert origin_ids to actual page TIDs)

The two-pass approach is necessary because we don't know a point's index page TID until we've written it, but neighbor tuples reference TIDs.
