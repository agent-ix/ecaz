-- M5 quality: folding delta inserts into the graph preserves recall. A/B on the
-- same 2000 real rows: index A is a full build; index B builds on 1800 then
-- inserts + folds the remaining 200. For 50 query vectors, compare each index's
-- top-10 to brute-force exact truth over the 2000 rows and report recall@10.
-- Fold-build recall should match full-build recall (folded nodes are properly
-- connected). Runs on ec_distann_m2 (real 10k corpus). Release build.
\set ON_ERROR_STOP on
SET enable_seqscan = off;
SET ec_distann.roster = '';

-- 2000-row working set + 50 real query vectors (real[]).
DROP TABLE IF EXISTS ab_corpus;
CREATE TABLE ab_corpus AS SELECT id, embedding FROM m2_10k_corpus WHERE id <= 2000;
DROP TABLE IF EXISTS ab_queries;
CREATE TABLE ab_queries AS SELECT id AS qid, source AS v FROM m2_10k_queries ORDER BY id LIMIT 50;

-- Index A: full build on all 2000.
CREATE INDEX ab_full_idx ON ab_corpus USING ec_distann (embedding ecvector_distann_ip_ops)
  WITH (graph_degree = 32);

-- Index B: build on 1800, then insert + fold the remaining 200.
DROP TABLE IF EXISTS ab_fold;
CREATE TABLE ab_fold AS SELECT id, embedding FROM ab_corpus WHERE id <= 1800;
CREATE INDEX ab_fold_idx ON ab_fold USING ec_distann (embedding ecvector_distann_ip_ops)
  WITH (graph_degree = 32);
INSERT INTO ab_fold SELECT id, embedding FROM ab_corpus WHERE id BETWEEN 1801 AND 2000;
SELECT ec_distann_fold_delta_into_graph('ab_fold_idx'::regclass::oid) AS folded;

-- Brute-force exact top-10 truth over the 2000 rows (seqscan, no index).
SET enable_seqscan = on; SET enable_indexscan = off;
DROP TABLE IF EXISTS truth;
CREATE TABLE truth AS
  SELECT q.qid, r.id, row_number() OVER (PARTITION BY q.qid ORDER BY c.embedding <#> q.v) AS rk
  FROM ab_queries q CROSS JOIN LATERAL
       (SELECT id, embedding FROM ab_corpus ORDER BY embedding <#> q.v LIMIT 10) r
  JOIN ab_corpus c ON c.id = r.id;
SET enable_indexscan = on; SET enable_seqscan = off;

-- recall@10 of each index vs truth.
SELECT 'A_full' AS index, round(avg(hit)::numeric, 4) AS recall_at_10 FROM (
  SELECT q.qid,
    (SELECT count(*) FROM (
       SELECT id FROM (SELECT id FROM ab_corpus ORDER BY embedding <#> q.v LIMIT 10) a
       INTERSECT SELECT id FROM truth WHERE truth.qid = q.qid) x)::float / 10 AS hit
  FROM ab_queries q) s
UNION ALL
SELECT 'B_fold', round(avg(hit)::numeric, 4) FROM (
  SELECT q.qid,
    (SELECT count(*) FROM (
       SELECT id FROM (SELECT id FROM ab_fold ORDER BY embedding <#> q.v LIMIT 10) b
       INTERSECT SELECT id FROM truth WHERE truth.qid = q.qid) x)::float / 10 AS hit
  FROM ab_queries q) s;
