CREATE TABLE t99_src_100k_corpus AS
SELECT id, source, embedding FROM real_100k_ivf_rabitq1_rerank_corpus;
ALTER TABLE t99_src_100k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t99_src_100k_queries AS
SELECT id, source FROM real_100k_ivf_rabitq1_rerank_queries;
ANALYZE t99_src_100k_corpus;
ANALYZE t99_src_100k_queries;
SELECT 'src_corpus_rows', count(*) FROM t99_src_100k_corpus;
SELECT 'src_queries_rows', count(*) FROM t99_src_100k_queries;
