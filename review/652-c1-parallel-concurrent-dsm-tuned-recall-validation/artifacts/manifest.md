# Artifact Manifest: Parallel Concurrent DSM Tuned Recall Validation

## pg18_parallel_concurrent_dsm_tuned_recall_validation.sql

- head SHA: `dbcc8a755ce9941ed508206e6e776be62fb459f1`
- packet/topic: `652-c1-parallel-concurrent-dsm-tuned-recall-validation`
- lane: PG18
- fixture: synthetic 10,000 corpus rows x 64 dimensions plus 100 query rows x 64 dimensions, `ecvector`
- storage format: default TurboQuant current format
- rerank mode: graph scan recall via existing external recall SQL helpers
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/652-c1-parallel-concurrent-dsm-tuned-recall-validation/artifacts/pg18_parallel_concurrent_dsm_tuned_recall_validation.sql --log-output review/652-c1-parallel-concurrent-dsm-tuned-recall-validation/artifacts/pg18_parallel_concurrent_dsm_tuned_recall_validation.log`
- timestamp: `2026-04-25T20:33:10-07:00`
- surface: shared corpus/query tables with one serial-built index and one concurrent-DSM-built index
- index settings: `m = 16`, `ef_construction = 128`; sweep at `ef_search = 128, 200, 400`

## pg18_parallel_concurrent_dsm_tuned_recall_validation.log

- head SHA: `dbcc8a755ce9941ed508206e6e776be62fb459f1`
- packet/topic: `652-c1-parallel-concurrent-dsm-tuned-recall-validation`
- lane: PG18
- fixture: synthetic 10,000 corpus rows x 64 dimensions plus 100 query rows x 64 dimensions, `ecvector`
- storage format: default TurboQuant current format
- rerank mode: graph scan recall via existing external recall SQL helpers
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/652-c1-parallel-concurrent-dsm-tuned-recall-validation/artifacts/pg18_parallel_concurrent_dsm_tuned_recall_validation.sql --log-output review/652-c1-parallel-concurrent-dsm-tuned-recall-validation/artifacts/pg18_parallel_concurrent_dsm_tuned_recall_validation.log`
- timestamp: `2026-04-25T20:33:10-07:00`
- surface: shared corpus/query tables with one serial-built index and one concurrent-DSM-built index
- key result lines:
  - serial build timing: `serial | requested_workers 0 | workers_launched 0 | heap_tuples 10000 | index_tuples 9998 | heap_ingest_us 308759 | flush_total_us 13893046 | graph_us 13767101 | stage_us 83780 | write_us 38469`
  - serial ef sweep: `ef=128 recall@10 0.505`, `ef=200 recall@10 0.534`, `ef=400 recall@10 0.599`
  - concurrent DSM build timing: `concurrent_dsm | requested_workers 4 | workers_launched 4 | heap_tuples 10000 | index_tuples 9998 | heap_ingest_us 173407 | flush_total_us 5171562 | graph_us 4988037 | stage_us 140243 | write_us 42041`
  - concurrent DSM graph workers launched: `4`
  - concurrent DSM ef sweep: `ef=128 recall@10 0.528`, `ef=200 recall@10 0.538`, `ef=400 recall@10 0.538`
  - ef=200 delta: `serial_graph_recall_at_10 0.534 | concurrent_dsm_graph_recall_at_10 0.538 | recall_delta 0.004000008 | serial_graph_recall_at_100 0.7189 | concurrent_dsm_graph_recall_at_100 0.714 | recall_100_delta -0.0049000382`
  - index bytes: `serial_index_bytes 3563520 | concurrent_dsm_index_bytes 3563520`
