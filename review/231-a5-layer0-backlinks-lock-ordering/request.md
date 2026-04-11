# Review Request: A5 Layer-0 Backlinks and Lock Ordering

## Context

Branch:
- `main`

Task / roadmap inputs:
- `plan/tasks/06-graph-insert.md`
- `review/199-aminsert-graph-aware-insertion-roadmap/request.md`
- `plan/status.md`

This is the next narrow A5 checkpoint after new-node forward links. It takes the
first slice that makes a live insert graph-reachable without trying to land
overflow pruning or upper-layer backlink mutation in the same jump.

Checkpoint scope:

1. reuse the already-selected layer-0 forward neighbors from the prior slice
2. add layer-0 backlinks on existing neighbors only when free capacity exists
3. define the physical lock order for multi-page backlink mutation
4. prove graph reachability via the graph-first runtime frontier

## Scope

- `src/am/insert.rs`
- `src/lib.rs`
- `plan/tasks/06-graph-insert.md`
- `spec/adr/ADR-026-live-insert-backlink-lock-ordering.md`

## What Landed

### 1. Live insert now writes layer-0 backlinks into selected existing neighbors

After appending the new node, `aminsert` now:

- takes the first `M` selected forward-link element tids from the new node
- loads each selected element to find its persisted neighbor tuple tid
- sorts those neighbor tuple tids by physical `(block_number, offset_number)`
- rewrites them page-by-page to add the new node tid into the layer-0 slice

This keeps the new node’s forward-link selection logic unchanged and adds the
minimum reverse-edge work needed for graph reachability.

### 2. Backlink mutation is intentionally limited to free layer-0 capacity

This checkpoint does **not** prune or shrink an existing neighbor list yet.

The rewrite helper:

- checks only the layer-0 window (`2 * M` slots)
- no-ops if the new node is already present
- inserts into the first `INVALID` slot when one exists
- skips full layer-0 slices for now

That keeps the slice narrow and avoids mixing lock protocol work with the
scoring/pruning policy that A5 still needs later.

### 3. Multi-page writes now have an explicit lock-ordering rule

Because backlink mutation can touch several pages, this checkpoint also lands
`ADR-026`.

The implementation rule is:

1. traversal and candidate discovery first
2. append the new node under one data-page `EXCLUSIVE` lock
3. release that append-page lock
4. sort backlink targets by physical neighbor-tuple tid
5. rewrite one data page at a time in ascending order
6. acquire the metadata-page `EXCLUSIVE` lock only after data-page writes finish

Within one page, multiple neighbor tuples are updated under one buffer lock and
one GenericXLog transaction.

### 4. A5 task tracking now records this as the first graph-reachability slice

`plan/tasks/06-graph-insert.md` now records:

- `50%` complete for layer-0 backlinks, graph reachability, and the ADR
- upper-layer insert search / backlinks still pending
- overflow pruning still pending

## Locking Decision

The new durable rule is:

- data pages first
- one data page at a time
- ascending physical order
- metadata last

`aminsert` never holds a metadata-page `EXCLUSIVE` lock at the same time as a
data-page `EXCLUSIVE` lock in this slice.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All passed on this checkpoint.

## New / Updated Coverage

- `test_tqhnsw_insert_populates_forward_links_from_live_entry_seed`
- `test_tqhnsw_live_insert_is_graph_reachable_via_backlinks`

The new assertions cover:

- the seeded live-entry element receives a persisted layer-0 backlink to the new node
- a non-promoted live insert becomes visible in the graph-seeded runtime frontier
  immediately after `amrescan`, before linear fallback can mask a disconnected node

Existing insert shape / duplicate / empty-index / rollover coverage stayed green.

## Review Focus

- Is the “selected layer-0 neighbors with free capacity only” boundary acceptable
  as the checkpoint before pruning lands?
- Is the page-grouped GenericXLog rewrite path sound for multiple neighbor tuples
  on the same page?
- Does `ADR-026` capture the right durable lock order for the later overflow and
  upper-layer backlink work?
