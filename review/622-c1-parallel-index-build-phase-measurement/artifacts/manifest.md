# Artifact Manifest

Packet: `622-c1-parallel-index-build-phase-measurement`

Head SHA: `46922960138b4aeb6fbe90d626e13b822d63595e`

Timestamp: `2026-04-25T10:07:34-07:00` through `2026-04-25T10:08:10-07:00`

Lane: PG18 pgrx local cluster, PostgreSQL 18.3, port 28818.

Fixture:
- Table: `ec_hnsw_parallel_build_phase_measure`
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
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/622-c1-parallel-index-build-phase-measurement/artifacts/pg18_parallel_build_phase_timing.sql --log-output review/622-c1-parallel-index-build-phase-measurement/artifacts/pg18_parallel_build_phase_timing.log
```

Artifacts:
- `pg18_parallel_build_phase_timing.sql`: SQL fixture and timing script.
- `pg18_parallel_build_phase_timing.log`: raw psql output captured by
  `ecaz dev sql --log-output`.

Key Result Lines:
- Fixture load: `INSERT 0 10000`, `Time: 284.156 ms`
- Serial round 1 create index: `Time: 9033.476 ms (00:09.033)`
- Parallel round 1 create index: `Time: 8676.363 ms (00:08.676)`
- Serial round 2 create index: `Time: 8992.727 ms (00:08.993)`
- Parallel round 2 create index: `Time: 8712.693 ms (00:08.713)`
- Serial round 1 phases: heap ingest `561114 us`, graph `8380357 us`,
  stage `49211 us`, write `20258 us`
- Parallel round 1 phases: heap ingest `379885 us`, begin `2517 us`,
  drain `118104 us`, sort/push `259237 us`, graph `8213592 us`,
  stage `47560 us`, write `18815 us`
- Serial round 2 phases: heap ingest `514021 us`, graph `8388182 us`,
  stage `52572 us`, write `19616 us`
- Parallel round 2 phases: heap ingest `415836 us`, begin `3043 us`,
  drain `158982 us`, sort/push `253804 us`, graph `8214958 us`,
  stage `49054 us`, write `19323 us`
- All measured index sizes: `2334720` bytes
