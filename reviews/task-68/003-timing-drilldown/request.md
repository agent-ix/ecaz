# Review Request: Task 68 Packet 003 Timing Drilldown

Code commit: `3471bc78bd9aea454dfa3b407d191a924b7e0447`

## Summary

This packet addresses the Task 68 packet 001/002 reviewer blockers before any Phase 2 P0 optimization slice:

- Makes `draft_ms` and `object_store_ms` exclusive top-level phase fields.
- Retains `draft_total_ms` and `object_store_total_ms` for the outer nested wall times.
- Emits recursive routing child counts and iterations to explain the prior `recursive_kmeans_calls=1` result.
- Adds drilldown timings inside recursive draft assembly.
- Reruns a 100k `CREATE INDEX` through `ecaz bench suite`.

Artifact manifest: `artifacts/manifest.md`

Rollup: `artifacts/drilldown-summary.md`

## Validation

Compile:

```text
cargo check -p ecaz --lib --no-default-features --features pg18
```

Result: passed.

Suite status:

```text
completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Key Result

New 100k notice:

```text
ec_spire_ambuild_timing index=task68_spire_100k_drilldown_idx phase=complete heap_tuples=100000 scanned_tuples=100000 index_tuples=100000 recursive_fanout=8 setup_ms=0 heap_scan_ms=1223 sample_collect_ms=0 kmeans_ms=490 kmeans_calls=1 assignment_ms=574 recursive_kmeans_ms=1 recursive_kmeans_calls=1 recursive_kmeans_max_level=1 recursive_assignment_ms=0 recursive_routing_initial_children=128 recursive_routing_final_children=8 recursive_routing_iterations=1 draft_ms=19247 draft_total_ms=19248 draft_input_clone_ms=47 draft_pid_alloc_ms=0 draft_recursive_routing_ms=2 draft_route_map_ms=0 draft_leaf_rows_ms=19182 draft_leaf_inputs_ms=10 draft_validation_ms=0 top_graph_ms=937 pq4_training_ms=0 object_store_ms=0 object_store_total_ms=937 publish_ms=8 total_ms=22482
```

Disjoint top-level phase sum:

```text
setup 0
+ heap_scan 1223
+ sample_collect 0
+ kmeans 490
+ assignment 574
+ recursive_kmeans 1
+ recursive_assignment 0
+ draft 19247
+ top_graph 937
+ object_store 0
+ publish 8
= 22480 ms, within 2 ms of total_ms=22482
```

## Blocker Closure

1. Draft drilldown: `draft_leaf_rows_ms=19182`, or 85.3 % of total wall time, is the hot subphase. The first P0 slice should target `build_recursive_leaf_rows_by_pid` / boundary leaf row placement.
2. Phase double-counting: `object_store_ms=0` and `object_store_total_ms=937` now show that the prior object-store bucket was top-graph-contained. `draft_ms` is exclusive while `draft_total_ms` retains the nested wall time.
3. Recursive k-means call count: the one call is expected for this algorithm. The run records `recursive_routing_initial_children=128`, `recursive_routing_final_children=8`, and `recursive_routing_iterations=1`; the hierarchy clusters current children directly to `target_fanout`.

## Estimated Caps

- `draft_leaf_rows_ms`: 19.182s / 22.482s = 85.3 % wall-time cap. Full elimination caps speedup at about 6.8x; a 50 % reduction caps improvement at about 1.7x.
- `top_graph_ms`: 0.937s / 22.482s = 4.2 % wall-time cap.
- `heap_scan_ms`: 1.223s / 22.482s = 5.4 % wall-time cap.

## Reviewer Ask

Please verify that the three Phase 2 blockers are closed and that the first P0 optimization slice should target `draft_leaf_rows_ms`, specifically the recursive leaf row construction/placement path.
