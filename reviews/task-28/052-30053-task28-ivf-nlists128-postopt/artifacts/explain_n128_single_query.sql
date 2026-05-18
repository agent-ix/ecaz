\timing on
SET statement_timeout = '30s';
SET ec_ivf.nprobe = 8;
EXPLAIN (ANALYZE, BUFFERS)
SELECT id
FROM task28_ivf_postopt10k_n128w25_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task28_ivf_postopt10k_n128w25_queries
  ORDER BY id
  LIMIT 1
)
LIMIT 10;
