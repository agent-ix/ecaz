\set ON_ERROR_STOP on

CREATE EXTENSION IF NOT EXISTS pg_prewarm;

SELECT relname,
       pg_size_pretty(pg_relation_size(oid)) AS size,
       pg_prewarm(oid) AS pages_loaded
FROM pg_class
WHERE relname LIKE 'real\_%' ESCAPE '\'
  AND relkind IN ('r', 'i')
ORDER BY relname;
