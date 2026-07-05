\timing on
\pset pager off
LOAD 'ecaz';
SET statement_timeout = '120s';
SET enable_seqscan = on;

SELECT version();
SHOW random_page_cost;
SHOW seq_page_cost;
SHOW cpu_operator_cost;
SHOW ec_ivf.nprobe;

SELECT relname
FROM pg_class
WHERE relname LIKE 'task28_ivf_postopt10k%'
ORDER BY relname;

SELECT source::text AS qsource
FROM task28_ivf_postopt10k_n128w25_queries
ORDER BY id
LIMIT 1
\gset

SET ec_ivf.nprobe = 8;
SELECT 'knn limit 10 n128 nprobe 8' AS section;
EXPLAIN (ANALYZE, BUFFERS)
SELECT id
FROM task28_ivf_postopt10k_n128w25_corpus
ORDER BY embedding <#> :'qsource'::real[]
LIMIT 10;

SET ec_ivf.nprobe = 32;
SELECT 'knn limit 10 n128 nprobe 32' AS section;
EXPLAIN (ANALYZE, BUFFERS)
SELECT id
FROM task28_ivf_postopt10k_n128w25_corpus
ORDER BY embedding <#> :'qsource'::real[]
LIMIT 10;

SET ec_ivf.nprobe = 64;
SELECT 'knn large limit n128 nprobe 64' AS section;
EXPLAIN (ANALYZE, BUFFERS)
SELECT id
FROM task28_ivf_postopt10k_n128w25_corpus
ORDER BY embedding <#> :'qsource'::real[]
LIMIT 1000;

SELECT 'non-knn count should not use IVF' AS section;
EXPLAIN (ANALYZE, BUFFERS)
SELECT count(*)
FROM task28_ivf_postopt10k_n128w25_corpus
WHERE id <= 100;

SELECT 'mixed predicate knn id <= 1000' AS section;
EXPLAIN (ANALYZE, BUFFERS)
SELECT id
FROM task28_ivf_postopt10k_n128w25_corpus
WHERE id <= 1000
ORDER BY embedding <#> :'qsource'::real[]
LIMIT 10;
