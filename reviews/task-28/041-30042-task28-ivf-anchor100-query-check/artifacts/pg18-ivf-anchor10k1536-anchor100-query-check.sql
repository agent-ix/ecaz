\timing on

SELECT 'sha', 'a9d70f3';

SELECT
  count(*) AS corpus_rows,
  cardinality((SELECT source FROM task28_ivf_anchor10k1536_heap_corpus ORDER BY id LIMIT 1)) AS dimensions
FROM task28_ivf_anchor10k1536_heap_corpus;

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_queries100;
CREATE TABLE task28_ivf_anchor10k1536_queries100 AS
SELECT id, source
FROM ec_hnsw_real_ann_benchmarks_anchor_queries
ORDER BY id
LIMIT 100;
ALTER TABLE task28_ivf_anchor10k1536_queries100 ADD PRIMARY KEY (id);
ANALYZE task28_ivf_anchor10k1536_queries100;

SELECT
  count(*) AS query_rows,
  cardinality((SELECT source FROM task28_ivf_anchor10k1536_queries100 ORDER BY id LIMIT 1)) AS dimensions
FROM task28_ivf_anchor10k1536_queries100;

SET enable_indexscan = off;
SET enable_bitmapscan = off;
SET enable_seqscan = on;

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_exact100_top10;
CREATE TABLE task28_ivf_anchor10k1536_exact100_top10 AS
SELECT q.id AS query_id, exact.id AS corpus_id, exact.rank
FROM task28_ivf_anchor10k1536_queries100 q
CROSS JOIN LATERAL (
  SELECT c.id, row_number() OVER (ORDER BY c.embedding <#> q.source, c.id) AS rank
  FROM task28_ivf_anchor10k1536_heap_corpus c
  ORDER BY c.embedding <#> q.source, c.id
  LIMIT 10
) exact;

SELECT
  count(*) AS exact_rows,
  count(DISTINCT query_id) AS exact_queries
FROM task28_ivf_anchor10k1536_exact100_top10;

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_anchor100_summary;
CREATE TABLE task28_ivf_anchor10k1536_anchor100_summary (
  nlists integer NOT NULL,
  nprobe integer NOT NULL,
  rerank_width integer NOT NULL,
  build_ms double precision NOT NULL,
  materialize_ms double precision NOT NULL,
  returned integer NOT NULL,
  exact_hits integer NOT NULL,
  recall_at_10 numeric NOT NULL
);

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_anchor100_latency;
CREATE TABLE task28_ivf_anchor10k1536_anchor100_latency (
  nlists integer NOT NULL,
  nprobe integer NOT NULL,
  rerank_width integer NOT NULL,
  query_id bigint NOT NULL,
  elapsed_ms double precision NOT NULL,
  returned integer NOT NULL
);

SET enable_indexscan = on;
SET enable_bitmapscan = on;
SET enable_seqscan = off;

DO $$
DECLARE
  point record;
  started timestamptz;
  build_ms double precision;
  materialize_ms double precision;
  query_row record;
  row_count integer;
BEGIN
  FOR point IN
    SELECT *
    FROM (VALUES
      (32, 16, 50),
      (32, 32, 25),
      (64, 16, 25)
    ) AS points(nlists, nprobe, rerank_width)
  LOOP
    DROP INDEX IF EXISTS task28_ivf_anchor10k1536_anchor100_idx;
    started := clock_timestamp();
    EXECUTE format(
      'CREATE INDEX task28_ivf_anchor10k1536_anchor100_idx
       ON task28_ivf_anchor10k1536_heap_corpus USING ec_ivf (embedding ecvector_ip_ops)
       WITH (
         nlists = %s,
         nprobe = %s,
         training_sample_rows = 2000,
         storage_format = ''turboquant'',
         rerank = ''heap_f32'',
         rerank_width = %s
       )',
      point.nlists,
      point.nprobe,
      point.rerank_width
    );
    build_ms := EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0;

    EXECUTE format('SET ec_ivf.nprobe = %s', point.nprobe);
    DROP TABLE IF EXISTS task28_ivf_anchor10k1536_anchor100_ivf_top10;
    started := clock_timestamp();
    CREATE TABLE task28_ivf_anchor10k1536_anchor100_ivf_top10 AS
    SELECT qq.id AS query_id, ivf.id AS corpus_id, ivf.rank
    FROM task28_ivf_anchor10k1536_queries100 qq
    CROSS JOIN LATERAL (
      SELECT c.id, row_number() OVER (ORDER BY c.embedding <#> qq.source, c.id) AS rank
      FROM task28_ivf_anchor10k1536_heap_corpus c
      ORDER BY c.embedding <#> qq.source, c.id
      LIMIT 10
    ) ivf;
    materialize_ms := EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0;

    INSERT INTO task28_ivf_anchor10k1536_anchor100_summary
    SELECT
      point.nlists,
      point.nprobe,
      point.rerank_width,
      build_ms,
      materialize_ms,
      count(*)::integer,
      count(e.corpus_id)::integer,
      round(count(e.corpus_id)::numeric / 1000.0, 4)
    FROM task28_ivf_anchor10k1536_anchor100_ivf_top10 r
    LEFT JOIN task28_ivf_anchor10k1536_exact100_top10 e
      ON e.query_id = r.query_id
     AND e.corpus_id = r.corpus_id;

    FOR query_row IN
      SELECT id, source
      FROM task28_ivf_anchor10k1536_queries100
      ORDER BY id
    LOOP
      started := clock_timestamp();
      SELECT count(*) INTO row_count
      FROM (
        SELECT id
        FROM task28_ivf_anchor10k1536_heap_corpus
        ORDER BY embedding <#> query_row.source
        LIMIT 10
      ) results;
      INSERT INTO task28_ivf_anchor10k1536_anchor100_latency
      VALUES (
        point.nlists,
        point.nprobe,
        point.rerank_width,
        query_row.id,
        EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0,
        row_count
      );
    END LOOP;
  END LOOP;
END $$;

SELECT
  nlists,
  nprobe,
  rerank_width,
  round(build_ms::numeric, 3) AS build_ms,
  round(materialize_ms::numeric, 3) AS materialize_ms,
  returned,
  exact_hits,
  recall_at_10
FROM task28_ivf_anchor10k1536_anchor100_summary
ORDER BY nlists, nprobe, rerank_width;

SELECT
  nlists,
  nprobe,
  rerank_width,
  count(*) AS queries,
  min(returned) AS min_returned,
  max(returned) AS max_returned,
  round(percentile_disc(0.50) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p50_ms,
  round(percentile_disc(0.95) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p95_ms,
  round(percentile_disc(0.99) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p99_ms,
  round(avg(elapsed_ms)::numeric, 3) AS avg_ms
FROM task28_ivf_anchor10k1536_anchor100_latency
GROUP BY nlists, nprobe, rerank_width
ORDER BY nlists, nprobe, rerank_width;

SELECT
  nlists,
  nprobe,
  rerank_width,
  query_id,
  round(elapsed_ms::numeric, 3) AS elapsed_ms,
  returned
FROM task28_ivf_anchor10k1536_anchor100_latency
ORDER BY elapsed_ms DESC
LIMIT 20;
