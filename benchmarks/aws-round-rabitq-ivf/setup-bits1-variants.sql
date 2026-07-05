\set ON_ERROR_STOP on
\timing on

-- 1-bit RaBitQ variants. Pair these with rerank='heap_f32' at scan
-- time (via SET ec_ivf.rerank_width = N) to recover recall — 1-bit
-- alone has a ceiling around 0.4-0.5 recall@10 at DBpedia scale.

DROP TABLE IF EXISTS real_10k_ivf_rabitq1_corpus;
CREATE TABLE real_10k_ivf_rabitq1_corpus AS TABLE real_10k_ivf_rabitq_corpus;
DROP TABLE IF EXISTS real_10k_ivf_rabitq1_queries;
CREATE TABLE real_10k_ivf_rabitq1_queries AS TABLE real_10k_ivf_rabitq_queries;
CREATE INDEX real_10k_ivf_rabitq1_idx ON real_10k_ivf_rabitq1_corpus
    USING ec_ivf (embedding ecvector_ip_ops)
    WITH (storage_format = 'rabitq', quant_bits = 1);

DROP TABLE IF EXISTS real_50k_ivf_rabitq1_corpus;
CREATE TABLE real_50k_ivf_rabitq1_corpus AS TABLE real_50k_ivf_rabitq_corpus;
DROP TABLE IF EXISTS real_50k_ivf_rabitq1_queries;
CREATE TABLE real_50k_ivf_rabitq1_queries AS TABLE real_50k_ivf_rabitq_queries;
CREATE INDEX real_50k_ivf_rabitq1_idx ON real_50k_ivf_rabitq1_corpus
    USING ec_ivf (embedding ecvector_ip_ops)
    WITH (storage_format = 'rabitq', quant_bits = 1);

-- 1-bit + heap_f32 rerank variant for the recall-matched comparison.
DROP TABLE IF EXISTS real_50k_ivf_rabitq1_rerank_corpus;
CREATE TABLE real_50k_ivf_rabitq1_rerank_corpus AS TABLE real_50k_ivf_rabitq_corpus;
DROP TABLE IF EXISTS real_50k_ivf_rabitq1_rerank_queries;
CREATE TABLE real_50k_ivf_rabitq1_rerank_queries AS TABLE real_50k_ivf_rabitq_queries;
CREATE INDEX real_50k_ivf_rabitq1_rerank_idx ON real_50k_ivf_rabitq1_rerank_corpus
    USING ec_ivf (embedding ecvector_ip_ops)
    WITH (storage_format = 'rabitq', quant_bits = 1, rerank = 'heap_f32');

SELECT relname,
       pg_size_pretty(pg_relation_size(oid)) AS size
FROM pg_class
WHERE relname LIKE 'real_%_rabitq1_%'
ORDER BY relname;
