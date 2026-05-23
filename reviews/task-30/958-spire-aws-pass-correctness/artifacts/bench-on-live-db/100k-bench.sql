-- ec_ivf RaBitQ kNN latency bench (k=10, 100 queries sequential).
-- Self-samples 100 corpus embeddings as probe vectors so we don't need to
-- encode the raw queries.source column on the fly. Placeholders 100k /
-- real_100k_ivf_rabitq1_rerank_corpus / real_100k_ivf_rabitq1_rerank_rabitq_idx are substituted by bench-on-live-db.sh.
\set ON_ERROR_STOP on
\timing on

\echo === scale=100k corpus=real_100k_ivf_rabitq1_rerank_corpus idx=real_100k_ivf_rabitq1_rerank_rabitq_idx ===

SELECT pg_size_pretty(pg_total_relation_size('real_100k_ivf_rabitq1_rerank_corpus'::regclass)) AS corpus_total,
       pg_size_pretty(pg_relation_size('real_100k_ivf_rabitq1_rerank_rabitq_idx'::regclass)) AS index_size;

SELECT count(*) AS n_rows FROM real_100k_ivf_rabitq1_rerank_corpus;

DO $bench$
DECLARE
  qrec record;
  total_ms double precision := 0;
  cnt int := 0;
  k int := 10;
  t_start timestamp;
  t_end timestamp;
  per_query double precision[];
BEGIN
  -- Self-sample: take 100 rows from the corpus and use their embeddings
  -- as probe vectors. Latency characteristics are representative of any
  -- in-distribution probe; this just avoids the source-vs-ecvector
  -- encoding step.
  FOR qrec IN SELECT id, embedding FROM real_100k_ivf_rabitq1_rerank_corpus
              WHERE id IN (SELECT id FROM real_100k_ivf_rabitq1_rerank_corpus ORDER BY id LIMIT 100) LOOP
    t_start := clock_timestamp();
    PERFORM (SELECT array_agg(id ORDER BY embedding <#> qrec.embedding)
             FROM (SELECT id, embedding FROM real_100k_ivf_rabitq1_rerank_corpus
                   ORDER BY embedding <#> qrec.embedding LIMIT k) s);
    t_end := clock_timestamp();
    cnt := cnt + 1;
    total_ms := total_ms + EXTRACT(EPOCH FROM (t_end - t_start)) * 1000.0;
    per_query := array_append(per_query, EXTRACT(EPOCH FROM (t_end - t_start)) * 1000.0);
  END LOOP;

  RAISE NOTICE 'BENCH-RESULT scale=100k corpus=real_100k_ivf_rabitq1_rerank_corpus idx=real_100k_ivf_rabitq1_rerank_rabitq_idx queries=% mean_ms=% total_ms=%',
    cnt,
    round((total_ms/cnt)::numeric, 3),
    round(total_ms::numeric, 1);

  RAISE NOTICE 'BENCH-LATENCY-PCTL scale=100k p50=% p95=% p99=% min=% max=%',
    round((SELECT percentile_cont(0.50) WITHIN GROUP (ORDER BY x) FROM unnest(per_query) x)::numeric, 3),
    round((SELECT percentile_cont(0.95) WITHIN GROUP (ORDER BY x) FROM unnest(per_query) x)::numeric, 3),
    round((SELECT percentile_cont(0.99) WITHIN GROUP (ORDER BY x) FROM unnest(per_query) x)::numeric, 3),
    round((SELECT min(x) FROM unnest(per_query) x)::numeric, 3),
    round((SELECT max(x) FROM unnest(per_query) x)::numeric, 3);
END
$bench$;

\echo --- planner sanity (EXPLAIN ANALYZE one query) ---
EXPLAIN (ANALYZE, BUFFERS, TIMING ON)
WITH probe AS (SELECT embedding FROM real_100k_ivf_rabitq1_rerank_corpus ORDER BY id LIMIT 1)
SELECT id FROM real_100k_ivf_rabitq1_rerank_corpus
ORDER BY embedding <#> (SELECT embedding FROM probe)
LIMIT 10;
