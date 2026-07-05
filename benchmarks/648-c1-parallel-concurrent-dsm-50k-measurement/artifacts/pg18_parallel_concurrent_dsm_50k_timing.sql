\set ON_ERROR_STOP on
\timing on

\echo === pg18 parallel concurrent DSM graph build 50k timing fixture ===
SELECT now() AS started_at,
       version() AS postgres_version,
       current_setting('server_version') AS server_version,
       current_setting('max_worker_processes') AS max_worker_processes;

DROP TABLE IF EXISTS ec_hnsw_parallel_concurrent_dsm_50k_measure CASCADE;
DROP EXTENSION IF EXISTS ecaz CASCADE;
CREATE EXTENSION ecaz;

CREATE TABLE ec_hnsw_parallel_concurrent_dsm_50k_measure (
    id bigint PRIMARY KEY,
    embedding ecvector NOT NULL
);

\echo === load 50000 rows x 64 dims ===
INSERT INTO ec_hnsw_parallel_concurrent_dsm_50k_measure
SELECT id,
       encode_to_ecvector(
           ARRAY(
               SELECT (
                   sin((id * dim)::double precision) +
                   cos((id + dim * 17)::double precision)
               )::real
               FROM generate_series(1, 64) AS dim
           ),
           4,
           42
       )
FROM generate_series(1, 50000) AS id;

VACUUM ANALYZE ec_hnsw_parallel_concurrent_dsm_50k_measure;
SELECT count(*) AS fixture_rows FROM ec_hnsw_parallel_concurrent_dsm_50k_measure;
SELECT relpages, reltuples
FROM pg_class
WHERE oid = 'ec_hnsw_parallel_concurrent_dsm_50k_measure'::regclass;

SET maintenance_work_mem = '1GB';
SET max_parallel_workers = 8;

\echo === serial create index 50k ===
SET ec_hnsw.enable_parallel_build_concurrent_dsm = off;
SET max_parallel_maintenance_workers = 0;
ALTER TABLE ec_hnsw_parallel_concurrent_dsm_50k_measure SET (parallel_workers = 0);
DROP INDEX IF EXISTS ec_hnsw_parallel_concurrent_dsm_50k_measure_idx;
CREATE INDEX ec_hnsw_parallel_concurrent_dsm_50k_measure_idx
    ON ec_hnsw_parallel_concurrent_dsm_50k_measure
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 6, ef_construction = 40);
SELECT 'serial_50k' AS round, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS serial_graph_workers_launched;
SELECT pg_relation_size('ec_hnsw_parallel_concurrent_dsm_50k_measure_idx') AS serial_50k_index_bytes;
DROP INDEX ec_hnsw_parallel_concurrent_dsm_50k_measure_idx;
CHECKPOINT;

\echo === parallel serial graph create index 50k ===
SET ec_hnsw.enable_parallel_build_concurrent_dsm = off;
SET max_parallel_maintenance_workers = 4;
ALTER TABLE ec_hnsw_parallel_concurrent_dsm_50k_measure SET (parallel_workers = 4);
DROP INDEX IF EXISTS ec_hnsw_parallel_concurrent_dsm_50k_measure_idx;
CREATE INDEX ec_hnsw_parallel_concurrent_dsm_50k_measure_idx
    ON ec_hnsw_parallel_concurrent_dsm_50k_measure
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 6, ef_construction = 40);
SELECT 'parallel_serial_graph_50k' AS round, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS parallel_serial_graph_workers_launched;
SELECT pg_relation_size('ec_hnsw_parallel_concurrent_dsm_50k_measure_idx') AS parallel_serial_graph_50k_index_bytes;
DROP INDEX ec_hnsw_parallel_concurrent_dsm_50k_measure_idx;
CHECKPOINT;

\echo === parallel concurrent DSM graph create index 50k ===
SET ec_hnsw.enable_parallel_build_concurrent_dsm = on;
SET max_parallel_maintenance_workers = 4;
ALTER TABLE ec_hnsw_parallel_concurrent_dsm_50k_measure SET (parallel_workers = 4);
DROP INDEX IF EXISTS ec_hnsw_parallel_concurrent_dsm_50k_measure_idx;
CREATE INDEX ec_hnsw_parallel_concurrent_dsm_50k_measure_idx
    ON ec_hnsw_parallel_concurrent_dsm_50k_measure
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 6, ef_construction = 40);
SELECT 'parallel_concurrent_dsm_50k' AS round, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS parallel_concurrent_dsm_graph_workers_launched;
SELECT pg_relation_size('ec_hnsw_parallel_concurrent_dsm_50k_measure_idx') AS parallel_concurrent_dsm_50k_index_bytes;

SELECT now() AS finished_at;
