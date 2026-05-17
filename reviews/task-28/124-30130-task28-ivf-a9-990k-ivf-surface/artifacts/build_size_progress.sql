\pset pager off
SELECT c.relname,
       c.reltuples::bigint AS estimated_rows,
       pg_relation_size(c.oid) AS relation_bytes,
       pg_total_relation_size(c.oid) AS total_bytes
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE n.nspname = 'public'
  AND c.relname LIKE 'task28_ivf_pqg990k_g8_n128%'
ORDER BY c.relname;
