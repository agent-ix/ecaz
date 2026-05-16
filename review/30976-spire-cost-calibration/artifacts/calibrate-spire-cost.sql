SET enable_seqscan = off;
SET enable_bitmapscan = off;
SET enable_sort = off;
SET ec_spire.remote_tuple_transport = 'pg_binary_attr_v1';

CREATE TEMP TABLE spire_cost_calibration_metrics(
    projection text,
    nprobe integer,
    k integer,
    qid integer,
    rows_returned integer,
    payload_bytes bigint,
    elapsed_ms double precision
);

CREATE TEMP TABLE spire_cost_calibration_costs(
    nprobe integer,
    dimensions integer,
    active_leaf_count bigint,
    local_store_count integer,
    recursive_fanout integer,
    estimated_routing_scores bigint,
    estimated_selected_leaves bigint,
    estimated_candidate_rows double precision,
    estimated_routing_pages double precision,
    estimated_leaf_pages double precision,
    storage_format text,
    effective_rerank_width integer,
    index_pages double precision,
    reltuples double precision,
    modeled_startup_cost double precision,
    modeled_total_cost double precision
);

DO $$
DECLARE
    nprobe_value integer;
    limit_value integer;
    projection_name text;
    projection_list text;
    payload_expr text;
    q record;
    rows_returned integer;
    payload_bytes bigint;
    started timestamptz;
    elapsed_ms double precision;
BEGIN
    FOREACH nprobe_value IN ARRAY ARRAY[1, 4, 8, 16]
    LOOP
        EXECUTE format('SET ec_spire.nprobe = %s', nprobe_value);
        INSERT INTO spire_cost_calibration_costs
        SELECT
            nprobe_value,
            dimensions,
            active_leaf_count,
            local_store_count,
            recursive_fanout,
            estimated_routing_scores,
            estimated_selected_leaves,
            estimated_candidate_rows,
            estimated_routing_pages,
            estimated_leaf_pages,
            storage_format,
            effective_rerank_width,
            index_pages,
            reltuples,
            modeled_startup_cost,
            modeled_total_cost
        FROM ec_spire_index_cost_snapshot('phase12_tuple_measure_coord_idx'::regclass);

        FOREACH limit_value IN ARRAY ARRAY[10, 50, 100]
        LOOP
            FOREACH projection_name IN ARRAY ARRAY['id_only', 'title_body']
            LOOP
                IF projection_name = 'id_only' THEN
                    projection_list := 'id';
                    payload_expr := '0';
                ELSE
                    projection_list := 'id, title, body';
                    payload_expr := 'length(title) + length(body)';
                END IF;

                FOR q IN
                    SELECT id, source
                    FROM phase12_tuple_measure_queries
                    ORDER BY id
                    LIMIT 20
                LOOP
                    started := clock_timestamp();
                    EXECUTE format(
                        'SELECT count(*), coalesce(sum(%s), 0)
                           FROM (
                             SELECT %s
                             FROM phase12_tuple_measure_corpus
                             ORDER BY embedding <#> %L::real[]
                             LIMIT %s
                           ) s',
                        payload_expr,
                        projection_list,
                        q.source::text,
                        limit_value
                    )
                    INTO rows_returned, payload_bytes;
                    elapsed_ms := EXTRACT(epoch FROM clock_timestamp() - started) * 1000.0;
                    INSERT INTO spire_cost_calibration_metrics
                    VALUES (
                        projection_name,
                        nprobe_value,
                        limit_value,
                        q.id,
                        rows_returned,
                        payload_bytes,
                        elapsed_ms
                    );
                END LOOP;
            END LOOP;
        END LOOP;
    END LOOP;
END $$;

SELECT
    projection,
    nprobe,
    k,
    count(*) AS query_count,
    sum(rows_returned) AS rows_returned,
    sum(payload_bytes) AS payload_bytes,
    round(avg(elapsed_ms)::numeric, 3) AS avg_ms,
    round(percentile_cont(0.50) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p50_ms,
    round(percentile_cont(0.95) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p95_ms,
    round(percentile_cont(0.99) WITHIN GROUP (ORDER BY elapsed_ms)::numeric, 3) AS p99_ms,
    round((count(*)::numeric / (sum(elapsed_ms) / 1000.0))::numeric, 3) AS qps
FROM spire_cost_calibration_metrics
GROUP BY projection, nprobe, k
ORDER BY projection, k, nprobe;

SELECT *
FROM spire_cost_calibration_costs
ORDER BY nprobe;
