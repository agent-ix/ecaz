\set ON_ERROR_STOP on
\timing on

\echo === pg18 source-score backlink-cache concurrent DSM real 50k timing ===
SELECT now() AS started_at,
       version() AS postgres_version,
       current_setting('server_version') AS server_version,
       current_setting('max_worker_processes') AS max_worker_processes;

CREATE EXTENSION IF NOT EXISTS ecaz;

\echo === fixture status ===
SELECT count(*) AS corpus_rows FROM tqhnsw_real_50k_reloaded_corpus;
SELECT count(*) AS query_rows FROM tqhnsw_real_50k_reloaded_queries;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_corpus') AS corpus_bytes;

SET maintenance_work_mem = '1GB';
SET max_parallel_workers = 8;
SET max_parallel_maintenance_workers = 4;
SET ec_hnsw.enable_parallel_build_concurrent_dsm = on;
ALTER TABLE tqhnsw_real_50k_reloaded_corpus SET (parallel_workers = 4);

\echo === concurrent DSM source-scored sidecar build after backlink target score reuse ===
DROP INDEX IF EXISTS tqhnsw_real_50k_reloaded_m16_parallel_dsm_backlink_cache_idx;
CREATE INDEX tqhnsw_real_50k_reloaded_m16_parallel_dsm_backlink_cache_idx
    ON tqhnsw_real_50k_reloaded_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'concurrent_dsm_source_backlink_cache' AS build_path, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS concurrent_dsm_graph_workers_launched;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_m16_parallel_dsm_backlink_cache_idx') AS concurrent_dsm_backlink_cache_index_bytes;

SELECT now() AS finished_at;
