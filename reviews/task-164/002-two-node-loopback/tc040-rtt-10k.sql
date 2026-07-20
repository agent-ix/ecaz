-- M2 two-node loopback on the real 10k / dim-1536 corpus (compute-representative
-- for the D4 transport-share evaluation). Result identity (TC-040/041) + latency.
\set ON_ERROR_STOP on
\set roster '0@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter;1@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter'

-- A real query vector (dim 1536) as real[].
DROP TABLE IF EXISTS q;
CREATE TEMP TABLE q AS
  SELECT source AS v FROM m2_10k_queries ORDER BY id LIMIT 1;

-- Single-node baseline.
SET ec_distann.roster = ''; SET ec_distann.local_node_id = 0; SET ec_distann.epoch = 0;
DROP TABLE IF EXISTS t_base;
CREATE TEMP TABLE t_base AS
  SELECT rank, vec_id, exact_dist FROM ec_distann_debug_expand_search(
    'm2_10k_idx'::regclass::oid, 'm2_10k_idx', (SELECT v FROM q), 4, 64, 10);

-- Two-node loopback.
SET ec_distann.roster = :'roster'; SET ec_distann.local_node_id = 0;
DROP TABLE IF EXISTS t_two;
CREATE TEMP TABLE t_two AS
  SELECT rank, vec_id, exact_dist FROM ec_distann_debug_expand_search(
    'm2_10k_idx'::regclass::oid, 'm2_10k_idx', (SELECT v FROM q), 4, 64, 10);

SELECT
  (SELECT count(*) FROM t_base) AS base_rows,
  (SELECT count(*) FROM t_two)  AS two_rows,
  (SELECT count(*) FROM (SELECT vec_id FROM t_base EXCEPT SELECT vec_id FROM t_two) d) AS base_minus_two,
  (SELECT count(*) FROM (SELECT vec_id FROM t_two EXCEPT SELECT vec_id FROM t_base) d) AS two_minus_base,
  (SELECT bool_and(b.vec_id = t.vec_id AND abs(b.exact_dist - t.exact_dist) < 1e-6)
     FROM t_base b JOIN t_two t USING (rank)) AS ranks_identical;

-- Latency: mean over N searches each way (warm; first-call head-graph build excluded).
DO $$
DECLARE
  qv real[]; n int := 40; i int; t0 timestamptz; t1 timestamptz; s float8; d float8;
BEGIN
  SELECT v INTO qv FROM q;
  SET ec_distann.roster = '';
  PERFORM count(*) FROM ec_distann_debug_expand_search('m2_10k_idx'::regclass::oid,'m2_10k_idx',qv,4,64,10);
  t0 := clock_timestamp();
  FOR i IN 1..n LOOP PERFORM count(*) FROM ec_distann_debug_expand_search('m2_10k_idx'::regclass::oid,'m2_10k_idx',qv,4,64,10); END LOOP;
  t1 := clock_timestamp(); s := extract(epoch FROM (t1-t0))*1000.0/n;

  SET ec_distann.roster = '0@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter;1@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter';
  PERFORM count(*) FROM ec_distann_debug_expand_search('m2_10k_idx'::regclass::oid,'m2_10k_idx',qv,4,64,10);
  t0 := clock_timestamp();
  FOR i IN 1..n LOOP PERFORM count(*) FROM ec_distann_debug_expand_search('m2_10k_idx'::regclass::oid,'m2_10k_idx',qv,4,64,10); END LOOP;
  t1 := clock_timestamp(); d := extract(epoch FROM (t1-t0))*1000.0/n;

  RAISE NOTICE 'REAL10K single_ms=% two_ms=% transport_delta_ms=% transport_share_pct=%',
    round(s::numeric,3), round(d::numeric,3), round((d-s)::numeric,3),
    round((100.0*(d-s)/nullif(d,0))::numeric,1);
END $$;
