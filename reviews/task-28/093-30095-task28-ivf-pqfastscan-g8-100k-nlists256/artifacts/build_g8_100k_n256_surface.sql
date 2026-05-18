\timing on

DROP TABLE IF EXISTS task28_ivf_pqg100k_g8_n256_corpus CASCADE;
DROP TABLE IF EXISTS task28_ivf_pqg100k_g8_n256_queries CASCADE;

CREATE TABLE task28_ivf_pqg100k_g8_n256_corpus AS
SELECT * FROM task28_ivf_pqg100k_g8_corpus;
ALTER TABLE task28_ivf_pqg100k_g8_n256_corpus ADD PRIMARY KEY (id);

CREATE TABLE task28_ivf_pqg100k_g8_n256_queries AS
SELECT * FROM task28_ivf_pqg100k_g8_queries;

CREATE INDEX task28_ivf_pqg100k_g8_n256_idx
ON task28_ivf_pqg100k_g8_n256_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 256,
  nprobe = 256,
  training_sample_rows = 2000,
  storage_format = 'pq_fastscan',
  pq_group_size = 8,
  rerank = 'heap_f32',
  rerank_width = 500
);

SELECT
  relname,
  pg_size_pretty(pg_relation_size(oid)) AS index_size,
  reloptions
FROM pg_class
WHERE relname = 'task28_ivf_pqg100k_g8_n256_idx';
