DROP INDEX IF EXISTS task28_a9_100k_hnsw_idx;

DO $$
DECLARE
  started_at timestamptz;
  elapsed_ms numeric;
BEGIN
  started_at := clock_timestamp();
  CREATE INDEX task28_a9_100k_hnsw_idx
    ON task28_a9_100k_hnsw_corpus USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128);
  elapsed_ms := extract(epoch FROM clock_timestamp() - started_at) * 1000.0;
  RAISE NOTICE 'task28_a9_100k_hnsw_idx build_ms=%', round(elapsed_ms, 3);
END $$;

ANALYZE task28_a9_100k_hnsw_corpus;
ANALYZE task28_a9_100k_hnsw_queries;

SELECT
  c.relname,
  pg_relation_size(c.oid) AS index_bytes,
  pg_size_pretty(pg_relation_size(c.oid)) AS index_size,
  c.reloptions
FROM pg_class c
WHERE c.relname = 'task28_a9_100k_hnsw_idx';
