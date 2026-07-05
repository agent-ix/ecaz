# Review Request: A6 Layer-0 Replacement Fill

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/07-vacuum.md`
- `plan/status.md`
- `plan/plan.md`
- `spec/functional/FR-022-vacuum-implementation.md`
- `spec/adr/ADR-027-vacuum-graph-repair-lock-ordering.md`

This is the next A6 checkpoint after dead-edge unlink. It keeps pass 2 narrow:
vacuum now repairs broken layer-0 edges on affected live nodes, but it still
defers upper-layer replacement, eviction/pruning, and concurrency validation.

Checkpoint scope:

1. collect affected live layer-0 nodes before dead-edge unlink
2. unlink deleted neighbor references as before
3. plan replacement candidates read-only, reusing insert/runtime traversal helpers
4. fill only currently free layer-0 slots during the ordered page write phase
5. add regression coverage for a real post-vacuum replacement event

## Scope

- `src/am/vacuum.rs`
- `src/lib.rs`
- `plan/tasks/07-vacuum.md`
- `plan/status.md`
- `plan/plan.md`
- `spec/functional/FR-022-vacuum-implementation.md`
- `spec/adr/ADR-027-vacuum-graph-repair-lock-ordering.md`

## What Landed

### 1. Pass 2 now records which live nodes actually lost a layer-0 edge

Before unlink runs, vacuum now scans live element tuples and records
`Layer0RepairRequest { source_tid, neighbor_tid }` only for layer-0 slices that
currently reference a soon-to-be-deleted element TID.

That keeps replacement planning targeted to the nodes whose persisted graph view
really changed, instead of broad rescoring across the full index.

### 2. Replacement planning stays read-only and reuses the existing graph helpers

After dead-edge unlink, vacuum now plans replacement candidates for those
affected live nodes by:

- loading the current metadata and persisted source code
- reusing `greedy_descend_from_entry(...)` plus
  `search_layer0_result_candidates(...)` first
- scoring candidates with the same code-to-code metric used by Task 06 insert
  planning
- deduping against deleted nodes, self, and already-surviving live neighbors

If graph search does not yield enough candidates, the planner tops up from a
linear live-node scan to keep the slice narrow and deterministic.

### 3. The write phase is still fill-only and page-ordered

ADR-027 is unchanged in spirit:

1. scan / planning first
2. rewrite one data page at a time in ascending block order
3. no metadata-page write in pass 2

The current repair write path is intentionally narrower than live insert:

- it only fills `INVALID` slots in the existing layer-0 slice
- it does not evict or shrink live neighbors
- it skips targets that no longer need repair by the time the write phase runs

So this slice restores some local connectivity without claiming final graph
quality or final concurrency semantics.

### 4. Tracking/docs now mark layer-0 replacement as landed

The task/status/spec surfaces now record that:

- A6 is about `70%` complete on `main`
- pass 2 dead-edge unlink plus layer-0 replacement fill are merged
- upper-layer repair and concurrency validation still remain

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed on this checkpoint.

## New / Updated Coverage

- `test_tqhnsw_vacuum_pass2_layer0_replacement_fills_broken_edges`

The new regression:

- builds a real graph fixture
- picks a deletable row that actually has at least one live inbound layer-0 edge
- runs the vacuum mark/remove path
- proves the deleted node is fully unlinked from persisted neighbor tuples
- proves at least one affected live node receives a new live layer-0 replacement
  candidate not present in its surviving pre-vacuum layer-0 set

## Review Focus

- Is graph-search-first with linear top-up an acceptable narrow A6 slice for
  layer-0 replacement, or is there still a correctness gap that forces a wider
  search/pruning policy immediately?
- Is the current fill-only write contract the right stopping point before
  upper-layer repair lands?
- Does ADR-027 still capture the correct concurrency boundary now that pass 2
  includes both unlink and replacement planning?
