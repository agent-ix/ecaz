SET enable_seqscan = off;
SET enable_bitmapscan = off;
SET enable_sort = off;
SET ec_spire.nprobe = 8;
SET ec_spire.remote_tuple_transport = 'pg_binary_attr_v1';

SELECT tuple_transport_default, tuple_transport_status, tuple_transport_capabilities
FROM ec_spire_remote_search_endpoint_identity('phase12_tuple_measure_coord_idx'::regclass);

CREATE TEMP TABLE phase12_tuple_measure_metrics(
    transport text,
    qid integer,
    rows_returned integer,
    payload_bytes bigint,
    elapsed_ms double precision
);

DO $$
DECLARE
    q record;
    rows_returned integer;
    payload_bytes bigint;
    started timestamptz;
    elapsed_ms double precision;
BEGIN
    FOR q IN
        SELECT id, source
        FROM phase12_tuple_measure_queries
        ORDER BY id
        LIMIT 20
    LOOP
        started := clock_timestamp();
        EXECUTE format(
            'SELECT count(*), coalesce(sum(length(title) + length(body)), 0)
               FROM (
                 SELECT id, title, body
                 FROM phase12_tuple_measure_corpus
                 ORDER BY embedding <#> %L::real[]
                 LIMIT 10
               ) s',
            q.source::text
        )
        INTO rows_returned, payload_bytes;
        elapsed_ms := EXTRACT(epoch FROM clock_timestamp() - started) * 1000.0;
        INSERT INTO phase12_tuple_measure_metrics
        VALUES ('pg_binary_attr_v1', q.id, rows_returned, payload_bytes, elapsed_ms);
        RAISE NOTICE 'transport=pg_binary_attr_v1 qid=% rows=% payload_bytes=% elapsed_ms=%',
            q.id, rows_returned, payload_bytes, round(elapsed_ms::numeric, 3);
    END LOOP;
END $$;

SELECT
    transport,
    count(*) AS query_count,
    sum(rows_returned) AS rows_returned,
    sum(payload_bytes) AS payload_bytes,
    round(avg(elapsed_ms)::numeric, 3) AS avg_ms,
    round(percentile_cont(0.50) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p50_ms,
    round(percentile_cont(0.95) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p95_ms,
    round(percentile_cont(0.99) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p99_ms,
    round((count(*)::numeric / (sum(elapsed_ms) / 1000.0))::numeric, 3) AS queries_per_second
FROM phase12_tuple_measure_metrics
GROUP BY transport;
