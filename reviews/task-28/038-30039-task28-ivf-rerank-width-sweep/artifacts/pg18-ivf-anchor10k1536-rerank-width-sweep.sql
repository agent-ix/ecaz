\timing on

SELECT 'sha', '4d894bd';

SELECT
  count(*) AS exact_rows,
  count(DISTINCT query_id) AS exact_queries
FROM task28_ivf_anchor10k1536_heap_exact_top10;

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_width_summary;
CREATE TABLE task28_ivf_anchor10k1536_width_summary (
  rerank_width integer NOT NULL,
  returned integer NOT NULL,
  exact_hits integer NOT NULL,
  recall_at_10 numeric NOT NULL,
  materialize_ms double precision NOT NULL
);

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_width_latency;
CREATE TABLE task28_ivf_anchor10k1536_width_latency (
  rerank_width integer NOT NULL,
  query_id bigint NOT NULL,
  elapsed_ms double precision NOT NULL,
  returned integer NOT NULL
);

SET enable_indexscan = on;
SET enable_bitmapscan = on;
SET enable_seqscan = off;
SET ec_ivf.nprobe = 32;

DROP INDEX IF EXISTS task28_ivf_anchor10k1536_width_idx;
CREATE INDEX task28_ivf_anchor10k1536_width_idx
ON task28_ivf_anchor10k1536_heap_corpus USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 32,
  nprobe = 32,
  training_sample_rows = 2000,
  storage_format = 'turboquant',
  rerank = 'heap_f32',
  rerank_width = 50
);

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_width_ivf_top10;
CREATE TABLE task28_ivf_anchor10k1536_width_ivf_top10 AS
SELECT q.id AS query_id, ivf.id AS corpus_id, ivf.rank
FROM (SELECT id, source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 20) q
CROSS JOIN LATERAL (
  SELECT c.id, row_number() OVER (ORDER BY c.embedding <#> q.source, c.id) AS rank
  FROM task28_ivf_anchor10k1536_heap_corpus c
  ORDER BY c.embedding <#> q.source, c.id
  LIMIT 10
) ivf;

INSERT INTO task28_ivf_anchor10k1536_width_summary
SELECT
  50,
  count(*)::integer,
  count(e.corpus_id)::integer,
  round(count(e.corpus_id)::numeric / 200.0, 4),
  0.0
FROM task28_ivf_anchor10k1536_width_ivf_top10 r
LEFT JOIN task28_ivf_anchor10k1536_heap_exact_top10 e
  ON e.query_id = r.query_id
 AND e.corpus_id = r.corpus_id;

DO $$
DECLARE
  q record;
  started timestamptz;
  row_count integer;
BEGIN
  FOR q IN SELECT id, source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 20 LOOP
    started := clock_timestamp();
    SELECT count(*) INTO row_count
    FROM (
      SELECT id
      FROM task28_ivf_anchor10k1536_heap_corpus
      ORDER BY embedding <#> q.source
      LIMIT 10
    ) results;
    INSERT INTO task28_ivf_anchor10k1536_width_latency
    VALUES (50, q.id, EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0, row_count);
  END LOOP;
END $$;

DROP INDEX task28_ivf_anchor10k1536_width_idx;
CREATE INDEX task28_ivf_anchor10k1536_width_idx
ON task28_ivf_anchor10k1536_heap_corpus USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 32,
  nprobe = 32,
  training_sample_rows = 2000,
  storage_format = 'turboquant',
  rerank = 'heap_f32',
  rerank_width = 200
);

DROP TABLE task28_ivf_anchor10k1536_width_ivf_top10;
CREATE TABLE task28_ivf_anchor10k1536_width_ivf_top10 AS
SELECT q.id AS query_id, ivf.id AS corpus_id, ivf.rank
FROM (SELECT id, source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 20) q
CROSS JOIN LATERAL (
  SELECT c.id, row_number() OVER (ORDER BY c.embedding <#> q.source, c.id) AS rank
  FROM task28_ivf_anchor10k1536_heap_corpus c
  ORDER BY c.embedding <#> q.source, c.id
  LIMIT 10
) ivf;

INSERT INTO task28_ivf_anchor10k1536_width_summary
SELECT
  200,
  count(*)::integer,
  count(e.corpus_id)::integer,
  round(count(e.corpus_id)::numeric / 200.0, 4),
  0.0
FROM task28_ivf_anchor10k1536_width_ivf_top10 r
LEFT JOIN task28_ivf_anchor10k1536_heap_exact_top10 e
  ON e.query_id = r.query_id
 AND e.corpus_id = r.corpus_id;

DO $$
DECLARE
  q record;
  started timestamptz;
  row_count integer;
BEGIN
  FOR q IN SELECT id, source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 20 LOOP
    started := clock_timestamp();
    SELECT count(*) INTO row_count
    FROM (
      SELECT id
      FROM task28_ivf_anchor10k1536_heap_corpus
      ORDER BY embedding <#> q.source
      LIMIT 10
    ) results;
    INSERT INTO task28_ivf_anchor10k1536_width_latency
    VALUES (200, q.id, EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0, row_count);
  END LOOP;
END $$;

DROP INDEX task28_ivf_anchor10k1536_width_idx;
CREATE INDEX task28_ivf_anchor10k1536_width_idx
ON task28_ivf_anchor10k1536_heap_corpus USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 32,
  nprobe = 32,
  training_sample_rows = 2000,
  storage_format = 'turboquant',
  rerank = 'heap_f32',
  rerank_width = 1000
);

