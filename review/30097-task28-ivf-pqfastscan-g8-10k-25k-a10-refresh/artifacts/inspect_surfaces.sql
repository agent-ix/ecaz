\timing on

SELECT relname, relkind, reltuples::bigint AS reltuples
FROM pg_class
WHERE relname LIKE 'task28_ivf_qcmp10k_%'
   OR relname LIKE 'task28_ivf_pqg10k_g8%'
   OR relname LIKE 'task28_ivf_pqg25k_g8%'
   OR relname LIKE 'task28_ivf_postopt25k_n64w25%'
ORDER BY relname;

SELECT
  relname,
  pg_size_pretty(pg_relation_size(oid)) AS size,
  reloptions
FROM pg_class
WHERE relname IN (
  'task28_ivf_qcmp10k_turboquant_idx',
  'task28_ivf_pqg10k_g8_idx',
  'task28_ivf_postopt25k_n64w25_idx',
  'task28_ivf_pqg25k_g8_idx'
)
ORDER BY relname;
