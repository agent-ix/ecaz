# Artifact Manifest: Concurrent DSM striped 50k tuned recall validation

## `pg18_concurrent_dsm_striped_50k_tuned_recall_validation.sql`

- head SHA: `63dcc92ffb2ee87e9a9220f8ad654af02662cf04`
- packet/topic: `655-c1-concurrent-dsm-striped-50k-tuned-recall-validation`
- lane: PG18 50k tuned serial vs striped concurrent DSM recall validation
- fixture: 50,000 corpus rows x 64 dimensions; 50 query rows x 64 dimensions; generated deterministic SQL fixture
- storage format: default current-format `ec_hnsw` / `ecvector` index
- rerank mode: default
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/655-c1-concurrent-dsm-striped-50k-tuned-recall-validation/artifacts/pg18_concurrent_dsm_striped_50k_tuned_recall_validation.sql --log-output review/655-c1-concurrent-dsm-striped-50k-tuned-recall-validation/artifacts/pg18_concurrent_dsm_striped_50k_tuned_recall_validation.log`
- timestamp: 2026-04-25 20:55:42-07 to 2026-04-25 21:01:07-07
- surface isolation: shared corpus/query tables, one serial index and one striped concurrent DSM index on the same table
- key result lines cited by request:
  - serial build timing: `serial | 0 requested | 0 launched | 50000 heap_tuples | 49982 index_tuples | flush_total_us 81740686 | graph_us 80949068 | stage_us 467232 | write_us 306623`
  - serial recall: `ef=128 recall@10 0.234`, `ef=200 recall@10 0.246`, `ef=400 recall@10 0.256`
  - striped concurrent DSM build timing: `concurrent_dsm_striped | 4 requested | 4 launched | 50000 heap_tuples | 49982 index_tuples | flush_total_us 29889276 | graph_us 28786109 | stage_us 750081 | write_us 340464`
  - concurrent DSM workers: `concurrent_dsm_graph_workers_launched = 4`
  - striped concurrent DSM recall: `ef=128 recall@10 0.256`, `ef=200 recall@10 0.256`, `ef=400 recall@10 0.256`
  - ef=200 delta row: `serial_graph_recall_at_10 0.246`, `concurrent_dsm_graph_recall_at_10 0.256`, `recall_delta 0.010000005`
  - index sizes: `serial_index_bytes 17752064`, `concurrent_dsm_index_bytes 17752064`

## `pg18_concurrent_dsm_striped_50k_tuned_recall_validation.log`

- head SHA: `63dcc92ffb2ee87e9a9220f8ad654af02662cf04`
- packet/topic: `655-c1-concurrent-dsm-striped-50k-tuned-recall-validation`
- lane / fixture / storage format / rerank mode: same as SQL artifact above
- command used: same as SQL artifact above
- timestamp: 2026-04-25 20:55:42-07 to 2026-04-25 21:01:07-07
- surface isolation: shared corpus/query tables, one serial index and one striped concurrent DSM index on the same table
- key result lines: see SQL artifact section above
