\timing on

INSERT INTO task89_drift_live_tqplus_corpus
SELECT *
FROM task89_drift_source50k_tq_corpus
ORDER BY id
OFFSET 12500
LIMIT 2500;

ANALYZE task89_drift_live_tqplus_corpus;

SELECT
  'live_after_50pct_insert' AS phase,
  count(*) AS rows,
  min(id) AS min_id,
  max(id) AS max_id
FROM task89_drift_live_tqplus_corpus;
