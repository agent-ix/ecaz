# Artifact Manifest: Parallel Concurrent DSM Recall Validation

## pg18_parallel_concurrent_dsm_recall_validation.sql

- head SHA: `63af31814ced4692e20d43ad7389e6ca3dbc327f`
- packet/topic: `650-c1-parallel-concurrent-dsm-recall-validation`
- lane: PG18
- fixture: synthetic 10,000 corpus rows x 64 dimensions plus 100 query rows x 64 dimensions, `ecvector`
- storage format: default TurboQuant current format
- rerank mode: graph scan recall via existing external recall SQL helpers
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/650-c1-parallel-concurrent-dsm-recall-validation/artifacts/pg18_parallel_concurrent_dsm_recall_validation.sql --log-output review/650-c1-parallel-concurrent-dsm-recall-validation/artifacts/pg18_parallel_concurrent_dsm_recall_validation.log`
- timestamp: `2026-04-25T19:09:07-07:00`
- surface: shared corpus/query tables with one serial-built index and one concurrent-DSM-built index

## pg18_parallel_concurrent_dsm_recall_validation.log

- head SHA: `63af31814ced4692e20d43ad7389e6ca3dbc327f`
- packet/topic: `650-c1-parallel-concurrent-dsm-recall-validation`
- lane: PG18
- fixture: synthetic 10,000 corpus rows x 64 dimensions plus 100 query rows x 64 dimensions, `ecvector`
- storage format: default TurboQuant current format
- rerank mode: graph scan recall via existing external recall SQL helpers
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/650-c1-parallel-concurrent-dsm-recall-validation/artifacts/pg18_parallel_concurrent_dsm_recall_validation.sql --log-output review/650-c1-parallel-concurrent-dsm-recall-validation/artifacts/pg18_parallel_concurrent_dsm_recall_validation.log`
- timestamp: `2026-04-25T19:09:07-07:00`
- surface: shared corpus/query tables with one serial-built index and one concurrent-DSM-built index
- key result lines:
  - serial build timing: `serial | requested_workers 0 | workers_launched 0 | heap_tuples 10000 | index_tuples 9998 | heap_ingest_us 316758 | flush_total_us 4609562 | graph_us 4529453 | stage_us 55714 | write_us 21435`
  - serial recall summary at ef=128: `graph_recall_at_10 0.343 | graph_recall_at_100 0.4446 | exact_quantized_recall_at_10 1 | graph_below_exact_queries 90 | worst_exact_gap 10`
  - serial ef sweep: `ef=64 recall@10 0.288`, `ef=128 recall@10 0.343`, `ef=200 recall@10 0.343`
  - concurrent DSM build timing: `concurrent_dsm | requested_workers 4 | workers_launched 4 | heap_tuples 10000 | index_tuples 9998 | heap_ingest_us 163860 | flush_total_us 1723607 | graph_us 1639823 | stage_us 62842 | write_us 19819`
  - concurrent DSM graph workers launched: `4`
  - concurrent DSM recall summary at ef=128: `graph_recall_at_10 0.403 | graph_recall_at_100 0.5094 | exact_quantized_recall_at_10 1 | graph_below_exact_queries 89 | worst_exact_gap 10`
  - concurrent DSM ef sweep: `ef=64 recall@10 0.369`, `ef=128 recall@10 0.403`, `ef=200 recall@10 0.411`
  - ef=128 delta: `serial_graph_recall_at_10 0.343 | concurrent_dsm_graph_recall_at_10 0.403 | recall_delta 0.060000002`
  - index bytes: `serial_index_bytes 2334720 | concurrent_dsm_index_bytes 2334720`
