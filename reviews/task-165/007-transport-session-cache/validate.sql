-- 006-P3 validation: after moving remote session setup to per-connection
-- (cached by roster/node/epoch), the 2-node loopback scan still returns the
-- same top-k as the single-node scan. Fresh v3 index (the delta_count format
-- bump invalidated older indexes). Runs on ec_distann_m2 (real 10k corpus).
\set ON_ERROR_STOP on
\set roster '0@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter;1@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter'
SET enable_seqscan = off;

DROP TABLE IF EXISTS p3_corpus;
CREATE TABLE p3_corpus AS SELECT id, embedding FROM m2_10k_corpus WHERE id <= 3000;
CREATE INDEX p3_idx ON p3_corpus USING ec_distann (embedding ecvector_distann_ip_ops)
  WITH (graph_degree = 32);

DROP TABLE IF EXISTS qv;
CREATE TEMP TABLE qv AS SELECT source AS v FROM m2_10k_queries ORDER BY id LIMIT 1;

SET ec_distann.roster = ''; SET ec_distann.local_node_id = 0; SET ec_distann.epoch = 0;
DROP TABLE IF EXISTS s_base;
CREATE TEMP TABLE s_base AS
  SELECT id, row_number() OVER () AS rn
  FROM (SELECT id FROM p3_corpus ORDER BY embedding <#> (SELECT v FROM qv) LIMIT 10) q;

SET ec_distann.roster = :'roster'; SET ec_distann.local_node_id = 0;
DROP TABLE IF EXISTS s_two;
CREATE TEMP TABLE s_two AS
  SELECT id, row_number() OVER () AS rn
  FROM (SELECT id FROM p3_corpus ORDER BY embedding <#> (SELECT v FROM qv) LIMIT 10) q;

SET ec_distann.roster = '';
SELECT
  (SELECT count(*) FROM (SELECT id FROM s_base EXCEPT SELECT id FROM s_two) d) AS base_minus_two,
  (SELECT count(*) FROM (SELECT id FROM s_two EXCEPT SELECT id FROM s_base) d) AS two_minus_base,
  (SELECT bool_and(b.id = t.id) FROM s_base b JOIN s_two t USING (rn)) AS order_identical;
