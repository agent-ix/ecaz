-- Task 105 fixtures @ 50k (sources: real_50k_ivf_tq_corpus/_queries)
CREATE TABLE t105_src_50k_corpus AS SELECT id, source, embedding FROM real_50k_ivf_tq_corpus;
ALTER TABLE t105_src_50k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_src_50k_queries AS SELECT id, source FROM real_50k_ivf_tq_queries;
ANALYZE t105_src_50k_corpus; ANALYZE t105_src_50k_queries;

CREATE TABLE t105_hnsw_tq_50k_corpus AS SELECT id, source, embedding FROM t105_src_50k_corpus;
ALTER TABLE t105_hnsw_tq_50k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_hnsw_tq_50k_queries AS SELECT id, source FROM t105_src_50k_queries;
CREATE INDEX t105_hnsw_tq_50k_idx ON t105_hnsw_tq_50k_corpus USING ec_hnsw (embedding) WITH (m='16', ef_construction='128', storage_format=turboquant);
ANALYZE t105_hnsw_tq_50k_corpus; ANALYZE t105_hnsw_tq_50k_queries;

CREATE TABLE t105_hnsw_rabitq_50k_corpus AS SELECT id, source, embedding FROM t105_src_50k_corpus;
ALTER TABLE t105_hnsw_rabitq_50k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_hnsw_rabitq_50k_queries AS SELECT id, source FROM t105_src_50k_queries;
CREATE INDEX t105_hnsw_rabitq_50k_idx ON t105_hnsw_rabitq_50k_corpus USING ec_hnsw (embedding) WITH (m='16', ef_construction='128', storage_format=rabitq);
ANALYZE t105_hnsw_rabitq_50k_corpus; ANALYZE t105_hnsw_rabitq_50k_queries;

CREATE TABLE t105_ivf_tq_50k_corpus AS SELECT id, source, embedding FROM t105_src_50k_corpus;
ALTER TABLE t105_ivf_tq_50k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_ivf_tq_50k_queries AS SELECT id, source FROM t105_src_50k_queries;
CREATE INDEX t105_ivf_tq_50k_idx ON t105_ivf_tq_50k_corpus USING ec_ivf (embedding) WITH (nlists='64', nprobe='64', training_sample_rows='2000', storage_format=turboquant, rerank=heap_f32, rerank_width='25');
ANALYZE t105_ivf_tq_50k_corpus; ANALYZE t105_ivf_tq_50k_queries;

CREATE TABLE t105_ivf_rabitq1_50k_corpus AS SELECT id, source, embedding FROM t105_src_50k_corpus;
ALTER TABLE t105_ivf_rabitq1_50k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_ivf_rabitq1_50k_queries AS SELECT id, source FROM t105_src_50k_queries;
CREATE INDEX t105_ivf_rabitq1_50k_idx ON t105_ivf_rabitq1_50k_corpus USING ec_ivf (embedding) WITH (nlists='64', nprobe='64', training_sample_rows='2000', storage_format=rabitq, quant_bits='1', rerank=heap_f32, rerank_width='50');
ANALYZE t105_ivf_rabitq1_50k_corpus; ANALYZE t105_ivf_rabitq1_50k_queries;

CREATE TABLE t105_ivf_rabitq4_50k_corpus AS SELECT id, source, embedding FROM t105_src_50k_corpus;
ALTER TABLE t105_ivf_rabitq4_50k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_ivf_rabitq4_50k_queries AS SELECT id, source FROM t105_src_50k_queries;
CREATE INDEX t105_ivf_rabitq4_50k_idx ON t105_ivf_rabitq4_50k_corpus USING ec_ivf (embedding) WITH (nlists='64', nprobe='64', training_sample_rows='2000', storage_format=rabitq, quant_bits='4', rerank=heap_f32, rerank_width='50');
ANALYZE t105_ivf_rabitq4_50k_corpus; ANALYZE t105_ivf_rabitq4_50k_queries;

CREATE TABLE t105_ivf_pqfs_50k_corpus AS SELECT id, source, embedding FROM t105_src_50k_corpus;
ALTER TABLE t105_ivf_pqfs_50k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_ivf_pqfs_50k_queries AS SELECT id, source FROM t105_src_50k_queries;
CREATE INDEX t105_ivf_pqfs_50k_idx ON t105_ivf_pqfs_50k_corpus USING ec_ivf (embedding) WITH (nlists='64', nprobe='64', training_sample_rows='2000', storage_format=pq_fastscan, pq_group_size='8');
ANALYZE t105_ivf_pqfs_50k_corpus; ANALYZE t105_ivf_pqfs_50k_queries;

CREATE TABLE t105_spire_tq_50k_corpus AS SELECT id, source, embedding FROM t105_src_50k_corpus;
ALTER TABLE t105_spire_tq_50k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_spire_tq_50k_queries AS SELECT id, source FROM t105_src_50k_queries;
CREATE INDEX t105_spire_tq_50k_idx ON t105_spire_tq_50k_corpus USING ec_spire (embedding) WITH (nlists='128', nprobe='24', rerank_width='25', local_store_count='1', storage_format=turboquant);
ANALYZE t105_spire_tq_50k_corpus; ANALYZE t105_spire_tq_50k_queries;

CREATE TABLE t105_spire_rabitq_50k_corpus AS SELECT id, source, embedding FROM t105_src_50k_corpus;
ALTER TABLE t105_spire_rabitq_50k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_spire_rabitq_50k_queries AS SELECT id, source FROM t105_src_50k_queries;
CREATE INDEX t105_spire_rabitq_50k_idx ON t105_spire_rabitq_50k_corpus USING ec_spire (embedding) WITH (nlists='128', nprobe='24', rerank_width='25', local_store_count='1', storage_format=rabitq);
ANALYZE t105_spire_rabitq_50k_corpus; ANALYZE t105_spire_rabitq_50k_queries;

CREATE TABLE t105_diskann_pqfs_50k_corpus AS SELECT id, source, embedding FROM t105_src_50k_corpus;
ALTER TABLE t105_diskann_pqfs_50k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_diskann_pqfs_50k_queries AS SELECT id, source FROM t105_src_50k_queries;
CREATE INDEX t105_diskann_pqfs_50k_idx ON t105_diskann_pqfs_50k_corpus USING ec_diskann (embedding);
ANALYZE t105_diskann_pqfs_50k_corpus; ANALYZE t105_diskann_pqfs_50k_queries;

CREATE TABLE t105_diskann_rabitq_50k_corpus AS SELECT id, source, embedding FROM t105_src_50k_corpus;
ALTER TABLE t105_diskann_rabitq_50k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_diskann_rabitq_50k_queries AS SELECT id, source FROM t105_src_50k_queries;
CREATE INDEX t105_diskann_rabitq_50k_idx ON t105_diskann_rabitq_50k_corpus USING ec_diskann (embedding) WITH (storage_format=rabitq);
ANALYZE t105_diskann_rabitq_50k_corpus; ANALYZE t105_diskann_rabitq_50k_queries;

CREATE TABLE t105_diskann_tq_50k_corpus AS SELECT id, source, embedding FROM t105_src_50k_corpus;
ALTER TABLE t105_diskann_tq_50k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t105_diskann_tq_50k_queries AS SELECT id, source FROM t105_src_50k_queries;
CREATE INDEX t105_diskann_tq_50k_idx ON t105_diskann_tq_50k_corpus USING ec_diskann (embedding) WITH (storage_format=turboquant);
ANALYZE t105_diskann_tq_50k_corpus; ANALYZE t105_diskann_tq_50k_queries;

