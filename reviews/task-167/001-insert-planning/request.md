# Review request — Task 167 M5 slice 1: incremental-insert planning

**Branch:** `task-165-ec-distann-m3` (M5 work continues here until its own
branch). First M5 slice: the pure planning half of the incremental graph insert.

## Context

M5 folds inserts into the persisted Vamana graph so they gain graph
connectivity (found by traversal, not just the M3 delta-buffer exact-scan tail)
and the buffer stays bounded. Full incremental insert is a multi-part on-disk
mutation (below); this slice lands the pure, unit-testable planning core with no
on-disk risk.

## What landed

New `ec_distann/insert.rs`:
- `select_insert_forward_neighbors(source, candidates, alpha, max_degree)` —
  `robust_prune` the inserted node's FR-081 search frontier to `max_degree` on
  exact `-inner_product` distance, α-diversified exactly like the batch build.
  Returns the kept neighbors' **vec_ids** (distann adjacency references neighbors
  by vec_id, never by record TID). Pure.
- `DistannForwardCandidate` (vec_id + co-placed source vector).

## Evidence (`artifacts/test-evidence.log`)

2 unit tests: degree bound respected, nearest kept, no duplicate edges, valid
vec_ids; bad input (empty source, alpha<1, max_degree 0, dim mismatch) rejected,
empty candidates → no neighbors.

## M5 remaining slices (on-disk mutation, each individually tested)

1. **Forward-neighbor search**: greedy-search the persisted graph for an
   inserted vector's candidate frontier (reader + exact rerank).
2. **Node append**: build the new `DistannNodeTuple` (search_code via the
   quantizer + embedded neighbor codes) and append it WAL-logged.
3. **Backlink amendment**: add the new node to each forward neighbor's adjacency
   in place; `robust_prune` the neighbor when its degree is full.
4. **Directory maintenance**: insert the new vec_id→record mapping into the
   sorted directory (rebuild for correctness first; incremental tail is a
   follow-up).
5. **Wire**: route `aminsert` / the write endpoint to fold delta entries into
   the graph; `node_count`/metadata update; multi-node routing to the owning
   node.

## Ask

Review the neighbor-selection metric/α-diversification and the slice plan. Not
closing the request.
