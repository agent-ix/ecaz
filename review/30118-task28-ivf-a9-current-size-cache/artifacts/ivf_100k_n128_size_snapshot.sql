SELECT
  c.relname,
  am.amname,
  c.reloptions,
  pg_relation_size(c.oid) AS index_bytes,
  pg_size_pretty(pg_relation_size(c.oid)) AS index_pretty
FROM pg_class c
JOIN pg_am am ON am.oid = c.relam
WHERE c.relname = 'task28_ivf_pqg100k_g8_n128_idx';

SELECT
  c.relname,
  pg_relation_size(c.oid) AS heap_bytes,
  pg_size_pretty(pg_relation_size(c.oid)) AS heap_pretty,
  pg_total_relation_size(c.oid) AS total_bytes,
  pg_size_pretty(pg_total_relation_size(c.oid)) AS total_pretty
FROM pg_class c
WHERE c.relname = 'task28_ivf_pqg100k_g8_n128_corpus';
