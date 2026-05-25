\pset pager off
\timing on

SET enable_seqscan = off;
SET ec_hnsw.ef_search = 200;

SELECT
current_setting('server_version') AS server_version,
current_setting('ec_hnsw.ef_search') AS sweep_value,
'ec_hnsw' AS profile;

SELECT
'task61_opt_50k_hnsw_m16_idx' AS index_name,
pg_relation_size('task61_opt_50k_hnsw_m16_idx'::regclass) AS index_bytes,
pg_size_pretty(pg_relation_size('task61_opt_50k_hnsw_m16_idx'::regclass)) AS index_size;

SELECT *
FROM ec_hnsw_index_cost_snapshot('task61_opt_50k_hnsw_m16_idx'::regclass);

EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task61_opt_50k_hnsw_corpus
ORDER BY embedding <#> (
SELECT source
FROM task61_opt_50k_hnsw_queries
ORDER BY id
LIMIT 1
)::real[]
LIMIT 10;

RESET enable_seqscan;
RESET ec_hnsw.ef_search;
