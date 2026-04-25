# Artifact Manifest

Packet: `630-c1-native-source-score-workspace-measurement`

Head SHA: `7f2d03e8069c29afc4ed81a2470cff455e75d00c`

Timestamp: `2026-04-25T13:05:54-07:00` through `2026-04-25T13:06:54-07:00`

Lane: PG18 pgrx local cluster, PostgreSQL 18.3, port 28818.

Fixture:
- Table: `ec_hnsw_native_source_score_workspace_50k_measure`
- Rows: 50,000
- Dimensions: 64
- Encoded type: `ecvector`
- Storage format: default `turboquant`
- Rerank mode: default, no `build_source_column`
- Index reloptions: `m = 6, ef_construction = 40`

Surface:
- Isolated one-table fixture.
- One `ec_hnsw` index existed at a time.
- Serial run dropped its index before the parallel run.
- Serial run used `max_parallel_maintenance_workers = 0` and table
  `parallel_workers = 0`.
- Parallel run used `max_parallel_maintenance_workers = 4` and table
  `parallel_workers = 4`.
- Phase counters came from `tests.ec_hnsw_debug_last_build_timing()`.

Command:

```sh
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/630-c1-native-source-score-workspace-measurement/artifacts/pg18_native_source_score_workspace_50k_timing.sql --log-output review/630-c1-native-source-score-workspace-measurement/artifacts/pg18_native_source_score_workspace_50k_timing.log
```

Artifacts:
- `pg18_native_source_score_workspace_50k_timing.sql`: SQL fixture and timing script.
- `pg18_native_source_score_workspace_50k_timing.log`: raw psql output captured by
  `ecaz dev sql --log-output`.

Key Result Lines:
- Fixture load: `INSERT 0 50000`, `Time: 1416.637 ms (00:01.417)`
- Serial create index: `Time: 29692.277 ms (00:29.692)`
- Parallel create index: `Time: 28794.693 ms (00:28.795)`
- Serial phases: heap ingest `1362500 us`, graph `27886740 us`,
  stage `262886 us`, write `122841 us`
- Parallel phases: heap ingest `584992 us`, begin `2735 us`,
  drain `430056 us`, sort/push `152194 us`, graph `27811377 us`,
  stage `253406 us`, write `104184 us`
- Both measured index sizes: `11624448` bytes
