\pset pager off
\timing on

SET enable_seqscan = off;
SET ec_ivf.nprobe = 64;
SET ec_ivf.stage2_final_rerank_width = 25;
SET ec_ivf.rerank_width = 100;

SELECT
current_setting('server_version') AS server_version,
current_setting('ec_ivf.nprobe') AS sweep_value,
current_setting('ec_ivf.rerank_width') AS rerank_width,
           'ec_ivf' AS profile;

SELECT
'task124_fw_tq_g100_100k_coarse_rerank_idx' AS index_name,
pg_relation_size('task124_fw_tq_g100_100k_coarse_rerank_idx'::regclass) AS index_bytes,
pg_size_pretty(pg_relation_size('task124_fw_tq_g100_100k_coarse_rerank_idx'::regclass)) AS index_size;

SELECT *
FROM ec_ivf_index_cost_snapshot('task124_fw_tq_g100_100k_coarse_rerank_idx'::regclass);

EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task124_fw_tq_g100_100k_corpus
ORDER BY embedding <#> (
SELECT source
FROM task124_fw_tq_g100_100k_queries
ORDER BY id
LIMIT 1
)::real[]
LIMIT 10;

RESET enable_seqscan;
RESET ec_ivf.nprobe;
RESET ec_ivf.stage2_final_rerank_width;
RESET ec_ivf.rerank_width;
