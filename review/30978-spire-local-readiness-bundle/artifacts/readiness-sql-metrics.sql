SET enable_seqscan = off;
SET enable_bitmapscan = off;
SET enable_sort = off;
SET ec_spire.nprobe = 1;
SET ec_spire.rerank_width = 0;
SET ec_spire.max_candidate_rows = 0;
SET ec_spire.remote_tuple_transport = 'pg_binary_attr_v1';

SELECT 'capacity_remote_search_max_nodes' AS metric, '8' AS value
UNION ALL SELECT 'capacity_remote_search_max_pids', '256'
UNION ALL SELECT 'capacity_remote_search_max_pids_per_node', '64'
UNION ALL SELECT 'capacity_remote_search_max_concurrent_dispatches', '8'
UNION ALL SELECT 'capacity_remote_search_max_concurrent_dispatches_per_node', '1';

SELECT
    tuple_transport_default,
    tuple_transport_status,
    tuple_transport_capabilities,
    (tuple_transport_status = 'ready'
     AND tuple_transport_capabilities @> ARRAY['pg_binary_attr_v1']) AS pg_binary_attr_v1_ready
FROM ec_spire_remote_search_endpoint_identity('phase12_ready_idx'::regclass);

SELECT 'corpus_rows' AS metric, count(*)::text AS value
FROM phase12_ready_corpus
UNION ALL
SELECT 'query_rows', count(*)::text
FROM phase12_ready_queries;

CREATE TEMP TABLE phase12_ready_metrics(
    qid bigint,
    rows_returned integer,
    recall_hits integer,
    recall_at_10 double precision,
    payload_bytes bigint,
    elapsed_ms double precision
);

DO $$
DECLARE
    q record;
    predicted_ids bigint[];
    exact_ids bigint[];
    rows_returned integer;
    recall_hits integer;
    payload_bytes bigint;
    started timestamptz;
    elapsed_ms double precision;
BEGIN
    FOR q IN
        SELECT id, source
        FROM phase12_ready_queries
        ORDER BY id
    LOOP
        PERFORM set_config('enable_seqscan', 'off', true);
        PERFORM set_config('enable_bitmapscan', 'off', true);
        PERFORM set_config('enable_sort', 'off', true);
        started := clock_timestamp();
        SELECT array_agg(id ORDER BY rank), count(*), coalesce(sum(length(title) + length(body)), 0)
        INTO predicted_ids, rows_returned, payload_bytes
        FROM (
            SELECT id, title, body, row_number() OVER () AS rank
            FROM phase12_ready_corpus
            ORDER BY embedding <#> q.source
            LIMIT 10
        ) predicted;
        elapsed_ms := EXTRACT(epoch FROM clock_timestamp() - started) * 1000.0;

        PERFORM set_config('enable_seqscan', 'on', true);
        PERFORM set_config('enable_indexscan', 'off', true);
        PERFORM set_config('enable_bitmapscan', 'off', true);
        PERFORM set_config('enable_sort', 'on', true);
        SELECT array_agg(id ORDER BY rank)
        INTO exact_ids
        FROM (
            SELECT id, row_number() OVER () AS rank
            FROM phase12_ready_corpus
            ORDER BY embedding <#> q.source
            LIMIT 10
        ) exact;

        SELECT count(*)::integer
        INTO recall_hits
        FROM unnest(predicted_ids) AS predicted_id
        WHERE predicted_id = ANY(exact_ids);

        INSERT INTO phase12_ready_metrics
        VALUES (
            q.id,
            rows_returned,
            recall_hits,
            recall_hits::double precision / 10.0,
            payload_bytes,
            elapsed_ms
        );

        RAISE NOTICE 'qid=% rows=% recall_hits=% payload_bytes=% elapsed_ms=%',
            q.id, rows_returned, recall_hits, payload_bytes, round(elapsed_ms::numeric, 3);
    END LOOP;
END $$;

SELECT
    count(*) AS query_count,
    sum(rows_returned) AS rows_returned,
    round(avg(recall_at_10)::numeric, 4) AS recall_at_10,
    sum(payload_bytes) AS payload_bytes,
    round(avg(elapsed_ms)::numeric, 3) AS avg_ms,
    round(percentile_cont(0.50) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p50_ms,
    round(percentile_cont(0.95) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p95_ms,
    round(percentile_cont(0.99) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p99_ms
FROM phase12_ready_metrics;

SELECT 'pipeline' AS section, *
FROM ec_spire_index_scan_pipeline_snapshot(
    'phase12_ready_idx'::regclass,
    (SELECT source FROM phase12_ready_queries ORDER BY id LIMIT 1)
);

SELECT 'routing' AS section, *
FROM ec_spire_index_scan_routing_snapshot(
    'phase12_ready_idx'::regclass,
    (SELECT source FROM phase12_ready_queries ORDER BY id LIMIT 1)
);

SELECT 'local_store_overlap' AS section, *
FROM ec_spire_index_scan_local_store_read_overlap_harness(
    'phase12_ready_idx'::regclass,
    (SELECT source FROM phase12_ready_queries ORDER BY id LIMIT 1)
);
