-- M3: multi-node SQL scan (amgettuple via RemoteNodeExpander + remote-hit
-- materialization) returns the same top-k as the single-node scan. Runs against
-- the committed ec_distann_m2 DB (loopback 2-node = same instance).
\set ON_ERROR_STOP on
\set roster '0@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter;1@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter'
SET enable_seqscan = off;

DROP TABLE IF EXISTS qv;
CREATE TEMP TABLE qv AS SELECT source AS v FROM m2_10k_queries ORDER BY id LIMIT 1;

-- Single-node baseline (empty roster).
SET ec_distann.roster = ''; SET ec_distann.local_node_id = 0; SET ec_distann.epoch = 0;
DROP TABLE IF EXISTS s_base;
CREATE TEMP TABLE s_base AS
  SELECT id, row_number() OVER () AS rn
  FROM (SELECT id FROM m2_10k_corpus ORDER BY embedding <#> (SELECT v FROM qv) LIMIT 10) q;

-- Two-node loopback scan (drives RemoteNodeExpander + remote-hit materialization).
SET ec_distann.roster = :'roster'; SET ec_distann.local_node_id = 0;
DROP TABLE IF EXISTS s_two;
CREATE TEMP TABLE s_two AS
  SELECT id, row_number() OVER () AS rn
  FROM (SELECT id FROM m2_10k_corpus ORDER BY embedding <#> (SELECT v FROM qv) LIMIT 10) q;

SELECT
  (SELECT count(*) FROM s_base) AS base_rows,
  (SELECT count(*) FROM s_two)  AS two_rows,
  (SELECT count(*) FROM (SELECT id FROM s_base EXCEPT SELECT id FROM s_two) d) AS base_minus_two,
  (SELECT count(*) FROM (SELECT id FROM s_two EXCEPT SELECT id FROM s_base) d) AS two_minus_base,
  (SELECT bool_and(b.id = t.id) FROM s_base b JOIN s_two t USING (rn)) AS order_identical;
