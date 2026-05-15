-- Phase 13b.8 — smoke verification: confirm the CustomScan path is wired
-- and remote fanout matches the registered remote count.

\set ON_ERROR_STOP on
\set prefix 'ec_spire_aws_synth_10k'

\echo === Registered remote nodes ===
SELECT node_id, descriptor_state, placement_count
FROM ec_spire_remote_node_snapshot(
  format('%s_idx', :'prefix')::regclass
)
ORDER BY node_id;

\echo === EXPLAIN ANALYZE: vector ORDER BY LIMIT through CustomScan ===
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT vec_id
FROM ec_spire_aws_synth_10k_corpus
ORDER BY embedding <#> (
  SELECT embedding FROM ec_spire_aws_synth_10k_queries WHERE vec_id = 0
)
LIMIT 10;

\echo === Handoff summary ===
SELECT *
FROM ec_spire_remote_search_production_scan_handoff_summary(
  format('%s_idx', :'prefix')::regclass,
  (SELECT embedding FROM ec_spire_aws_synth_10k_queries WHERE vec_id = 0)::real[],
  10
);
