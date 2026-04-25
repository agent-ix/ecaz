# Artifact Manifest

Packet: `626-c1-parallel-index-build-50k-scale-measurement`

Head SHA: `0c0984c5068bb97cc3cf6b74e0977fe0c80aab55`

Timestamp: `2026-04-25T11:12:56-07:00` through `2026-04-25T11:14:43-07:00`

Lane: PG18 pgrx local cluster, PostgreSQL 18.3, port 28818.

Fixture:
- Table: `ec_hnsw_parallel_build_50k_scale_measure`
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
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/626-c1-parallel-index-build-50k-scale-measurement/artifacts/pg18_parallel_build_50k_scale_timing.sql --log-output review/626-c1-parallel-index-build-50k-scale-measurement/artifacts/pg18_parallel_build_50k_scale_timing.log
```

Artifacts:
- `pg18_parallel_build_50k_scale_timing.sql`: SQL fixture and timing script.
- `pg18_parallel_build_50k_scale_timing.log`: raw psql output captured by
  `ecaz dev sql --log-output`.

Key Result Lines:
- Fixture load: `INSERT 0 50000`, `Time: 1357.215 ms (00:01.357)`
- Serial create index: `Time: 53683.587 ms (00:53.684)`
- Parallel create index: `Time: 52391.576 ms (00:52.392)`
- Serial phases: heap ingest `8618142 us`, graph `44626766 us`,
  stage `274579 us`, write `127113 us`
- Parallel phases: heap ingest `7121196 us`, begin `2697 us`,
  drain `412374 us`, sort/push `6706119 us`, graph `44867640 us`,
  stage `257123 us`, write `101357 us`
- Both measured index sizes: `11624448` bytes
