\set ON_ERROR_STOP on
\timing on

\echo === pg18 source-score backlink-cache concurrent DSM real 50k recall check ===
SELECT now() AS started_at,
       version() AS postgres_version,
       current_setting('server_version') AS server_version;

CREATE EXTENSION IF NOT EXISTS ecaz;
SET ec_hnsw.ef_search = 200;

DROP TABLE IF EXISTS tqhnsw_real_50k_reloaded_backlink_cache_queries_10;
CREATE TABLE tqhnsw_real_50k_reloaded_backlink_cache_queries_10 AS
SELECT *
FROM tqhnsw_real_50k_reloaded_queries
ORDER BY id
LIMIT 10;
ALTER TABLE tqhnsw_real_50k_reloaded_backlink_cache_queries_10 ADD PRIMARY KEY (id);
VACUUM ANALYZE tqhnsw_real_50k_reloaded_backlink_cache_queries_10;

\echo === serial baseline recall ===
CREATE TEMP TABLE serial_real_50k_recall AS
SELECT 'serial_existing' AS build_path, *
FROM tests.ec_hnsw_graph_scan_recall_external_summary(
    'tqhnsw_real_50k_reloaded_corpus',
    'tqhnsw_real_50k_reloaded_backlink_cache_queries_10',
    'tqhnsw_real_50k_reloaded_m16_idx',
    16,
    200
);
SELECT build_path,
       graph_recall_at_10,
       graph_recall_at_100,
       exact_quantized_recall_at_10,
       ndcg_at_10,
       graph_below_exact_queries,
       worst_exact_gap
FROM serial_real_50k_recall;

\echo === backlink-cache concurrent DSM recall ===
CREATE TEMP TABLE backlink_cache_dsm_real_50k_recall AS
SELECT 'concurrent_dsm_source_backlink_cache' AS build_path, *
FROM tests.ec_hnsw_graph_scan_recall_external_summary(
    'tqhnsw_real_50k_reloaded_corpus',
    'tqhnsw_real_50k_reloaded_backlink_cache_queries_10',
    'tqhnsw_real_50k_reloaded_m16_parallel_dsm_backlink_cache_idx',
    16,
    200
);
SELECT build_path,
       graph_recall_at_10,
       graph_recall_at_100,
       exact_quantized_recall_at_10,
       ndcg_at_10,
       graph_below_exact_queries,
       worst_exact_gap
FROM backlink_cache_dsm_real_50k_recall;

\echo === recall delta ===
SELECT serial_summary.graph_recall_at_10 AS serial_graph_recall_at_10,
       backlink_cache_summary.graph_recall_at_10 AS backlink_cache_dsm_graph_recall_at_10,
       backlink_cache_summary.graph_recall_at_10 - serial_summary.graph_recall_at_10 AS recall_delta,
       serial_summary.graph_recall_at_100 AS serial_graph_recall_at_100,
       backlink_cache_summary.graph_recall_at_100 AS backlink_cache_dsm_graph_recall_at_100,
       backlink_cache_summary.graph_recall_at_100 - serial_summary.graph_recall_at_100 AS recall_100_delta,
       backlink_cache_summary.graph_below_exact_queries AS backlink_cache_dsm_graph_below_exact_queries,
       backlink_cache_summary.worst_exact_gap AS backlink_cache_dsm_worst_exact_gap
FROM serial_real_50k_recall AS serial_summary
CROSS JOIN backlink_cache_dsm_real_50k_recall AS backlink_cache_summary;

SELECT now() AS finished_at;
