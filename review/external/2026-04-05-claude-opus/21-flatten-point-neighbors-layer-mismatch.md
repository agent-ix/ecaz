# Review: flatten_point_neighbors Layer Boundary

**File:** `src/am/mod.rs:1932-1951`
**Severity:** Low (correctness concern)
**Category:** Correctness

## Finding

```rust
fn flatten_point_neighbors(
    origin_id: usize,
    level: u8,
    neighbors_per_layer: &[Vec<hnsw_rs::hnsw::Neighbour>],
) -> Vec<usize> {
    let mut seen = HashSet::new();
    let mut flattened = Vec::new();

    for layer in 0..=usize::from(level) {
        if let Some(layer_neighbors) = neighbors_per_layer.get(layer) {
            for neighbor in layer_neighbors {
                if neighbor.d_id != origin_id && seen.insert(neighbor.d_id) {
                    flattened.push(neighbor.d_id);
                }
            }
        }
    }

    flattened
}
```

This flattens all neighbors from all layers into a single list for the neighbor tuple. The HNSW paper specifies that each layer has a max degree of `M` (layers > 0) or `2*M` (layer 0). By flattening, the stored neighbor count can exceed `2*M`.

The `TqNeighborTuple` stores `count` as `u16` (line 244) but the allocated `neighbor_slots(level, m)` function accounts for this: `2*m + level*m` slots.

**This is correct** -- the page layout allocates enough slots for all flattened neighbors. The concern is whether the flattened representation loses the layer structure, which matters for HNSW search (graph traversal starts at the top layer and descends).

Since the current scan implementation is a linear scan (not graph traversal), the flattened representation is fine. When graph-based scan is implemented, the neighbor tuple format may need to be restructured to separate neighbors by layer.

## Recommendation

No change needed for the current linear scan. When implementing graph traversal (FR-009), the neighbor tuple format should be revisited to support per-layer neighbor access. Consider adding a layer-delimiter or layer-offset array to the neighbor tuple encoding.

## Action Required

None for current stage. Flag for FR-009 implementation.
