\timing on
SELECT version();
SHOW shared_buffers;
SHOW random_page_cost;
SHOW seq_page_cost;
SHOW cpu_operator_cost;
SELECT current_setting('ec_ivf.nprobe', true) AS ec_ivf_nprobe;
SELECT pg_relation_size('task28_ivf_postopt10k_n128w25_idx') AS index_bytes,
       pg_relation_size('task28_ivf_postopt10k_n128w25_corpus') AS corpus_bytes,
       (SELECT count(*) FROM task28_ivf_postopt10k_n128w25_corpus) AS corpus_rows,
       (SELECT count(*) FROM task28_ivf_postopt10k_n128w25_queries) AS query_rows;
