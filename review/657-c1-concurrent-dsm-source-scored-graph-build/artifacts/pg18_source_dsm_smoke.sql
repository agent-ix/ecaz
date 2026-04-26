\set ON_ERROR_STOP on
\timing on

DROP TABLE IF EXISTS ec_hnsw_source_dsm_smoke;
DROP EXTENSION IF EXISTS ecaz CASCADE;
CREATE EXTENSION ecaz;

CREATE TABLE ec_hnsw_source_dsm_smoke (
    id bigint PRIMARY KEY,
    source real[] NOT NULL,
    embedding ecvector NOT NULL
);

INSERT INTO ec_hnsw_source_dsm_smoke
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
               FROM generate_series(1, 16) AS dim
           ) AS source
    FROM generate_series(1, 2000) AS id
) AS fixture;

VACUUM ANALYZE ec_hnsw_source_dsm_smoke;
SET maintenance_work_mem = '512MB';
SET max_parallel_workers = 8;
SET ec_hnsw.enable_parallel_build_concurrent_dsm = on;
SET max_parallel_maintenance_workers = 2;
ALTER TABLE ec_hnsw_source_dsm_smoke SET (parallel_workers = 2);

CREATE INDEX ec_hnsw_source_dsm_smoke_idx
    ON ec_hnsw_source_dsm_smoke
    USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 8, ef_construction = 64, build_source_column = source);

SELECT *
FROM tests.ec_hnsw_debug_last_build_timing();
SELECT tests.ec_hnsw_debug_parallel_graph_build_workers_launched()
    AS concurrent_dsm_graph_workers_launched;
