\pset pager off
\timing off

SET client_min_messages = info;
SET enable_seqscan = off;
SET ec_diskann.prefilter_kind = 'binary_sidecar';
SET ec_diskann.log_scan_profile = on;

\echo diskann_cost_split width=64
SET ec_diskann.list_size = 64;
SET ec_diskann.rerank_budget = 64;
SELECT id
FROM profile_r10k_dann_pf_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM profile_r10k_dann_pf_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

\echo diskann_cost_split width=200
SET ec_diskann.list_size = 200;
SET ec_diskann.rerank_budget = 200;
SELECT id
FROM profile_r10k_dann_pf_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM profile_r10k_dann_pf_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

\echo diskann_cost_split width=800
SET ec_diskann.list_size = 800;
SET ec_diskann.rerank_budget = 800;
SELECT id
FROM profile_r10k_dann_pf_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM profile_r10k_dann_pf_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

RESET ec_diskann.log_scan_profile;
RESET ec_diskann.prefilter_kind;
RESET ec_diskann.list_size;
RESET ec_diskann.rerank_budget;
RESET enable_seqscan;
