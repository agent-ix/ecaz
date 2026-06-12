-- Task 99 profile — LANE-LOCAL source mapping (Intel desktop, postgres DB).
-- Creates the canonical t99 source pair from this lane's real-DBpedia 100k
-- fixture. The shared t99-fixtures.sql replicates per-variant tables from
-- these two tables, so the per-variant SQL and the SuiteConfig stay
-- byte-identical across lanes (local / Graviton 4 / AWS Intel).
--
-- On the AWS lanes, write the equivalent two statements against the
-- corpus tables present in the snapshot restore (discover with
-- `ecaz corpus list`), commit that file to the trip packet, and run it
-- instead of this one. Embeddings are raw f32 in the ecvector container
-- (verified byte-identical across AM profiles), so any profile's corpus
-- table is a valid source.

CREATE TABLE t99_src_100k_corpus AS
SELECT id, source, embedding
FROM current_intel_real100k_hnsw_corpus;

ALTER TABLE t99_src_100k_corpus ADD PRIMARY KEY (id);

CREATE TABLE t99_src_100k_queries AS
SELECT id, source
FROM current_intel_real100k_hnsw_queries;

ANALYZE t99_src_100k_corpus;
ANALYZE t99_src_100k_queries;
