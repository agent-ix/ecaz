# Artifact Manifest

Packet: `631-c1-native-neighbor-flatten-measurement`

Head SHA: `6815332567bafa9a0f6840397512dbf00c1d349b`

Timestamp: `2026-04-25T13:21:42-07:00` through `2026-04-25T13:22:41-07:00`

Lane: PG18 pgrx local cluster, PostgreSQL 18.3, port 28818.

Fixture:
- Table: `ec_hnsw_native_neighbor_flatten_50k_measure`
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
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/631-c1-native-neighbor-flatten-measurement/artifacts/pg18_native_neighbor_flatten_50k_timing.sql --log-output review/631-c1-native-neighbor-flatten-measurement/artifacts/pg18_native_neighbor_flatten_50k_timing.log
```

Artifacts:
- `pg18_native_neighbor_flatten_50k_timing.sql`: SQL fixture and timing script.
- `pg18_native_neighbor_flatten_50k_timing.log`: raw psql output captured by
  `ecaz dev sql --log-output`.

Key Result Lines:
- Fixture load: `INSERT 0 50000`, `Time: 1467.559 ms (00:01.468)`
- Serial create index: `Time: 29432.464 ms (00:29.432)`
- Parallel create index: `Time: 28119.478 ms (00:28.119)`
- Serial phases: heap ingest `1344187 us`, graph `27560080 us`,
  stage `261799 us`, write `200027 us`
- Parallel phases: heap ingest `656217 us`, begin `2994 us`,
  drain `478669 us`, sort/push `174548 us`, graph `27029382 us`,
  stage `255759 us`, write `133604 us`
- Both measured index sizes: `11624448` bytes
