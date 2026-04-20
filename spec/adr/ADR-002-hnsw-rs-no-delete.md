---
id: ADR-002
title: "hnsw_rs has no delete — own the graph in Postgres pages"
status: SUPERSEDED
superseded_by: ADR-042
impact: HIGH for FR-010 (HNSW vacuum)
date: 2026-04-03
---
# ADR-002: hnsw_rs has no delete — own the graph in Postgres pages

## Context

The `hnsw_rs` crate exposes insert and search but **no delete method**.

## Investigation Results

### Graph topology is accessible

- `Point::get_neighborhood_id() -> Vec<Vec<Neighbour>>` — returns per-layer neighbor lists
- `Hnsw::get_point_indexation() -> &PointIndexation` — access to all points
- `PointIndexation::get_layer_iterator(layer) -> IterPointLayer` — iterate points per layer
- `Point::get_point_id() -> PointId` — (layer: u8, index: i32)
- `Point::get_origin_id() -> usize` — the external ID we assigned

The `flatten` module also provides `FlatPoint` which is a reduced version with just IDs and neighbors — exactly what we need for page serialization.

### hnsw_rs is useful for build, not runtime

`hnsw_rs` manages its own memory (Arcs, RwLocks per point). This is incompatible with Postgres buffer-managed pages. We cannot keep an `hnsw_rs::Hnsw` instance alive across transactions.

## Decision

**Option A: Use hnsw_rs for bulk build only, own the graph in Postgres pages at runtime.**

### Build path (ambuild)
1. Scan heap, collect all tqvector codes
2. Build `hnsw_rs::Hnsw` in memory with our custom `Distance` impl
3. Walk the completed graph: extract each point's layer assignment and per-layer neighbor lists
4. Write to Postgres pages as TqElementTuple + TqNeighborTuple
5. Drop the `hnsw_rs::Hnsw` struct

### Runtime operations (insert, scan, vacuum)
Operate directly on Postgres pages:
- **Insert**: read neighbor tuples from pages, find neighbors via beam search over pages, write new tuples
- **Scan**: beam search over pages using `tqvector_inner_product`
- **Vacuum**: three-pass algorithm directly on page tuples (same as pgvector)

This matches how pgvector works — it implements its own HNSW entirely in C with no library dependency for runtime graph ops. We use `hnsw_rs` to avoid reimplementing the construction algorithm, but own everything at the storage layer.

### Distance impl for hnsw_rs build

```rust
struct TqDistance {
    quantizer: TurboQuantizer,
}

impl Distance<u8> for TqDistance {
    fn eval(&self, va: &[u8], vb: &[u8]) -> f32 {
        // Deserialize TurboCode from bytes, decode one side, score asymmetrically
        // Only used during bulk build — not latency-sensitive
    }
}
```

Or more practically, store raw f32 vectors during build (since we're scanning the heap which has them) and use a standard distance function, then extract the graph topology and write compressed codes to pages.
