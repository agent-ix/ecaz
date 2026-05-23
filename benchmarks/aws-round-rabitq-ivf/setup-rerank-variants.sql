\set ON_ERROR_STOP on
\timing on

DROP TABLE IF EXISTS real_10k_ivf_rabitq_rerank_corpus;
CREATE TABLE real_10k_ivf_rabitq_rerank_corpus AS TABLE real_10k_ivf_rabitq_corpus;
DROP TABLE IF EXISTS real_10k_ivf_rabitq_rerank_queries;
CREATE TABLE real_10k_ivf_rabitq_rerank_queries AS TABLE real_10k_ivf_rabitq_queries;
CREATE INDEX real_10k_ivf_rabitq_rerank_idx ON real_10k_ivf_rabitq_rerank_corpus
    USING ec_ivf (embedding ecvector_ip_ops)
    WITH (storage_format = 'rabitq', rerank = 'heap_f32');

DROP TABLE IF EXISTS real_50k_ivf_rabitq_rerank_corpus;
CREATE TABLE real_50k_ivf_rabitq_rerank_corpus AS TABLE real_50k_ivf_rabitq_corpus;
DROP TABLE IF EXISTS real_50k_ivf_rabitq_rerank_queries;
CREATE TABLE real_50k_ivf_rabitq_rerank_queries AS TABLE real_50k_ivf_rabitq_queries;
CREATE INDEX real_50k_ivf_rabitq_rerank_idx ON real_50k_ivf_rabitq_rerank_corpus
    USING ec_ivf (embedding ecvector_ip_ops)
    WITH (storage_format = 'rabitq', rerank = 'heap_f32');

SELECT relname,
       pg_size_pretty(pg_relation_size(oid)) AS size
FROM pg_class
WHERE relname LIKE 'real_%_rerank_%'
ORDER BY relname;
