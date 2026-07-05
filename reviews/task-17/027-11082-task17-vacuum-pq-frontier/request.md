# Review Request: use the PQ frontier for DiskANN vacuum repair planning

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `src/am/ec_diskann/routine.rs`

## What this packet is

This is the next DiskANN AM slice after packet `11081`.

`11081` bounded the vacuum repair budget and restored interrupt servicing, but
the repair planner was still doing more work than the algorithm needs: it ran a
full `vamana_scan_with(...)` pass with exact heap rerank just to assemble the
candidate frontier that `RobustPrune` immediately re-evaluates on exact source
vectors.

This packet removes that redundant exact rerank from the vacuum repair planning
path. It keeps the repair semantics the same: collect a bounded candidate
frontier, decode live source vectors for those candidates, then let
`select_insert_forward_neighbors(...)` do the exact alpha-prune selection.

## What changed

### `src/am/ec_diskann/routine.rs`

`plan_vacuum_fill_candidates_for_target(...)` now uses the grouped-PQ frontier
directly:

```rust
scan::greedy_descent_with(
    &reader,
    visited,
    entry_point,
    repair_scan_budget,
    &|tuple: &VamanaNodeTuple| {
        -grouped_pq_score_f32(&query_lut, group_count, &tuple.search_code)
    },
)?
```

instead of calling `scan::vamana_scan_with(...)` with:

- `list_size = repair_scan_budget`
- `rerank_budget = repair_scan_budget`
- `top_k = repair_scan_budget`
- an `exact_heap_rerank_distance(...)` callback for every frontier candidate

The rest of the repair planner stays intact:

- skip dead / duplicate tids
- fetch each surviving candidate's source vector from heap
- build `ForwardNeighborCandidate` rows
- run `select_insert_forward_neighbors(...)` for the exact final selection

So the only behavioral change is that the repair frontier no longer pays for an
intermediate exact rerank pass that the final prune step does not need.

## Why this slice

- DiskANN-only and AM-local; no CLI changes.
- Keeps the vacuum follow-up focused on algorithm/runtime work instead of local
  machine tuning.
- Matches the actual repair contract from ADR-047: fill-only repair still uses
  exact source vectors at the prune step, but does not need an earlier exact
  rerank just to choose the frontier window.
- Avoids making local performance claims on the slower machine. This is a
  structural work reduction that should carry to the faster final-bench box.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 218 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also passed on `pg18` for this checkpoint:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Notable DiskANN coverage in that run:

- `am::ec_diskann::routine::tests::pg_test_ec_diskann_vacuum_refills_broken_neighbor_slot`
- `am::ec_diskann::routine::tests::pg_test_ec_diskann_vacuum_replans_on_stale_repair_tuple`
- `am::ec_diskann::routine::tests::vacuum_repair_scan_budget_caps_at_graph_degree`

## Follow-ups intentionally not in this packet

- Any new recall / latency / vacuum timing claims. Final benches belong on the
  faster machine.
- A broader redesign of DiskANN vacuum repair. This slice only removes the
  redundant exact-rerank stage from the existing repair planner.
- Any `ecaz-cli` changes. This packet is purely in the DiskANN AM.
