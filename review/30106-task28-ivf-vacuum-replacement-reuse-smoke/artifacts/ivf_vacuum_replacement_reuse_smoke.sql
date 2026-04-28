\timing on

SELECT version();
SELECT 'setup' AS section;
CREATE EXTENSION IF NOT EXISTS ecaz;

DROP TABLE IF EXISTS task28_ivf_vacuum_replace_8 CASCADE;
DROP TABLE IF EXISTS task28_ivf_vacuum_replace_32 CASCADE;
DROP TABLE IF EXISTS task28_ivf_vacuum_replace_64 CASCADE;

CREATE TABLE task28_ivf_vacuum_replace_8 (
    id bigint PRIMARY KEY,
    embedding ecvector NOT NULL
);
CREATE TABLE task28_ivf_vacuum_replace_32 (
    id bigint PRIMARY KEY,
    embedding ecvector NOT NULL
);
CREATE TABLE task28_ivf_vacuum_replace_64 (
    id bigint PRIMARY KEY,
    embedding ecvector NOT NULL
);

INSERT INTO task28_ivf_vacuum_replace_8
SELECT
    gs,
    encode_to_ecvector(
        ARRAY[
            sin((base_id * 0.013)::double precision)::real,
            cos((base_id * 0.013)::double precision)::real,
            sin((base_id * 0.021)::double precision)::real,
            cos((base_id * 0.021)::double precision)::real
        ]::real[],
        4,
        42
    )
FROM (
    SELECT gs, ((gs - 1) % 2500) + 1 AS base_id
    FROM generate_series(1, 5000) AS gs
) s;
INSERT INTO task28_ivf_vacuum_replace_32 SELECT * FROM task28_ivf_vacuum_replace_8;
INSERT INTO task28_ivf_vacuum_replace_64 SELECT * FROM task28_ivf_vacuum_replace_8;

CREATE INDEX task28_ivf_vacuum_replace_8_idx
ON task28_ivf_vacuum_replace_8 USING ec_ivf (embedding ecvector_ip_ops)
WITH (nlists = 8, nprobe = 8, training_sample_rows = 1000, rerank = 'heap_f32');

CREATE INDEX task28_ivf_vacuum_replace_32_idx
ON task28_ivf_vacuum_replace_32 USING ec_ivf (embedding ecvector_ip_ops)
WITH (nlists = 32, nprobe = 32, training_sample_rows = 1000, rerank = 'heap_f32');

CREATE INDEX task28_ivf_vacuum_replace_64_idx
ON task28_ivf_vacuum_replace_64 USING ec_ivf (embedding ecvector_ip_ops)
WITH (nlists = 64, nprobe = 64, training_sample_rows = 1000, rerank = 'heap_f32');

ANALYZE task28_ivf_vacuum_replace_8;
ANALYZE task28_ivf_vacuum_replace_32;
ANALYZE task28_ivf_vacuum_replace_64;

SELECT 'after build' AS phase, relname, pg_relation_size(oid) AS bytes, pg_size_pretty(pg_relation_size(oid)) AS pretty
FROM pg_class
WHERE relname IN (
    'task28_ivf_vacuum_replace_8_idx',
    'task28_ivf_vacuum_replace_32_idx',
    'task28_ivf_vacuum_replace_64_idx'
)
ORDER BY relname;

DELETE FROM task28_ivf_vacuum_replace_8 WHERE id BETWEEN 2501 AND 5000;
DELETE FROM task28_ivf_vacuum_replace_32 WHERE id BETWEEN 2501 AND 5000;
DELETE FROM task28_ivf_vacuum_replace_64 WHERE id BETWEEN 2501 AND 5000;

VACUUM (ANALYZE) task28_ivf_vacuum_replace_8;
VACUUM (ANALYZE) task28_ivf_vacuum_replace_32;
VACUUM (ANALYZE) task28_ivf_vacuum_replace_64;

SELECT 'after delete vacuum' AS phase, relname, pg_relation_size(oid) AS bytes, pg_size_pretty(pg_relation_size(oid)) AS pretty
FROM pg_class
WHERE relname IN (
    'task28_ivf_vacuum_replace_8_idx',
    'task28_ivf_vacuum_replace_32_idx',
    'task28_ivf_vacuum_replace_64_idx'
)
ORDER BY relname;

INSERT INTO task28_ivf_vacuum_replace_8
SELECT
    gs,
    encode_to_ecvector(
        ARRAY[
            sin((base_id * 0.013)::double precision)::real,
            cos((base_id * 0.013)::double precision)::real,
            sin((base_id * 0.021)::double precision)::real,
            cos((base_id * 0.021)::double precision)::real
        ]::real[],
        4,
        42
    )
FROM (
    SELECT gs, ((gs - 1) % 2500) + 1 AS base_id
    FROM generate_series(5001, 7500) AS gs
) s;
INSERT INTO task28_ivf_vacuum_replace_32 SELECT * FROM task28_ivf_vacuum_replace_8 WHERE id BETWEEN 5001 AND 7500;
INSERT INTO task28_ivf_vacuum_replace_64 SELECT * FROM task28_ivf_vacuum_replace_8 WHERE id BETWEEN 5001 AND 7500;

ANALYZE task28_ivf_vacuum_replace_8;
ANALYZE task28_ivf_vacuum_replace_32;
ANALYZE task28_ivf_vacuum_replace_64;

SELECT 'after refill' AS phase, relname, pg_relation_size(oid) AS bytes, pg_size_pretty(pg_relation_size(oid)) AS pretty
FROM pg_class
WHERE relname IN (
    'task28_ivf_vacuum_replace_8_idx',
    'task28_ivf_vacuum_replace_32_idx',
    'task28_ivf_vacuum_replace_64_idx'
)
ORDER BY relname;

SELECT 'final row counts' AS section, surface, live_rows
FROM (
    SELECT 'n8' AS surface, count(*) AS live_rows FROM task28_ivf_vacuum_replace_8
    UNION ALL
    SELECT 'n32' AS surface, count(*) AS live_rows FROM task28_ivf_vacuum_replace_32
    UNION ALL
    SELECT 'n64' AS surface, count(*) AS live_rows FROM task28_ivf_vacuum_replace_64
) s
ORDER BY surface;
