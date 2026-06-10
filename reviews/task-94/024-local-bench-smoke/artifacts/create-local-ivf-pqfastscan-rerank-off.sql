CREATE TABLE task94_local_pqfs10k_roff_corpus AS
SELECT id, source, embedding
FROM task28_ivf_pqg10k_g8_corpus;

ALTER TABLE task94_local_pqfs10k_roff_corpus
ADD PRIMARY KEY (id);

CREATE TABLE task94_local_pqfs10k_roff_queries AS
SELECT id, source
FROM task28_ivf_pqg10k_g8_queries;

CREATE INDEX task94_local_pqfs10k_roff_idx
ON task94_local_pqfs10k_roff_corpus
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
    nlists = 64,
    nprobe = 64,
    training_sample_rows = 2000,
    storage_format = 'pq_fastscan',
    pq_group_size = 8,
    rerank = 'off'
);

ANALYZE task94_local_pqfs10k_roff_corpus;
ANALYZE task94_local_pqfs10k_roff_queries;
