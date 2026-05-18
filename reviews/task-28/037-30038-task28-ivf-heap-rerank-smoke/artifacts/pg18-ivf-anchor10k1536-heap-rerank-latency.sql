\timing on

SELECT 'sha', '9b42a71';

SET enable_indexscan = on;
SET enable_bitmapscan = on;
SET enable_seqscan = off;
SET ec_ivf.nprobe = 32;

DROP TABLE IF EXISTS task28_ivf_anchor10k1536_heap_latency;
CREATE TABLE task28_ivf_anchor10k1536_heap_latency (
  query_id bigint primary key,
  elapsed_ms double precision NOT NULL,
  returned integer NOT NULL
);

DO $$
DECLARE
  q record;
  started timestamptz;
  row_count integer;
BEGIN
  FOR q IN
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
      ORDER BY embedding <#> q.source
      LIMIT 10
    ) results;
    INSERT INTO task28_ivf_anchor10k1536_heap_latency
    VALUES (
      q.id,
      EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0,
      row_count
    );
  END LOOP;
END $$;

SELECT
  count(*) AS queries,
  min(returned) AS min_returned,
  max(returned) AS max_returned,
  round(percentile_disc(0.50) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p50_ms,
  round(percentile_disc(0.95) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p95_ms,
  round(percentile_disc(0.99) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p99_ms,
  round(avg(elapsed_ms)::numeric, 3) AS avg_ms
FROM task28_ivf_anchor10k1536_heap_latency;

SELECT *
FROM task28_ivf_anchor10k1536_heap_latency
ORDER BY elapsed_ms DESC, query_id
LIMIT 5;
