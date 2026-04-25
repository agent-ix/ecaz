# Artifact Manifest

Packet: `620-c1-parallel-index-build-pg18-measurement`

Head SHA: `4305d77f1cfe37fa5f4e91729fa0c0b7aa41950c`

Timestamp: `2026-04-25T09:06:28-07:00` through `2026-04-25T09:07:05-07:00`

Lane: PG18 pgrx local cluster, PostgreSQL 18.3, port 28818.

Fixture:
- Table: `ec_hnsw_parallel_build_measure`
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

Command:

```sh
script --quiet --return --flush --log-out review/620-c1-parallel-index-build-pg18-measurement/artifacts/pg18_parallel_build_timing.log --command "/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d postgres -v ON_ERROR_STOP=1 -f review/620-c1-parallel-index-build-pg18-measurement/artifacts/pg18_parallel_build_timing.sql"
```

Artifacts:
- `pg18_parallel_build_timing.sql`: SQL fixture and timing script.
- `pg18_parallel_build_timing.log`: raw psql output captured by
  `script --log-out`.

Key Result Lines:
- Fixture load: `INSERT 0 10000`, `Time: 332.608 ms`
- Serial round 1 create index: `Time: 9139.351 ms (00:09.139)`
- Parallel round 1 create index: `Time: 9633.868 ms (00:09.634)`
- Serial round 2 create index: `Time: 9011.378 ms (00:09.011)`
- Parallel round 2 create index: `Time: 9350.732 ms (00:09.351)`
- All measured index sizes: `2334720` bytes
