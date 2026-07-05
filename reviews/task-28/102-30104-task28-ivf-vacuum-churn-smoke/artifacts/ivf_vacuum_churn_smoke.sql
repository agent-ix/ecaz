\timing on

SELECT version();
SELECT 'setup' AS section;
CREATE EXTENSION IF NOT EXISTS ecaz;

DROP TABLE IF EXISTS task28_ivf_vacuum_churn_8 CASCADE;
DROP TABLE IF EXISTS task28_ivf_vacuum_churn_32 CASCADE;
DROP TABLE IF EXISTS task28_ivf_vacuum_churn_64 CASCADE;

CREATE TABLE task28_ivf_vacuum_churn_8 (
    id bigint PRIMARY KEY,
    embedding ecvector NOT NULL
);
CREATE TABLE task28_ivf_vacuum_churn_32 (
    id bigint PRIMARY KEY,
    embedding ecvector NOT NULL
);
CREATE TABLE task28_ivf_vacuum_churn_64 (
    id bigint PRIMARY KEY,
    embedding ecvector NOT NULL
);

INSERT INTO task28_ivf_vacuum_churn_8
SELECT
    gs,
    encode_to_ecvector(
        ARRAY[
            sin((gs * 0.013)::double precision)::real,
            cos((gs * 0.013)::double precision)::real,
            sin((gs * 0.021)::double precision)::real,
            cos((gs * 0.021)::double precision)::real
        ]::real[],
        4,
        42
    )
FROM generate_series(1, 5000) AS gs;
INSERT INTO task28_ivf_vacuum_churn_32 SELECT * FROM task28_ivf_vacuum_churn_8;
INSERT INTO task28_ivf_vacuum_churn_64 SELECT * FROM task28_ivf_vacuum_churn_8;

CREATE INDEX task28_ivf_vacuum_churn_8_idx
ON task28_ivf_vacuum_churn_8 USING ec_ivf (embedding ecvector_ip_ops)
WITH (nlists = 8, nprobe = 8, training_sample_rows = 1000, rerank = 'heap_f32');

CREATE INDEX task28_ivf_vacuum_churn_32_idx
ON task28_ivf_vacuum_churn_32 USING ec_ivf (embedding ecvector_ip_ops)
WITH (nlists = 32, nprobe = 32, training_sample_rows = 1000, rerank = 'heap_f32');

CREATE INDEX task28_ivf_vacuum_churn_64_idx
ON task28_ivf_vacuum_churn_64 USING ec_ivf (embedding ecvector_ip_ops)
WITH (nlists = 64, nprobe = 64, training_sample_rows = 1000, rerank = 'heap_f32');

ANALYZE task28_ivf_vacuum_churn_8;
ANALYZE task28_ivf_vacuum_churn_32;
ANALYZE task28_ivf_vacuum_churn_64;

SELECT 'after build' AS phase, relname, pg_relation_size(oid) AS bytes, pg_size_pretty(pg_relation_size(oid)) AS pretty
FROM pg_class
WHERE relname IN (
    'task28_ivf_vacuum_churn_8_idx',
    'task28_ivf_vacuum_churn_32_idx',
    'task28_ivf_vacuum_churn_64_idx'
)
ORDER BY relname;

DELETE FROM task28_ivf_vacuum_churn_8 WHERE id BETWEEN 2501 AND 5000;
DELETE FROM task28_ivf_vacuum_churn_32 WHERE id BETWEEN 2501 AND 5000;
DELETE FROM task28_ivf_vacuum_churn_64 WHERE id BETWEEN 2501 AND 5000;

VACUUM (ANALYZE) task28_ivf_vacuum_churn_8;
VACUUM (ANALYZE) task28_ivf_vacuum_churn_32;
VACUUM (ANALYZE) task28_ivf_vacuum_churn_64;

SELECT 'after delete vacuum' AS phase, relname, pg_relation_size(oid) AS bytes, pg_size_pretty(pg_relation_size(oid)) AS pretty
FROM pg_class
WHERE relname IN (
    'task28_ivf_vacuum_churn_8_idx',
    'task28_ivf_vacuum_churn_32_idx',
    'task28_ivf_vacuum_churn_64_idx'
)
ORDER BY relname;

INSERT INTO task28_ivf_vacuum_churn_8
SELECT
    gs,
    encode_to_ecvector(
        ARRAY[
            sin((gs * 0.013)::double precision)::real,
            cos((gs * 0.013)::double precision)::real,
            sin((gs * 0.021)::double precision)::real,
            cos((gs * 0.021)::double precision)::real
        ]::real[],
        4,
        42
    )
FROM generate_series(5001, 7500) AS gs;
INSERT INTO task28_ivf_vacuum_churn_32 SELECT * FROM task28_ivf_vacuum_churn_8 WHERE id BETWEEN 5001 AND 7500;
INSERT INTO task28_ivf_vacuum_churn_64 SELECT * FROM task28_ivf_vacuum_churn_8 WHERE id BETWEEN 5001 AND 7500;

ANALYZE task28_ivf_vacuum_churn_8;
ANALYZE task28_ivf_vacuum_churn_32;
ANALYZE task28_ivf_vacuum_churn_64;

SELECT 'after refill' AS phase, relname, pg_relation_size(oid) AS bytes, pg_size_pretty(pg_relation_size(oid)) AS pretty
FROM pg_class
WHERE relname IN (
    'task28_ivf_vacuum_churn_8_idx',
    'task28_ivf_vacuum_churn_32_idx',
    'task28_ivf_vacuum_churn_64_idx'
)
ORDER BY relname;

SELECT 'final row counts' AS section, surface, live_rows
FROM (
    SELECT 'n8' AS surface, count(*) AS live_rows FROM task28_ivf_vacuum_churn_8
    UNION ALL
    SELECT 'n32' AS surface, count(*) AS live_rows FROM task28_ivf_vacuum_churn_32
    UNION ALL
    SELECT 'n64' AS surface, count(*) AS live_rows FROM task28_ivf_vacuum_churn_64
) s
ORDER BY surface;
