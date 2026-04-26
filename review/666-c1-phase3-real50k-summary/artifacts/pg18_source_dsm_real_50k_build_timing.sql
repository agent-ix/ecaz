\set ON_ERROR_STOP on
\timing on

\echo === pg18 source-scored real 50k serial vs concurrent DSM build timing ===
SELECT now() AS started_at,
       version() AS postgres_version,
       current_setting('server_version') AS server_version,
       current_setting('max_worker_processes') AS max_worker_processes;

CREATE EXTENSION IF NOT EXISTS ecaz;

\echo === fixture status ===
SELECT count(*) AS corpus_rows FROM tqhnsw_real_50k_reloaded_corpus;
SELECT count(*) AS query_rows FROM tqhnsw_real_50k_reloaded_queries;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_corpus') AS corpus_bytes;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_m16_idx') AS existing_serial_index_bytes;

SET maintenance_work_mem = '1GB';
SET max_parallel_workers = 8;
SET max_parallel_maintenance_workers = 4;
ALTER TABLE tqhnsw_real_50k_reloaded_corpus SET (parallel_workers = 4);

\echo === serial source-scored sidecar build ===
DROP INDEX IF EXISTS tqhnsw_real_50k_reloaded_m16_serial_timing_idx;
SET ec_hnsw.enable_parallel_build_concurrent_dsm = off;
CREATE INDEX tqhnsw_real_50k_reloaded_m16_serial_timing_idx
    ON tqhnsw_real_50k_reloaded_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'serial_source' AS build_path, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS serial_graph_workers_launched;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_m16_serial_timing_idx') AS serial_timing_index_bytes;

\echo === concurrent DSM source-scored sidecar build ===
DROP INDEX IF EXISTS tqhnsw_real_50k_reloaded_m16_parallel_dsm_timing_idx;
SET ec_hnsw.enable_parallel_build_concurrent_dsm = on;
CREATE INDEX tqhnsw_real_50k_reloaded_m16_parallel_dsm_timing_idx
    ON tqhnsw_real_50k_reloaded_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'concurrent_dsm_source' AS build_path, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS concurrent_dsm_graph_workers_launched;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_m16_parallel_dsm_timing_idx') AS concurrent_dsm_timing_index_bytes;

\echo === timing sidecar size comparison ===
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_m16_serial_timing_idx') AS serial_timing_index_bytes,
       pg_relation_size('tqhnsw_real_50k_reloaded_m16_parallel_dsm_timing_idx') AS concurrent_dsm_timing_index_bytes;

SELECT now() AS finished_at;
