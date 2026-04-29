DROP TABLE IF EXISTS task28_ivf_qcmp25k_rabitq_corpus CASCADE;
DROP TABLE IF EXISTS task28_ivf_qcmp25k_rabitq_queries CASCADE;

CREATE TABLE task28_ivf_qcmp25k_rabitq_corpus AS
SELECT id, source, embedding
FROM task28_ivf_postopt25k_n64w25_corpus;

ALTER TABLE task28_ivf_qcmp25k_rabitq_corpus
  ADD PRIMARY KEY (id);

CREATE TABLE task28_ivf_qcmp25k_rabitq_queries AS
SELECT id, source
FROM task28_ivf_postopt25k_n64w25_queries;

ALTER TABLE task28_ivf_qcmp25k_rabitq_queries
  ADD PRIMARY KEY (id);

DO $$
DECLARE
  started_at timestamptz;
  elapsed_ms numeric;
BEGIN
  started_at := clock_timestamp();
  CREATE INDEX task28_ivf_qcmp25k_rabitq_idx
    ON task28_ivf_qcmp25k_rabitq_corpus USING ec_ivf (embedding ecvector_ip_ops)
    WITH (
      nlists = 64,
      nprobe = 64,
      training_sample_rows = 2000,
      storage_format = 'rabitq',
      rerank = 'heap_f32',
      rerank_width = 750
    );
  elapsed_ms := extract(epoch FROM clock_timestamp() - started_at) * 1000.0;
  RAISE NOTICE 'task28_ivf_qcmp25k_rabitq_idx build_ms=%', round(elapsed_ms, 3);
END $$;

ANALYZE task28_ivf_qcmp25k_rabitq_corpus;
ANALYZE task28_ivf_qcmp25k_rabitq_queries;

SELECT
  c.relname,
  pg_relation_size(c.oid) AS index_bytes,
  pg_size_pretty(pg_relation_size(c.oid)) AS index_size,
  c.reloptions
FROM pg_class c
WHERE c.relname = 'task28_ivf_qcmp25k_rabitq_idx';
