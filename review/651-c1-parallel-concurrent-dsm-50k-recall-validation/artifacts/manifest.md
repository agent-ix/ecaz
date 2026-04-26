# Artifact Manifest: Parallel Concurrent DSM 50k Recall Validation

## pg18_parallel_concurrent_dsm_50k_recall_validation.sql

- head SHA: `2f4de5293820b41b5c702a829478c7144eca070c`
- packet/topic: `651-c1-parallel-concurrent-dsm-50k-recall-validation`
- lane: PG18
- fixture: synthetic 50,000 corpus rows x 64 dimensions plus 50 query rows x 64 dimensions, `ecvector`
- storage format: default TurboQuant current format
- rerank mode: graph scan recall via existing external recall SQL helpers
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/651-c1-parallel-concurrent-dsm-50k-recall-validation/artifacts/pg18_parallel_concurrent_dsm_50k_recall_validation.sql --log-output review/651-c1-parallel-concurrent-dsm-50k-recall-validation/artifacts/pg18_parallel_concurrent_dsm_50k_recall_validation.log`
- timestamp: `2026-04-25T19:24:07-07:00`
- surface: shared corpus/query tables with one serial-built index and one concurrent-DSM-built index

## pg18_parallel_concurrent_dsm_50k_recall_validation.log

- head SHA: `2f4de5293820b41b5c702a829478c7144eca070c`
- packet/topic: `651-c1-parallel-concurrent-dsm-50k-recall-validation`
- lane: PG18
- fixture: synthetic 50,000 corpus rows x 64 dimensions plus 50 query rows x 64 dimensions, `ecvector`
- storage format: default TurboQuant current format
- rerank mode: graph scan recall via existing external recall SQL helpers
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/651-c1-parallel-concurrent-dsm-50k-recall-validation/artifacts/pg18_parallel_concurrent_dsm_50k_recall_validation.sql --log-output review/651-c1-parallel-concurrent-dsm-50k-recall-validation/artifacts/pg18_parallel_concurrent_dsm_50k_recall_validation.log`
- timestamp: `2026-04-25T19:24:07-07:00`
- surface: shared corpus/query tables with one serial-built index and one concurrent-DSM-built index
- key result lines:
  - serial build timing: `serial | requested_workers 0 | workers_launched 0 | heap_tuples 50000 | index_tuples 49982 | heap_ingest_us 1347700 | flush_total_us 27745645 | graph_us 27277551 | stage_us 270089 | write_us 183913`
  - serial recall summary at ef=128: `graph_recall_at_10 0.088 | graph_recall_at_100 0.2444 | exact_quantized_recall_at_10 1 | graph_below_exact_queries 49 | worst_exact_gap 10`
  - concurrent DSM build timing: `concurrent_dsm | requested_workers 4 | workers_launched 4 | heap_tuples 50000 | index_tuples 49982 | heap_ingest_us 579369 | flush_total_us 11083147 | graph_us 10577078 | stage_us 325105 | write_us 171050`
  - concurrent DSM graph workers launched: `4`
  - concurrent DSM recall summary at ef=128: `graph_recall_at_10 0.154 | graph_recall_at_100 0.3752 | exact_quantized_recall_at_10 1 | graph_below_exact_queries 47 | worst_exact_gap 10`
  - ef=128 delta: `serial_graph_recall_at_10 0.088 | concurrent_dsm_graph_recall_at_10 0.154 | recall_delta 0.066 | serial_graph_recall_at_100 0.2444 | concurrent_dsm_graph_recall_at_100 0.3752 | recall_100_delta 0.13080001`
  - index bytes: `serial_index_bytes 11616256 | concurrent_dsm_index_bytes 11616256`
