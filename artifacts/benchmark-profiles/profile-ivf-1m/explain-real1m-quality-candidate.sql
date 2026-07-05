\pset pager off
\timing on

SET enable_seqscan = off;
SET ec_ivf.nprobe = 128;
SET ec_ivf.rerank_width = 1000;

SELECT
current_setting('server_version') AS server_version,
current_setting('ec_ivf.nprobe') AS nprobe,
current_setting('ec_ivf.rerank_width') AS rerank_width;

SELECT
'ec_hnsw_real_ann_benchmarks_anchor_idx' AS index_name,
pg_relation_size('ec_hnsw_real_ann_benchmarks_anchor_idx'::regclass) AS index_bytes,
pg_size_pretty(pg_relation_size('ec_hnsw_real_ann_benchmarks_anchor_idx'::regclass)) AS index_size;

EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM ec_hnsw_real_ann_benchmarks_anchor_corpus
ORDER BY embedding <#> (
SELECT source
FROM ec_hnsw_real_ann_benchmarks_anchor_queries
ORDER BY id
LIMIT 1
)::real[]
LIMIT 10;

RESET enable_seqscan;
RESET ec_ivf.nprobe;
RESET ec_ivf.rerank_width;
