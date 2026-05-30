# Artifact Manifest

- Task bucket: `reviews/task-68/007-top-graph-cache-measurement`
- Head SHA while writing packet: `1a4aca35e4aa7b0c6af88cc38751626c218780d4`
- Code SHA under measurement: `fe7d5e6892dc1e7154eb95d8e620b22bef070d10`
- Installed backend SHA-256: `4aa5e735046ceb62ac02a87c7d14e513030e98111dec515cd05aeccb5a3551a8`
- Timestamp: `2026-05-30T05:28:36Z`
- Lane: Task 68 SPIRE build Phase 2, top-graph distance cache measurement
- Fixture/storage/rerank: M5 DBpedia 10k and 100k, `turboquant`, `rerank_width=25`
- Surface isolation: one measured index per table, using existing packet 002 fixture tables

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `install-current-extension.log` | `/Users/peter/.cargo/bin/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-68/007-top-graph-cache-measurement/artifacts/install-current-extension.log` | Installed backend SHA-256 `4aa5e735046ceb62ac02a87c7d14e513030e98111dec515cd05aeccb5a3551a8`. |
| `suite.json` | Checked-in `ecaz bench suite` config | Defines precheck plus 10k and 100k measured `CREATE INDEX` steps. |
| `suite-dry-run-manifest.json` | `/Users/peter/.cargo/bin/ecaz --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-68/007-top-graph-cache-measurement/artifacts/suite.json --dry-run --manifest-output reviews/task-68/007-top-graph-cache-measurement/artifacts/suite-dry-run-manifest.json` | Expanded all 3 commands without execution. |
| `suite-manifest.json` | `/Users/peter/.cargo/bin/ecaz --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-68/007-top-graph-cache-measurement/artifacts/suite.json --manifest-output reviews/task-68/007-top-graph-cache-measurement/artifacts/suite-manifest.json` | Suite completed 3/3 steps. |
| `precheck-host-and-tables.log` | Suite step `precheck-host-and-tables` | Confirmed 10k and 100k corpus row counts, extension `ecaz`, and AM `ec_spire`. |
| `create-10k-spire-topgraph-cache-index.log` | Suite step `create-10k-spire-topgraph-cache-index` | `total_ms=384`, `top_graph_ms=68`, `draft_total_ms=10`, `draft_leaf_rows_ms=1`. |
| `create-100k-spire-topgraph-cache-index.log` | Suite step `create-100k-spire-topgraph-cache-index` | `total_ms=3236`, `top_graph_ms=847`, `draft_total_ms=90`, `draft_leaf_rows_ms=19`. |
| `results.jsonl` | Written by `ecaz bench suite run` | Normalized suite result rows. |
| `results-from-report.jsonl` | `/Users/peter/.cargo/bin/ecaz --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite report --manifest reviews/task-68/007-top-graph-cache-measurement/artifacts/suite-manifest.json --results-output reviews/task-68/007-top-graph-cache-measurement/artifacts/results-from-report.jsonl` | Report extraction completed against the suite manifest. |
| `suite-status.md` | Captured status summary | `completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`. |
| `measurement-summary.md` | Manual summary from packet-local logs | Before/after tables and updated phase ranking. |

## Cited Notice Lines

10k:

```text
NOTICE:  ec_spire_ambuild_timing index=task68_spire_10k_topgraph_cache_idx phase=complete heap_tuples=10000 scanned_tuples=10000 index_tuples=10000 recursive_fanout=8 setup_ms=0 heap_scan_ms=138 sample_collect_ms=0 kmeans_ms=148 kmeans_calls=1 assignment_ms=15 recursive_kmeans_ms=0 recursive_kmeans_calls=1 recursive_kmeans_max_level=1 recursive_assignment_ms=0 recursive_routing_initial_children=32 recursive_routing_final_children=8 recursive_routing_iterations=1 draft_ms=10 draft_total_ms=10 draft_input_clone_ms=4 draft_pid_alloc_ms=0 draft_recursive_routing_ms=1 draft_route_map_ms=0 draft_leaf_rows_ms=1 draft_leaf_inputs_ms=0 draft_validation_ms=0 top_graph_ms=68 pq4_training_ms=0 object_store_ms=0 object_store_total_ms=68 publish_ms=2 total_ms=384
```

100k:

```text
NOTICE:  ec_spire_ambuild_timing index=task68_spire_100k_topgraph_cache_idx phase=complete heap_tuples=100000 scanned_tuples=100000 index_tuples=100000 recursive_fanout=8 setup_ms=0 heap_scan_ms=1234 sample_collect_ms=0 kmeans_ms=489 kmeans_calls=1 assignment_ms=573 recursive_kmeans_ms=1 recursive_kmeans_calls=1 recursive_kmeans_max_level=1 recursive_assignment_ms=0 recursive_routing_initial_children=128 recursive_routing_final_children=8 recursive_routing_iterations=1 draft_ms=89 draft_total_ms=90 draft_input_clone_ms=47 draft_pid_alloc_ms=0 draft_recursive_routing_ms=2 draft_route_map_ms=0 draft_leaf_rows_ms=19 draft_leaf_inputs_ms=10 draft_validation_ms=0 top_graph_ms=847 pq4_training_ms=0 object_store_ms=0 object_store_total_ms=847 publish_ms=0 total_ms=3236
```
