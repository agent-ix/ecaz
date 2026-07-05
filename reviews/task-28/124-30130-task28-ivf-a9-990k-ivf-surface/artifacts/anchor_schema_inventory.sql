\pset pager off
\timing on

SELECT current_setting('server_version') AS server_version;

SELECT c.relname AS table_name,
       c.reltuples::bigint AS estimated_rows,
       pg_relation_size(c.oid) AS table_bytes,
       pg_size_pretty(pg_relation_size(c.oid)) AS table_pretty
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE n.nspname = 'public'
  AND c.relkind = 'r'
  AND c.relname LIKE 'ec_hnsw_real_ann_benchmarks_anchor%'
ORDER BY c.relname;

SELECT a.attrelid::regclass AS table_name,
       a.attnum,
       a.attname,
       format_type(a.atttypid, a.atttypmod) AS data_type,
       a.attnotnull
FROM pg_attribute a
WHERE a.attrelid::regclass::text LIKE 'ec_hnsw_real_ann_benchmarks_anchor%'
  AND a.attnum > 0
  AND NOT a.attisdropped
ORDER BY a.attrelid::regclass::text, a.attnum;

SELECT i.indrelid::regclass AS table_name,
       ci.relname AS index_name,
       pg_relation_size(ci.oid) AS index_bytes,
       pg_size_pretty(pg_relation_size(ci.oid)) AS index_pretty,
       pg_get_indexdef(i.indexrelid) AS indexdef
FROM pg_index i
JOIN pg_class ci ON ci.oid = i.indexrelid
WHERE i.indrelid::regclass::text LIKE 'ec_hnsw_real_ann_benchmarks_anchor%'
ORDER BY i.indrelid::regclass::text, ci.relname;
