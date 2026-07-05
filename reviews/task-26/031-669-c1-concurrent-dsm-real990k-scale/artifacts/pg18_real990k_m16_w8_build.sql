\set ON_ERROR_STOP on
\timing on

\echo === pg18 concurrent DSM real 990k m16 w8 build ===
SELECT now() AS started_at,
       version() AS postgres_version,
       current_setting('server_version') AS server_version,
       current_setting('max_worker_processes') AS max_worker_processes,
       current_setting('max_parallel_workers') AS max_parallel_workers,
       current_setting('max_parallel_maintenance_workers') AS max_parallel_maintenance_workers;

CREATE EXTENSION IF NOT EXISTS ecaz;

\echo === fixture status ===
SELECT count(*) AS corpus_rows FROM ec_hnsw_real_ann_benchmarks_anchor_corpus;
SELECT count(*) AS query_rows FROM ec_hnsw_real_ann_benchmarks_anchor_queries;
SELECT chunk_kind, count(*) AS chunks, sum(row_count) AS rows
FROM ecaz_corpus_load_state
WHERE prefix = 'ec_hnsw_real_ann_benchmarks_anchor'
GROUP BY chunk_kind
ORDER BY chunk_kind;

SET maintenance_work_mem = '1GB';
SET ec_hnsw.enable_parallel_build_concurrent_dsm = on;
SET max_parallel_workers = 16;
SET max_parallel_maintenance_workers = 8;
ALTER TABLE ec_hnsw_real_ann_benchmarks_anchor_corpus SET (parallel_workers = 8);

\echo === controlled concurrent DSM source-scored build: m16 w8 ===
DROP INDEX IF EXISTS ec_hnsw_real_ann_benchmarks_anchor_m16_w8_idx;
CREATE INDEX ec_hnsw_real_ann_benchmarks_anchor_m16_w8_idx
    ON ec_hnsw_real_ann_benchmarks_anchor_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'real990k_m16_w8' AS build_path, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS concurrent_dsm_graph_workers_launched;
SELECT pg_relation_size('ec_hnsw_real_ann_benchmarks_anchor_m16_w8_idx') AS index_bytes;

SELECT now() AS finished_at;
