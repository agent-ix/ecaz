# Review Request: A5 Upper-Layer Insert Links

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/06-graph-insert.md`
- `review/199-aminsert-graph-aware-insertion-roadmap/request.md`
- `plan/status.md`
- `spec/adr/ADR-026-live-insert-backlink-lock-ordering.md`

This is the next narrow A5 checkpoint after layer-0 backlinks. It extends the
same insert-time graph mutation path into upper layers without mixing in
overflow pruning yet.

Checkpoint scope:

1. discover insert-time candidates above layer 0
2. persist simple upper-layer forward links on the new node
3. apply matching upper-layer backlinks when target slices still have free
   capacity
4. keep the existing page-order lock protocol unchanged

## Scope

- `src/am/insert.rs`
- `src/lib.rs`
- `plan/tasks/06-graph-insert.md`

## What Landed

### 1. Insert-side neighbor discovery is now layer-aware above layer 0

`discover_insert_forward_neighbor_slots(...)` now returns both:

- the flattened neighbor-slot payload for the new node
- the selected `(layer, element_tid)` pairs that should receive backlinks

For upper layers, the new helper path:

- starts from the current entry candidate
- runs `graph::search_layer_result_candidates(...)` from
  `metadata.max_level` down to layer `1`
- writes simple top-`M` forward candidates into each upper-layer slice that the
  new node actually participates in

Layer 0 still uses the existing narrow path:

- `greedy_descend_from_entry(...)`
- `search_layer0_result_candidates(...)`

So this checkpoint extends the previous traversal seam rather than replacing it.

### 2. Backlink mutation now carries layer identity through the write phase

The prior checkpoint only knew “selected layer-0 neighbor tuple tids”.

This slice keeps explicit layer information through the write path:

- `LayerForwardSelection { layer, element_tid }`
- `BacklinkMutation { neighbor_tid, layer }`

That allows one existing neighbor tuple to be rewritten once and updated in
multiple logical layers if the same target was selected more than once.

### 3. Upper-layer backlinks reuse the same narrow free-capacity contract

This checkpoint still does **not** prune or shrink existing neighbor lists.

For every selected layer:

- if the new node is already present, the update is a no-op
- if the target slice contains an `INVALID` slot, the new node is inserted
- if the target slice is full, the update is skipped

This is the same boundary as the layer-0 backlink checkpoint, now generalized
to upper layers.

### 4. Lock ordering is unchanged

No new ADR was needed because this slice reuses ADR-026 exactly:

1. traverse first
2. append under one data-page `EXCLUSIVE` lock
3. release that append-page lock
4. sort backlink targets by physical neighbor-tuple tid
5. rewrite one data page at a time in ascending order
6. metadata last

The main extension is that the on-page rewrite now knows which logical layer
within each neighbor tuple should be updated.

### 5. A5 task tracking now marks upper-layer search / links as landed

`plan/tasks/06-graph-insert.md` now records:

- `75%` complete for upper-layer insert search and upper-layer backlink coverage
- overflow pruning still pending
- drift accounting and concurrency hardening still pending

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed on this checkpoint.

## New / Updated Coverage

- `test_tqhnsw_insert_populates_forward_links_from_live_entry_seed`
- `test_tqhnsw_insert_populates_forward_links_against_built_graph`
- `test_tqhnsw_insert_populates_upper_layer_links_when_available`

The new upper-layer regression uses a sparse live-insert fixture instead of a
built graph so it tests this checkpoint’s actual contract:

- an upper-level live insert gets a populated layer-1 forward slice once the
  graph already has an upper layer
- upper-layer forward targets participate in layer 1
- at least one of those sparse upper-layer targets receives the matching
  layer-1 backlink

The existing layer-0 tests were also tightened so they assert only the second
half of the layer-0 forward window stays invalid, instead of assuming all
upper-layer slots stay empty forever.

## Review Focus

- Is carrying explicit `(layer, element_tid)` selections through the insert path
  the right seam before overflow pruning lands?
- Is it acceptable that upper-layer backlink coverage is still “free capacity
  only”, matching the current layer-0 contract?
- Should the next slice focus directly on overflow pruning / neighbor shrinking,
  or is there another narrower correction needed before that?
