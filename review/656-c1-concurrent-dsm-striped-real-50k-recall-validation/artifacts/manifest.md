# Artifact Manifest: Concurrent DSM striped real 50k source-scored blocker

## `pg18_concurrent_dsm_striped_real_50k_recall_validation.sql`

- head SHA: `85e72b40b1694d7df6c211777b3d16b9234e3ba6`
- packet/topic: `656-c1-concurrent-dsm-striped-real-50k-recall-validation`
- lane: PG18 real 50k source-scored serial baseline plus striped concurrent DSM sidecar attempt
- fixture: `/home/peter/dev/datasets/tqhnsw_real_50k`, 50,000 corpus rows x 1536 dimensions; 1,000 query rows x 1536 dimensions; 10-query subset for this smoke attempt
- storage format: default current-format `ec_hnsw` / `ecvector` index
- rerank mode: default
- command used: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/656-c1-concurrent-dsm-striped-real-50k-recall-validation/artifacts/pg18_concurrent_dsm_striped_real_50k_recall_validation.sql --log-output review/656-c1-concurrent-dsm-striped-real-50k-recall-validation/artifacts/pg18_concurrent_dsm_striped_real_50k_recall_validation.log`
- timestamp: 2026-04-25 22:07:39-07
- surface isolation: shared real corpus table; existing serial source-scored m16 index; attempted striped concurrent DSM source-scored sidecar index
- outcome: failed during concurrent sidecar `CREATE INDEX`
- key result lines cited by request:
  - fixture rows: `corpus_rows 50000`, `query_rows 1000`, `query_subset_rows 10`
  - serial source-scored index size: `serial_index_bytes 68280320`
  - serial source-scored recall at `ef_search=200`: `graph_recall_at_10 0.91`, `graph_recall_at_100 0.762`, `exact_quantized_recall_at_10 1`, `ndcg_at_10 0.947258`
  - concurrent sidecar failure: `ERROR: concurrent DSM graph assembly does not support source-scored builds yet`

## `pg18_concurrent_dsm_striped_real_50k_recall_validation.log`

- head SHA: `85e72b40b1694d7df6c211777b3d16b9234e3ba6`
- packet/topic: `656-c1-concurrent-dsm-striped-real-50k-recall-validation`
- lane / fixture / storage format / rerank mode: same as SQL artifact above
- command used: same as SQL artifact above
- timestamp: 2026-04-25 22:07:39-07
- surface isolation: shared real corpus table; existing serial source-scored m16 index; attempted striped concurrent DSM source-scored sidecar index
- outcome and key result lines: see SQL artifact section above
