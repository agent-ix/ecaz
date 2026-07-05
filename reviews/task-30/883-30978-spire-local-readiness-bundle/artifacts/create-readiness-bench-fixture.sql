CREATE EXTENSION ecaz;

CREATE TABLE phase12_ready_corpus (
    id bigint PRIMARY KEY,
    source real[] NOT NULL,
    embedding ecvector NOT NULL,
    title text NOT NULL,
    body text NOT NULL
);

WITH generated AS (
    SELECT
        i,
        ARRAY[
            sin(i::real * 0.11),
            cos(i::real * 0.07),
            sin(i::real * 0.13 + 0.5),
            cos(i::real * 0.17 + 0.25),
            sin(i::real * 0.19 + 0.75),
            cos(i::real * 0.23 + 1.0),
            sin(i::real * 0.29 + 1.25),
            cos(i::real * 0.31 + 1.5)
        ]::real[] AS source
    FROM generate_series(1, 600) AS i
)
INSERT INTO phase12_ready_corpus (id, source, embedding, title, body)
SELECT
    i,
    source,
    encode_to_ecvector(source, 4, 42),
    format('ready-title-%s', i),
    repeat(format('ready-body-%s ', i), 4)
FROM generated;

CREATE TABLE phase12_ready_queries (
    id bigint PRIMARY KEY,
    source real[] NOT NULL
);

INSERT INTO phase12_ready_queries (id, source)
SELECT row_number() OVER (ORDER BY id), source
FROM phase12_ready_corpus
WHERE id % 37 = 0
ORDER BY id
LIMIT 12;

CREATE INDEX phase12_ready_idx
ON phase12_ready_corpus
USING ec_spire (embedding ecvector_spire_ip_ops)
WITH (nlists = 1, nprobe = 1, rerank_width = 0, storage_format = 'rabitq');

SELECT 'corpus_rows' AS metric, count(*)::text AS value
FROM phase12_ready_corpus
UNION ALL
SELECT 'query_rows', count(*)::text
FROM phase12_ready_queries
UNION ALL
SELECT 'index_reloptions', reloptions::text
FROM pg_class
WHERE relname = 'phase12_ready_idx';
