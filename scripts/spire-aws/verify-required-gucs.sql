-- Phase 13b.6 / 13b.10.d — verify every load-bearing PG / SPIRE GUC.
-- Operator invokes via `ecaz dev sql --file ...` against each node.

\echo === PostgreSQL version ===
SELECT current_setting('server_version') AS server_version;

\echo === Prepared-xact capacity (must be >= 64 per Phase 13a.1.b) ===
SELECT current_setting('max_prepared_transactions') AS max_prepared_transactions;

\echo === SPIRE GUCs (consistency / transport / payload caps) ===
SELECT name, setting, unit
FROM pg_settings
WHERE name LIKE 'ec_spire.%'
ORDER BY name;

\echo === Buffer / work memory (Phase 13a.1.b) ===
SELECT name, setting, unit
FROM pg_settings
WHERE name IN ('shared_buffers', 'work_mem', 'maintenance_work_mem');

\echo === ecaz extension version ===
SELECT extname, extversion FROM pg_extension WHERE extname = 'ecaz';
