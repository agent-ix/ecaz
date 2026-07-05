# Artifact Manifest

- Task bucket: `reviews/task-68/005-zero-replica-fast-path-measurement`
- Head SHA while writing packet: `cd98e26ba7a8a8f1ecb2da8715dd782961161e2a`
- Code SHA under measurement: `c8f98a71da07e8d1417642fcbbe558ce0ae942d9`
- Installed backend SHA-256: `0a47749823f1bea04783eee15ce670ca03e3933a0ec1be5548ae98b92f5bd6ec`
- Timestamp: `2026-05-30T04:51:24Z`
- Lane: Task 68 SPIRE build Phase 2, zero-replica leaf-row fast path
- Fixture/storage/rerank: M5 DBpedia 10k and 100k, `turboquant`, `rerank_width=25`
- Surface isolation: one measured index per table, using existing packet 002 fixture tables

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `install-current-extension.log` | `/Users/peter/.cargo/bin/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/install-current-extension.log` | Installed backend SHA-256 `0a47749823f1bea04783eee15ce670ca03e3933a0ec1be5548ae98b92f5bd6ec`. |
| `suite.json` | Checked-in `ecaz bench suite` config | Defines precheck plus 10k and 100k measured `CREATE INDEX` steps. |
| `suite-dry-run-manifest.json` | `/Users/peter/.cargo/bin/ecaz --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/suite.json --dry-run --manifest-output reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/suite-dry-run-manifest.json` | Expanded all 3 commands without execution. |
| `suite-manifest.json` | `/Users/peter/.cargo/bin/ecaz --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/suite.json --manifest-output reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/suite-manifest.json` | Suite completed 3/3 steps. |
| `precheck-host-and-tables.log` | Suite step `precheck-host-and-tables` | Confirmed 10k and 100k corpus row counts, extension `ecaz`, and AM `ec_spire`. |
| `create-10k-spire-fastpath-index.log` | Suite step `create-10k-spire-fastpath-index` | `total_ms=372`, `draft_total_ms=10`, `draft_leaf_rows_ms=3`, `top_graph_ms=59`. |
| `create-100k-spire-fastpath-index.log` | Suite step `create-100k-spire-fastpath-index` | `total_ms=3362`, `draft_total_ms=92`, `draft_leaf_rows_ms=25`, `top_graph_ms=935`. |
| `results.jsonl` | Written by `ecaz bench suite run` | Normalized suite result rows. |
| `results-from-report.jsonl` | `/Users/peter/.cargo/bin/ecaz --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite report --manifest reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/suite-manifest.json --results-output reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/results-from-report.jsonl` | Report extraction completed against the suite manifest. |
| `suite-status.md` | Captured status summary | `completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`. |
| `measurement-summary.md` | Manual summary from packet-local logs | Before/after tables and updated phase ranking. |

## Cited Notice Lines

10k:

```text
NOTICE:  ec_spire_ambuild_timing index=task68_spire_10k_fastpath_idx phase=complete heap_tuples=10000 scanned_tuples=10000 index_tuples=10000 recursive_fanout=8 setup_ms=0 heap_scan_ms=137 sample_collect_ms=0 kmeans_ms=149 kmeans_calls=1 assignment_ms=14 recursive_kmeans_ms=0 recursive_kmeans_calls=1 recursive_kmeans_max_level=1 recursive_assignment_ms=0 recursive_routing_initial_children=32 recursive_routing_final_children=8 recursive_routing_iterations=1 draft_ms=10 draft_total_ms=10 draft_input_clone_ms=4 draft_pid_alloc_ms=0 draft_recursive_routing_ms=1 draft_route_map_ms=0 draft_leaf_rows_ms=3 draft_leaf_inputs_ms=0 draft_validation_ms=0 top_graph_ms=59 pq4_training_ms=0 object_store_ms=0 object_store_total_ms=59 publish_ms=0 total_ms=372
```

100k:

```text
NOTICE:  ec_spire_ambuild_timing index=task68_spire_100k_fastpath_idx phase=complete heap_tuples=100000 scanned_tuples=100000 index_tuples=100000 recursive_fanout=8 setup_ms=0 heap_scan_ms=1252 sample_collect_ms=0 kmeans_ms=495 kmeans_calls=1 assignment_ms=580 recursive_kmeans_ms=1 recursive_kmeans_calls=1 recursive_kmeans_max_level=1 recursive_assignment_ms=0 recursive_routing_initial_children=128 recursive_routing_final_children=8 recursive_routing_iterations=1 draft_ms=91 draft_total_ms=92 draft_input_clone_ms=48 draft_pid_alloc_ms=0 draft_recursive_routing_ms=2 draft_route_map_ms=0 draft_leaf_rows_ms=25 draft_leaf_inputs_ms=10 draft_validation_ms=0 top_graph_ms=935 pq4_training_ms=0 object_store_ms=0 object_store_total_ms=935 publish_ms=7 total_ms=3362
```
