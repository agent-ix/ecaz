-- M3 exit gate: 100k multinode distinct_recall >= single-node - 0.001.
-- Stronger form: for 50 queries, the 2-node loopback top-10 is compared id-for-id
-- against the single-node top-10. Byte-identical top-k => recall delta is exactly
-- 0 (>= -0.001 trivially). Runs on the committed ec_distann_bench / m1_50k_mono.
\set ON_ERROR_STOP on
\set roster '0@host=/home/peter/.pgrx port=28818 dbname=ec_distann_bench user=peter;1@host=/home/peter/.pgrx port=28818 dbname=ec_distann_bench user=peter'
SET enable_seqscan = off;

DROP TABLE IF EXISTS queries;
CREATE TEMP TABLE queries AS SELECT id AS qid, source AS v FROM m1_100k_mono_corpus WHERE id <= 50;

-- Single-node baseline (empty roster).
SET ec_distann.roster = ''; SET ec_distann.local_node_id = 0; SET ec_distann.epoch = 0;
DROP TABLE IF EXISTS base;
CREATE TEMP TABLE base AS
  SELECT q.qid, r.id
  FROM queries q CROSS JOIN LATERAL
       (SELECT id FROM m1_100k_mono_corpus ORDER BY embedding <#> q.v LIMIT 10) r;

-- Two-node loopback scan (RemoteNodeExpander + remote-hit materialization).
SET ec_distann.roster = :'roster'; SET ec_distann.local_node_id = 0;
DROP TABLE IF EXISTS two;
CREATE TEMP TABLE two AS
  SELECT q.qid, r.id
  FROM queries q CROSS JOIN LATERAL
       (SELECT id FROM m1_100k_mono_corpus ORDER BY embedding <#> q.v LIMIT 10) r;

SET ec_distann.roster = '';
SELECT
  count(*) AS n_queries,
  count(*) FILTER (WHERE mismatch = 0) AS identical_queries,
  sum(mismatch) AS total_mismatched_ids
FROM (
  SELECT q.qid,
    (SELECT count(*) FROM (SELECT id FROM base WHERE qid = q.qid
                           EXCEPT SELECT id FROM two WHERE qid = q.qid) d)
  + (SELECT count(*) FROM (SELECT id FROM two WHERE qid = q.qid
                           EXCEPT SELECT id FROM base WHERE qid = q.qid) d) AS mismatch
  FROM queries q
) s;
