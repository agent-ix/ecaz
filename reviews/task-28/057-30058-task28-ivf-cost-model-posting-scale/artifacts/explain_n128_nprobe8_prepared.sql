\timing on
SET statement_timeout = '30s';
SET ec_ivf.nprobe = 8;

SELECT source::text AS qsource
FROM task28_ivf_postopt10k_n128w25_queries
ORDER BY id
LIMIT 1
\gset

PREPARE task28_ivf_n128_knn(real[], bigint) AS
SELECT id
FROM task28_ivf_postopt10k_n128w25_corpus
ORDER BY embedding <#> $1::real[]
LIMIT $2;

EXPLAIN (ANALYZE, BUFFERS)
EXECUTE task28_ivf_n128_knn(:'qsource'::real[], 10);
