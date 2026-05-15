-- Phase 13b.9.c — DELETE driver via the DML CustomScan delete executor.
-- Variables:
--   :prefix  corpus prefix
--   :per_tx  rows deleted per transaction
--   :rows    total rows to delete

\set ON_ERROR_STOP on

\echo === Pre-delete placement snapshot ===
SELECT * FROM ec_spire_index_active_snapshot_diagnostics(
  format('%s_idx', :'prefix')::regclass
);

SELECT clock_timestamp() AS delete_started_at \gset

WITH victims AS (
  SELECT vec_id FROM ec_spire_aws_repr_1m_corpus
  ORDER BY vec_id DESC LIMIT (:'rows')::int
)
DELETE FROM ec_spire_aws_repr_1m_corpus c
USING victims v
WHERE c.vec_id = v.vec_id;

SELECT clock_timestamp() - :'delete_started_at'::timestamptz AS elapsed;

\echo === Post-delete placement snapshot ===
SELECT * FROM ec_spire_index_active_snapshot_diagnostics(
  format('%s_idx', :'prefix')::regclass
);
