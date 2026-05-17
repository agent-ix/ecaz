\timing on

DROP TABLE IF EXISTS task28_ivf_qcmp10k_turboquant_corpus CASCADE;
DROP TABLE IF EXISTS task28_ivf_qcmp10k_turboquant_queries CASCADE;
DROP TABLE IF EXISTS task28_ivf_qcmp10k_pqfastscan_corpus CASCADE;
DROP TABLE IF EXISTS task28_ivf_qcmp10k_pqfastscan_queries CASCADE;
DROP TABLE IF EXISTS task28_ivf_qcmp10k_rabitq_corpus CASCADE;
DROP TABLE IF EXISTS task28_ivf_qcmp10k_rabitq_queries CASCADE;

CREATE TABLE task28_ivf_qcmp10k_turboquant_corpus AS
SELECT * FROM task28_ivf_postopt10k_n64w25_corpus;
ALTER TABLE task28_ivf_qcmp10k_turboquant_corpus ADD PRIMARY KEY (id);
CREATE TABLE task28_ivf_qcmp10k_turboquant_queries AS
SELECT * FROM task28_ivf_postopt10k_n64w25_queries;

CREATE TABLE task28_ivf_qcmp10k_pqfastscan_corpus AS
SELECT * FROM task28_ivf_postopt10k_n64w25_corpus;
ALTER TABLE task28_ivf_qcmp10k_pqfastscan_corpus ADD PRIMARY KEY (id);
CREATE TABLE task28_ivf_qcmp10k_pqfastscan_queries AS
SELECT * FROM task28_ivf_postopt10k_n64w25_queries;

CREATE TABLE task28_ivf_qcmp10k_rabitq_corpus AS
SELECT * FROM task28_ivf_postopt10k_n64w25_corpus;
ALTER TABLE task28_ivf_qcmp10k_rabitq_corpus ADD PRIMARY KEY (id);
CREATE TABLE task28_ivf_qcmp10k_rabitq_queries AS
SELECT * FROM task28_ivf_postopt10k_n64w25_queries;

CREATE INDEX task28_ivf_qcmp10k_turboquant_idx
ON task28_ivf_qcmp10k_turboquant_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 64,
  nprobe = 64,
  training_sample_rows = 2000,
  storage_format = 'turboquant',
  rerank = 'heap_f32',
  rerank_width = 25
);

CREATE INDEX task28_ivf_qcmp10k_pqfastscan_idx
ON task28_ivf_qcmp10k_pqfastscan_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 64,
  nprobe = 64,
  training_sample_rows = 2000,
  storage_format = 'pq_fastscan',
  rerank = 'heap_f32',
  rerank_width = 25
);

CREATE INDEX task28_ivf_qcmp10k_rabitq_idx
ON task28_ivf_qcmp10k_rabitq_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 64,
  nprobe = 64,
  training_sample_rows = 2000,
  storage_format = 'rabitq',
  rerank = 'heap_f32',
  rerank_width = 25
);

SELECT
  relname,
  pg_size_pretty(pg_relation_size(oid)) AS index_size,
  reloptions
FROM pg_class
WHERE relname IN (
  'task28_ivf_qcmp10k_turboquant_idx',
  'task28_ivf_qcmp10k_pqfastscan_idx',
  'task28_ivf_qcmp10k_rabitq_idx'
)
ORDER BY relname;
