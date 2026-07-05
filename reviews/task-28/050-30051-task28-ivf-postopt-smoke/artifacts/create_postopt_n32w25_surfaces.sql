\timing on
DROP TABLE IF EXISTS task28_ivf_postopt10k_n32w25_queries;
DROP TABLE IF EXISTS task28_ivf_postopt10k_n32w25_corpus;
DROP TABLE IF EXISTS task28_ivf_postopt25k_n32w25_queries;
DROP TABLE IF EXISTS task28_ivf_postopt25k_n32w25_corpus;

CREATE TABLE task28_ivf_postopt10k_n32w25_corpus AS
SELECT id, source, embedding
FROM task28_ivf_anchor10k1536_corpus
ORDER BY id;
ALTER TABLE task28_ivf_postopt10k_n32w25_corpus ADD PRIMARY KEY (id);

CREATE TABLE task28_ivf_postopt10k_n32w25_queries AS
SELECT id, source
FROM task28_ivf_anchor10k1536_queries100
ORDER BY id;
ALTER TABLE task28_ivf_postopt10k_n32w25_queries ADD PRIMARY KEY (id);

CREATE INDEX task28_ivf_postopt10k_n32w25_idx
ON task28_ivf_postopt10k_n32w25_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 32,
  nprobe = 32,
  training_sample_rows = 2000,
  storage_format = turboquant,
  rerank = heap_f32,
  rerank_width = 25
);

CREATE TABLE task28_ivf_postopt25k_n32w25_corpus AS
SELECT id, source, embedding
FROM task28_ivf_anchor25k_corpus
ORDER BY id;
ALTER TABLE task28_ivf_postopt25k_n32w25_corpus ADD PRIMARY KEY (id);

CREATE TABLE task28_ivf_postopt25k_n32w25_queries AS
SELECT id, source
FROM task28_ivf_anchor10k1536_queries100
ORDER BY id;
ALTER TABLE task28_ivf_postopt25k_n32w25_queries ADD PRIMARY KEY (id);

CREATE INDEX task28_ivf_postopt25k_n32w25_idx
ON task28_ivf_postopt25k_n32w25_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 32,
  nprobe = 32,
  training_sample_rows = 2000,
  storage_format = turboquant,
  rerank = heap_f32,
  rerank_width = 25
);

SELECT
  c.relname,
  pg_size_pretty(pg_relation_size(c.oid)) AS index_size,
  c.reloptions
FROM pg_class c
WHERE c.relname IN (
  'task28_ivf_postopt10k_n32w25_idx',
  'task28_ivf_postopt25k_n32w25_idx'
)
ORDER BY c.relname;
