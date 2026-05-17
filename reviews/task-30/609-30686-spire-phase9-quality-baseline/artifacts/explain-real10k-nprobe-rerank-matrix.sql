\pset pager off
\timing on

SET enable_seqscan = off;

SELECT
  current_setting('server_version') AS server_version,
  'task30_p9_quality_base_c5ed545_idx' AS index_name,
  pg_relation_size('task30_p9_quality_base_c5ed545_idx'::regclass) AS index_bytes,
  pg_size_pretty(pg_relation_size('task30_p9_quality_base_c5ed545_idx'::regclass)) AS index_size;

SELECT * FROM ec_spire_index_cost_snapshot('task30_p9_quality_base_c5ed545_idx'::regclass);

SET ec_spire.rerank_width = 0;
SET ec_spire.nprobe = 8;
SELECT 'nprobe=8 rerank_width=0' AS baseline_point;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task30_p9_quality_base_c5ed545_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_p9_quality_base_c5ed545_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_spire.nprobe = 16;
SELECT 'nprobe=16 rerank_width=0' AS baseline_point;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task30_p9_quality_base_c5ed545_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_p9_quality_base_c5ed545_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_spire.nprobe = 24;
SELECT 'nprobe=24 rerank_width=0' AS baseline_point;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task30_p9_quality_base_c5ed545_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_p9_quality_base_c5ed545_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_spire.nprobe = 32;
SELECT 'nprobe=32 rerank_width=0' AS baseline_point;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task30_p9_quality_base_c5ed545_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_p9_quality_base_c5ed545_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_spire.rerank_width = 25;
SET ec_spire.nprobe = 8;
SELECT 'nprobe=8 rerank_width=25' AS baseline_point;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task30_p9_quality_base_c5ed545_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_p9_quality_base_c5ed545_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_spire.nprobe = 16;
SELECT 'nprobe=16 rerank_width=25' AS baseline_point;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task30_p9_quality_base_c5ed545_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_p9_quality_base_c5ed545_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_spire.nprobe = 24;
SELECT 'nprobe=24 rerank_width=25' AS baseline_point;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task30_p9_quality_base_c5ed545_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_p9_quality_base_c5ed545_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_spire.nprobe = 32;
SELECT 'nprobe=32 rerank_width=25' AS baseline_point;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task30_p9_quality_base_c5ed545_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_p9_quality_base_c5ed545_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_spire.rerank_width = 50;
SET ec_spire.nprobe = 8;
SELECT 'nprobe=8 rerank_width=50' AS baseline_point;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task30_p9_quality_base_c5ed545_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_p9_quality_base_c5ed545_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_spire.nprobe = 16;
SELECT 'nprobe=16 rerank_width=50' AS baseline_point;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task30_p9_quality_base_c5ed545_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_p9_quality_base_c5ed545_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_spire.nprobe = 24;
SELECT 'nprobe=24 rerank_width=50' AS baseline_point;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task30_p9_quality_base_c5ed545_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_p9_quality_base_c5ed545_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

SET ec_spire.nprobe = 32;
SELECT 'nprobe=32 rerank_width=50' AS baseline_point;
EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task30_p9_quality_base_c5ed545_corpus
ORDER BY embedding <#> (
  SELECT source
  FROM task30_p9_quality_base_c5ed545_queries
  ORDER BY id
  LIMIT 1
)::real[]
LIMIT 10;

RESET enable_seqscan;
RESET ec_spire.nprobe;
RESET ec_spire.rerank_width;
