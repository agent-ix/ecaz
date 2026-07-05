\pset pager off
\timing on

SET enable_seqscan = off;
SET ec_ivf.nprobe = 48;
SET ec_ivf.rerank_width = 750;

SELECT
  current_setting('server_version') AS server_version,
  current_setting('ec_ivf.nprobe') AS nprobe,
  current_setting('ec_ivf.rerank_width') AS rerank_width;

SELECT
  'task31_m5_real50k_pqg8_n64_idx' AS index_name,
  pg_relation_size('task31_m5_real50k_pqg8_n64_idx'::regclass) AS index_bytes,
  pg_size_pretty(pg_relation_size('task31_m5_real50k_pqg8_n64_idx'::regclass)) AS index_size;

EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task31_m5_real50k_pqg8_n64_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task31_m5_real50k_pqg8_n64_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

RESET enable_seqscan;
RESET ec_ivf.nprobe;
RESET ec_ivf.rerank_width;
