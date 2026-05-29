CREATE EXTENSION IF NOT EXISTS ecaz;

DROP TABLE IF EXISTS task60_diskann_pq;
DROP TABLE IF EXISTS task60_diskann_rabitq;

CREATE TABLE task60_diskann_pq (
    id bigint PRIMARY KEY,
    embedding ecvector
);

CREATE TABLE task60_diskann_rabitq (
    id bigint PRIMARY KEY,
    embedding ecvector
);

INSERT INTO task60_diskann_pq VALUES
    (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42)),
    (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.0, 0.0], 4, 42)),
    (3, encode_to_ecvector(ARRAY[0.0, 0.0, 1.0, 0.0], 4, 42)),
    (4, encode_to_ecvector(ARRAY[0.0, 0.0, 0.0, 1.0], 4, 42)),
    (5, encode_to_ecvector(ARRAY[0.70710677, 0.70710677, 0.0, 0.0], 4, 42)),
    (6, encode_to_ecvector(ARRAY[0.70710677, 0.0, 0.70710677, 0.0], 4, 42));

INSERT INTO task60_diskann_rabitq SELECT * FROM task60_diskann_pq;

CREATE INDEX task60_diskann_pq_idx
    ON task60_diskann_pq USING ec_diskann (embedding ecvector_diskann_ip_ops)
    WITH (
        graph_degree = 4,
        build_list_size = 10,
        list_size = 64,
        storage_format = 'pq_fastscan'
    );

CREATE INDEX task60_diskann_rabitq_idx
    ON task60_diskann_rabitq USING ec_diskann (embedding ecvector_diskann_ip_ops)
    WITH (
        graph_degree = 4,
        build_list_size = 10,
        list_size = 64,
        storage_format = 'rabitq'
    );

SET enable_seqscan = off;
SET enable_bitmapscan = off;
SET enable_sort = off;

DO $$
DECLARE
    got bigint;
    line text;
    plan text := '';
BEGIN
    FOR line IN EXECUTE 'EXPLAIN (COSTS OFF) SELECT id FROM task60_diskann_pq ORDER BY embedding <#> ARRAY[1.0,0.0,0.0,0.0]::real[] LIMIT 3'
    LOOP
        plan := plan || E'\n' || line;
    END LOOP;
    IF position('Index Scan using task60_diskann_pq_idx' in plan) = 0 THEN
        RAISE EXCEPTION 'pq_fastscan plan did not use DiskANN index: %', plan;
    END IF;

    SELECT id INTO got
    FROM task60_diskann_pq
    ORDER BY embedding <#> ARRAY[1.0,0.0,0.0,0.0]::real[]
    LIMIT 1;
    IF got <> 1 THEN
        RAISE EXCEPTION 'pq_fastscan nearest id %, expected 1', got;
    END IF;

    plan := '';
    FOR line IN EXECUTE 'EXPLAIN (COSTS OFF) SELECT id FROM task60_diskann_rabitq ORDER BY embedding <#> ARRAY[1.0,0.0,0.0,0.0]::real[] LIMIT 3'
    LOOP
        plan := plan || E'\n' || line;
    END LOOP;
    IF position('Index Scan using task60_diskann_rabitq_idx' in plan) = 0 THEN
        RAISE EXCEPTION 'rabitq plan did not use DiskANN index: %', plan;
    END IF;

    SELECT id INTO got
    FROM task60_diskann_rabitq
    ORDER BY embedding <#> ARRAY[1.0,0.0,0.0,0.0]::real[]
    LIMIT 1;
    IF got <> 1 THEN
        RAISE EXCEPTION 'rabitq nearest id %, expected 1', got;
    END IF;
END
$$;

SELECT
    'task60_diskann_format_smoke_passed' AS status,
    pg_relation_size('task60_diskann_pq_idx'::regclass) AS pq_index_bytes,
    pg_relation_size('task60_diskann_rabitq_idx'::regclass) AS rabitq_index_bytes;
