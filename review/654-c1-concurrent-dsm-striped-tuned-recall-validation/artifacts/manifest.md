# Artifact Manifest: Concurrent DSM striped tuned recall validation

## `pg18_concurrent_dsm_striped_tuned_recall_validation.sql`

- head SHA: `2b756518d463f7b8ad3a72b7d0842139a9942b05`
- packet/topic: `654-c1-concurrent-dsm-striped-tuned-recall-validation`
- lane: PG18 tuned serial vs concurrent DSM recall validation after striped insertion scheduling
- fixture: 10,000 corpus rows x 64 dimensions; 100 query rows x 64 dimensions; generated deterministic SQL fixture
- storage format: default current-format `ec_hnsw` / `ecvector` index
- rerank mode: default
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/654-c1-concurrent-dsm-striped-tuned-recall-validation/artifacts/pg18_concurrent_dsm_striped_tuned_recall_validation.sql --log-output review/654-c1-concurrent-dsm-striped-tuned-recall-validation/artifacts/pg18_concurrent_dsm_striped_tuned_recall_validation.log`
- timestamp: 2026-04-25 20:51:07-07 to 2026-04-25 20:53:08-07
- surface isolation: shared corpus/query tables, one serial index and one concurrent DSM index on the same table
- key result lines cited by request:
  - serial build timing: `serial | 0 requested | 0 launched | 10000 heap_tuples | 9998 index_tuples | flush_total_us 13294592 | graph_us 13146880 | stage_us 97121 | write_us 47190`
  - serial recall: `ef=128 recall@10 0.505`, `ef=200 recall@10 0.534`, `ef=400 recall@10 0.599`
  - concurrent DSM build timing: `concurrent_dsm | 4 requested | 4 launched | 10000 heap_tuples | 9998 index_tuples | flush_total_us 5073365 | graph_us 4904564 | stage_us 137323 | write_us 30310`
  - concurrent DSM workers: `concurrent_dsm_graph_workers_launched = 4`
  - concurrent DSM recall: `ef=128 recall@10 0.552`, `ef=200 recall@10 0.558`, `ef=400 recall@10 0.582`
  - ef=200 delta row: `serial_graph_recall_at_10 0.534`, `concurrent_dsm_graph_recall_at_10 0.558`, `recall_delta 0.024000049`
  - index sizes: `serial_index_bytes 3563520`, `concurrent_dsm_index_bytes 3563520`

## `pg18_concurrent_dsm_striped_tuned_recall_validation.log`

- head SHA: `2b756518d463f7b8ad3a72b7d0842139a9942b05`
- packet/topic: `654-c1-concurrent-dsm-striped-tuned-recall-validation`
- lane / fixture / storage format / rerank mode: same as SQL artifact above
- command used: same as SQL artifact above
- timestamp: 2026-04-25 20:51:07-07 to 2026-04-25 20:53:08-07
- surface isolation: shared corpus/query tables, one serial index and one concurrent DSM index on the same table
- key result lines: see SQL artifact section above
