# Artifact Manifest

Packet: `627-c1-build-state-dedup-index-measurement`

Head SHA: `cc23d286d9ac0015c385ad702bb43f5ec7579efa`

Timestamp: `2026-04-25T11:37:19-07:00` through `2026-04-25T11:38:55-07:00`

Lane: PG18 pgrx local cluster, PostgreSQL 18.3, port 28818.

Fixture:
- Table: `ec_hnsw_build_state_dedup_50k_measure`
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
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/627-c1-build-state-dedup-index-measurement/artifacts/pg18_build_state_dedup_index_50k_timing.sql --log-output review/627-c1-build-state-dedup-index-measurement/artifacts/pg18_build_state_dedup_index_50k_timing.log
```

Artifacts:
- `pg18_build_state_dedup_index_50k_timing.sql`: SQL fixture and timing script.
- `pg18_build_state_dedup_index_50k_timing.log`: raw psql output captured by
  `ecaz dev sql --log-output`.

Key Result Lines:
- Fixture load: `INSERT 0 50000`, `Time: 1436.506 ms (00:01.437)`
- Serial create index: `Time: 48926.700 ms (00:48.927)`
- Parallel create index: `Time: 46103.259 ms (00:46.103)`
- Serial phases: heap ingest `1471241 us`, graph `47023806 us`,
  stage `273917 us`, write `105676 us`
- Parallel phases: heap ingest `604500 us`, begin `2943 us`,
  drain `444276 us`, sort/push `157275 us`, graph `45105472 us`,
  stage `254752 us`, write `100398 us`
- Both measured index sizes: `11624448` bytes
