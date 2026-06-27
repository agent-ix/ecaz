-- Task 99 profile — shared per-variant fixture creation (all lanes).
-- Requires t99_src_100k_corpus / t99_src_100k_queries (see the lane
-- sources file). One index per replicated table (index-isolation rule).
-- Index shapes mirror the established per-AM fixture conventions:
--   HNSW:    task87_phase6 shape + explicit storage_format
--   IVF:     task87/94 shape (nlists=64, training_sample_rows=2000)
--   SPIRE:   task87_phase6_real50k shape (nlists=128)
--   DiskANN: task102/103 shape (storage_format only)

-- ---------------------------------------------------------------- HNSW
CREATE TABLE t99_hnsw_tq_100k_corpus AS SELECT id, source, embedding FROM t99_src_100k_corpus;
ALTER TABLE t99_hnsw_tq_100k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t99_hnsw_tq_100k_queries AS SELECT id, source FROM t99_src_100k_queries;
CREATE INDEX t99_hnsw_tq_100k_idx ON t99_hnsw_tq_100k_corpus
USING ec_hnsw (embedding) WITH (m='16', ef_construction='128', storage_format=turboquant);
ANALYZE t99_hnsw_tq_100k_corpus; ANALYZE t99_hnsw_tq_100k_queries;

CREATE TABLE t99_hnsw_rabitq_100k_corpus AS SELECT id, source, embedding FROM t99_src_100k_corpus;
ALTER TABLE t99_hnsw_rabitq_100k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t99_hnsw_rabitq_100k_queries AS SELECT id, source FROM t99_src_100k_queries;
CREATE INDEX t99_hnsw_rabitq_100k_idx ON t99_hnsw_rabitq_100k_corpus
USING ec_hnsw (embedding) WITH (m='16', ef_construction='128', storage_format=rabitq);
ANALYZE t99_hnsw_rabitq_100k_corpus; ANALYZE t99_hnsw_rabitq_100k_queries;

-- ----------------------------------------------------------------- IVF
CREATE TABLE t99_ivf_tq_100k_corpus AS SELECT id, source, embedding FROM t99_src_100k_corpus;
ALTER TABLE t99_ivf_tq_100k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t99_ivf_tq_100k_queries AS SELECT id, source FROM t99_src_100k_queries;
CREATE INDEX t99_ivf_tq_100k_idx ON t99_ivf_tq_100k_corpus
USING ec_ivf (embedding) WITH (nlists='64', nprobe='64', training_sample_rows='2000',
    storage_format=turboquant, rerank=heap_f32, rerank_width='25');
ANALYZE t99_ivf_tq_100k_corpus; ANALYZE t99_ivf_tq_100k_queries;

CREATE TABLE t99_ivf_rabitq1_100k_corpus AS SELECT id, source, embedding FROM t99_src_100k_corpus;
ALTER TABLE t99_ivf_rabitq1_100k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t99_ivf_rabitq1_100k_queries AS SELECT id, source FROM t99_src_100k_queries;
CREATE INDEX t99_ivf_rabitq1_100k_idx ON t99_ivf_rabitq1_100k_corpus
USING ec_ivf (embedding) WITH (nlists='64', nprobe='64', training_sample_rows='2000',
    storage_format=rabitq, quant_bits='1', rerank=heap_f32, rerank_width='50');
ANALYZE t99_ivf_rabitq1_100k_corpus; ANALYZE t99_ivf_rabitq1_100k_queries;

CREATE TABLE t99_ivf_rabitq4_100k_corpus AS SELECT id, source, embedding FROM t99_src_100k_corpus;
ALTER TABLE t99_ivf_rabitq4_100k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t99_ivf_rabitq4_100k_queries AS SELECT id, source FROM t99_src_100k_queries;
CREATE INDEX t99_ivf_rabitq4_100k_idx ON t99_ivf_rabitq4_100k_corpus
USING ec_ivf (embedding) WITH (nlists='64', nprobe='64', training_sample_rows='2000',
    storage_format=rabitq, quant_bits='4', rerank=heap_f32, rerank_width='50');
ANALYZE t99_ivf_rabitq4_100k_corpus; ANALYZE t99_ivf_rabitq4_100k_queries;

