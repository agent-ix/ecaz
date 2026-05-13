SET enable_indexscan = off;
SET enable_bitmapscan = off;

SELECT 'remote_rows' AS metric, count(*)::text AS value
FROM phase12_tuple_measure_remote
UNION ALL
SELECT 'coordinator_rows', count(*)::text
FROM phase12_tuple_measure_corpus
UNION ALL
SELECT 'query_rows', count(*)::text
FROM phase12_tuple_measure_queries;

SELECT relname, reloptions
FROM pg_class
WHERE relname IN ('phase12_tuple_measure_remote_idx', 'phase12_tuple_measure_coord_idx')
ORDER BY relname;

SELECT *
FROM ec_spire_remote_node_snapshot('phase12_tuple_measure_coord_idx'::regclass)
ORDER BY node_id;

SELECT tuple_transport_default, tuple_transport_status, tuple_transport_capabilities
FROM ec_spire_remote_search_endpoint_identity('phase12_tuple_measure_coord_idx'::regclass);
