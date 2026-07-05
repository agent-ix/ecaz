\pset pager off
\timing on

SET enable_seqscan = off;
SET ec_diskann.list_size = 200;

SELECT
current_setting('server_version') AS server_version,
current_setting('ec_diskann.list_size') AS sweep_value,
'ec_diskann' AS profile;

SELECT
'task47_gate_diskann_idx' AS index_name,
pg_relation_size('task47_gate_diskann_idx'::regclass) AS index_bytes,
pg_size_pretty(pg_relation_size('task47_gate_diskann_idx'::regclass)) AS index_size;

SELECT *
FROM ec_diskann_index_cost_snapshot('task47_gate_diskann_idx'::regclass);

EXPLAIN (FORMAT JSON, ecaz, ANALYZE, COSTS OFF)
SELECT id
FROM task47_gate_diskann_corpus
ORDER BY embedding <#> (
SELECT source
FROM task47_gate_diskann_queries
ORDER BY id
LIMIT 1
)::real[]
LIMIT 10;

RESET enable_seqscan;
RESET ec_diskann.list_size;
