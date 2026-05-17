\timing on

SELECT 'sha', '9b42a71';
SELECT current_database(), version();

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_heap_corpus CASCADE;
CREATE TABLE task28_ivf_anchor10k1536_heap_corpus AS
SELECT id, source, embedding
FROM task28_ivf_anchor10k1536_corpus
ORDER BY id;

ALTER TABLE task28_ivf_anchor10k1536_heap_corpus ADD PRIMARY KEY (id);
ANALYZE task28_ivf_anchor10k1536_heap_corpus;

SELECT
  count(*) AS corpus_rows,
  cardinality((SELECT source FROM task28_ivf_anchor10k1536_heap_corpus ORDER BY id LIMIT 1)) AS source_dimensions;

CREATE INDEX task28_ivf_anchor10k1536_n32_heap_idx
ON task28_ivf_anchor10k1536_heap_corpus USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 32,
  nprobe = 32,
  training_sample_rows = 2000,
  storage_format = 'turboquant',
  rerank = 'heap_f32'
);

ANALYZE task28_ivf_anchor10k1536_heap_corpus;

SELECT
  pg_relation_size('task28_ivf_anchor10k1536_n32_heap_idx'::regclass) AS index_bytes,
  pg_size_pretty(pg_relation_size('task28_ivf_anchor10k1536_n32_heap_idx'::regclass)) AS index_pretty,
  pg_relation_size('task28_ivf_anchor10k1536_heap_corpus'::regclass) AS heap_bytes,
  pg_size_pretty(pg_relation_size('task28_ivf_anchor10k1536_heap_corpus'::regclass)) AS heap_pretty;

SET enable_indexscan = off;
SET enable_bitmapscan = off;
SET enable_seqscan = on;

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_heap_exact_top10;
CREATE TABLE task28_ivf_anchor10k1536_heap_exact_top10 AS
SELECT q.id AS query_id, exact.id AS corpus_id, exact.rank
FROM (SELECT id, source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 20) q
CROSS JOIN LATERAL (
  SELECT c.id, row_number() OVER (ORDER BY c.embedding <#> q.source, c.id) AS rank
  FROM task28_ivf_anchor10k1536_heap_corpus c
  ORDER BY c.embedding <#> q.source, c.id
  LIMIT 10
) exact;

SET enable_indexscan = on;
SET enable_bitmapscan = on;
SET enable_seqscan = off;
SET ec_ivf.nprobe = 32;

EXPLAIN (ANALYZE, BUFFERS, COSTS OFF)
SELECT id
FROM task28_ivf_anchor10k1536_heap_corpus
ORDER BY embedding <#> (SELECT source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 1)
LIMIT 10;

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_heap_ivf_top10;
CREATE TABLE task28_ivf_anchor10k1536_heap_ivf_top10 AS
SELECT q.id AS query_id, ivf.id AS corpus_id, ivf.rank
FROM (SELECT id, source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 20) q
CROSS JOIN LATERAL (
  SELECT c.id, row_number() OVER (ORDER BY c.embedding <#> q.source, c.id) AS rank
  FROM task28_ivf_anchor10k1536_heap_corpus c
  ORDER BY c.embedding <#> q.source, c.id
  LIMIT 10
) ivf;

SELECT
  count(*) AS returned,
  count(e.corpus_id) AS exact_hits,
  round(count(e.corpus_id)::numeric / 200.0, 4) AS recall_at_10
FROM task28_ivf_anchor10k1536_heap_ivf_top10 r
LEFT JOIN task28_ivf_anchor10k1536_heap_exact_top10 e
  ON e.query_id = r.query_id
 AND e.corpus_id = r.corpus_id;

SELECT
  count(*) FILTER (WHERE rank_count != 10) AS queries_without_10_results,
  min(rank_count) AS min_results,
  max(rank_count) AS max_results
FROM (
  SELECT query_id, count(*) AS rank_count
  FROM task28_ivf_anchor10k1536_heap_ivf_top10
  GROUP BY query_id
) counts;
