SELECT
  c.relname,
  pg_relation_size(c.oid) AS index_bytes,
  pg_size_pretty(pg_relation_size(c.oid)) AS index_size,
  c.reloptions
FROM pg_class c
WHERE c.relname IN (
  'task28_ivf_qcmp10k_rabitq_idx',
  'task28_ivf_qcmp25k_rabitq_idx'
)
ORDER BY c.relname;
