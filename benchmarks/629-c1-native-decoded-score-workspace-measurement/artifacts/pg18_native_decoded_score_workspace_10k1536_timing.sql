\set ON_ERROR_STOP on
\timing on

\echo === pg18 native decoded score workspace 10k x 1536 timing fixture ===
SELECT now() AS started_at,
       version() AS postgres_version,
       current_setting('server_version') AS server_version,
       current_setting('max_worker_processes') AS max_worker_processes;

DROP TABLE IF EXISTS ec_hnsw_native_decoded_score_10k1536_measure CASCADE;
DROP EXTENSION IF EXISTS ecaz CASCADE;
CREATE EXTENSION ecaz;

CREATE TABLE ec_hnsw_native_decoded_score_10k1536_measure (
    id bigint PRIMARY KEY,
    embedding tqvector NOT NULL
);

\echo === load 10000 rows x 1536 dims ===
INSERT INTO ec_hnsw_native_decoded_score_10k1536_measure
SELECT id,
       encode_to_tqvector(
           ARRAY(
               SELECT (
                   sin((id * dim)::double precision) +
                   cos((id + dim * 17)::double precision)
               )::real
               FROM generate_series(1, 1536) AS dim
           ),
           4,
           42
       )
FROM generate_series(1, 10000) AS id;

VACUUM ANALYZE ec_hnsw_native_decoded_score_10k1536_measure;
SELECT count(*) AS fixture_rows FROM ec_hnsw_native_decoded_score_10k1536_measure;
SELECT relpages, reltuples
FROM pg_class
WHERE oid = 'ec_hnsw_native_decoded_score_10k1536_measure'::regclass;

SET maintenance_work_mem = '1GB';
SET max_parallel_workers = 8;

\echo === serial create index 10k x 1536 ===
SET max_parallel_maintenance_workers = 0;
ALTER TABLE ec_hnsw_native_decoded_score_10k1536_measure SET (parallel_workers = 0);
DROP INDEX IF EXISTS ec_hnsw_native_decoded_score_10k1536_measure_idx;
CREATE INDEX ec_hnsw_native_decoded_score_10k1536_measure_idx
    ON ec_hnsw_native_decoded_score_10k1536_measure
    USING ec_hnsw (embedding tqvector_ip_ops)
    WITH (m = 6, ef_construction = 40);
SELECT 'serial_10k1536' AS round, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT pg_relation_size('ec_hnsw_native_decoded_score_10k1536_measure_idx') AS serial_10k1536_index_bytes;
DROP INDEX ec_hnsw_native_decoded_score_10k1536_measure_idx;
CHECKPOINT;

\echo === parallel create index 10k x 1536 ===
SET max_parallel_maintenance_workers = 4;
ALTER TABLE ec_hnsw_native_decoded_score_10k1536_measure SET (parallel_workers = 4);
DROP INDEX IF EXISTS ec_hnsw_native_decoded_score_10k1536_measure_idx;
CREATE INDEX ec_hnsw_native_decoded_score_10k1536_measure_idx
    ON ec_hnsw_native_decoded_score_10k1536_measure
    USING ec_hnsw (embedding tqvector_ip_ops)
    WITH (m = 6, ef_construction = 40);
SELECT 'parallel_10k1536' AS round, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT pg_relation_size('ec_hnsw_native_decoded_score_10k1536_measure_idx') AS parallel_10k1536_index_bytes;

SELECT now() AS finished_at;
