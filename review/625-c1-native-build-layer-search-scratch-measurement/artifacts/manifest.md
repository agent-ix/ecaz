# Artifact Manifest

Packet: `625-c1-native-build-layer-search-scratch-measurement`

Head SHA: `2515e4b1375b3d57534cba01ac2fb17377528eb3`

Timestamp: `2026-04-25T10:56:40-07:00` through `2026-04-25T10:57:11-07:00`

Lane: PG18 pgrx local cluster, PostgreSQL 18.3, port 28818.

Fixture:
- Table: `ec_hnsw_layer_search_scratch_measure`
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
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/625-c1-native-build-layer-search-scratch-measurement/artifacts/pg18_layer_search_scratch_timing.sql --log-output review/625-c1-native-build-layer-search-scratch-measurement/artifacts/pg18_layer_search_scratch_timing.log
```

Artifacts:
- `pg18_layer_search_scratch_timing.sql`: SQL fixture and timing script.
- `pg18_layer_search_scratch_timing.log`: raw psql output captured by
  `ecaz dev sql --log-output`.

Key Result Lines:
- Fixture load: `INSERT 0 10000`, `Time: 273.179 ms`
- Serial round 1 create index: `Time: 7929.252 ms (00:07.929)`
- Parallel round 1 create index: `Time: 7610.618 ms (00:07.611)`
- Serial round 2 create index: `Time: 7921.307 ms (00:07.921)`
- Parallel round 2 create index: `Time: 7720.030 ms (00:07.720)`
- Serial round 1 phases: heap ingest `548274 us`, graph `7288562 us`,
  stage `50398 us`, write `19851 us`
- Parallel round 1 phases: heap ingest `386427 us`, begin `2763 us`,
  drain `121870 us`, sort/push `261787 us`, graph `7144312 us`,
  stage `50662 us`, write `18804 us`
- Serial round 2 phases: heap ingest `517276 us`, graph `7319489 us`,
  stage `49673 us`, write `19147 us`
- Parallel round 2 phases: heap ingest `410733 us`, begin `2990 us`,
  drain `148440 us`, sort/push `259296 us`, graph `7223762 us`,
  stage `55842 us`, write `19505 us`
- All measured index sizes: `2334720` bytes
