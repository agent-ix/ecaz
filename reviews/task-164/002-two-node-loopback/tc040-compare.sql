-- TC-040/041: 2-node loopback top-k identical to single-node build.
-- Run in ec_distann_m2 (committed) with a released .so. The "2-node" roster
-- points both entries at this same instance; each remote call sets its target
-- local_node_id, so one instance serves both partitions (ADR-085 D2 loopback).
\set ON_ERROR_STOP on
\timing off

\set conn 'host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter'
\set roster '0@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter;1@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter'

-- Single-node baseline (empty roster => coordinator owns everything locally).
SET ec_distann.roster = '';
SET ec_distann.local_node_id = 0;
SET ec_distann.epoch = 0;
DROP TABLE IF EXISTS t_base;
CREATE TEMP TABLE t_base AS
  SELECT rank, vec_id, exact_dist FROM ec_distann_debug_expand_search(
    'm2_idx'::regclass::oid, 'm2_idx',
    ARRAY[0.35, 0.62, -0.28, 0.64]::real[], 4, 100, 10);

-- Two-node loopback (hash placement splits ownership; node-1 ids go remote).
SET ec_distann.roster = :'roster';
SET ec_distann.local_node_id = 0;
SET ec_distann.epoch = 0;
DROP TABLE IF EXISTS t_two;
CREATE TEMP TABLE t_two AS
  SELECT rank, vec_id, exact_dist FROM ec_distann_debug_expand_search(
    'm2_idx'::regclass::oid, 'm2_idx',
    ARRAY[0.35, 0.62, -0.28, 0.64]::real[], 4, 100, 10);

-- Verdict: same row count, same set, identical rank->vec_id mapping.
SELECT
  (SELECT count(*) FROM t_base)                                          AS base_rows,
  (SELECT count(*) FROM t_two)                                           AS two_rows,
  (SELECT count(*) FROM (SELECT vec_id FROM t_base EXCEPT SELECT vec_id FROM t_two) d) AS base_minus_two,
  (SELECT count(*) FROM (SELECT vec_id FROM t_two EXCEPT SELECT vec_id FROM t_base) d) AS two_minus_base,
  (SELECT bool_and(b.vec_id = t.vec_id AND abs(b.exact_dist - t.exact_dist) < 1e-6)
     FROM t_base b JOIN t_two t USING (rank))                            AS ranks_identical;
