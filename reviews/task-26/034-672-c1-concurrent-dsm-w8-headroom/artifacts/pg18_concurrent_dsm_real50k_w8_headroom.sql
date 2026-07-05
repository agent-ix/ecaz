\set ON_ERROR_STOP on
\timing on

\echo === pg18 concurrent DSM real 50k w8 headroom diagnostic ===
SELECT now() AS started_at,
       version() AS postgres_version,
       current_setting('server_version') AS server_version,
       current_setting('max_worker_processes') AS max_worker_processes,
       current_setting('max_parallel_workers') AS max_parallel_workers,
       current_setting('max_parallel_maintenance_workers') AS max_parallel_maintenance_workers;

CREATE EXTENSION IF NOT EXISTS ecaz;

-- The long-lived benchmark database may already have the extension installed
-- with an older test-only debug function signature. Refresh this debug surface
-- without dropping the extension or benchmark indexes.
ALTER EXTENSION ecaz DROP FUNCTION tests.ec_hnsw_debug_last_build_timing();
DROP FUNCTION IF EXISTS tests.ec_hnsw_debug_last_build_timing();
CREATE FUNCTION tests.ec_hnsw_debug_last_build_timing() RETURNS TABLE (
    requested_workers bigint,
    workers_launched bigint,
    heap_workers_launched bigint,
    graph_workers_launched bigint,
    heap_tuples bigint,
    index_tuples bigint,
    heap_ingest_us bigint,
    parallel_begin_us bigint,
    parallel_drain_us bigint,
    parallel_sort_push_us bigint,
    flush_total_us bigint,
    graph_us bigint,
    stage_us bigint,
    write_us bigint
)
STRICT
LANGUAGE c
AS '$libdir/ecaz', 'ec_hnsw_debug_last_build_timing_wrapper';
ALTER EXTENSION ecaz ADD FUNCTION tests.ec_hnsw_debug_last_build_timing();

\echo === fixture status ===
SELECT count(*) AS corpus_rows FROM tqhnsw_real_50k_reloaded_corpus;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_corpus') AS corpus_bytes;

SET maintenance_work_mem = '1GB';
SET ec_hnsw.enable_parallel_build_concurrent_dsm = on;
SET max_parallel_workers = 16;
SET max_parallel_maintenance_workers = 8;
ALTER TABLE tqhnsw_real_50k_reloaded_corpus SET (parallel_workers = 8);

\echo === concurrent DSM source-scored build: 8 workers with headroom ===
DROP INDEX IF EXISTS tqhnsw_real_50k_reloaded_m16_parallel_dsm_w8_headroom_idx;
CREATE INDEX tqhnsw_real_50k_reloaded_m16_parallel_dsm_w8_headroom_idx
    ON tqhnsw_real_50k_reloaded_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'concurrent_dsm_source_w8_headroom' AS build_path, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS concurrent_dsm_graph_workers_launched;
SELECT pg_relation_size('tqhnsw_real_50k_reloaded_m16_parallel_dsm_w8_headroom_idx') AS index_bytes;

SELECT now() AS finished_at;
