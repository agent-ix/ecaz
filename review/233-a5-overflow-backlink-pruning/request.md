# Review Request: A5 Overflow Backlink Pruning

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/06-graph-insert.md`
- `review/199-aminsert-graph-aware-insertion-roadmap/request.md`
- `spec/adr/ADR-026-live-insert-backlink-lock-ordering.md`

This is the next A5 checkpoint after upper-layer insert links. It closes the
remaining graph-mutation gap for single-session live insert behavior by letting
selected neighbors admit the new node even when the target layer slice is
already full.

Checkpoint scope:

1. plan overflow-aware backlink mutation from the existing insert selections
2. prune full target slices with simple score-ordered top-`M` / top-`2M`
   retention
3. keep ADR-026 lock ordering unchanged
4. add regression coverage for a real full-slice layer-0 rewrite event

## Scope

- `src/am/insert.rs`
- `src/lib.rs`
- `plan/tasks/06-graph-insert.md`

## What Landed

### 1. Backlink planning now distinguishes free-slot inserts from full-slice rewrites

The live insert path still starts from the existing `(layer, element_tid)`
forward selections, but it now plans one of two concrete actions per selected
target layer:

- `InsertIfFree`
- `RewriteFullSlice { expected_slice, replacement_slice }`

Free-capacity behavior is unchanged. The new behavior is that a full selected
target slice can now be rewritten instead of being skipped outright.

### 2. Full-slice pruning uses the current simple score contract

For a full target layer slice, planning now scores:

- every existing neighbor in that layer slice
- the newly inserted node

against the target element’s persisted code using the same raw code scorer
already used by insert/search. The planner keeps the best:

- `2M` tids at layer 0
- `M` tids above layer 0

Tie-breaking prefers existing neighbors before the new node, then physical TID
order, so equal-score cases do not churn the graph unnecessarily.

This is intentionally the simple selection contract from the roadmap, not the
full heuristic/Algorithm-4 selector.

### 3. Page-order lock protocol is unchanged, with one new guard

ADR-026 still governs the write phase:

1. traversal and planning first
2. append the new node
3. sort selected neighbor tuples by physical order
4. rewrite one data page at a time
5. metadata last

The new guard is specific to full-slice rewrites:

- planning records the expected pre-write layer slice
- under the page `EXCLUSIVE` lock, the rewrite only applies if the live slice
  still matches that expected snapshot
- if the layer drifted meanwhile, the rewrite is skipped instead of overwriting
  a concurrently changed full slice

If the live layer now has a free slot or already contains the new node, the
write path falls back to the existing narrow behavior.

So this checkpoint improves single-session correctness without claiming final
concurrent insert hardening yet.

### 4. Coverage now includes a real full layer-0 rewrite event

The new pg regression:

- builds a dense graph
- performs bounded live inserts until one insert actually targets a previously
  full layer-0 slice
- asserts that the admitted full target:
  - contains the new element after insert
  - stays at full `2M` capacity
  - differs from its pre-insert slice, proving an eviction happened

Existing insert tests for level sizing, promotion, forward links, upper-layer
links, reachability, duplicates, and empty-index behavior all remain in place
and passed unchanged.

### 5. A5 tracking now marks overflow handling as landed

`plan/tasks/06-graph-insert.md` now records:

- `90%` complete for neighbor overflow handling / shrinking
- concurrency retry/hardening still pending in the `100%` milestone
- drift accounting still pending

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed on this checkpoint.

## New / Updated Coverage

- `test_tqhnsw_insert_rewrites_full_layer0_backlink_slice`
- `test_tqhnsw_insert_populates_forward_links_from_live_entry_seed`
- `test_tqhnsw_insert_populates_forward_links_against_built_graph`
- `test_tqhnsw_insert_populates_upper_layer_links_when_available`
- `test_tqhnsw_live_insert_is_graph_reachable_via_backlinks`

## Review Focus

- Is the `expected_slice` guard the right narrow contract for full-slice
  rewrites before the final concurrency-hardening checkpoint?
- Is simple score-ordered pruning an acceptable stopping point for A5, or is
  there a correctness gap that still forces heuristic selection sooner?
- Should the next slice go directly to `inserted_since_rebuild`, or is there a
  smaller concurrency-retry improvement worth landing first?
