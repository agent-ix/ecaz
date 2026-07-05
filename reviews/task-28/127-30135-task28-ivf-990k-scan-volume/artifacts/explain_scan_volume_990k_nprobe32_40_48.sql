\pset pager off
\timing on

SET enable_seqscan = off;
SET ec_ivf.rerank_width = 500;

SELECT
  current_setting('server_version') AS server_version,
  current_setting('ec_ivf.rerank_width') AS rerank_width,
  pg_relation_size('task28_ivf_pqg990k_g8_n128_idx'::regclass) AS index_bytes,
  pg_size_pretty(pg_relation_size('task28_ivf_pqg990k_g8_n128_idx'::regclass)) AS index_size;

SELECT count(*) AS corpus_rows FROM task28_ivf_pqg990k_g8_n128_corpus;
SELECT count(*) AS query_rows FROM task28_ivf_pqg990k_g8_n128_queries;

SET ec_ivf.nprobe = 32;
SELECT current_setting('ec_ivf.nprobe') AS nprobe;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task28_ivf_pqg990k_g8_n128_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task28_ivf_pqg990k_g8_n128_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_ivf.nprobe = 40;
SELECT current_setting('ec_ivf.nprobe') AS nprobe;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task28_ivf_pqg990k_g8_n128_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task28_ivf_pqg990k_g8_n128_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_ivf.nprobe = 48;
SELECT current_setting('ec_ivf.nprobe') AS nprobe;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task28_ivf_pqg990k_g8_n128_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task28_ivf_pqg990k_g8_n128_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

RESET enable_seqscan;
RESET ec_ivf.nprobe;
RESET ec_ivf.rerank_width;
