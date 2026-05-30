\set ON_ERROR_STOP on
\timing on

\echo === Task 33 HNSW M5 real50K worker sweep ===
SELECT now() AS started_at,
       version() AS postgres_version,
       current_setting('server_version') AS server_version,
       current_setting('max_worker_processes') AS max_worker_processes,
       current_setting('max_parallel_workers') AS max_parallel_workers,
       current_setting('max_parallel_maintenance_workers') AS max_parallel_maintenance_workers,
       current_setting('maintenance_work_mem') AS maintenance_work_mem;

CREATE EXTENSION IF NOT EXISTS ecaz;

\echo === fixture status ===
SELECT count(*) AS corpus_rows
FROM task33_m5_hnsw_real50k_corpus;
SELECT pg_relation_size('task33_m5_hnsw_real50k_corpus') AS corpus_bytes;

SET maintenance_work_mem = '1GB';
SET max_parallel_workers = 8;
SET ec_hnsw.enable_parallel_build_concurrent_dsm = on;

\echo === drop setup index from load step before worker sweep ===
DROP INDEX IF EXISTS task33_m5_hnsw_real50k_m16_idx;

\echo === concurrent DSM source-scored build: 1 worker ===
SET max_parallel_maintenance_workers = 1;
ALTER TABLE task33_m5_hnsw_real50k_corpus SET (parallel_workers = 1);
DROP INDEX IF EXISTS task33_m5_hnsw_real50k_w1_idx;
SELECT pg_stat_get_db_parallel_workers_launched(oid) AS w1_parallel_before
FROM pg_database
WHERE datname = current_database()
\gset
SELECT clock_timestamp() AS w1_started_at \gset
CREATE INDEX task33_m5_hnsw_real50k_w1_idx
    ON task33_m5_hnsw_real50k_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'task33_hnsw_real50k_w1' AS build_path,
       1 AS requested_workers,
       :'w1_parallel_before'::bigint AS parallel_workers_before,
       pg_stat_get_db_parallel_workers_launched(oid) AS parallel_workers_after,
       pg_stat_get_db_parallel_workers_launched(oid) - :'w1_parallel_before'::bigint
           AS parallel_workers_launched_delta,
       :'w1_started_at'::timestamptz AS started_at,
       clock_timestamp() AS finished_at,
       EXTRACT(EPOCH FROM clock_timestamp() - :'w1_started_at'::timestamptz)
           AS build_wall_seconds,
       pg_relation_size('task33_m5_hnsw_real50k_w1_idx') AS index_bytes
FROM pg_database
WHERE datname = current_database();

\echo === concurrent DSM source-scored build: 2 workers ===
SET max_parallel_maintenance_workers = 2;
ALTER TABLE task33_m5_hnsw_real50k_corpus SET (parallel_workers = 2);
DROP INDEX IF EXISTS task33_m5_hnsw_real50k_w2_idx;
SELECT pg_stat_get_db_parallel_workers_launched(oid) AS w2_parallel_before
FROM pg_database
WHERE datname = current_database()
\gset
SELECT clock_timestamp() AS w2_started_at \gset
CREATE INDEX task33_m5_hnsw_real50k_w2_idx
    ON task33_m5_hnsw_real50k_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'task33_hnsw_real50k_w2' AS build_path,
       2 AS requested_workers,
       :'w2_parallel_before'::bigint AS parallel_workers_before,
       pg_stat_get_db_parallel_workers_launched(oid) AS parallel_workers_after,
       pg_stat_get_db_parallel_workers_launched(oid) - :'w2_parallel_before'::bigint
           AS parallel_workers_launched_delta,
       :'w2_started_at'::timestamptz AS started_at,
       clock_timestamp() AS finished_at,
       EXTRACT(EPOCH FROM clock_timestamp() - :'w2_started_at'::timestamptz)
           AS build_wall_seconds,
       pg_relation_size('task33_m5_hnsw_real50k_w2_idx') AS index_bytes
FROM pg_database
WHERE datname = current_database();

\echo === concurrent DSM source-scored build: 4 workers ===
SET max_parallel_maintenance_workers = 4;
ALTER TABLE task33_m5_hnsw_real50k_corpus SET (parallel_workers = 4);
DROP INDEX IF EXISTS task33_m5_hnsw_real50k_w4_idx;
SELECT pg_stat_get_db_parallel_workers_launched(oid) AS w4_parallel_before
FROM pg_database
WHERE datname = current_database()
\gset
SELECT clock_timestamp() AS w4_started_at \gset
CREATE INDEX task33_m5_hnsw_real50k_w4_idx
    ON task33_m5_hnsw_real50k_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'task33_hnsw_real50k_w4' AS build_path,
       4 AS requested_workers,
       :'w4_parallel_before'::bigint AS parallel_workers_before,
       pg_stat_get_db_parallel_workers_launched(oid) AS parallel_workers_after,
       pg_stat_get_db_parallel_workers_launched(oid) - :'w4_parallel_before'::bigint
           AS parallel_workers_launched_delta,
       :'w4_started_at'::timestamptz AS started_at,
       clock_timestamp() AS finished_at,
       EXTRACT(EPOCH FROM clock_timestamp() - :'w4_started_at'::timestamptz)
           AS build_wall_seconds,
       pg_relation_size('task33_m5_hnsw_real50k_w4_idx') AS index_bytes
FROM pg_database
WHERE datname = current_database();

\echo === concurrent DSM source-scored build: 8 workers ===
SET max_parallel_maintenance_workers = 8;
ALTER TABLE task33_m5_hnsw_real50k_corpus SET (parallel_workers = 8);
DROP INDEX IF EXISTS task33_m5_hnsw_real50k_w8_idx;
SELECT pg_stat_get_db_parallel_workers_launched(oid) AS w8_parallel_before
FROM pg_database
WHERE datname = current_database()
\gset
SELECT clock_timestamp() AS w8_started_at \gset
CREATE INDEX task33_m5_hnsw_real50k_w8_idx
    ON task33_m5_hnsw_real50k_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128, build_source_column = source);
SELECT 'task33_hnsw_real50k_w8' AS build_path,
       8 AS requested_workers,
       :'w8_parallel_before'::bigint AS parallel_workers_before,
       pg_stat_get_db_parallel_workers_launched(oid) AS parallel_workers_after,
       pg_stat_get_db_parallel_workers_launched(oid) - :'w8_parallel_before'::bigint
           AS parallel_workers_launched_delta,
       :'w8_started_at'::timestamptz AS started_at,
       clock_timestamp() AS finished_at,
       EXTRACT(EPOCH FROM clock_timestamp() - :'w8_started_at'::timestamptz)
           AS build_wall_seconds,
       pg_relation_size('task33_m5_hnsw_real50k_w8_idx') AS index_bytes
FROM pg_database
WHERE datname = current_database();

\echo === select best worker index for follow-on recall/latency/storage steps ===
DROP INDEX IF EXISTS task33_m5_hnsw_real50k_m16_idx;
ALTER INDEX task33_m5_hnsw_real50k_w4_idx
    RENAME TO task33_m5_hnsw_real50k_m16_idx;

SELECT now() AS finished_at;