CREATE TABLE t99_ivf_pqfs_100k_corpus AS SELECT id, source, embedding FROM t99_src_100k_corpus;
ALTER TABLE t99_ivf_pqfs_100k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t99_ivf_pqfs_100k_queries AS SELECT id, source FROM t99_src_100k_queries;
CREATE INDEX t99_ivf_pqfs_100k_idx ON t99_ivf_pqfs_100k_corpus
USING ec_ivf (embedding) WITH (nlists='64', nprobe='64', training_sample_rows='2000',
    storage_format=pq_fastscan, pq_group_size='8');
ANALYZE t99_ivf_pqfs_100k_corpus; ANALYZE t99_ivf_pqfs_100k_queries;

-- --------------------------------------------------------------- SPIRE
CREATE TABLE t99_spire_tq_100k_corpus AS SELECT id, source, embedding FROM t99_src_100k_corpus;
ALTER TABLE t99_spire_tq_100k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t99_spire_tq_100k_queries AS SELECT id, source FROM t99_src_100k_queries;
CREATE INDEX t99_spire_tq_100k_idx ON t99_spire_tq_100k_corpus
USING ec_spire (embedding) WITH (nlists='128', nprobe='24', rerank_width='25',
    local_store_count='1', storage_format=turboquant);
ANALYZE t99_spire_tq_100k_corpus; ANALYZE t99_spire_tq_100k_queries;

CREATE TABLE t99_spire_rabitq_100k_corpus AS SELECT id, source, embedding FROM t99_src_100k_corpus;
ALTER TABLE t99_spire_rabitq_100k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t99_spire_rabitq_100k_queries AS SELECT id, source FROM t99_src_100k_queries;
CREATE INDEX t99_spire_rabitq_100k_idx ON t99_spire_rabitq_100k_corpus
USING ec_spire (embedding) WITH (nlists='128', nprobe='24', rerank_width='25',
    local_store_count='1', storage_format=rabitq);
ANALYZE t99_spire_rabitq_100k_corpus; ANALYZE t99_spire_rabitq_100k_queries;

-- NOTE: no SPIRE pq_fastscan fixture — structurally absent (product gap,
-- Task 104 finding: encode_assignment_payload requires a persisted
-- grouped-PQ model; no fixture flow can build the index on any host).

-- ------------------------------------------------------------- DiskANN
CREATE TABLE t99_diskann_pqfs_100k_corpus AS SELECT id, source, embedding FROM t99_src_100k_corpus;
ALTER TABLE t99_diskann_pqfs_100k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t99_diskann_pqfs_100k_queries AS SELECT id, source FROM t99_src_100k_queries;
CREATE INDEX t99_diskann_pqfs_100k_idx ON t99_diskann_pqfs_100k_corpus
USING ec_diskann (embedding);
ANALYZE t99_diskann_pqfs_100k_corpus; ANALYZE t99_diskann_pqfs_100k_queries;

CREATE TABLE t99_diskann_rabitq_100k_corpus AS SELECT id, source, embedding FROM t99_src_100k_corpus;
ALTER TABLE t99_diskann_rabitq_100k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t99_diskann_rabitq_100k_queries AS SELECT id, source FROM t99_src_100k_queries;
CREATE INDEX t99_diskann_rabitq_100k_idx ON t99_diskann_rabitq_100k_corpus
USING ec_diskann (embedding) WITH (storage_format=rabitq);
ANALYZE t99_diskann_rabitq_100k_corpus; ANALYZE t99_diskann_rabitq_100k_queries;

CREATE TABLE t99_diskann_tq_100k_corpus AS SELECT id, source, embedding FROM t99_src_100k_corpus;
ALTER TABLE t99_diskann_tq_100k_corpus ADD PRIMARY KEY (id);
CREATE TABLE t99_diskann_tq_100k_queries AS SELECT id, source FROM t99_src_100k_queries;
CREATE INDEX t99_diskann_tq_100k_idx ON t99_diskann_tq_100k_corpus
USING ec_diskann (embedding) WITH (storage_format=turboquant);
ANALYZE t99_diskann_tq_100k_corpus; ANALYZE t99_diskann_tq_100k_queries;
