-- EXPLAIN counters per the reviewer's evidence ask.
-- Run after pg_prewarm so the buffer pool is hot.
\set ON_ERROR_STOP on
\pset format unaligned

SET enable_seqscan = off;

-- One query at each of the cited nprobe cells, against the no-rerank
-- 50k variant. Each EXPLAIN run exposes the 11 IvfExplainCounters
-- per src/am/common/explain.rs (centroid_scores, selected_lists,
-- posting_pages_read, postings_visited / scored / pruned_by_bound,
-- heap_tids_scored, candidates_scored / inserted, rerank_rows,
-- filtered_duplicates).

\echo === 50k nprobe=8 ===
SET ec_ivf.nprobe = 8;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF, BUFFERS OFF)
SELECT id
FROM real_50k_ivf_rabitq_corpus
ORDER BY embedding <#> (
  SELECT source FROM real_50k_ivf_rabitq_queries ORDER BY id LIMIT 1
)::real[]
LIMIT 10;

\echo === 50k nprobe=16 ===
SET ec_ivf.nprobe = 16;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF, BUFFERS OFF)
SELECT id
FROM real_50k_ivf_rabitq_corpus
ORDER BY embedding <#> (
  SELECT source FROM real_50k_ivf_rabitq_queries ORDER BY id LIMIT 1
)::real[]
LIMIT 10;

\echo === 50k nprobe=32 ===
SET ec_ivf.nprobe = 32;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF, BUFFERS OFF)
SELECT id
FROM real_50k_ivf_rabitq_corpus
ORDER BY embedding <#> (
  SELECT source FROM real_50k_ivf_rabitq_queries ORDER BY id LIMIT 1
)::real[]
LIMIT 10;

\echo === 50k nprobe=64 ===
SET ec_ivf.nprobe = 64;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF, BUFFERS OFF)
SELECT id
FROM real_50k_ivf_rabitq_corpus
ORDER BY embedding <#> (
  SELECT source FROM real_50k_ivf_rabitq_queries ORDER BY id LIMIT 1
)::real[]
LIMIT 10;

\echo === 50k nprobe=64 + rerank=heap_f32 width=200 ===
SET ec_ivf.nprobe = 64;
SET ec_ivf.rerank_width = 200;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF, BUFFERS OFF)
SELECT id
FROM real_50k_ivf_rabitq_rerank_corpus
ORDER BY embedding <#> (
  SELECT source FROM real_50k_ivf_rabitq_rerank_queries ORDER BY id LIMIT 1
)::real[]
LIMIT 10;

RESET ec_ivf.nprobe;
RESET ec_ivf.rerank_width;
