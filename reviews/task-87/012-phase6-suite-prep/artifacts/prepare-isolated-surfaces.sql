\set ON_ERROR_STOP on

CREATE TABLE IF NOT EXISTS task87_phase6_real10k_hnsw_corpus AS
SELECT * FROM task28_ivf_qcmp10k_turboquant_corpus;
CREATE TABLE IF NOT EXISTS task87_phase6_real10k_hnsw_queries AS
SELECT * FROM task28_ivf_qcmp10k_turboquant_queries;
CREATE INDEX IF NOT EXISTS task87_phase6_real10k_hnsw_m16_idx
ON task87_phase6_real10k_hnsw_corpus
USING ec_hnsw (embedding ecvector_ip_ops)
WITH (m = 16, ef_construction = 128);

CREATE TABLE IF NOT EXISTS task87_phase6_real10k_ivf_corpus AS
SELECT * FROM task28_ivf_qcmp10k_turboquant_corpus;
CREATE TABLE IF NOT EXISTS task87_phase6_real10k_ivf_queries AS
SELECT * FROM task28_ivf_qcmp10k_turboquant_queries;
CREATE INDEX IF NOT EXISTS task87_phase6_real10k_ivf_tq_idx
ON task87_phase6_real10k_ivf_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
    nlists = 64,
    nprobe = 64,
    training_sample_rows = 2000,
    storage_format = 'turboquant',
    rerank = 'heap_f32',
    rerank_width = 25
);

CREATE TABLE IF NOT EXISTS task87_phase6_real10k_spire_corpus AS
SELECT * FROM task30_spire_real10k_tq_corpus;
CREATE TABLE IF NOT EXISTS task87_phase6_real10k_spire_queries AS
SELECT * FROM task30_spire_real10k_tq_queries;
CREATE INDEX IF NOT EXISTS task87_phase6_real10k_spire_tq_idx
ON task87_phase6_real10k_spire_corpus
USING ec_spire (embedding ecvector_spire_ip_ops)
WITH (
    nlists = 32,
    nprobe = 24,
    rerank_width = 25,
    local_store_count = 1,
    storage_format = 'turboquant'
);

CREATE TABLE IF NOT EXISTS task87_phase6_real50k_hnsw_corpus AS
SELECT * FROM task67_local_fullq_50k_hnsw_corpus;
CREATE TABLE IF NOT EXISTS task87_phase6_real50k_hnsw_queries AS
SELECT * FROM task67_local_fullq_50k_hnsw_queries;
CREATE INDEX IF NOT EXISTS task87_phase6_real50k_hnsw_m16_idx
ON task87_phase6_real50k_hnsw_corpus
USING ec_hnsw (embedding ecvector_ip_ops)
WITH (m = 16, ef_construction = 128, build_source_column = 'source');

CREATE TABLE IF NOT EXISTS task87_phase6_real50k_ivf_corpus AS
SELECT * FROM task67_local_fullq_50k_hnsw_corpus;
CREATE TABLE IF NOT EXISTS task87_phase6_real50k_ivf_queries AS
SELECT * FROM task67_local_fullq_50k_hnsw_queries;
CREATE INDEX IF NOT EXISTS task87_phase6_real50k_ivf_tq_idx
ON task87_phase6_real50k_ivf_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
    nlists = 64,
    nprobe = 64,
    training_sample_rows = 2000,
    storage_format = 'turboquant',
    rerank = 'heap_f32',
    rerank_width = 25
);

CREATE TABLE IF NOT EXISTS task87_phase6_real50k_spire_corpus AS
SELECT * FROM task67_local_fullq_50k_hnsw_corpus;
CREATE TABLE IF NOT EXISTS task87_phase6_real50k_spire_queries AS
SELECT * FROM task67_local_fullq_50k_hnsw_queries;
CREATE INDEX IF NOT EXISTS task87_phase6_real50k_spire_tq_idx
ON task87_phase6_real50k_spire_corpus
USING ec_spire (embedding ecvector_spire_ip_ops)
WITH (
    nlists = 128,
    nprobe = 24,
    rerank_width = 25,
    local_store_count = 1,
    storage_format = 'turboquant'
);

CREATE TABLE IF NOT EXISTS task87_phase6_real100k_hnsw_corpus AS
SELECT * FROM task67_local_fullq_100k_hnsw_corpus;
CREATE TABLE IF NOT EXISTS task87_phase6_real100k_hnsw_queries AS
SELECT * FROM task67_local_fullq_100k_hnsw_queries;
CREATE INDEX IF NOT EXISTS task87_phase6_real100k_hnsw_m16_idx
ON task87_phase6_real100k_hnsw_corpus
USING ec_hnsw (embedding ecvector_ip_ops)
WITH (m = 16, ef_construction = 128, build_source_column = 'source');

CREATE TABLE IF NOT EXISTS task87_phase6_real100k_ivf_corpus AS
SELECT * FROM task67_local_fullq_100k_hnsw_corpus;
CREATE TABLE IF NOT EXISTS task87_phase6_real100k_ivf_queries AS
SELECT * FROM task67_local_fullq_100k_hnsw_queries;
CREATE INDEX IF NOT EXISTS task87_phase6_real100k_ivf_tq_idx
ON task87_phase6_real100k_ivf_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
    nlists = 64,
    nprobe = 64,
    training_sample_rows = 2000,
    storage_format = 'turboquant',
    rerank = 'heap_f32',
    rerank_width = 25
);

CREATE TABLE IF NOT EXISTS task87_phase6_real100k_spire_corpus AS
SELECT * FROM task67_local_fullq_100k_hnsw_corpus;
CREATE TABLE IF NOT EXISTS task87_phase6_real100k_spire_queries AS
SELECT * FROM task67_local_fullq_100k_hnsw_queries;
CREATE INDEX IF NOT EXISTS task87_phase6_real100k_spire_tq_idx
ON task87_phase6_real100k_spire_corpus
USING ec_spire (embedding ecvector_spire_ip_ops)
WITH (
    nlists = 128,
    nprobe = 24,
    rerank_width = 25,
    local_store_count = 1,
    storage_format = 'turboquant'
);

ANALYZE task87_phase6_real10k_hnsw_corpus;
ANALYZE task87_phase6_real10k_hnsw_queries;
ANALYZE task87_phase6_real10k_ivf_corpus;
ANALYZE task87_phase6_real10k_ivf_queries;
ANALYZE task87_phase6_real10k_spire_corpus;
ANALYZE task87_phase6_real10k_spire_queries;
ANALYZE task87_phase6_real50k_hnsw_corpus;
ANALYZE task87_phase6_real50k_hnsw_queries;
ANALYZE task87_phase6_real50k_ivf_corpus;
ANALYZE task87_phase6_real50k_ivf_queries;
ANALYZE task87_phase6_real50k_spire_corpus;
ANALYZE task87_phase6_real50k_spire_queries;
ANALYZE task87_phase6_real100k_hnsw_corpus;
ANALYZE task87_phase6_real100k_hnsw_queries;
ANALYZE task87_phase6_real100k_ivf_corpus;
ANALYZE task87_phase6_real100k_ivf_queries;
ANALYZE task87_phase6_real100k_spire_corpus;
ANALYZE task87_phase6_real100k_spire_queries;
