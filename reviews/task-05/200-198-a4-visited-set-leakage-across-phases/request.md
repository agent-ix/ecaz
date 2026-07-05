# Review Request: A4 Visited Set Leakage Across Search Phases

## Summary

In the standard HNSW algorithm, the visited set is managed per search phase: upper-layer greedy descent and layer-0 beam search each start with a fresh visited set (or the descent carries only the single best node forward). In tqvector, a single `visited_tids` HashSet persists across greedy descent and the subsequent layer-0 bootstrap search, and then continues into the incremental frontier refill. This could cause the beam search to skip nodes that were visited during descent but are actually important layer-0 neighbors.

## Current behavior

**Visited set lifecycle** (`scan.rs`):
1. `reset_scan_position` calls `reset_scan_visited_state` → clears the set
2. `initialize_scan_entry_candidate` calls `greedy_descend_from_entry` → **does not mark visited nodes** (descent doesn't use the visited set directly; it uses its own callback pattern)
3. `search_layer0_result_candidates` filters with `|neighbor_tid| !visited_contains_element(opaque, neighbor_tid)` → **the layer-0 search filters against the same visited set**
4. Frontier refill uses the same `visited_contains_element` check

Looking more carefully at `graph::greedy_descend_from_entry` (`graph.rs:133-155`): the descent loads neighbors via `load_neighbor_tids_for_layer` and scores them, but it doesn't write to the scan-level visited set. The visited set tracking happens in the BeamSearch's internal `visited: HashSet<NodeId>` inside `search_layer0_result_candidates_with_successors`.

**However**, the bootstrap trace seeding (`scan.rs:847-878`, `seed_bootstrap_trace`) calls `mark_visited_element` for every discovered candidate. Once the bootstrap results are seeded into the visible frontier, subsequent refill operations (`refill_candidate_frontier_from_source_into` at scan.rs:960-991) filter neighbors with `!visited_contains_element`. This means:

- Nodes discovered during bootstrap but not selected for the visible frontier are marked visited
- When a frontier source is later refilled, its neighbors that were already seen during bootstrap are skipped
- This is correct behavior IF the bootstrap search was comprehensive enough

**The risk**: if `bootstrap_frontier_limit` (= `ef_search`) is small, the bootstrap search may discover many nodes but only keep a few. All discovered nodes get marked visited. Later refill operations cannot rediscover them through different graph paths, even if they'd score better in the new context.

## Comparison with reference implementations

**hnswlib-rs** (`hnsw.rs:1421-1470`, `search_general`):
- Upper layers: no visited set (simple greedy, one neighbor at a time)
- Layer 0: fresh visited set per `search_layer` call
- No carryover between phases

**instant-distance** (`lib.rs:729-737`, `cull`):
- Explicit `cull()` between layers: clears candidates AND clears visited
- Only preserves the `nearest` list
- Each layer starts with a clean slate

**swarc** (`search.rs:73-106`):
- Visited set is **not cleared** between layers (same HashSet throughout)
- But swarc's layer search is different — it considers all nodes in layer 0

## Suggested investigation

1. **Visited count audit**: After `initialize_scan_entry_candidate` completes, log how many nodes are in `visited_tids`. If this is a significant fraction of the index (e.g., >10% of 10K nodes), the refill path is operating with a heavily pre-filtered neighbor set.

2. **Visited-set-reset experiment**: After `search_layer0_result_candidates` returns but before seeding the bootstrap trace, clear the visited set (keeping only the frontier candidates as visited). Re-run recall gate.

3. **Check if greedy descent pollutes the visited set**: Trace whether any code path between `greedy_descend_from_entry` and `search_layer0_result_candidates` calls `mark_visited_element`.

## Files to read

- `src/am/scan.rs:788-878` — `initialize_scan_entry_candidate` and `seed_bootstrap_trace`
- `src/am/scan.rs:695-724` — visited set management functions
- `src/am/scan.rs:960-1030` — refill and top-up (use visited filter)
- `src/am/graph.rs:133-207` — descent and layer-0 search (their internal visited sets)

## Review focus

- Whether the visited set correctly separates descent-phase and search-phase state
- Whether bootstrap discovery over-marks visited nodes, starving later refill operations
- Whether the reference implementations' visited-set hygiene should be adopted
