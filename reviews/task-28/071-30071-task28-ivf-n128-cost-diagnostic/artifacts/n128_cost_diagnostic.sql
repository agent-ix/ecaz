\timing on
\pset pager off
LOAD 'ecaz';
SET statement_timeout = '60s';
SET enable_seqscan = on;

SELECT version();
SHOW random_page_cost;
SHOW seq_page_cost;
SHOW cpu_operator_cost;
SHOW ec_ivf.nprobe;

SELECT
    'n64' AS surface,
    pg_relation_size('task28_ivf_postopt10k_n64w25_idx'::regclass) AS index_bytes,
    (SELECT count(*) FROM task28_ivf_postopt10k_n64w25_corpus) AS corpus_rows,
    (SELECT count(*) FROM task28_ivf_postopt10k_n64w25_queries) AS query_rows
UNION ALL
SELECT
    'n128' AS surface,
    pg_relation_size('task28_ivf_postopt10k_n128w25_idx'::regclass) AS index_bytes,
    (SELECT count(*) FROM task28_ivf_postopt10k_n128w25_corpus) AS corpus_rows,
    (SELECT count(*) FROM task28_ivf_postopt10k_n128w25_queries) AS query_rows;

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

SET ec_ivf.nprobe = 8;
SELECT 'normal planner explain n128 nprobe 8' AS section;
EXPLAIN (ANALYZE, BUFFERS)
EXECUTE task28_ivf_n128_knn(:'qsource'::real[], 10);

SET ec_ivf.nprobe = 16;
SELECT 'normal planner explain n128 nprobe 16' AS section;
EXPLAIN (ANALYZE, BUFFERS)
EXECUTE task28_ivf_n128_knn(:'qsource'::real[], 10);

SET ec_ivf.nprobe = 24;
SELECT 'normal planner explain n128 nprobe 24' AS section;
EXPLAIN (ANALYZE, BUFFERS)
EXECUTE task28_ivf_n128_knn(:'qsource'::real[], 10);

SET ec_ivf.nprobe = 32;
SELECT 'normal planner explain n128 nprobe 32' AS section;
EXPLAIN (ANALYZE, BUFFERS)
EXECUTE task28_ivf_n128_knn(:'qsource'::real[], 10);
