\timing on

SELECT relname
FROM pg_class
WHERE relname LIKE 'task28_ivf%100k%'
   OR relname LIKE 'task28%100k%'
ORDER BY relname
LIMIT 50;

SELECT
  relname,
  reltuples::bigint AS estimated_rows
FROM pg_class
WHERE relkind = 'r'
  AND relname LIKE '%_corpus'
ORDER BY reltuples DESC, relname
LIMIT 80;
