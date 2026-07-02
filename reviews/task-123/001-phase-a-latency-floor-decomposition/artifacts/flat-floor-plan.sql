\pset pager off
\timing on

SET enable_seqscan = on;
SET enable_indexscan = off;
SET enable_indexonlyscan = off;
SET enable_bitmapscan = off;

SELECT
  'task123_phase_a_flat_floor_plan' AS evidence,
  current_setting('server_version') AS server_version,
  current_setting('port') AS port,
  current_setting('enable_seqscan') AS enable_seqscan,
  current_setting('enable_indexscan') AS enable_indexscan,
  current_setting('enable_indexonlyscan') AS enable_indexonlyscan,
  current_setting('enable_bitmapscan') AS enable_bitmapscan;

SELECT
  c.relname AS table_name,
  i.relname AS index_name,
  am.amname,
  pg_relation_size(i.oid) AS index_bytes,
  cidx.reloptions
FROM pg_index x
JOIN pg_class c ON c.oid = x.indrelid
JOIN pg_class i ON i.oid = x.indexrelid
JOIN pg_class cidx ON cidx.oid = x.indexrelid
JOIN pg_am am ON am.oid = i.relam
WHERE c.relname IN (
  't121_s3_10k_b4_tr50_f8_b64_corpus',
  't121_s3_50k_b4_tr50_f8_b64_corpus',
  't121_s3_100k_b4_tr50_f8_b64_corpus'
)
ORDER BY c.relname, i.relname;

EXPLAIN (ANALYZE, COSTS OFF)
SELECT id
FROM t121_s3_10k_b4_tr50_f8_b64_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM t121_s3_10k_b4_tr50_f8_b64_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

EXPLAIN (ANALYZE, COSTS OFF)
SELECT id
FROM t121_s3_50k_b4_tr50_f8_b64_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM t121_s3_50k_b4_tr50_f8_b64_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

EXPLAIN (ANALYZE, COSTS OFF)
SELECT id
FROM t121_s3_100k_b4_tr50_f8_b64_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM t121_s3_100k_b4_tr50_f8_b64_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

RESET enable_seqscan;
RESET enable_indexscan;
RESET enable_indexonlyscan;
RESET enable_bitmapscan;
