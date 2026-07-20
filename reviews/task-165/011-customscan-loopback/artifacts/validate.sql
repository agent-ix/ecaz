\set ON_ERROR_STOP on
\set roster '0@host=/home/peter/.pgrx port=28818 dbname=ec_distann_cs user=peter;1@host=/home/peter/.pgrx port=28818 dbname=ec_distann_cs user=peter'
SET enable_seqscan = off;
DROP TABLE IF EXISTS q; CREATE TEMP TABLE q AS SELECT id AS qid, source AS v FROM cs WHERE id <= 20;

-- Confirm the planner replaces the index scan with the multi-node CustomScan.
SET ec_distann.roster = :'roster'; SET ec_distann.local_node_id = 0; SET ec_distann.epoch = 1;
EXPLAIN (COSTS OFF)
  SELECT q.qid, r.id FROM q CROSS JOIN LATERAL
    (SELECT id FROM cs ORDER BY embedding <#> q.v LIMIT 10) r;

-- Single-node baseline (empty roster -> local amgettuple path).
SET ec_distann.roster = ''; SET ec_distann.local_node_id = 0; SET ec_distann.epoch = 0;
DROP TABLE IF EXISTS base; CREATE TEMP TABLE base AS
  SELECT q.qid, r.id FROM q CROSS JOIN LATERAL
    (SELECT id FROM cs ORDER BY embedding <#> q.v LIMIT 10) r;

-- Multi-node loopback (CustomScan -> owner-shipped row payloads).
SET ec_distann.roster = :'roster'; SET ec_distann.local_node_id = 0; SET ec_distann.epoch = 1;
DROP TABLE IF EXISTS two; CREATE TEMP TABLE two AS
  SELECT q.qid, r.id FROM q CROSS JOIN LATERAL
    (SELECT id FROM cs ORDER BY embedding <#> q.v LIMIT 10) r;

SET ec_distann.roster = '';
SELECT count(DISTINCT qid) AS n_queries,
  count(DISTINCT qid) FILTER (WHERE mismatch = 0) AS identical_queries,
  coalesce(sum(mismatch),0) AS total_mismatched_ids
FROM (
  SELECT q.qid,
    (SELECT count(*) FROM (SELECT id FROM base WHERE qid=q.qid EXCEPT SELECT id FROM two WHERE qid=q.qid) d)
  + (SELECT count(*) FROM (SELECT id FROM two WHERE qid=q.qid EXCEPT SELECT id FROM base WHERE qid=q.qid) d) AS mismatch
  FROM q
) s;
