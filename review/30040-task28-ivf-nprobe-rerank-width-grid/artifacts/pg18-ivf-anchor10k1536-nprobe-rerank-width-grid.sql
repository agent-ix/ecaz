\timing on

SELECT 'sha', '697f49e';

SELECT
  count(*) AS corpus_rows,
  cardinality((SELECT source FROM task28_ivf_anchor10k1536_heap_corpus ORDER BY id LIMIT 1)) AS dimensions
FROM task28_ivf_anchor10k1536_heap_corpus;

SELECT
  count(*) AS exact_rows,
  count(DISTINCT query_id) AS exact_queries
FROM task28_ivf_anchor10k1536_heap_exact_top10;

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_grid_summary;
CREATE TABLE task28_ivf_anchor10k1536_grid_summary (
  nprobe integer NOT NULL,
  rerank_width integer NOT NULL,
  build_ms double precision NOT NULL,
  materialize_ms double precision NOT NULL,
  returned integer NOT NULL,
  exact_hits integer NOT NULL,
  recall_at_10 numeric NOT NULL
);

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_grid_latency;
CREATE TABLE task28_ivf_anchor10k1536_grid_latency (
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
  np integer;
  rw integer;
  started timestamptz;
  build_ms double precision;
  materialize_ms double precision;
  query_row record;
  row_count integer;
BEGIN
  FOREACH np IN ARRAY ARRAY[4, 8, 16, 32] LOOP
    FOREACH rw IN ARRAY ARRAY[25, 50, 100, 200] LOOP
      DROP INDEX IF EXISTS task28_ivf_anchor10k1536_grid_idx;
      started := clock_timestamp();
      EXECUTE format(
        'CREATE INDEX task28_ivf_anchor10k1536_grid_idx
         ON task28_ivf_anchor10k1536_heap_corpus USING ec_ivf (embedding ecvector_ip_ops)
         WITH (
           nlists = 32,
           nprobe = %s,
           training_sample_rows = 2000,
           storage_format = ''turboquant'',
           rerank = ''heap_f32'',
           rerank_width = %s
         )',
        np,
        rw
      );
      build_ms := EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0;

      EXECUTE format('SET ec_ivf.nprobe = %s', np);
      DROP TABLE IF EXISTS task28_ivf_anchor10k1536_grid_ivf_top10;
      started := clock_timestamp();
      CREATE TABLE task28_ivf_anchor10k1536_grid_ivf_top10 AS
      SELECT qq.id AS query_id, ivf.id AS corpus_id, ivf.rank
      FROM (SELECT id, source FROM task28_ivf_anchor10k1536_queries ORDER BY id LIMIT 20) qq
      CROSS JOIN LATERAL (
        SELECT c.id, row_number() OVER (ORDER BY c.embedding <#> qq.source, c.id) AS rank
        FROM task28_ivf_anchor10k1536_heap_corpus c
        ORDER BY c.embedding <#> qq.source, c.id
        LIMIT 10
      ) ivf;
      materialize_ms := EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0;

      INSERT INTO task28_ivf_anchor10k1536_grid_summary
      SELECT
        np,
        rw,
        build_ms,
        materialize_ms,
        count(*)::integer,
        count(e.corpus_id)::integer,
        round(count(e.corpus_id)::numeric / 200.0, 4)
      FROM task28_ivf_anchor10k1536_grid_ivf_top10 r
      LEFT JOIN task28_ivf_anchor10k1536_heap_exact_top10 e
        ON e.query_id = r.query_id
       AND e.corpus_id = r.corpus_id;

      FOR query_row IN
        SELECT id, source
        FROM task28_ivf_anchor10k1536_queries
        ORDER BY id
        LIMIT 20
      LOOP
        started := clock_timestamp();
        SELECT count(*) INTO row_count
        FROM (
          SELECT id
          FROM task28_ivf_anchor10k1536_heap_corpus
          ORDER BY embedding <#> query_row.source
          LIMIT 10
        ) results;
        INSERT INTO task28_ivf_anchor10k1536_grid_latency
        VALUES (
          np,
          rw,
          query_row.id,
          EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0,
          row_count
        );
      END LOOP;
    END LOOP;
  END LOOP;
END $$;

SELECT
  nprobe,
  rerank_width,
  round(build_ms::numeric, 3) AS build_ms,
  round(materialize_ms::numeric, 3) AS materialize_ms,
  returned,
  exact_hits,
  recall_at_10
FROM task28_ivf_anchor10k1536_grid_summary
ORDER BY nprobe, rerank_width;

SELECT
  nprobe,
  rerank_width,
  count(*) AS queries,
  min(returned) AS min_returned,
  max(returned) AS max_returned,
  round(percentile_disc(0.50) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p50_ms,
  round(percentile_disc(0.95) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p95_ms,
  round(percentile_disc(0.99) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p99_ms,
  round(avg(elapsed_ms)::numeric, 3) AS avg_ms
FROM task28_ivf_anchor10k1536_grid_latency
GROUP BY nprobe, rerank_width
ORDER BY nprobe, rerank_width;

SELECT
  nprobe,
  rerank_width,
  query_id,
  round(elapsed_ms::numeric, 3) AS elapsed_ms,
  returned
FROM task28_ivf_anchor10k1536_grid_latency
ORDER BY elapsed_ms DESC
LIMIT 20;
