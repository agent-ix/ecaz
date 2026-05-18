SELECT
  c.relname,
  c.reltuples::bigint AS estimated_rows,
  pg_relation_size(c.oid) AS heap_bytes,
  pg_total_relation_size(c.oid) AS total_bytes
FROM pg_class c
WHERE c.relname IN (
  'task28_a9_100k_hnsw_corpus',
  'task28_a9_100k_hnsw_queries',
  'task28_a9_100k_ivf_corpus',
  'task28_a9_100k_ivf_queries',
  'ec_hnsw_real_ann_benchmarks_anchor_corpus',
  'ec_hnsw_real_ann_benchmarks_anchor_queries'
)
ORDER BY c.relname;

SELECT
  i.tablename,
  i.indexname,
  c.reltuples::bigint AS estimated_rows,
  pg_relation_size(c.oid) AS index_bytes,
  pg_size_pretty(pg_relation_size(c.oid)) AS index_size,
  i.indexdef
FROM pg_indexes i
JOIN pg_class c ON c.relname = i.indexname
WHERE i.tablename IN (
  'task28_a9_100k_hnsw_corpus',
  'task28_a9_100k_ivf_corpus',
  'ec_hnsw_real_ann_benchmarks_anchor_corpus'
)
ORDER BY i.tablename, i.indexname;
