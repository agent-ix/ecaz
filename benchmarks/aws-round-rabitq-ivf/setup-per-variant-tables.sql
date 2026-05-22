-- Phase A per-variant table fixtures.
--
-- snap-054feaffc50ecf1c9 ships `ec_hnsw_real_10k_corpus`, `_queries`, and
-- `ec_hnsw_real_50k_corpus`, `_queries` with a default `ec_ivf` (TurboQuant)
-- index on the corpus tables. We need isolated per-AM × per-storage_format
-- corpus tables per ADR-050 so the planner can't pick across variants.
--
-- This script replicates the corpus rows once per (am, storage_format)
-- variant on the existing data, builds the right index, and leaves the
-- queries table shared (queries are read-only and identical across runs).
--
-- Idempotent: each block is wrapped in IF NOT EXISTS / DROP IF EXISTS.

\set ON_ERROR_STOP on
\timing on

-- Shared queries get cloned per-variant so `ecaz bench` finds
-- `{prefix}_queries` via its naming convention. Queries are tiny
-- (200 / 1000 rows) so the duplication cost is negligible.

DROP TABLE IF EXISTS real_10k_queries_shared;
CREATE TABLE real_10k_queries_shared AS TABLE ec_hnsw_real_10k_queries;

DROP TABLE IF EXISTS real_50k_queries_shared;
CREATE TABLE real_50k_queries_shared AS TABLE ec_hnsw_real_50k_queries;

-- 10k variants ----------------------------------------------------------

-- IVF TurboQuant variant (mirror the existing default-index corpus into
-- a typed sibling so the per-AM table rule holds even for the baseline).
DROP TABLE IF EXISTS real_10k_ivf_tq_corpus;
CREATE TABLE real_10k_ivf_tq_corpus AS TABLE ec_hnsw_real_10k_corpus;
DROP TABLE IF EXISTS real_10k_ivf_tq_queries;
CREATE TABLE real_10k_ivf_tq_queries AS TABLE real_10k_queries_shared;
CREATE INDEX real_10k_ivf_tq_idx ON real_10k_ivf_tq_corpus
    USING ec_ivf (embedding ecvector_ip_ops);

DROP TABLE IF EXISTS real_10k_ivf_rabitq_corpus;
CREATE TABLE real_10k_ivf_rabitq_corpus AS TABLE ec_hnsw_real_10k_corpus;
DROP TABLE IF EXISTS real_10k_ivf_rabitq_queries;
CREATE TABLE real_10k_ivf_rabitq_queries AS TABLE real_10k_queries_shared;
CREATE INDEX real_10k_ivf_rabitq_idx ON real_10k_ivf_rabitq_corpus
    USING ec_ivf (embedding ecvector_ip_ops)
    WITH (storage_format = 'rabitq');

DROP TABLE IF EXISTS real_10k_ivf_pqfs_corpus;
CREATE TABLE real_10k_ivf_pqfs_corpus AS TABLE ec_hnsw_real_10k_corpus;
DROP TABLE IF EXISTS real_10k_ivf_pqfs_queries;
CREATE TABLE real_10k_ivf_pqfs_queries AS TABLE real_10k_queries_shared;
CREATE INDEX real_10k_ivf_pqfs_idx ON real_10k_ivf_pqfs_corpus
    USING ec_ivf (embedding ecvector_ip_ops)
    WITH (storage_format = 'pq_fastscan');

-- 50k variants ----------------------------------------------------------

DROP TABLE IF EXISTS real_50k_ivf_tq_corpus;
CREATE TABLE real_50k_ivf_tq_corpus AS TABLE ec_hnsw_real_50k_corpus;
DROP TABLE IF EXISTS real_50k_ivf_tq_queries;
CREATE TABLE real_50k_ivf_tq_queries AS TABLE real_50k_queries_shared;
CREATE INDEX real_50k_ivf_tq_idx ON real_50k_ivf_tq_corpus
    USING ec_ivf (embedding ecvector_ip_ops);

DROP TABLE IF EXISTS real_50k_ivf_rabitq_corpus;
CREATE TABLE real_50k_ivf_rabitq_corpus AS TABLE ec_hnsw_real_50k_corpus;
DROP TABLE IF EXISTS real_50k_ivf_rabitq_queries;
CREATE TABLE real_50k_ivf_rabitq_queries AS TABLE real_50k_queries_shared;
CREATE INDEX real_50k_ivf_rabitq_idx ON real_50k_ivf_rabitq_corpus
    USING ec_ivf (embedding ecvector_ip_ops)
    WITH (storage_format = 'rabitq');

DROP TABLE IF EXISTS real_50k_ivf_pqfs_corpus;
CREATE TABLE real_50k_ivf_pqfs_corpus AS TABLE ec_hnsw_real_50k_corpus;
DROP TABLE IF EXISTS real_50k_ivf_pqfs_queries;
CREATE TABLE real_50k_ivf_pqfs_queries AS TABLE real_50k_queries_shared;
CREATE INDEX real_50k_ivf_pqfs_idx ON real_50k_ivf_pqfs_corpus
    USING ec_ivf (embedding ecvector_ip_ops)
    WITH (storage_format = 'pq_fastscan');

-- Sanity check ----------------------------------------------------------

SELECT relname,
       pg_size_pretty(pg_relation_size(oid)) AS size,
       reltuples::bigint AS rows
FROM pg_class
WHERE relname LIKE 'real_%_corpus' OR relname LIKE 'real_%_idx'
ORDER BY relname;
