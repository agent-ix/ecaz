# Task 68 Packet 003 Artifact Manifest

- head SHA: `3471bc78bd9aea454dfa3b407d191a924b7e0447`
- task bucket: `reviews/task-68/003-timing-drilldown`
- timestamp: `2026-05-30T04:37:30Z`
- lane: Task 68 timing-shape fix and draft drilldown
- fixture/storage/rerank: M5 DBpedia 100k table from packet 002, `storage_format=turboquant`, `rerank_width=25`
- isolated one-index-per-table or shared-table surface: one measured index on the existing 100k table

## Artifacts

### `install-current-extension.log`

- command: `/Users/peter/.cargo/bin/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-68/003-timing-drilldown/artifacts/install-current-extension.log`
- result: passed
- key lines:
  - `[install] backend artifact assertion passed`
  - `[install] installed_backend=/opt/homebrew/lib/postgresql@18/ecaz.dylib`
  - `[install] sha256=bc2cf979984481b6a1bb727c555190010972d11442f4909293ebb76728766149`

### `cargo-check-ecaz-lib-pg18.log`

- command: `cargo check -p ecaz --lib --no-default-features --features pg18`
- result: passed
- key line: `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.09s`

### `suite.json`

- command source: checked-in `ecaz bench suite` config
- result: covers host/table precheck and one 100k measured `CREATE INDEX` run

### `suite-dry-run-manifest.json`

- command: `/Users/peter/.cargo/bin/ecaz --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-68/003-timing-drilldown/artifacts/suite.json --dry-run --manifest-output reviews/task-68/003-timing-drilldown/artifacts/suite-dry-run-manifest.json`
- result: passed

### `suite-manifest.json`

- command: `/Users/peter/.cargo/bin/ecaz --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-68/003-timing-drilldown/artifacts/suite.json --manifest-output reviews/task-68/003-timing-drilldown/artifacts/suite-manifest.json`
- result: passed
- status: `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

### `precheck-host-and-table.log`

- command: suite step `precheck-host-and-table`
- result: passed
- key lines:
  - PostgreSQL 18.3, `shared_buffers=128MB`, `maintenance_work_mem=64MB`
  - `corpus_rows=100000`
  - extension `ecaz 0.1.1`
  - AM list includes `ec_spire`

### `create-100k-spire-drilldown-index.log`

- command: suite step `create-100k-spire-drilldown-index`
- result: passed
- key notice:
  - `ec_spire_ambuild_timing index=task68_spire_100k_drilldown_idx phase=complete heap_tuples=100000 scanned_tuples=100000 index_tuples=100000 recursive_fanout=8 setup_ms=0 heap_scan_ms=1223 sample_collect_ms=0 kmeans_ms=490 kmeans_calls=1 assignment_ms=574 recursive_kmeans_ms=1 recursive_kmeans_calls=1 recursive_kmeans_max_level=1 recursive_assignment_ms=0 recursive_routing_initial_children=128 recursive_routing_final_children=8 recursive_routing_iterations=1 draft_ms=19247 draft_total_ms=19248 draft_input_clone_ms=47 draft_pid_alloc_ms=0 draft_recursive_routing_ms=2 draft_route_map_ms=0 draft_leaf_rows_ms=19182 draft_leaf_inputs_ms=10 draft_validation_ms=0 top_graph_ms=937 pq4_training_ms=0 object_store_ms=0 object_store_total_ms=937 publish_ms=8 total_ms=22482`

### `results.jsonl`

- command: emitted by `ecaz bench suite run`
- result: normalized suite result rows

### `drilldown-summary.md`

- command source: manual rollup from the logs above
- result: records the disjoint phase reconciliation, recursive k-means audit, draft drilldown, and estimated caps
