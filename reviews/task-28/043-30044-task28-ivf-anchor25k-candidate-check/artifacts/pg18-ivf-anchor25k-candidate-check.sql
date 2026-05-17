\timing on

SELECT 'sha', '81f5468';

DROP TABLE IF EXISTS task28_ivf_anchor25k_corpus CASCADE;
CREATE TABLE task28_ivf_anchor25k_corpus AS
SELECT id, source, embedding
FROM ec_hnsw_real_ann_benchmarks_anchor_corpus
ORDER BY id
LIMIT 25000;
ALTER TABLE task28_ivf_anchor25k_corpus ADD PRIMARY KEY (id);
ANALYZE task28_ivf_anchor25k_corpus;

SELECT
  count(*) AS corpus_rows,
  cardinality((SELECT source FROM task28_ivf_anchor25k_corpus ORDER BY id LIMIT 1)) AS dimensions
FROM task28_ivf_anchor25k_corpus;

SELECT
  count(*) AS query_rows,
  cardinality((SELECT source FROM task28_ivf_anchor10k1536_queries100 ORDER BY id LIMIT 1)) AS dimensions
FROM task28_ivf_anchor10k1536_queries100;

SET enable_indexscan = off;
SET enable_bitmapscan = off;
SET enable_seqscan = on;

DROP TABLE IF EXISTS task28_ivf_anchor25k_exact100_top10;
CREATE TABLE task28_ivf_anchor25k_exact100_top10 AS
SELECT q.id AS query_id, exact.id AS corpus_id, exact.rank
FROM task28_ivf_anchor10k1536_queries100 q
CROSS JOIN LATERAL (
  SELECT c.id, row_number() OVER (ORDER BY c.embedding <#> q.source, c.id) AS rank
  FROM task28_ivf_anchor25k_corpus c
  ORDER BY c.embedding <#> q.source, c.id
  LIMIT 10
) exact;

SELECT
  count(*) AS exact_rows,
  count(DISTINCT query_id) AS exact_queries
FROM task28_ivf_anchor25k_exact100_top10;

DROP TABLE IF EXISTS task28_ivf_anchor25k_summary;
CREATE TABLE task28_ivf_anchor25k_summary (
  nlists integer NOT NULL,
  nprobe integer NOT NULL,
  rerank_width integer NOT NULL,
  build_ms double precision NOT NULL,
  materialize_ms double precision NOT NULL,
  returned integer NOT NULL,
  exact_hits integer NOT NULL,
  recall_at_10 numeric NOT NULL,
  index_bytes bigint NOT NULL
);

DROP TABLE IF EXISTS task28_ivf_anchor25k_latency;
CREATE TABLE task28_ivf_anchor25k_latency (
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
  index_bytes bigint;
  query_row record;
  row_count integer;
BEGIN
  FOR point IN
    SELECT *
    FROM (VALUES
      (32, 24, 25),
      (32, 32, 25)
    ) AS points(nlists, nprobe, rerank_width)
  LOOP
    DROP INDEX IF EXISTS task28_ivf_anchor25k_idx;
    started := clock_timestamp();
    EXECUTE format(
      'CREATE INDEX task28_ivf_anchor25k_idx
       ON task28_ivf_anchor25k_corpus USING ec_ivf (embedding ecvector_ip_ops)
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
    SELECT pg_relation_size('task28_ivf_anchor25k_idx'::regclass) INTO index_bytes;

    EXECUTE format('SET ec_ivf.nprobe = %s', point.nprobe);
    DROP TABLE IF EXISTS task28_ivf_anchor25k_ivf_top10;
    started := clock_timestamp();
    CREATE TABLE task28_ivf_anchor25k_ivf_top10 AS
    SELECT qq.id AS query_id, ivf.id AS corpus_id, ivf.rank
    FROM task28_ivf_anchor10k1536_queries100 qq
    CROSS JOIN LATERAL (
      SELECT c.id, row_number() OVER (ORDER BY c.embedding <#> qq.source, c.id) AS rank
      FROM task28_ivf_anchor25k_corpus c
      ORDER BY c.embedding <#> qq.source, c.id
      LIMIT 10
    ) ivf;
    materialize_ms := EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0;

    INSERT INTO task28_ivf_anchor25k_summary
    SELECT
      point.nlists,
      point.nprobe,
      point.rerank_width,
      build_ms,
      materialize_ms,
      count(*)::integer,
      count(e.corpus_id)::integer,
      round(count(e.corpus_id)::numeric / 1000.0, 4),
      index_bytes
    FROM task28_ivf_anchor25k_ivf_top10 r
    LEFT JOIN task28_ivf_anchor25k_exact100_top10 e
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
        FROM task28_ivf_anchor25k_corpus
        ORDER BY embedding <#> query_row.source
        LIMIT 10
      ) results;
      INSERT INTO task28_ivf_anchor25k_latency
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
  recall_at_10,
  index_bytes,
  pg_size_pretty(index_bytes) AS index_pretty
FROM task28_ivf_anchor25k_summary
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
FROM task28_ivf_anchor25k_latency
GROUP BY nlists, nprobe, rerank_width
ORDER BY nlists, nprobe, rerank_width;
