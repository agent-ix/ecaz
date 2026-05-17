\timing on

SELECT
  c.relname,
  c.reltuples::bigint AS estimated_rows
FROM pg_class c
WHERE c.relname IN (
  'ec_hnsw_real_ann_benchmarks_anchor_corpus',
  'ec_hnsw_real_ann_benchmarks_anchor_queries'
)
ORDER BY c.relname;

SELECT
  table_name,
  column_name,
  data_type,
  udt_name
FROM information_schema.columns
WHERE table_name IN (
  'ec_hnsw_real_ann_benchmarks_anchor_corpus',
  'ec_hnsw_real_ann_benchmarks_anchor_queries'
)
ORDER BY table_name, ordinal_position;
