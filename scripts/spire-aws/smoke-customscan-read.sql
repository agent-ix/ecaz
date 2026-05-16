-- Phase 13b.8 — smoke verification: confirm the CustomScan path is wired
-- and remote fanout matches the registered remote count.

\set ON_ERROR_STOP on
\if :{?prefix}
\else
\set prefix 'ec_spire_aws_synth_10k'
\endif

SELECT format('%I_corpus', :'prefix') AS corpus_table,
       format('%I_queries', :'prefix') AS queries_table,
       format('%I_idx', :'prefix') AS index_name
\gset

\echo === Registered remote nodes ===
SELECT node_id, descriptor_state, placement_count
FROM ec_spire_remote_node_snapshot(
  :'index_name'::regclass
)
ORDER BY node_id;

\echo === EXPLAIN ANALYZE: vector ORDER BY LIMIT through CustomScan ===
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT vec_id
FROM :corpus_table
ORDER BY embedding <#> (
  SELECT embedding FROM :queries_table WHERE vec_id = 0
)
LIMIT 10;

\echo === Handoff summary ===
SELECT *
FROM ec_spire_remote_search_production_scan_handoff_summary(
  :'index_name'::regclass,
  (SELECT embedding FROM :queries_table WHERE vec_id = 0)::real[],
  10
);

\echo === Production read profile ===
SELECT *
FROM ec_spire_remote_search_production_read_profile(
  :'index_name'::regclass,
  (SELECT embedding FROM :queries_table WHERE vec_id = 0)::real[],
  10
);