DROP TABLE task28_ivf_anchor10k1536_width_ivf_top10;
CREATE TABLE task28_ivf_anchor10k1536_width_ivf_top10 AS
SELECT q.id AS query_id, ivf.id AS corpus_id, ivf.rank
FROM (SELECT id, source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 20) q
CROSS JOIN LATERAL (
  SELECT c.id, row_number() OVER (ORDER BY c.embedding <#> q.source, c.id) AS rank
  FROM task28_ivf_anchor10k1536_heap_corpus c
  ORDER BY c.embedding <#> q.source, c.id
  LIMIT 10
) ivf;

INSERT INTO task28_ivf_anchor10k1536_width_summary
SELECT
  1000,
  count(*)::integer,
  count(e.corpus_id)::integer,
  round(count(e.corpus_id)::numeric / 200.0, 4),
  0.0
FROM task28_ivf_anchor10k1536_width_ivf_top10 r
LEFT JOIN task28_ivf_anchor10k1536_heap_exact_top10 e
  ON e.query_id = r.query_id
 AND e.corpus_id = r.corpus_id;

DO $$
DECLARE
  q record;
  started timestamptz;
  row_count integer;
BEGIN
  FOR q IN SELECT id, source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 20 LOOP
    started := clock_timestamp();
    SELECT count(*) INTO row_count
    FROM (
      SELECT id
      FROM task28_ivf_anchor10k1536_heap_corpus
      ORDER BY embedding <#> q.source
      LIMIT 10
    ) results;
    INSERT INTO task28_ivf_anchor10k1536_width_latency
    VALUES (1000, q.id, EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0, row_count);
  END LOOP;
END $$;

DROP INDEX task28_ivf_anchor10k1536_width_idx;
CREATE INDEX task28_ivf_anchor10k1536_width_idx
ON task28_ivf_anchor10k1536_heap_corpus USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 32,
  nprobe = 32,
  training_sample_rows = 2000,
  storage_format = 'turboquant',
  rerank = 'heap_f32',
  rerank_width = 0
);

DROP TABLE task28_ivf_anchor10k1536_width_ivf_top10;
CREATE TABLE task28_ivf_anchor10k1536_width_ivf_top10 AS
SELECT q.id AS query_id, ivf.id AS corpus_id, ivf.rank
FROM (SELECT id, source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 20) q
CROSS JOIN LATERAL (
  SELECT c.id, row_number() OVER (ORDER BY c.embedding <#> q.source, c.id) AS rank
  FROM task28_ivf_anchor10k1536_heap_corpus c
  ORDER BY c.embedding <#> q.source, c.id
  LIMIT 10
) ivf;

INSERT INTO task28_ivf_anchor10k1536_width_summary
SELECT
  0,
  count(*)::integer,
  count(e.corpus_id)::integer,
  round(count(e.corpus_id)::numeric / 200.0, 4),
  0.0
FROM task28_ivf_anchor10k1536_width_ivf_top10 r
LEFT JOIN task28_ivf_anchor10k1536_heap_exact_top10 e
  ON e.query_id = r.query_id
 AND e.corpus_id = r.corpus_id;

DO $$
DECLARE
  q record;
  started timestamptz;
  row_count integer;
BEGIN
  FOR q IN SELECT id, source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 20 LOOP
    started := clock_timestamp();
    SELECT count(*) INTO row_count
    FROM (
      SELECT id
      FROM task28_ivf_anchor10k1536_heap_corpus
      ORDER BY embedding <#> q.source
      LIMIT 10
    ) results;
    INSERT INTO task28_ivf_anchor10k1536_width_latency
    VALUES (0, q.id, EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0, row_count);
  END LOOP;
END $$;

SELECT *
FROM task28_ivf_anchor10k1536_width_summary
ORDER BY CASE WHEN rerank_width = 0 THEN 10000000 ELSE rerank_width END;

SELECT
  rerank_width,
  count(*) AS queries,
  min(returned) AS min_returned,
  max(returned) AS max_returned,
  round(percentile_disc(0.50) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p50_ms,
  round(percentile_disc(0.95) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p95_ms,
  round(percentile_disc(0.99) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p99_ms,
  round(avg(elapsed_ms)::numeric, 3) AS avg_ms
FROM task28_ivf_anchor10k1536_width_latency
GROUP BY rerank_width
ORDER BY CASE WHEN rerank_width = 0 THEN 10000000 ELSE rerank_width END;

SELECT
  rerank_width,
  query_id,
  round(elapsed_ms::numeric, 3) AS elapsed_ms,
  returned
FROM task28_ivf_anchor10k1536_width_latency
ORDER BY elapsed_ms DESC
LIMIT 10;
