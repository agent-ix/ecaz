\timing on

DROP TABLE IF EXISTS task89_drift_rebuild12500_tqplus_corpus CASCADE;
DROP TABLE IF EXISTS task89_drift_rebuild12500_tqplus_queries CASCADE;

CREATE TABLE task89_drift_rebuild12500_tqplus_corpus AS
SELECT *
FROM task89_drift_source50k_tq_corpus
ORDER BY id
LIMIT 12500;

ALTER TABLE task89_drift_rebuild12500_tqplus_corpus
ADD PRIMARY KEY (id);

CREATE TABLE task89_drift_rebuild12500_tqplus_queries AS
SELECT *
FROM task89_drift_source50k_tq_queries
ORDER BY id
LIMIT 200;

CREATE INDEX task89_drift_rebuild12500_tqplus_turboquant_idx
ON task89_drift_rebuild12500_tqplus_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
  nlists = 64,
  nprobe = 48,
  storage_format = 'turboquant',
  turboquant_calibration = 'tqplus_experimental'
);

ANALYZE task89_drift_rebuild12500_tqplus_corpus;
ANALYZE task89_drift_rebuild12500_tqplus_queries;

SELECT
  'rebuild12500_ready' AS phase,
  count(*) AS rows,
  min(id) AS min_id,
  max(id) AS max_id
FROM task89_drift_rebuild12500_tqplus_corpus;
