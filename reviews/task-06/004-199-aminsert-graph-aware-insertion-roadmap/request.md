# Review Request: aminsert Graph-Aware Insertion Roadmap (Post-A4)

## Summary

This review documents the full gap between the current `aminsert` and a graph-aware HNSW insertion, based on comparison with four reference implementations (hnswlib-rs, instant-distance, swarc, hnsw). This is not actionable until A4 passes — it's a roadmap for A5 (FR-016).

## Current aminsert behavior (insert.rs:9-73)

1. Decode quantized vector from datum
2. Check for duplicate by linear scan (gamma + code match)
3. Create neighbor tuple: `TqNeighborTuple { count: 0, tids: Vec::new() }`
4. Create element tuple: `level: 0`, `deleted: false`, empty neighbor reference
5. If first element: set `metadata.entry_point = element_tid`
6. No graph connections. Node is completely disconnected.

## What a graph-aware insert requires (from reference implementations)

### Step 1: Random level assignment

All four references assign a level from an exponential distribution:
```
level = floor(-ln(uniform_random()) * m_L)
```
where `m_L = 1 / ln(M)`. This determines which layers the new node participates in.

**tqvector gap**: Always inserts at level 0. No random level generation.

### Step 2: Greedy descent through upper layers

Starting from the entry point at its max level, descend to `level + 1`:
```
for layer in (level+1..=entry_point_level).rev() {
    search_layer(query, entry_point, ef=1, layer)  // find nearest in this layer
    entry_point = nearest
}
```

**tqvector status**: `greedy_descend_from_entry` (graph.rs:133-155) already implements this. Can be reused directly.

### Step 3: Search for neighbors at each insertion layer

From `level` down to 0:
```
for layer in (0..=level).rev() {
    candidates = search_layer(query, entry_point, ef_construction, layer)
    neighbors = select_neighbors(candidates, M_layer)
    ...
}
```
where `M_layer = 2*M` for layer 0 and `M` for upper layers.

**tqvector status**: `search_layer0_result_candidates` exists for layer 0. Upper-layer search would need a new `search_layer_result_candidates` that works with the layer-specific neighbor slots (using `load_neighbor_tids_for_layer` which already exists).

### Step 4: Neighbor selection heuristic

Two strategies from the HNSW paper:
1. **Simple**: keep M closest candidates by distance
2. **Heuristic** (Algorithm 4): prune candidates that are closer to already-selected neighbors than to the query. This improves graph diversity and long-range connectivity.

hnswlib-rs implements the heuristic with `keep_pruned` and `extend_candidates` flags. instant-distance has `select_heuristic` with the same options.

**tqvector gap**: No neighbor selection logic exists. The build path gets this from hnsw-rs, but aminsert has nothing.

### Step 5: Bidirectional edge creation

After selecting neighbors for the new node, each neighbor's neighbor list must also be updated to include the new node:
```
for neighbor in selected_neighbors:
    neighbor.connections[layer].push(new_node)
    if neighbor.connections[layer].len() > M_max:
        shrink(neighbor.connections[layer])  // keep only M_max best
```

This requires:
- Reading the neighbor's element tuple
- Loading its neighbor tuple
- Adding the new node's ItemPointer to the appropriate layer slice
- Possibly shrinking (removing the worst connection) if the list exceeds M (or 2*M for layer 0)
- Writing the updated neighbor tuple back with WAL logging

**tqvector gap**: No bidirectional updates. This is the most complex part because it requires in-place mutation of neighbor tuples on existing pages, which must be WAL-logged and handle concurrent access.

### Step 6: Entry point update

If the new node's level exceeds the current max level:
```
if level > metadata.max_level:
    metadata.entry_point = new_node
    metadata.max_level = level
```

**tqvector gap**: Entry point is only set for the very first element. Never updated afterward.

### Step 7: Neighbor tuple resizing

Current neighbor tuples are created with `count: 0, tids: Vec::new()`. For graph-aware insert, the neighbor tuple must be pre-allocated with `neighbor_slots(level, m)` slots, since the level determines how many layers (and thus how many neighbor slots) the node needs.

If a node is inserted at level 3 with m=8, it needs `2*8 + 3*8 = 40` neighbor slots (240 bytes of ItemPointers). This must be allocated upfront or handled via tuple replacement.

**tqvector status**: `page::neighbor_slots(level, m)` already computes the correct count. Build uses it. aminsert would need to use it with the randomly-assigned level.

## Implementation order suggestion

1. **Random level assignment** — simple, no graph reads needed
2. **Pre-sized neighbor tuples** — allocate correct slot count at insert time
3. **Entry point update** — metadata page write under lock (pattern exists in `with_locked_metadata_page`)
4. **Greedy descent** — reuse `greedy_descend_from_entry`
5. **Layer-0 neighbor search + simple selection** — reuse beam search, add simple top-M selection
6. **Forward edge creation** — write new node's neighbor slots
7. **Bidirectional edge creation** — update existing neighbor tuples (hardest part)
8. **Neighbor list shrinking** — prune oversized lists after bidirectional update
9. **Heuristic neighbor selection** — replace simple selection for better graph quality

## Dependency on A4

Steps 4-8 reuse the same page-level traversal and scoring code that search uses. If search scoring is broken (the current A4 failure), neighbor selection during insert would inherit the same bug — the insert would connect to the wrong neighbors. This is why A5 is blocked on A4.

## Files to read

- `src/am/insert.rs` — current aminsert implementation
- `src/am/graph.rs:133-155` — greedy descent (reusable)
- `src/am/graph.rs:157-207` — layer-0 beam search (reusable)
- `src/am/shared.rs:86-130` — `with_locked_metadata_page` (pattern for metadata update)
- `src/am/page.rs` — `neighbor_slots`, `TqNeighborTuple` encoding
- Reference: `hnswlib-rs/src/hnsw.rs:1068-1277` — full insert + reverse_update_neighborhood

## Review focus

- Whether the suggested implementation order is correct given the existing code surface
- Whether bidirectional edge updates can be done in-place on existing pages or require tuple replacement
- Whether the simple neighbor selection (step 5) is sufficient for initial A5 or if heuristic selection is needed from the start
