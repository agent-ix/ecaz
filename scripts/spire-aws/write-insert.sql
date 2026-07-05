-- Phase 13b.9.c — coordinator-routed INSERT driver.
-- Variables:
--   :prefix  corpus prefix (table is <prefix>_corpus)
--   :batch   rows per INSERT
--   :rows    total rows to insert

\set ON_ERROR_STOP on

\echo === Pre-insert placement snapshot ===
SELECT * FROM ec_spire_index_active_snapshot_diagnostics(
  format('%s_idx', :'prefix')::regclass
);

\set start_id 0
SELECT clock_timestamp() AS insert_started_at \gset

INSERT INTO ec_spire_aws_repr_1m_corpus (vec_id, embedding)
SELECT g + (SELECT coalesce(max(vec_id), 0) + 1 FROM ec_spire_aws_repr_1m_corpus),
       ARRAY(SELECT random()::real FROM generate_series(1, 1536))::ecvector
FROM generate_series(1, (:'rows')::int) g;

SELECT clock_timestamp() AS insert_ended_at,
       clock_timestamp() - :'insert_started_at'::timestamptz AS elapsed;

\echo === Post-insert placement snapshot ===
SELECT * FROM ec_spire_index_active_snapshot_diagnostics(
  format('%s_idx', :'prefix')::regclass
);
