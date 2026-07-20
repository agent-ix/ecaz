-- Task 165 004-P1: transport-level fault drills against the real AM scan path.
-- Each drill asserts the NFR-020 contract: a remote fault is an ERROR (fail
-- closed) with a distinct machine-readable class, never a wrong/partial result.
-- Uses the committed v3 p3_idx (ec_distann, 3000 rows) in ec_distann_m2. The
-- query's beam expands node-1-owned vec_ids, so a fault on node 1 is exercised.
\set ON_ERROR_STOP off
\set good '0@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter;1@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter'
\set deadport '0@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter;1@host=/home/peter/.pgrx port=1 dbname=ec_distann_m2 user=peter'
\set baddb '0@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter;1@host=/home/peter/.pgrx port=28818 dbname=ec_distann_nonexistent user=peter'
SET enable_seqscan = off;
DROP TABLE IF EXISTS qv;
CREATE TEMP TABLE qv AS SELECT source AS v FROM m2_10k_queries ORDER BY id LIMIT 1;

\echo '=== DRILL 0: baseline (good 2-node roster) — expect 10 rows ==='
SET ec_distann.roster = :'good'; SET ec_distann.local_node_id = 0; SET ec_distann.epoch = 0;
SELECT count(*) AS baseline_rows
  FROM (SELECT id FROM p3_corpus ORDER BY embedding <#> (SELECT v FROM qv) LIMIT 10) q;

\echo '=== DRILL 1: connection_reset (node 1 unreachable port) — expect ERROR, fail-closed ==='
SET ec_distann.roster = :'deadport';
SELECT count(*) FROM (SELECT id FROM p3_corpus ORDER BY embedding <#> (SELECT v FROM qv) LIMIT 10) q;

\echo '=== DRILL 2: missing_remote_target (node 1 nonexistent database) — expect ERROR, fail-closed ==='
SET ec_distann.roster = :'baddb';
SELECT count(*) FROM (SELECT id FROM p3_corpus ORDER BY embedding <#> (SELECT v FROM qv) LIMIT 10) q;

\echo '=== DRILL 3: no-false-reject — session epoch bump w/o content change — expect 10 rows ==='
-- The FR-082 epoch fingerprint is CONTENT-based (roster + metadata
-- content_digest), not a bare GUC. Bumping ec_distann.epoch on the same index
-- does not diverge the fingerprint, so it must NOT falsely reject. A genuine
-- epoch_mismatch needs actually-divergent content between coordinator and
-- owner; that path is covered at the endpoint by the pg_test fault drill
-- (bogus fingerprint bytea -> [EC_EPOCH_MISMATCH], retriable).
SET ec_distann.roster = :'good'; SET ec_distann.epoch = 999999;
SELECT count(*) AS no_false_reject_rows
  FROM (SELECT id FROM p3_corpus ORDER BY embedding <#> (SELECT v FROM qv) LIMIT 10) q;

\echo '=== DRILL 4: recovery — restore good roster/epoch — expect 10 rows again ==='
SET ec_distann.roster = :'good'; SET ec_distann.epoch = 0;
SELECT count(*) AS recovery_rows
  FROM (SELECT id FROM p3_corpus ORDER BY embedding <#> (SELECT v FROM qv) LIMIT 10) q;
