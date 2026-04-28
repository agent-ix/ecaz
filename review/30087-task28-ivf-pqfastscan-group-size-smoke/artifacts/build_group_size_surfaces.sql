\timing on

DROP TABLE IF EXISTS task28_ivf_pqg10k_g8_corpus CASCADE;
DROP TABLE IF EXISTS task28_ivf_pqg10k_g8_queries CASCADE;
DROP TABLE IF EXISTS task28_ivf_pqg10k_g16_corpus CASCADE;
DROP TABLE IF EXISTS task28_ivf_pqg10k_g16_queries CASCADE;
DROP TABLE IF EXISTS task28_ivf_pqg10k_g32_corpus CASCADE;
DROP TABLE IF EXISTS task28_ivf_pqg10k_g32_queries CASCADE;

CREATE TABLE task28_ivf_pqg10k_g8_corpus AS
SELECT * FROM task28_ivf_postopt10k_n64w25_corpus;
ALTER TABLE task28_ivf_pqg10k_g8_corpus ADD PRIMARY KEY (id);
CREATE TABLE task28_ivf_pqg10k_g8_queries AS
SELECT * FROM task28_ivf_postopt10k_n64w25_queries;

CREATE TABLE task28_ivf_pqg10k_g16_corpus AS
SELECT * FROM task28_ivf_postopt10k_n64w25_corpus;
ALTER TABLE task28_ivf_pqg10k_g16_corpus ADD PRIMARY KEY (id);
CREATE TABLE task28_ivf_pqg10k_g16_queries AS
SELECT * FROM task28_ivf_postopt10k_n64w25_queries;

CREATE TABLE task28_ivf_pqg10k_g32_corpus AS
SELECT * FROM task28_ivf_postopt10k_n64w25_corpus;
ALTER TABLE task28_ivf_pqg10k_g32_corpus ADD PRIMARY KEY (id);
CREATE TABLE task28_ivf_pqg10k_g32_queries AS
SELECT * FROM task28_ivf_postopt10k_n64w25_queries;

CREATE INDEX task28_ivf_pqg10k_g8_idx
ON task28_ivf_pqg10k_g8_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 64,
  nprobe = 64,
  training_sample_rows = 2000,
  storage_format = 'pq_fastscan',
  pq_group_size = 8,
  rerank = 'heap_f32',
  rerank_width = 25
);

CREATE INDEX task28_ivf_pqg10k_g16_idx
ON task28_ivf_pqg10k_g16_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 64,
  nprobe = 64,
  training_sample_rows = 2000,
  storage_format = 'pq_fastscan',
  pq_group_size = 16,
  rerank = 'heap_f32',
  rerank_width = 25
);

CREATE INDEX task28_ivf_pqg10k_g32_idx
ON task28_ivf_pqg10k_g32_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 64,
  nprobe = 64,
  training_sample_rows = 2000,
  storage_format = 'pq_fastscan',
  pq_group_size = 32,
  rerank = 'heap_f32',
  rerank_width = 25
);

SELECT
  relname,
  pg_size_pretty(pg_relation_size(oid)) AS index_size,
  reloptions
FROM pg_class
WHERE relname IN (
  'task28_ivf_pqg10k_g8_idx',
  'task28_ivf_pqg10k_g16_idx',
  'task28_ivf_pqg10k_g32_idx'
)
ORDER BY relname;
