# Artifact Manifest

Head SHA: `b9eba6670c9d5774e87e6e6f5ea42de38a43fefa`

Packet: `review/11102-task29c-vamana-core-profile`

Lane: Task 29c DiskANN build performance tuning.

Fixture: local PG18, real-10k 1536-d corpus copied into isolated prefix
`task29c_phase_profile`, 200 query rows.

Storage format: `ec_diskann` `pq_fastscan` tuple format with persisted binary
sidecar payload.

DiskANN reloptions: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`.

HNSW reference reloptions: `m=32`, `ef_construction=100`,
`build_source_column=source`.

Table model: isolated one-index-per-table prefix `task29c_phase_profile`.

Cache state: warm local run on the existing PG18 scratch instance.

Timestamp: 2026-04-30T13:34:48-07:00

## Artifacts

### `create-index-task29c-vamana-core-profile.log`

Command:

`target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11102-task29c-vamana-core-profile/artifacts/create-index-task29c-vamana-core-profile.log --sql "DROP INDEX IF EXISTS task29c_phase_profile_idx; CREATE INDEX task29c_phase_profile_idx ON task29c_phase_profile_corpus USING ec_diskann (embedding ecvector_diskann_ip_ops) WITH (graph_degree=32, build_list_size=100, alpha=1.2);"`

Key result lines:

- debug/dev-installed extension total: `497.950s`
- `build_persist_ms=478699`
- `core_medoid_ms=10052`
- `core_graph_ms=468545`
- `core_persist_ms=100`
- `write_pages_ms=41`
- pass 0 elapsed: `139.692s`
- pass 1 elapsed: `328.836s`

This run is retained as the debug/dev-profile caveat for packet `11101`.

### `create-index-task29c-vamana-core-profile-release-extension.log`

Command:

`target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11102-task29c-vamana-core-profile/artifacts/create-index-task29c-vamana-core-profile-release-extension.log --sql "DROP INDEX IF EXISTS task29c_phase_profile_idx; CREATE INDEX task29c_phase_profile_idx ON task29c_phase_profile_corpus USING ec_diskann (embedding ecvector_diskann_ip_ops) WITH (graph_degree=32, build_list_size=100, alpha=1.2);"`

Key result lines:

- release-installed extension total: `79.238s`
- `heap_scan_ms=1261`
- `training_ms=130`
- `sidecar_setup_ms=2`
- `payload_derivation_ms=293`
- `build_persist_ms=77485`
- `core_medoid_ms=1566`
- `core_graph_ms=75903`
- `core_persist_ms=14`
- `write_pages_ms=59`
- pass 0 elapsed: `21.539s`
- pass 1 elapsed: `54.363s`

Pass 1 details cited by `request.md`:

- `greedy_search_ms=21015`
- `robust_prune_ms=6886`
- `backlink_ms=9876`
- `greedy_distance_calls=12864074`
- `robust_prune_distance_calls=17837238`
- `backlink_distance_calls=614031`

### `create-index-task29c-hnsw-reference-release-extension.log`

Command:

`target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11102-task29c-vamana-core-profile/artifacts/create-index-task29c-hnsw-reference-release-extension.log --sql "DROP INDEX IF EXISTS task29c_hnsw_reference_idx; CREATE INDEX task29c_hnsw_reference_idx ON task29c_phase_profile_corpus USING ec_hnsw (embedding ecvector_ip_ops) WITH (m=32, ef_construction=100, build_source_column=source);"`

Key result: raw DDL path completed with `CREATE INDEX`. This artifact did not
include wall-clock timing, so the loader path below is the timing source of
truth for HNSW.

### `drop-task29c-hnsw-loader-index.log`

Command:

`target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11102-task29c-vamana-core-profile/artifacts/drop-task29c-hnsw-loader-index.log --sql "DROP INDEX IF EXISTS task29c_phase_profile_m32_idx;"`

Key result: old loader-managed HNSW reference index was absent.

### `load-task29c-hnsw-reference-release-extension.log`

Command:

`target/release/ecaz --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11102-task29c-vamana-core-profile/artifacts/load-task29c-hnsw-reference-release-extension.log corpus load --prefix task29c_phase_profile --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --profile ec_hnsw --m 32 --ef-construction 100 --allow-manifest-mismatch`

Key result lines:

- existing corpus table reused: `10000 rows`
- existing queries table reused: `200 rows`
- built `task29c_phase_profile_m32_idx` in `5.23s`
- completed prefix in `7.24s`

### `size-task29c-diskann-hnsw-release-extension.log`

Command:

`target/release/ecaz --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11102-task29c-vamana-core-profile/artifacts/size-task29c-diskann-hnsw-release-extension.log --sql "SELECT relname, pg_size_pretty(pg_relation_size(oid)) AS relation_size, pg_relation_size(oid) AS bytes FROM pg_class WHERE relname IN ('task29c_phase_profile_idx', 'task29c_phase_profile_m32_idx', 'task29c_hnsw_reference_idx') ORDER BY relname;"`

Key result rows:

- `task29c_phase_profile_idx`: `4824 kB`, `4939776` bytes
- `task29c_phase_profile_m32_idx`: `14 MB`, `15130624` bytes
- `task29c_hnsw_reference_idx`: `14 MB`, `15130624` bytes
