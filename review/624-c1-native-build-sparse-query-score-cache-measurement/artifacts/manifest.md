# Artifact Manifest

Packet: `624-c1-native-build-sparse-query-score-cache-measurement`

Head SHA: `373a3632cee700aaa34d6a0eb582e0e48a7d8345`

Timestamp: `2026-04-25T10:40:14-07:00` through `2026-04-25T10:40:49-07:00`

Lane: PG18 pgrx local cluster, PostgreSQL 18.3, port 28818.

Fixture:
- Table: `ec_hnsw_sparse_query_score_cache_measure`
- Rows: 10,000
- Dimensions: 64
- Encoded type: `ecvector`
- Storage format: default `turboquant`
- Rerank mode: default, no `build_source_column`
- Index reloptions: `m = 6, ef_construction = 40`

Surface:
- Isolated one-table fixture.
- One `ec_hnsw` index existed at a time.
- Each serial/parallel round dropped the prior index before the next round.
- Serial rounds used `max_parallel_maintenance_workers = 0` and table
  `parallel_workers = 0`.
- Parallel rounds used `max_parallel_maintenance_workers = 4` and table
  `parallel_workers = 4`.
- Phase counters came from `tests.ec_hnsw_debug_last_build_timing()`.

Command:

```sh
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/624-c1-native-build-sparse-query-score-cache-measurement/artifacts/pg18_sparse_query_score_cache_timing.sql --log-output review/624-c1-native-build-sparse-query-score-cache-measurement/artifacts/pg18_sparse_query_score_cache_timing.log
```

Artifacts:
- `pg18_sparse_query_score_cache_timing.sql`: SQL fixture and timing script.
- `pg18_sparse_query_score_cache_timing.log`: raw psql output captured by
  `ecaz dev sql --log-output`.

Key Result Lines:
- Fixture load: `INSERT 0 10000`, `Time: 276.617 ms`
- Serial round 1 create index: `Time: 9086.669 ms (00:09.087)`
- Parallel round 1 create index: `Time: 8882.599 ms (00:08.883)`
- Serial round 2 create index: `Time: 8559.489 ms (00:08.559)`
- Parallel round 2 create index: `Time: 8514.682 ms (00:08.515)`
- Serial round 1 phases: heap ingest `573407 us`, graph `8427858 us`,
  stage `50469 us`, write `21975 us`
- Parallel round 1 phases: heap ingest `399834 us`, begin `2959 us`,
  drain `137035 us`, sort/push `259834 us`, graph `8401233 us`,
  stage `50271 us`, write `19311 us`
- Serial round 2 phases: heap ingest `510251 us`, graph `7967080 us`,
  stage `50940 us`, write `19555 us`
- Parallel round 2 phases: heap ingest `392953 us`, begin `2378 us`,
  drain `130408 us`, sort/push `260161 us`, graph `8040487 us`,
  stage `50169 us`, write `19796 us`
- All measured index sizes: `2334720` bytes
