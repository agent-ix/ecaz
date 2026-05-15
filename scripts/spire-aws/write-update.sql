-- Phase 13b.9.c — non-embedding UPDATE driver.
-- Variables:
--   :prefix  corpus prefix
--   :per_tx  rows updated per transaction
--   :rows    total rows to touch

\set ON_ERROR_STOP on

\echo === Pre-update placement snapshot ===
SELECT * FROM ec_spire_index_active_snapshot_diagnostics(
  format('%s_idx', :'prefix')::regclass
);

SELECT clock_timestamp() AS update_started_at \gset

WITH targets AS (
  SELECT vec_id FROM ec_spire_aws_repr_1m_corpus ORDER BY vec_id LIMIT (:'rows')::int
)
UPDATE ec_spire_aws_repr_1m_corpus c
SET label = format('updated-%s', now())
FROM targets t
WHERE c.vec_id = t.vec_id;

SELECT clock_timestamp() - :'update_started_at'::timestamptz AS elapsed;

\echo === Post-update placement snapshot (should be unchanged for non-embedding UPDATE) ===
SELECT * FROM ec_spire_index_active_snapshot_diagnostics(
  format('%s_idx', :'prefix')::regclass
);
