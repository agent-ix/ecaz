\pset pager off
\timing on

SELECT version() AS postgres_version;

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_corpus CASCADE;
DROP TABLE IF EXISTS task28_ivf_anchor10k1536_queries CASCADE;

CREATE TABLE task28_ivf_anchor10k1536_corpus AS
SELECT id, source, embedding
FROM ec_hnsw_real_ann_benchmarks_anchor_corpus
ORDER BY id
LIMIT 10000;

CREATE TABLE task28_ivf_anchor10k1536_queries AS
SELECT id, source
FROM ec_hnsw_real_ann_benchmarks_anchor_queries
ORDER BY id
LIMIT 20;

ALTER TABLE task28_ivf_anchor10k1536_corpus ADD PRIMARY KEY (id);
ALTER TABLE task28_ivf_anchor10k1536_queries ADD PRIMARY KEY (id);
ANALYZE task28_ivf_anchor10k1536_corpus;
ANALYZE task28_ivf_anchor10k1536_queries;

SELECT
  (SELECT count(*) FROM task28_ivf_anchor10k1536_corpus) AS corpus_rows,
  (SELECT count(*) FROM task28_ivf_anchor10k1536_queries) AS query_rows,
  cardinality((SELECT source FROM task28_ivf_anchor10k1536_corpus ORDER BY id LIMIT 1)) AS source_dimensions;

CREATE INDEX task28_ivf_anchor10k1536_n32_idx
ON task28_ivf_anchor10k1536_corpus USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 32,
  nprobe = 32,
  training_sample_rows = 2000,
  storage_format = 'turboquant',
  rerank = 'off'
);

ANALYZE task28_ivf_anchor10k1536_corpus;

SELECT
  pg_relation_size('task28_ivf_anchor10k1536_n32_idx'::regclass) AS index_bytes,
  pg_size_pretty(pg_relation_size('task28_ivf_anchor10k1536_n32_idx'::regclass)) AS index_pretty,
  pg_relation_size('task28_ivf_anchor10k1536_corpus'::regclass) AS heap_bytes,
  pg_size_pretty(pg_relation_size('task28_ivf_anchor10k1536_corpus'::regclass)) AS heap_pretty;

SET enable_seqscan = off;

SET ec_ivf.nprobe = 1;
EXPLAIN (ANALYZE, COSTS OFF, SUMMARY ON, TIMING OFF, ecaz)
SELECT id
FROM task28_ivf_anchor10k1536_corpus
ORDER BY embedding <#> (SELECT source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 1)
LIMIT 10;

SET ec_ivf.nprobe = 4;
EXPLAIN (ANALYZE, COSTS OFF, SUMMARY ON, TIMING OFF, ecaz)
SELECT id
FROM task28_ivf_anchor10k1536_corpus
ORDER BY embedding <#> (SELECT source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 1)
LIMIT 10;

SET ec_ivf.nprobe = 16;
EXPLAIN (ANALYZE, COSTS OFF, SUMMARY ON, TIMING OFF, ecaz)
SELECT id
FROM task28_ivf_anchor10k1536_corpus
ORDER BY embedding <#> (SELECT source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 1)
LIMIT 10;

SET ec_ivf.nprobe = 32;
EXPLAIN (ANALYZE, COSTS OFF, SUMMARY ON, TIMING OFF, ecaz)
SELECT id
FROM task28_ivf_anchor10k1536_corpus
ORDER BY embedding <#> (SELECT source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 1)
LIMIT 10;

RESET enable_seqscan;
RESET ec_ivf.nprobe;

SET enable_indexscan = off;
SET enable_bitmapscan = off;
SET enable_seqscan = on;

CREATE TEMP TABLE task28_ivf_anchor10k1536_exact_top10 AS
SELECT q.id AS query_id, exact.id AS corpus_id
FROM task28_ivf_anchor10k1536_queries q
CROSS JOIN LATERAL (
  SELECT c.id
  FROM task28_ivf_anchor10k1536_corpus c
  ORDER BY c.embedding <#> q.source
  LIMIT 10
) exact;

RESET enable_indexscan;
RESET enable_bitmapscan;
RESET enable_seqscan;

CREATE TEMP TABLE task28_ivf_anchor10k1536_results (
  nprobe integer NOT NULL,
  query_id bigint NOT NULL,
  corpus_id bigint NOT NULL
);

SET enable_seqscan = off;

SET ec_ivf.nprobe = 1;
INSERT INTO task28_ivf_anchor10k1536_results
SELECT 1, q.id, ann.id
FROM task28_ivf_anchor10k1536_queries q
CROSS JOIN LATERAL (
  SELECT c.id
  FROM task28_ivf_anchor10k1536_corpus c
  ORDER BY c.embedding <#> q.source
  LIMIT 10
) ann;

SET ec_ivf.nprobe = 4;
INSERT INTO task28_ivf_anchor10k1536_results
SELECT 4, q.id, ann.id
FROM task28_ivf_anchor10k1536_queries q
CROSS JOIN LATERAL (
  SELECT c.id
  FROM task28_ivf_anchor10k1536_corpus c
  ORDER BY c.embedding <#> q.source
  LIMIT 10
) ann;

SET ec_ivf.nprobe = 16;
INSERT INTO task28_ivf_anchor10k1536_results
SELECT 16, q.id, ann.id
FROM task28_ivf_anchor10k1536_queries q
CROSS JOIN LATERAL (
  SELECT c.id
  FROM task28_ivf_anchor10k1536_corpus c
  ORDER BY c.embedding <#> q.source
  LIMIT 10
) ann;

SET ec_ivf.nprobe = 32;
INSERT INTO task28_ivf_anchor10k1536_results
SELECT 32, q.id, ann.id
FROM task28_ivf_anchor10k1536_queries q
CROSS JOIN LATERAL (
  SELECT c.id
  FROM task28_ivf_anchor10k1536_corpus c
  ORDER BY c.embedding <#> q.source
  LIMIT 10
) ann;

RESET enable_seqscan;
RESET ec_ivf.nprobe;

SELECT
  r.nprobe,
  count(*) AS returned,
  count(e.corpus_id) AS exact_hits,
  round(count(e.corpus_id)::numeric / (20 * 10), 4) AS recall_at_10
FROM task28_ivf_anchor10k1536_results r
LEFT JOIN task28_ivf_anchor10k1536_exact_top10 e
  ON e.query_id = r.query_id
 AND e.corpus_id = r.corpus_id
GROUP BY r.nprobe
ORDER BY r.nprobe;

SELECT
  nprobe,
  query_id,
  count(*) AS returned_per_query
FROM task28_ivf_anchor10k1536_results
GROUP BY nprobe, query_id
HAVING count(*) <> 10
ORDER BY nprobe, query_id;
