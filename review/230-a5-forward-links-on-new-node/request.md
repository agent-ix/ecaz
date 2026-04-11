# Review Request: A5 Insert Forward Links on the New Node

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/06-graph-insert.md`
- `review/199-aminsert-graph-aware-insertion-roadmap/request.md`
- `plan/status.md`

This is the second narrow A5 checkpoint after level allocation / tuple sizing /
metadata promotion. It takes the next acceptable slice from review 199 without
trying to land backlinks or neighbor shrinking yet.

Checkpoint scope:

1. reuse `greedy_descend_from_entry`
2. reuse the existing layer-0 search helper for insert-side candidate discovery
3. write simple top-`M` forward links on the **new node only**
4. keep backlinks, upper-layer insert search, and shrinking deferred

## Scope

- `src/am/insert.rs`
- `src/lib.rs`
- `plan/tasks/06-graph-insert.md`

## What Landed

### 1. Live insert now computes forward-link seeds from the current graph

Non-empty `aminsert` no longer appends a fully disconnected node.

The new insert-side helper path:

- loads the current metadata entry point if present
- scores it against the incoming code using the existing code-to-code scorer
- runs `graph::greedy_descend_from_entry(...)`
- runs `graph::search_layer0_result_candidates(...)` with
  `ef_construction.max(1)` as the candidate window

This keeps insert-side traversal aligned with the traversal seams already used
by runtime graph code instead of introducing a second search implementation.

### 2. The new node now persists a simple top-`M` layer-0 forward set

The inserted neighbor tuple still allocates the full `neighbor_slots(level, m)`
shape, but the insert path now fills only the first `M` slots from the layer-0
candidate results.

Everything after that remains `INVALID` in this slice:

- the second half of the layer-0 window
- every upper-layer slot when `level > 0`

That keeps the change narrow and avoids pretending the graph is fully connected
before backlink mutation exists.

### 3. Empty-index / first-insert behavior stays on the existing narrow path

The empty-index path still:

- initializes shape metadata
- allocates the correct level-shaped neighbor tuple
- appends the new node
- promotes metadata when needed

It does **not** try to search for neighbors before an entry point exists.

### 4. A5 task tracking now exposes milestone progress in-tree

`plan/tasks/06-graph-insert.md` now records:

- `20%` complete for level allocation / tuple sizing / promotion
- `35%` complete for greedy descent, layer-0 candidate search, and new-node
  forward links

The remaining milestones for upper-layer search, backlinks, shrinking, and
concurrency hardening are also called out explicitly.

## Locking Decision

This packet still does **not** introduce multi-page neighbor/backlink mutation,
so there is no new ADR yet.

Current lock behavior remains:

1. duplicate scan outside the metadata EXCLUSIVE lock
2. graph reads through the existing shared traversal/page helpers
3. append under one target data-page EXCLUSIVE buffer lock
4. metadata repair/promotion afterward under the metadata-page EXCLUSIVE lock only when needed

No insert in this slice holds multiple data-page locks at once.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed for this checkpoint. The `cargo test` / `cargo pgrx test` runs still
include the usual long `proptest_quant` tail (`payload_len_matches_actual`).

## New / Updated Coverage

- `test_tqhnsw_insert_populates_forward_links_from_live_entry_seed`
- `test_tqhnsw_insert_populates_forward_links_against_built_graph`
- existing insert shape / duplicate / entry-point / rollover coverage remains green

The new tests assert:

- the second live insert into an initially empty index links forward to the
  original entry element
- a live insert into a built graph persists at least one forward link to a
  pre-existing element
- only the first `M` slots are populated in this slice, with the remaining
  layer-0 / upper-layer slots left `INVALID`

## Review Focus

- Is the insert-side reuse of `greedy_descend_from_entry` plus
  `search_layer0_result_candidates` the right narrow traversal seam here?
- Is filling only the first `M` slots on the new node an acceptable interim
  contract until backlinks and shrinking land?
- Should the next slice prioritize upper-layer insert search first, or go
  directly to backlink mutation with the lock-ordering ADR?
