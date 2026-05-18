\set ON_ERROR_STOP on
\timing on

\echo === pg18 parallel concurrent DSM tuned recall validation fixture ===
SELECT now() AS started_at,
       version() AS postgres_version,
       current_setting('server_version') AS server_version,
       current_setting('max_worker_processes') AS max_worker_processes;

DROP TABLE IF EXISTS ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus CASCADE;
DROP TABLE IF EXISTS ec_hnsw_parallel_concurrent_dsm_tuned_recall_queries CASCADE;
DROP EXTENSION IF EXISTS ecaz CASCADE;
CREATE EXTENSION ecaz;

CREATE TABLE ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus (
    id bigint PRIMARY KEY,
    source real[] NOT NULL,
    embedding ecvector NOT NULL
);

CREATE TABLE ec_hnsw_parallel_concurrent_dsm_tuned_recall_queries (
    id bigint PRIMARY KEY,
    source real[] NOT NULL
);

\echo === load 10000 corpus rows x 64 dims ===
INSERT INTO ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus
SELECT id,
       source,
       encode_to_ecvector(source, 4, 42)
FROM (
    SELECT id,
           ARRAY(
               SELECT (
                   sin((id * dim)::double precision) +
                   cos((id + dim * 17)::double precision)
               )::real
               FROM generate_series(1, 64) AS dim
           ) AS source
    FROM generate_series(1, 10000) AS id
) AS fixture;

\echo === load 100 query rows x 64 dims ===
INSERT INTO ec_hnsw_parallel_concurrent_dsm_tuned_recall_queries
SELECT id,
       ARRAY(
           SELECT (
               sin(((id + 100000) * dim)::double precision) +
               cos((id + 100000 + dim * 17)::double precision)
           )::real
           FROM generate_series(1, 64) AS dim
       ) AS source
FROM generate_series(1, 100) AS id;

VACUUM ANALYZE ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus;
VACUUM ANALYZE ec_hnsw_parallel_concurrent_dsm_tuned_recall_queries;
SELECT count(*) AS corpus_rows FROM ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus;
SELECT count(*) AS query_rows FROM ec_hnsw_parallel_concurrent_dsm_tuned_recall_queries;

SET maintenance_work_mem = '1GB';
SET max_parallel_workers = 8;
SET ec_hnsw.ef_search = 200;

\echo === serial-built tuned index ===
SET ec_hnsw.enable_parallel_build_concurrent_dsm = off;
SET max_parallel_maintenance_workers = 0;
ALTER TABLE ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus SET (parallel_workers = 0);
CREATE INDEX ec_hnsw_parallel_concurrent_dsm_tuned_recall_serial_idx
    ON ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128);
SELECT 'serial' AS build_path, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT 'serial' AS build_path, *
FROM tests.ec_hnsw_graph_scan_recall_ef_sweep(
    'ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus',
    'ec_hnsw_parallel_concurrent_dsm_tuned_recall_queries',
    'ec_hnsw_parallel_concurrent_dsm_tuned_recall_serial_idx',
    16,
    ARRAY[128, 200, 400]
);

\echo === parallel concurrent DSM-built tuned index ===
SET ec_hnsw.enable_parallel_build_concurrent_dsm = on;
SET max_parallel_maintenance_workers = 4;
ALTER TABLE ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus SET (parallel_workers = 4);
CREATE INDEX ec_hnsw_parallel_concurrent_dsm_tuned_recall_dsm_idx
    ON ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 16, ef_construction = 128);
SELECT 'concurrent_dsm' AS build_path, *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS concurrent_dsm_graph_workers_launched;
SELECT 'concurrent_dsm' AS build_path, *
FROM tests.ec_hnsw_graph_scan_recall_ef_sweep(
    'ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus',
    'ec_hnsw_parallel_concurrent_dsm_tuned_recall_queries',
    'ec_hnsw_parallel_concurrent_dsm_tuned_recall_dsm_idx',
    16,
    ARRAY[128, 200, 400]
);

\echo === tuned serial vs concurrent DSM recall delta at ef_search 200 ===
WITH serial_summary AS (
    SELECT *
    FROM tests.ec_hnsw_graph_scan_recall_external_summary(
        'ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus',
        'ec_hnsw_parallel_concurrent_dsm_tuned_recall_queries',
        'ec_hnsw_parallel_concurrent_dsm_tuned_recall_serial_idx',
        16,
        200
    )
),
dsm_summary AS (
    SELECT *
    FROM tests.ec_hnsw_graph_scan_recall_external_summary(
        'ec_hnsw_parallel_concurrent_dsm_tuned_recall_corpus',
        'ec_hnsw_parallel_concurrent_dsm_tuned_recall_queries',
        'ec_hnsw_parallel_concurrent_dsm_tuned_recall_dsm_idx',
        16,
        200
    )
)
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
FROM serial_summary
CROSS JOIN dsm_summary;

SELECT pg_relation_size('ec_hnsw_parallel_concurrent_dsm_tuned_recall_serial_idx') AS serial_index_bytes,
       pg_relation_size('ec_hnsw_parallel_concurrent_dsm_tuned_recall_dsm_idx') AS concurrent_dsm_index_bytes;

SELECT now() AS finished_at;
