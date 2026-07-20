-- M2 latency: 2-node loopback vs single-node, and the transport share (D4).
-- Times N orchestrated searches each way over the committed loopback fixture.
\set ON_ERROR_STOP on
\set roster '0@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter;1@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter'

-- Warm both paths (first call builds the in-memory head graph, per M0 trap).
SET ec_distann.roster = '';
SET ec_distann.local_node_id = 0; SET ec_distann.epoch = 0;
SELECT count(*) FROM ec_distann_debug_expand_search('m2_idx'::regclass::oid, 'm2_idx', ARRAY[0.35,0.62,-0.28,0.64]::real[], 4, 100, 10);
SET ec_distann.roster = :'roster';
SELECT count(*) FROM ec_distann_debug_expand_search('m2_idx'::regclass::oid, 'm2_idx', ARRAY[0.35,0.62,-0.28,0.64]::real[], 4, 100, 10);

DO $$
DECLARE
  n int := 50;
  i int;
  t0 timestamptz; t1 timestamptz;
  single_ms float8; two_ms float8;
BEGIN
  -- Single-node.
  SET ec_distann.roster = '';
  t0 := clock_timestamp();
  FOR i IN 1..n LOOP
    PERFORM count(*) FROM ec_distann_debug_expand_search(
      'm2_idx'::regclass::oid, 'm2_idx', ARRAY[0.35,0.62,-0.28,0.64]::real[], 4, 100, 10);
  END LOOP;
  t1 := clock_timestamp();
  single_ms := extract(epoch FROM (t1 - t0)) * 1000.0 / n;

  -- Two-node loopback.
  SET ec_distann.roster = '0@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter;1@host=/home/peter/.pgrx port=28818 dbname=ec_distann_m2 user=peter';
  t0 := clock_timestamp();
  FOR i IN 1..n LOOP
    PERFORM count(*) FROM ec_distann_debug_expand_search(
      'm2_idx'::regclass::oid, 'm2_idx', ARRAY[0.35,0.62,-0.28,0.64]::real[], 4, 100, 10);
  END LOOP;
  t1 := clock_timestamp();
  two_ms := extract(epoch FROM (t1 - t0)) * 1000.0 / n;

  RAISE NOTICE 'single_node_mean_ms=% two_node_mean_ms=% transport_delta_ms=% transport_share_pct=%',
    round(single_ms::numeric, 3), round(two_ms::numeric, 3),
    round((two_ms - single_ms)::numeric, 3),
    round((100.0 * (two_ms - single_ms) / nullif(two_ms,0))::numeric, 1);
END $$;
