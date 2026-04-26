\set ON_ERROR_STOP on
\timing on

\echo === pg18 concurrent DSM real 50k worker sweep ===
SELECT now() AS started_at,
       version() AS postgres_version,
       current_setting('server_version') AS server_version,
       current_setting('max_worker_processes') AS max_worker_processes,
       current_setting('max_parallel_workers') AS max_parallel_workers,
       current_setting('max_parallel_maintenance_workers') AS max_parallel_maintenance_workers;

CREATE EXTENSION IF NOT EXISTS ecaz;

\echo === fixture status ===
SELECT count(*) AS corpus_rows FROM tqhnsw_real_50k_reloaded_corpus;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_corpus') AS corpus_bytes;

SET maintenance_work_mem = '1GB';
SET ec_hnsw.enable_parallel_build_concurrent_dsm = on;

\echo === concurrent DSM source-scored build: 1 worker ===
SET max_parallel_workers = 8;
SET max_parallel_maintenance_workers = 1;
ALTER TABLE tqhnsw_real_50k_reloaded_corpus SET (parallel_workers = 1);
DROP INDEX IF EXISTS tqhnsw_real_50k_reloaded_m16_parallel_dsm_sweep_w1_idx;
CREATE INDEX tqhnsw_real_50k_reloaded_m16_parallel_dsm_sweep_w1_idx
    ON tqhnsw_real_50k_reloaded_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'concurrent_dsm_source_w1' AS build_path, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS concurrent_dsm_graph_workers_launched;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_m16_parallel_dsm_sweep_w1_idx') AS index_bytes;

\echo === concurrent DSM source-scored build: 2 workers ===
SET max_parallel_maintenance_workers = 2;
ALTER TABLE tqhnsw_real_50k_reloaded_corpus SET (parallel_workers = 2);
DROP INDEX IF EXISTS tqhnsw_real_50k_reloaded_m16_parallel_dsm_sweep_w2_idx;
CREATE INDEX tqhnsw_real_50k_reloaded_m16_parallel_dsm_sweep_w2_idx
    ON tqhnsw_real_50k_reloaded_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'concurrent_dsm_source_w2' AS build_path, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS concurrent_dsm_graph_workers_launched;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_m16_parallel_dsm_sweep_w2_idx') AS index_bytes;

\echo === concurrent DSM source-scored build: 4 workers ===
SET max_parallel_maintenance_workers = 4;
ALTER TABLE tqhnsw_real_50k_reloaded_corpus SET (parallel_workers = 4);
DROP INDEX IF EXISTS tqhnsw_real_50k_reloaded_m16_parallel_dsm_sweep_w4_idx;
CREATE INDEX tqhnsw_real_50k_reloaded_m16_parallel_dsm_sweep_w4_idx
    ON tqhnsw_real_50k_reloaded_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'concurrent_dsm_source_w4' AS build_path, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS concurrent_dsm_graph_workers_launched;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_m16_parallel_dsm_sweep_w4_idx') AS index_bytes;

\echo === concurrent DSM source-scored build: 8 workers ===
SET max_parallel_maintenance_workers = 8;
ALTER TABLE tqhnsw_real_50k_reloaded_corpus SET (parallel_workers = 8);
DROP INDEX IF EXISTS tqhnsw_real_50k_reloaded_m16_parallel_dsm_sweep_w8_idx;
CREATE INDEX tqhnsw_real_50k_reloaded_m16_parallel_dsm_sweep_w8_idx
    ON tqhnsw_real_50k_reloaded_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'concurrent_dsm_source_w8' AS build_path, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS concurrent_dsm_graph_workers_launched;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_m16_parallel_dsm_sweep_w8_idx') AS index_bytes;

SELECT now() AS finished_at;
