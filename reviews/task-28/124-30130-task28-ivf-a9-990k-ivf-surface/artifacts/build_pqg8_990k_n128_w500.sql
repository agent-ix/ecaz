\pset pager off
\timing on

DROP TABLE IF EXISTS task28_ivf_pqg990k_g8_n128_corpus CASCADE;
DROP TABLE IF EXISTS task28_ivf_pqg990k_g8_n128_queries CASCADE;

CREATE TABLE task28_ivf_pqg990k_g8_n128_corpus AS
SELECT * FROM ec_hnsw_real_ann_benchmarks_anchor_corpus;

ALTER TABLE task28_ivf_pqg990k_g8_n128_corpus ADD PRIMARY KEY (id);

CREATE TABLE task28_ivf_pqg990k_g8_n128_queries AS
SELECT * FROM ec_hnsw_real_ann_benchmarks_anchor_queries;

CREATE INDEX task28_ivf_pqg990k_g8_n128_idx
ON task28_ivf_pqg990k_g8_n128_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 128,
  nprobe = 128,
  training_sample_rows = 2000,
  quantizer = 'pq_fastscan',
  pq_group_size = 8,
  rerank = 'heap_f32',
  rerank_width = 500
);

ANALYZE task28_ivf_pqg990k_g8_n128_corpus;
ANALYZE task28_ivf_pqg990k_g8_n128_queries;

SELECT
  relname,
  pg_relation_size(oid) AS relation_bytes,
  pg_size_pretty(pg_relation_size(oid)) AS relation_size,
  reloptions
FROM pg_class
WHERE relname IN (
  'task28_ivf_pqg990k_g8_n128_corpus',
  'task28_ivf_pqg990k_g8_n128_queries',
  'task28_ivf_pqg990k_g8_n128_idx'
)
ORDER BY relname;
