# Review Request: A6 Upper-Layer Replacement Fill

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/07-vacuum.md`
- `plan/status.md`
- `plan/plan.md`
- `spec/functional/FR-022-vacuum-implementation.md`
- `spec/adr/ADR-027-vacuum-graph-repair-lock-ordering.md`

This is the next A6 checkpoint after layer-0 replacement fill. It extends the
same pass-2 repair shape into upper layers without widening the write-side lock
scope or introducing eviction/pruning.

Checkpoint scope:

1. generalize repair requests from layer-0-only to layer-aware
2. plan layer-aware replacements with existing graph search helpers first
3. keep linear top-up fallback for sparse / exhausted search results
4. keep the write phase fill-only on currently free slots
5. prove a real post-vacuum upper-layer reconnection event

## Scope

- `src/am/vacuum.rs`
- `src/lib.rs`
- `plan/tasks/07-vacuum.md`
- `plan/status.md`
- `plan/plan.md`
- `spec/functional/FR-022-vacuum-implementation.md`
- `spec/adr/ADR-027-vacuum-graph-repair-lock-ordering.md`

## What Landed

### 1. Pass 2 repair requests are now layer-aware

Vacuum now records `LayerRepairRequest { source_tid, neighbor_tid, layer }`
instead of only a layer-0 repair request.

Before unlink runs, it scans each live element tuple and records one repair
request for every logical layer whose persisted slice still references a
soon-to-be-deleted element TID.

### 2. Replacement planning now covers upper layers too

The replacement planner now:

- reuses the existing entry-point scoring path
- for layer 0, keeps the current `greedy_descend_from_entry(...)` +
  `search_layer0_result_candidates(...)` behavior
- for upper layers, descends/searches with
  `search_layer_result_candidates(...)` at the requested layer
- seeds the search from surviving neighbors already present in that layer
- filters out deleted nodes, self, and already-surviving live neighbors

If graph search still does not yield enough candidates, the planner tops up
from a linear live-node scan restricted to elements that actually participate in
the requested layer.

### 3. The write phase still stays narrow

ADR-027 remains the governing rule:

1. scan / plan first
2. rewrite one data page at a time in ascending physical order
3. no metadata-page write in pass 2

The new behavior does **not** add pruning or eviction. The page write phase:

- groups plans by neighbor tuple
- decodes each tuple once
- clears any deleted refs still present
- fills only `INVALID` slots in the requested layer slices
- re-encodes once per changed tuple

So the repair surface is wider across layers, but the write-side concurrency
boundary is unchanged.

### 4. Tracking/spec docs now mark layer-aware repair as landed

The task/status/spec surfaces now record that:

- A6 is about `85%` complete on `main`
- pass 2 now repairs broken edges across all persisted layers
- the remaining A6 work is concurrency validation

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed on this checkpoint.

## New / Updated Coverage

- `test_tqhnsw_vacuum_pass2_layer0_replacement_fills_broken_edges`
- `test_tqhnsw_vacuum_pass2_upper_replacement_fills_broken_edges`

The new upper-layer regression:

- builds a real graph fixture with at least one upper layer
- dynamically chooses a deletable row that actually has a live inbound
  layer-1 edge
- runs the vacuum mark/remove path
- proves the deleted element tid is still fully unlinked from persisted
  neighbor tuples
- proves at least one affected live upper-layer slice receives a new live
  replacement candidate not present in its surviving pre-vacuum layer-1 set

## Review Focus

- Is layer-aware fill-only repair the right stopping point before the final A6
  concurrency slice, or does upper-layer repair force eviction/retry sooner?
- Is the graph-search-first plus linear top-up fallback still a reasonable
  narrow policy once upper layers are included?
- Does ADR-027 still describe the real lock boundary accurately now that pass 2
  can mutate multiple logical layers within one persisted neighbor tuple?
