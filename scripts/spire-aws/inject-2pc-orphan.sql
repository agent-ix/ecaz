-- Phase 13b.10.c — 2PC orphan injection helper.
--
-- Strategy: open a transaction, INSERT one row through the coordinator
-- DML path (which prepares on the remote), then PREPARE TRANSACTION
-- locally and *abandon* it. The remote prepare survives; the
-- coordinator side is left as an in-doubt local prepared xact that
-- ec_spire_reap_orphaned_remote_prepared_xacts() must converge.
--
-- The operator is expected to run this script and *not* commit/rollback
-- the prepared xact afterward — the reap call in fault.sh is the
-- recovery step.

\set ON_ERROR_STOP on
\set prefix 'ec_spire_aws_repr_1m'

BEGIN;
  INSERT INTO ec_spire_aws_repr_1m_corpus (vec_id, embedding)
  VALUES (
    (SELECT coalesce(max(vec_id), 0) + 1 FROM ec_spire_aws_repr_1m_corpus),
    ARRAY(SELECT random()::real FROM generate_series(1, 1536))::ecvector
  );
PREPARE TRANSACTION 'ecaz_spire_aws_orphan';

\echo === Coordinator prepared-xact list (orphan is the one named ecaz_spire_aws_orphan) ===
SELECT gid, prepared, owner, database FROM pg_prepared_xacts;
