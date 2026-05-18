# Review Request: A4 Neighbor Slot Packing Audit (hnsw-rs → Page Format)

## Summary

The build path extracts per-layer neighbor lists from hnsw-rs and packs them into a flattened `TqNeighborTuple`. The search path unpacks them using `layer_slot_bounds`. If these two sides disagree on layout — layer ordering, slot boundaries, or neighbor-within-layer ordering — the graph becomes nonsensical at read time even though it was correct at write time.

## The write side (build.rs)

`pack_point_neighbor_slots` (`build.rs:798-820`):
```rust
fn pack_point_neighbor_slots(origin_id, level, m, neighbors_per_layer) -> Vec<Option<usize>> {
    let mut slots = vec![None; neighbor_slots(level, m as u16)];
    // Layer 0: slots [0, 2*m)
    fill_point_neighbor_layer_slots(&mut slots, origin_id, 0, 0, m * 2, neighbors_per_layer);
    // Layer L: slots [2*m + (L-1)*m, 2*m + L*m)
    for layer in 1..=level {
        let start = m * 2 + ((layer - 1) * m);
        fill_point_neighbor_layer_slots(&mut slots, origin_id, layer, start, m, neighbors_per_layer);
    }
    slots
}
```

`fill_point_neighbor_layer_slots` (`build.rs:822-851`) iterates `neighbors_per_layer.get(layer)` and fills slots sequentially, skipping self-references.

## The read side (graph.rs)

`layer_slot_bounds` (`graph.rs:289-301`):
```rust
fn layer_slot_bounds(element_level, m, layer) -> Option<(usize, usize)> {
    if layer > element_level { return None; }
    if layer == 0 { return Some((0, m * 2)); }
    let start = m * 2 + (usize::from(layer) - 1) * m;
    Some((start, start + m))
}
```

## Potential mismatch vectors

### 1. hnsw-rs layer indexing vs tqvector layer indexing

hnsw-rs uses `point.get_point_id().0` as the level and `point.get_neighborhood_id()` as a `Vec<Vec<Neighbour>>` indexed by layer. **Critical question**: does hnsw-rs index layer 0 as `neighbors_per_layer[0]`? Or does it use a different convention (e.g., highest layer first)?

If hnsw-rs returns layers in reverse order, `fill_point_neighbor_layer_slots` would pack layer N neighbors into the layer 0 slots and vice versa. The graph would be structurally valid (correct slot counts) but semantically wrong (wrong neighbors in wrong layers).

### 2. Neighbor ordering within a layer

hnsw-rs returns neighbors sorted by distance (closest first). `fill_point_neighbor_layer_slots` preserves this order. The search side treats all neighbors in a layer equally (no ordering assumption). This should be fine, but worth confirming.

### 3. Level capping

Build caps level: `let level = point.get_point_id().0.min(max_level_cap)` (`build.rs:650`). If hnsw-rs assigned a level higher than `max_level_cap`, the node's actual connections at those upper layers get silently dropped. The node still has valid layer-0 connections, but its upper-layer neighbors (which hnsw-rs optimized for long-range traversal) are lost.

## Suggested investigation

1. **Dump a small graph**: Build a 20-element index with `m=4`. For each element, print:
   - hnsw-rs: `get_point_id().0` (level) and `get_neighborhood_id()` (neighbors per layer)
   - tqvector page: decoded `TqNeighborTuple.tids` with `layer_slot_bounds` applied
   - Verify they match exactly.

2. **Layer index probe**: Check hnsw-rs source for `get_neighborhood_id()` return convention. Specifically whether index 0 is the base layer or the node's insertion layer.

3. **Level cap impact**: Check how many nodes in the 10K recall corpus have levels capped by `max_level_that_fits`. If significant, the graph's upper-layer connectivity is degraded.

## Files to read

- `src/am/build.rs:596-662` — `build_hnsw_graph` (graph construction)
- `src/am/build.rs:798-851` — `pack_point_neighbor_slots`, `fill_point_neighbor_layer_slots`
- `src/am/graph.rs:270-301` — `valid_neighbor_tids_for_layer`, `layer_slot_bounds`
- `src/am/page.rs` — `neighbor_slots`, `TqNeighborTuple` encoding
- hnsw-rs source: `get_neighborhood_id()` and `get_point_id()` semantics

## Review focus

- Whether the write-side layer indexing matches the read-side layer indexing
- Whether the hnsw-rs `get_neighborhood_id()` Vec is indexed from layer 0 upward
- Whether level capping silently discards important connectivity
