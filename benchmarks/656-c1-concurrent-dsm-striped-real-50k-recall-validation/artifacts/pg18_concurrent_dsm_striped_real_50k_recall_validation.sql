\set ON_ERROR_STOP on
\timing on

\echo === pg18 striped concurrent DSM real 50k recall validation ===
SELECT now() AS started_at,
       version() AS postgres_version,
       current_setting('server_version') AS server_version,
       current_setting('max_worker_processes') AS max_worker_processes;

CREATE EXTENSION IF NOT EXISTS ecaz;

\echo === real corpus fixture status ===
SELECT count(*) AS corpus_rows FROM tqhnsw_real_50k_corpus;
SELECT count(*) AS query_rows FROM tqhnsw_real_50k_queries;
SELECT pg_relation_size('tqhnsw_real_50k_m16_idx') AS serial_index_bytes;

DROP TABLE IF EXISTS tqhnsw_real_50k_parallel_queries_10;
CREATE TABLE tqhnsw_real_50k_parallel_queries_10 AS
SELECT *
FROM tqhnsw_real_50k_queries
ORDER BY id
LIMIT 10;
ALTER TABLE tqhnsw_real_50k_parallel_queries_10 ADD PRIMARY KEY (id);
VACUUM ANALYZE tqhnsw_real_50k_parallel_queries_10;
SELECT count(*) AS query_subset_rows FROM tqhnsw_real_50k_parallel_queries_10;

SET maintenance_work_mem = '1GB';
SET max_parallel_workers = 8;
SET ec_hnsw.ef_search = 200;

\echo === existing serial-built real 50k m16 index recall at ef_search 200 ===
CREATE TEMP TABLE serial_real_50k_recall AS
SELECT 'serial_existing' AS build_path, *
FROM tests.ec_hnsw_graph_scan_recall_external_summary(
    'tqhnsw_real_50k_corpus',
    'tqhnsw_real_50k_parallel_queries_10',
    'tqhnsw_real_50k_m16_idx',
    16,
    200
);
SELECT build_path,
       graph_recall_at_10,
       graph_recall_at_100,
       exact_quantized_recall_at_10,
       ndcg_at_10,
       mean_abs_score_error,
       spearman_rho_at_10,
       graph_below_exact_queries,
       worst_exact_gap
FROM serial_real_50k_recall;

\echo === striped concurrent DSM-built real 50k m16 sidecar index ===
DROP INDEX IF EXISTS tqhnsw_real_50k_m16_parallel_dsm_idx;
SET ec_hnsw.enable_parallel_build_concurrent_dsm = on;
SET max_parallel_maintenance_workers = 4;
ALTER TABLE tqhnsw_real_50k_corpus SET (parallel_workers = 4);
CREATE INDEX tqhnsw_real_50k_m16_parallel_dsm_idx
    ON tqhnsw_real_50k_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'concurrent_dsm_striped' AS build_path, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS concurrent_dsm_graph_workers_launched;
SELECT pg_relation_size('tqhnsw_real_50k_m16_parallel_dsm_idx') AS concurrent_dsm_index_bytes;

\echo === striped concurrent DSM real 50k m16 recall at ef_search 200 ===
CREATE TEMP TABLE concurrent_dsm_real_50k_recall AS
SELECT 'concurrent_dsm_striped' AS build_path, *
FROM tests.ec_hnsw_graph_scan_recall_external_summary(
    'tqhnsw_real_50k_corpus',
    'tqhnsw_real_50k_parallel_queries_10',
    'tqhnsw_real_50k_m16_parallel_dsm_idx',
    16,
    200
);
SELECT build_path,
       graph_recall_at_10,
       graph_recall_at_100,
       exact_quantized_recall_at_10,
       ndcg_at_10,
       mean_abs_score_error,
       spearman_rho_at_10,
       graph_below_exact_queries,
       worst_exact_gap
FROM concurrent_dsm_real_50k_recall;

\echo === real 50k serial vs striped concurrent DSM recall delta at ef_search 200 ===
SELECT serial_summary.graph_recall_at_10 AS serial_graph_recall_at_10,
       dsm_summary.graph_recall_at_10 AS concurrent_dsm_graph_recall_at_10,
       dsm_summary.graph_recall_at_10 - serial_summary.graph_recall_at_10 AS recall_delta,
       serial_summary.graph_recall_at_100 AS serial_graph_recall_at_100,
       dsm_summary.graph_recall_at_100 AS concurrent_dsm_graph_recall_at_100,
       dsm_summary.graph_recall_at_100 - serial_summary.graph_recall_at_100 AS recall_100_delta,
       serial_summary.exact_quantized_recall_at_10 AS serial_exact_quantized_recall_at_10,
       dsm_summary.exact_quantized_recall_at_10 AS concurrent_dsm_exact_quantized_recall_at_10,
       dsm_summary.graph_below_exact_queries AS concurrent_dsm_graph_below_exact_queries,
       dsm_summary.worst_exact_gap AS concurrent_dsm_worst_exact_gap
FROM serial_real_50k_recall AS serial_summary
CROSS JOIN concurrent_dsm_real_50k_recall AS dsm_summary;

SELECT pg_relation_size('tqhnsw_real_50k_m16_idx') AS serial_index_bytes,
       pg_relation_size('tqhnsw_real_50k_m16_parallel_dsm_idx') AS concurrent_dsm_index_bytes;

SELECT now() AS finished_at;
