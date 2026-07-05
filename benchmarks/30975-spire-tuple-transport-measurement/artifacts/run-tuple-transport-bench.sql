SET enable_seqscan = off;
SET enable_indexscan = off;

DROP TABLE IF EXISTS phase12_tuple_transport_bench_results;
CREATE TEMP TABLE phase12_tuple_transport_bench_results (
    mode text NOT NULL,
    query_id bigint NOT NULL,
    elapsed_ms double precision NOT NULL,
    row_count integer NOT NULL,
    payload_bytes integer NOT NULL
);

DO $$
DECLARE
    mode text;
    q record;
    row record;
    started timestamptz;
    row_count integer;
    payload_bytes integer;
BEGIN
    FOREACH mode IN ARRAY ARRAY['json_tuple_payload_v1', 'pg_binary_attr_v1'] LOOP
        EXECUTE format('SET ec_spire.remote_tuple_transport = %L', mode);
        FOR q IN
            SELECT id, source
              FROM phase12_tuple_measure_queries
             ORDER BY id
             LIMIT 20
        LOOP
            started := clock_timestamp();
            row_count := 0;
            payload_bytes := 0;
            FOR row IN EXECUTE
                'SELECT id, title, body
                   FROM phase12_tuple_measure_corpus
                  ORDER BY embedding <#> $1::real[]
                  LIMIT 10'
                USING q.source
            LOOP
                row_count := row_count + 1;
                payload_bytes := payload_bytes
                    + octet_length(row.title)
                    + octet_length(row.body);
            END LOOP;
            INSERT INTO phase12_tuple_transport_bench_results
            VALUES (
                mode,
                q.id,
                EXTRACT(EPOCH FROM clock_timestamp() - started) * 1000.0,
                row_count,
                payload_bytes
            );
        END LOOP;
    END LOOP;
END $$;

SELECT mode,
       count(*) AS query_count,
       min(row_count) AS min_rows,
       max(row_count) AS max_rows,
       sum(payload_bytes) AS payload_bytes,
       round(avg(elapsed_ms)::numeric, 3) AS avg_ms,
       round(percentile_cont(0.50) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p50_ms,
       round(percentile_cont(0.95) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p95_ms,
       round(percentile_cont(0.99) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p99_ms,
       round((1000.0 / avg(elapsed_ms))::numeric, 3) AS queries_per_second
  FROM phase12_tuple_transport_bench_results
 GROUP BY mode
 ORDER BY mode;
