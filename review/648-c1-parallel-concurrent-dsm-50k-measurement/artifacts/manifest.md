# Artifact Manifest: Parallel Concurrent DSM 50k Measurement

## pg18_parallel_concurrent_dsm_50k_timing.sql

- head SHA: `ded6b9531bafe514ac0fd493105d98de0ce71abb`
- packet/topic: `648-c1-parallel-concurrent-dsm-50k-measurement`
- lane: PG18
- fixture: synthetic 50,000 rows x 64 dimensions, `ecvector`, generated from deterministic `sin`/`cos` expression
- storage format: default TurboQuant current format
- rerank mode: build-only timing; no scan rerank mode
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/648-c1-parallel-concurrent-dsm-50k-measurement/artifacts/pg18_parallel_concurrent_dsm_50k_timing.sql --log-output review/648-c1-parallel-concurrent-dsm-50k-measurement/artifacts/pg18_parallel_concurrent_dsm_50k_timing.log`
- timestamp: `2026-04-25T19:03:11-07:00`
- surface: shared-table fixture with one index built/dropped per path
- notes: SQL fixture used `maintenance_work_mem = '1GB'`, `m = 6`, `ef_construction = 40`; parallel paths requested 4 maintenance workers and table `parallel_workers = 4`.

## pg18_parallel_concurrent_dsm_50k_timing.log

- head SHA: `ded6b9531bafe514ac0fd493105d98de0ce71abb`
- packet/topic: `648-c1-parallel-concurrent-dsm-50k-measurement`
- lane: PG18
- fixture: synthetic 50,000 rows x 64 dimensions, `ecvector`
- storage format: default TurboQuant current format
- rerank mode: build-only timing; no scan rerank mode
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/648-c1-parallel-concurrent-dsm-50k-measurement/artifacts/pg18_parallel_concurrent_dsm_50k_timing.sql --log-output review/648-c1-parallel-concurrent-dsm-50k-measurement/artifacts/pg18_parallel_concurrent_dsm_50k_timing.log`
- timestamp: `2026-04-25T19:03:11-07:00`
- surface: shared-table fixture with one index built/dropped per path
- key result lines:
  - serial wall time: `CREATE INDEX Time: 29582.270 ms`
  - serial debug timing: `serial_50k | requested_workers 0 | workers_launched 0 | heap_tuples 50000 | index_tuples 49982 | heap_ingest_us 1349785 | flush_total_us 28102264 | graph_us 27703505 | stage_us 264451 | write_us 119547`
  - parallel serial-graph wall time: `CREATE INDEX Time: 28463.356 ms`
  - parallel serial-graph debug timing: `parallel_serial_graph_50k | requested_workers 4 | workers_launched 4 | heap_tuples 50000 | index_tuples 49982 | heap_ingest_us 587231 | parallel_begin_us 2880 | parallel_drain_us 433080 | parallel_sort_push_us 151265 | flush_total_us 27853166 | graph_us 27478823 | stage_us 261621 | write_us 97746`
  - parallel concurrent DSM wall time: `CREATE INDEX Time: 11419.608 ms`
  - parallel concurrent DSM debug timing: `parallel_concurrent_dsm_50k | requested_workers 4 | workers_launched 4 | heap_tuples 50000 | index_tuples 49982 | heap_ingest_us 590878 | parallel_begin_us 2284 | parallel_drain_us 430571 | parallel_sort_push_us 158018 | flush_total_us 10805349 | graph_us 10365380 | stage_us 329886 | write_us 100993`
  - graph workers launched: `parallel_concurrent_dsm_graph_workers_launched = 4`
  - index bytes: all paths reported `11624448`
