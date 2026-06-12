-- Task 105 fixtures @ 1m (sources: real_1m_ivf_rabitq1_rerank_corpus/_queries)
CREATE TABLE t105_src_1m_corpus AS SELECT id, source, embedding FROM real_1m_ivf_rabitq1_rerank_corpus;
ALTER TABLE t105_src_1m_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_src_1m_queries AS SELECT id, source FROM real_1m_ivf_rabitq1_rerank_queries;
ANALYZE t105_src_1m_corpus; ANALYZE t105_src_1m_queries;

CREATE TABLE t105_hnsw_tq_1m_corpus AS SELECT id, source, embedding FROM t105_src_1m_corpus;
ALTER TABLE t105_hnsw_tq_1m_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_hnsw_tq_1m_queries AS SELECT id, source FROM t105_src_1m_queries;
CREATE INDEX t105_hnsw_tq_1m_idx ON t105_hnsw_tq_1m_corpus USING ec_hnsw (embedding) WITH (m='16', ef_construction='128', storage_format=turboquant);
ANALYZE t105_hnsw_tq_1m_corpus; ANALYZE t105_hnsw_tq_1m_queries;

CREATE TABLE t105_hnsw_rabitq_1m_corpus AS SELECT id, source, embedding FROM t105_src_1m_corpus;
ALTER TABLE t105_hnsw_rabitq_1m_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_hnsw_rabitq_1m_queries AS SELECT id, source FROM t105_src_1m_queries;
CREATE INDEX t105_hnsw_rabitq_1m_idx ON t105_hnsw_rabitq_1m_corpus USING ec_hnsw (embedding) WITH (m='16', ef_construction='128', storage_format=rabitq);
ANALYZE t105_hnsw_rabitq_1m_corpus; ANALYZE t105_hnsw_rabitq_1m_queries;

CREATE TABLE t105_ivf_tq_1m_corpus AS SELECT id, source, embedding FROM t105_src_1m_corpus;
ALTER TABLE t105_ivf_tq_1m_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_ivf_tq_1m_queries AS SELECT id, source FROM t105_src_1m_queries;
CREATE INDEX t105_ivf_tq_1m_idx ON t105_ivf_tq_1m_corpus USING ec_ivf (embedding) WITH (nlists='256', nprobe='64', training_sample_rows='2000', storage_format=turboquant, rerank=heap_f32, rerank_width='25');
ANALYZE t105_ivf_tq_1m_corpus; ANALYZE t105_ivf_tq_1m_queries;

CREATE TABLE t105_ivf_rabitq1_1m_corpus AS SELECT id, source, embedding FROM t105_src_1m_corpus;
ALTER TABLE t105_ivf_rabitq1_1m_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_ivf_rabitq1_1m_queries AS SELECT id, source FROM t105_src_1m_queries;
CREATE INDEX t105_ivf_rabitq1_1m_idx ON t105_ivf_rabitq1_1m_corpus USING ec_ivf (embedding) WITH (nlists='256', nprobe='64', training_sample_rows='2000', storage_format=rabitq, quant_bits='1', rerank=heap_f32, rerank_width='50');
ANALYZE t105_ivf_rabitq1_1m_corpus; ANALYZE t105_ivf_rabitq1_1m_queries;

CREATE TABLE t105_ivf_rabitq4_1m_corpus AS SELECT id, source, embedding FROM t105_src_1m_corpus;
ALTER TABLE t105_ivf_rabitq4_1m_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_ivf_rabitq4_1m_queries AS SELECT id, source FROM t105_src_1m_queries;
CREATE INDEX t105_ivf_rabitq4_1m_idx ON t105_ivf_rabitq4_1m_corpus USING ec_ivf (embedding) WITH (nlists='256', nprobe='64', training_sample_rows='2000', storage_format=rabitq, quant_bits='4', rerank=heap_f32, rerank_width='50');
ANALYZE t105_ivf_rabitq4_1m_corpus; ANALYZE t105_ivf_rabitq4_1m_queries;

CREATE TABLE t105_ivf_pqfs_1m_corpus AS SELECT id, source, embedding FROM t105_src_1m_corpus;
ALTER TABLE t105_ivf_pqfs_1m_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_ivf_pqfs_1m_queries AS SELECT id, source FROM t105_src_1m_queries;
CREATE INDEX t105_ivf_pqfs_1m_idx ON t105_ivf_pqfs_1m_corpus USING ec_ivf (embedding) WITH (nlists='256', nprobe='64', training_sample_rows='2000', storage_format=pq_fastscan, pq_group_size='8');
ANALYZE t105_ivf_pqfs_1m_corpus; ANALYZE t105_ivf_pqfs_1m_queries;

CREATE TABLE t105_spire_tq_1m_corpus AS SELECT id, source, embedding FROM t105_src_1m_corpus;
ALTER TABLE t105_spire_tq_1m_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_spire_tq_1m_queries AS SELECT id, source FROM t105_src_1m_queries;
CREATE INDEX t105_spire_tq_1m_idx ON t105_spire_tq_1m_corpus USING ec_spire (embedding) WITH (nlists='512', nprobe='24', rerank_width='25', local_store_count='1', storage_format=turboquant);
ANALYZE t105_spire_tq_1m_corpus; ANALYZE t105_spire_tq_1m_queries;

CREATE TABLE t105_spire_rabitq_1m_corpus AS SELECT id, source, embedding FROM t105_src_1m_corpus;
ALTER TABLE t105_spire_rabitq_1m_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_spire_rabitq_1m_queries AS SELECT id, source FROM t105_src_1m_queries;
CREATE INDEX t105_spire_rabitq_1m_idx ON t105_spire_rabitq_1m_corpus USING ec_spire (embedding) WITH (nlists='512', nprobe='24', rerank_width='25', local_store_count='1', storage_format=rabitq);
ANALYZE t105_spire_rabitq_1m_corpus; ANALYZE t105_spire_rabitq_1m_queries;

CREATE TABLE t105_diskann_pqfs_1m_corpus AS SELECT id, source, embedding FROM t105_src_1m_corpus;
ALTER TABLE t105_diskann_pqfs_1m_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_diskann_pqfs_1m_queries AS SELECT id, source FROM t105_src_1m_queries;
CREATE INDEX t105_diskann_pqfs_1m_idx ON t105_diskann_pqfs_1m_corpus USING ec_diskann (embedding);
ANALYZE t105_diskann_pqfs_1m_corpus; ANALYZE t105_diskann_pqfs_1m_queries;

CREATE TABLE t105_diskann_rabitq_1m_corpus AS SELECT id, source, embedding FROM t105_src_1m_corpus;
ALTER TABLE t105_diskann_rabitq_1m_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_diskann_rabitq_1m_queries AS SELECT id, source FROM t105_src_1m_queries;
CREATE INDEX t105_diskann_rabitq_1m_idx ON t105_diskann_rabitq_1m_corpus USING ec_diskann (embedding) WITH (storage_format=rabitq);
ANALYZE t105_diskann_rabitq_1m_corpus; ANALYZE t105_diskann_rabitq_1m_queries;

CREATE TABLE t105_diskann_tq_1m_corpus AS SELECT id, source, embedding FROM t105_src_1m_corpus;
ALTER TABLE t105_diskann_tq_1m_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_diskann_tq_1m_queries AS SELECT id, source FROM t105_src_1m_queries;
CREATE INDEX t105_diskann_tq_1m_idx ON t105_diskann_tq_1m_corpus USING ec_diskann (embedding) WITH (storage_format=turboquant);
ANALYZE t105_diskann_tq_1m_corpus; ANALYZE t105_diskann_tq_1m_queries;

