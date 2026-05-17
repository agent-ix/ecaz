\timing on
\pset pager off
LOAD 'ecaz';
SET statement_timeout = '300s';
SET enable_seqscan = on;
SET ec_ivf.nprobe = 32;
SET ec_hnsw.ef_search = 64;

SELECT version();
SHOW random_page_cost;
SHOW seq_page_cost;
SHOW cpu_operator_cost;
SHOW ec_ivf.nprobe;
SHOW ec_hnsw.ef_search;

CREATE INDEX IF NOT EXISTS task28_ivf_postopt10k_n128w25_hnsw_idx
ON task28_ivf_postopt10k_n128w25_corpus
USING ec_hnsw (embedding ecvector_ip_ops)
WITH (m = 16, ef_construction = 128);

ANALYZE task28_ivf_postopt10k_n128w25_corpus;

SELECT
    relname,
    amname,
    pg_relation_size(pg_class.oid) AS bytes
FROM pg_class
JOIN pg_index ON pg_index.indexrelid = pg_class.oid
JOIN pg_am ON pg_am.oid = pg_class.relam
WHERE indrelid = 'task28_ivf_postopt10k_n128w25_corpus'::regclass
ORDER BY relname;

SELECT 'shape 1: non-prepared knn limit 10' AS section;
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

SELECT 'shape 2: non-prepared knn limit 1000' AS section;
EXPLAIN (ANALYZE, BUFFERS)
SELECT id
FROM task28_ivf_postopt10k_n128w25_corpus
ORDER BY embedding <#> (
    SELECT source
    FROM task28_ivf_postopt10k_n128w25_queries
    ORDER BY id
    LIMIT 1
)
LIMIT 1000;

SELECT 'shape 3: mixed predicate id <= 1000 limit 10' AS section;
EXPLAIN (ANALYZE, BUFFERS)
SELECT id
FROM task28_ivf_postopt10k_n128w25_corpus
WHERE id <= 1000
ORDER BY embedding <#> (
    SELECT source
    FROM task28_ivf_postopt10k_n128w25_queries
    ORDER BY id
    LIMIT 1
)
LIMIT 10;

SELECT 'shape 4: non-knn selective count' AS section;
EXPLAIN (ANALYZE, BUFFERS)
SELECT count(*)
FROM task28_ivf_postopt10k_n128w25_corpus
WHERE id <= 100;

SELECT 'shape 5: low-selectivity non-knn count' AS section;
EXPLAIN (ANALYZE, BUFFERS)
SELECT count(*)
FROM task28_ivf_postopt10k_n128w25_corpus
WHERE id > 0;
