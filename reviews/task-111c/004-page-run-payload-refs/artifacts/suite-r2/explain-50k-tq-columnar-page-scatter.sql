\pset pager off
\timing on

SET enable_seqscan = off;
SET ec_ivf.nprobe = 32;
SET ec_ivf.dense_posting_coalescing = on;
SET ec_ivf.dense_posting_typed_views = off;
SET ec_ivf.columnar_page_scatter = on;
SET ec_ivf.scratch_soa_batch_decode = on;
SET ec_ivf.rerank_width = 0;

SELECT
current_setting('server_version') AS server_version,
current_setting('ec_ivf.nprobe') AS sweep_value,
current_setting('ec_ivf.scratch_soa_batch_decode') AS scratch_soa_batch_decode,
           current_setting('ec_ivf.rerank_width') AS rerank_width,
           'ec_ivf' AS profile;

SELECT
'task111b_008_50k_tq_columnar_turboquant_idx' AS index_name,
pg_relation_size('task111b_008_50k_tq_columnar_turboquant_idx'::regclass) AS index_bytes,
pg_size_pretty(pg_relation_size('task111b_008_50k_tq_columnar_turboquant_idx'::regclass)) AS index_size;

SELECT *
FROM ec_ivf_index_cost_snapshot('task111b_008_50k_tq_columnar_turboquant_idx'::regclass);

EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task111b_008_50k_tq_columnar_corpus
ORDER BY embedding <#> (
SELECT source
FROM task111b_008_50k_tq_columnar_queries
ORDER BY id
LIMIT 1
)::real[]
LIMIT 10;

RESET enable_seqscan;
RESET ec_ivf.nprobe;
RESET ec_ivf.dense_posting_coalescing;
RESET ec_ivf.dense_posting_typed_views;
RESET ec_ivf.columnar_page_scatter;
RESET ec_ivf.scratch_soa_batch_decode;
RESET ec_ivf.rerank_width;
