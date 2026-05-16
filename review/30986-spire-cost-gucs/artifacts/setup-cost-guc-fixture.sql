DROP TABLE IF EXISTS phase12a_cost_guc_corpus, phase12a_cost_guc_queries;

CREATE TABLE phase12a_cost_guc_corpus(
    id bigint primary key,
    source real[],
    embedding ecvector
);

INSERT INTO phase12a_cost_guc_corpus
VALUES
    (1, ARRAY[1.0, 0.0]::real[], encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
    (2, ARRAY[0.0, 1.0]::real[], encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)),
    (3, ARRAY[-1.0, 0.0]::real[], encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)),
    (4, ARRAY[0.0, -1.0]::real[], encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42));

CREATE TABLE phase12a_cost_guc_queries(
    id bigint primary key,
    source real[]
);

INSERT INTO phase12a_cost_guc_queries
VALUES (1, ARRAY[1.0, 0.0]::real[]);

CREATE INDEX phase12a_cost_guc_idx
    ON phase12a_cost_guc_corpus
    USING ec_spire (embedding ecvector_spire_ip_ops)
    WITH (nlists = 2, rerank_width = 0);

ANALYZE phase12a_cost_guc_corpus;
