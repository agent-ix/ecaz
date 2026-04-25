# Artifact Manifest

Packet: `628-c1-native-graph-scratch-cache-measurement`

Head SHA: `0be01d11b079b538f1858edff91d564916c2e14f`

Timestamp: `2026-04-25T11:55:40-07:00` through `2026-04-25T11:56:43-07:00`

Lane: PG18 pgrx local cluster, PostgreSQL 18.3, port 28818.

Fixture:
- Table: `ec_hnsw_native_graph_scratch_cache_50k_measure`
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
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/628-c1-native-graph-scratch-cache-measurement/artifacts/pg18_native_graph_scratch_cache_50k_timing.sql --log-output review/628-c1-native-graph-scratch-cache-measurement/artifacts/pg18_native_graph_scratch_cache_50k_timing.log
```

Artifacts:
- `pg18_native_graph_scratch_cache_50k_timing.sql`: SQL fixture and timing script.
- `pg18_native_graph_scratch_cache_50k_timing.log`: raw psql output captured by
  `ecaz dev sql --log-output`.

Key Result Lines:
- Fixture load: `INSERT 0 50000`, `Time: 1397.614 ms (00:01.398)`
- Serial create index: `Time: 30333.870 ms (00:30.334)`
- Parallel create index: `Time: 30845.279 ms (00:30.845)`
- Serial phases: heap ingest `1324566 us`, graph `28556153 us`,
  stage `263868 us`, write `134544 us`
- Parallel phases: heap ingest `636071 us`, begin `2861 us`,
  drain `451520 us`, sort/push `181685 us`, graph `29816052 us`,
  stage `256141 us`, write `103613 us`
- Both measured index sizes: `11624448` bytes
