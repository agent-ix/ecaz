# Task 68 Packet 002 Artifact Manifest

- head SHA: `c580065eacec32170931e0a44be5b041f09229cd`
- task bucket: `reviews/task-68/002-characterization`
- timestamp: `2026-05-30T04:06:40Z`
- lane: Task 68 SPIRE build performance characterization
- fixture/storage/rerank: M5 DBpedia 10k and 100k fixtures, `storage_format=turboquant`, `rerank_width=25`
- isolated one-index-per-table or shared-table surface: one index per table for measured `CREATE INDEX` steps

## Artifacts

### `suite.json`

- command source: checked-in `ecaz bench suite` config
- result: covers host precheck, extension setup, cleanup, 10k load, 10k measured create-index, 100k load, and 100k measured create-index

### `suite-dry-run-manifest.json`

- command: `/Users/peter/.cargo/bin/ecaz --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-68/002-characterization/artifacts/suite.json --dry-run --manifest-output reviews/task-68/002-characterization/artifacts/suite-dry-run-manifest.json`
- result: passed

### `suite-manifest.json`

- command: `/Users/peter/.cargo/bin/ecaz --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-68/002-characterization/artifacts/suite.json --manifest-output reviews/task-68/002-characterization/artifacts/suite-manifest.json`
- result: passed
- status: `completed=7 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

### `results.jsonl`

- command: emitted by `ecaz bench suite run`
- result: normalized suite result rows

### `create-task68-database-precheck.log`

- command: `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --raw --sql "SELECT datname FROM pg_database WHERE datname = 'task68_spire_char';" --log-output reviews/task-68/002-characterization/artifacts/create-task68-database-precheck.log`
- result: database did not exist before this packet

### `create-task68-database.log`

- command: `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --raw --sql "CREATE DATABASE task68_spire_char;" --log-output reviews/task-68/002-characterization/artifacts/create-task68-database.log`
- result: passed
- key line: `CREATE DATABASE`

### `precheck-host.log`

- command: suite step `precheck-host`
- result: passed
- key line: PostgreSQL 18.3, `shared_buffers=128MB`, `maintenance_work_mem=64MB`, `effective_cache_size=4GB`

### `setup-extension.log`

- command: suite step `setup-extension`
- result: passed
- key lines:
  - `CREATE EXTENSION`
  - `ecaz | 0.1.1`
  - AM list includes `ec_spire`

### `cleanup-prior-task68-relations.log`

- command: suite step `cleanup-prior-task68-relations`
- result: passed

### `load-10k-spire-turboquant.log`

- command: suite step `load-10k-spire-turboquant`
- result: passed
- key lines:
  - `[loader] built task68_spire_10k_load_turboquant_idx in 805.46ms`
  - `[loader] completed prefix task68_spire_10k_load in 3.39s`

### `create-10k-spire-profile-index.log`

- command: suite step `create-10k-spire-profile-index`
- result: passed
- key notice:
  - `ec_spire_ambuild_timing index=task68_spire_10k_profile_idx phase=complete heap_tuples=10000 scanned_tuples=10000 index_tuples=10000 recursive_fanout=8 setup_ms=0 heap_scan_ms=119 sample_collect_ms=0 kmeans_ms=147 kmeans_calls=1 assignment_ms=15 recursive_kmeans_ms=0 recursive_kmeans_calls=1 recursive_kmeans_max_level=1 recursive_assignment_ms=0 draft_ms=499 top_graph_ms=24 pq4_training_ms=0 object_store_ms=24 publish_ms=0 total_ms=806`

### `load-100k-spire-turboquant.log`

- command: suite step `load-100k-spire-turboquant`
- result: passed
- key lines:
  - `[loader] built task68_spire_100k_load_turboquant_idx in 21.80s`
  - `[loader] completed prefix task68_spire_100k_load in 46.26s`

### `create-100k-spire-profile-index.log`

- command: suite step `create-100k-spire-profile-index`
- result: passed
- key notice:
  - `ec_spire_ambuild_timing index=task68_spire_100k_profile_idx phase=complete heap_tuples=100000 scanned_tuples=100000 index_tuples=100000 recursive_fanout=8 setup_ms=0 heap_scan_ms=1220 sample_collect_ms=0 kmeans_ms=486 kmeans_calls=1 assignment_ms=570 recursive_kmeans_ms=2 recursive_kmeans_calls=1 recursive_kmeans_max_level=1 recursive_assignment_ms=0 draft_ms=19282 top_graph_ms=252 pq4_training_ms=0 object_store_ms=252 publish_ms=1 total_ms=21814`

### `characterization-summary.md`

- command source: manual rollup from the suite logs above
- result: records phase split, k-means rollup, static call audit reference, and ranked P0 recommendations
