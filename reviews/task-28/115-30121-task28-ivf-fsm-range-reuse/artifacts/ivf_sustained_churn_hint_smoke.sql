\timing on

CREATE EXTENSION IF NOT EXISTS ecaz;

DROP TABLE IF EXISTS task28_ivf_churn_n32 CASCADE;
DROP TABLE IF EXISTS task28_ivf_churn_n64 CASCADE;

CREATE TABLE task28_ivf_churn_n32 (
  id bigint PRIMARY KEY,
  embedding ecvector NOT NULL
);

CREATE TABLE task28_ivf_churn_n64 (
  id bigint PRIMARY KEY,
  embedding ecvector NOT NULL
);

INSERT INTO task28_ivf_churn_n32 (id, embedding)
SELECT gs, encode_to_ecvector(
  ARRAY[
    sin(((gs % 50000)::double precision * 0.013)::double precision)::real,
    cos(((gs % 50000)::double precision * 0.013)::double precision)::real,
    sin(((gs % 50000)::double precision * 0.021)::double precision)::real,
    cos(((gs % 50000)::double precision * 0.021)::double precision)::real
  ]::real[],
  4,
  42
)
FROM generate_series(1, 50000) AS gs;

INSERT INTO task28_ivf_churn_n64
SELECT * FROM task28_ivf_churn_n32;

CREATE INDEX task28_ivf_churn_n32_idx
ON task28_ivf_churn_n32 USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 32,
  nprobe = 8,
  training_sample_rows = 10000,
  quantizer = 'turboquant',
  rerank = 'heap_f32'
);

CREATE INDEX task28_ivf_churn_n64_idx
ON task28_ivf_churn_n64 USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 64,
  nprobe = 8,
  training_sample_rows = 10000,
  quantizer = 'turboquant',
  rerank = 'heap_f32'
);

SELECT 'cycle0_build' AS phase, c.relname, pg_relation_size(c.oid) AS index_bytes, pg_size_pretty(pg_relation_size(c.oid)) AS index_size
FROM pg_class c
WHERE c.relname IN ('task28_ivf_churn_n32_idx', 'task28_ivf_churn_n64_idx')
ORDER BY c.relname;

DELETE FROM task28_ivf_churn_n32 WHERE id <= 25000;
DELETE FROM task28_ivf_churn_n64 WHERE id <= 25000;
VACUUM (ANALYZE) task28_ivf_churn_n32;
VACUUM (ANALYZE) task28_ivf_churn_n64;
INSERT INTO task28_ivf_churn_n32 (id, embedding)
SELECT 50000 + gs, encode_to_ecvector(
  ARRAY[
    sin(((gs % 50000)::double precision * 0.013)::double precision)::real,
    cos(((gs % 50000)::double precision * 0.013)::double precision)::real,
    sin(((gs % 50000)::double precision * 0.021)::double precision)::real,
    cos(((gs % 50000)::double precision * 0.021)::double precision)::real
  ]::real[],
  4,
  42
)
FROM generate_series(1, 25000) AS gs;
INSERT INTO task28_ivf_churn_n64 SELECT * FROM task28_ivf_churn_n32 WHERE id > 50000 AND id <= 75000;
SELECT 'cycle1_refill' AS phase, c.relname, pg_relation_size(c.oid) AS index_bytes, pg_size_pretty(pg_relation_size(c.oid)) AS index_size
FROM pg_class c
WHERE c.relname IN ('task28_ivf_churn_n32_idx', 'task28_ivf_churn_n64_idx')
ORDER BY c.relname;

DELETE FROM task28_ivf_churn_n32 WHERE id > 25000 AND id <= 50000;
DELETE FROM task28_ivf_churn_n64 WHERE id > 25000 AND id <= 50000;
VACUUM (ANALYZE) task28_ivf_churn_n32;
VACUUM (ANALYZE) task28_ivf_churn_n64;
INSERT INTO task28_ivf_churn_n32 (id, embedding)
SELECT 75000 + gs, encode_to_ecvector(
  ARRAY[
    sin(((gs % 50000)::double precision * 0.013)::double precision)::real,
    cos(((gs % 50000)::double precision * 0.013)::double precision)::real,
    sin(((gs % 50000)::double precision * 0.021)::double precision)::real,
    cos(((gs % 50000)::double precision * 0.021)::double precision)::real
  ]::real[],
  4,
  42
)
FROM generate_series(1, 25000) AS gs;
INSERT INTO task28_ivf_churn_n64 SELECT * FROM task28_ivf_churn_n32 WHERE id > 75000 AND id <= 100000;
SELECT 'cycle2_refill' AS phase, c.relname, pg_relation_size(c.oid) AS index_bytes, pg_size_pretty(pg_relation_size(c.oid)) AS index_size
FROM pg_class c
WHERE c.relname IN ('task28_ivf_churn_n32_idx', 'task28_ivf_churn_n64_idx')
ORDER BY c.relname;

DELETE FROM task28_ivf_churn_n32 WHERE id > 50000 AND id <= 75000;
DELETE FROM task28_ivf_churn_n64 WHERE id > 50000 AND id <= 75000;
VACUUM (ANALYZE) task28_ivf_churn_n32;
VACUUM (ANALYZE) task28_ivf_churn_n64;
INSERT INTO task28_ivf_churn_n32 (id, embedding)
SELECT 100000 + gs, encode_to_ecvector(
  ARRAY[
    sin(((gs % 50000)::double precision * 0.013)::double precision)::real,
    cos(((gs % 50000)::double precision * 0.013)::double precision)::real,
    sin(((gs % 50000)::double precision * 0.021)::double precision)::real,
    cos(((gs % 50000)::double precision * 0.021)::double precision)::real
  ]::real[],
  4,
  42
)
FROM generate_series(1, 25000) AS gs;
INSERT INTO task28_ivf_churn_n64 SELECT * FROM task28_ivf_churn_n32 WHERE id > 100000 AND id <= 125000;
SELECT 'cycle3_refill' AS phase, c.relname, pg_relation_size(c.oid) AS index_bytes, pg_size_pretty(pg_relation_size(c.oid)) AS index_size
FROM pg_class c
WHERE c.relname IN ('task28_ivf_churn_n32_idx', 'task28_ivf_churn_n64_idx')
ORDER BY c.relname;
