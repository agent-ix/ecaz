#!/usr/bin/env bash
# Storage accounting for tqvector NFR-002.
# Requires: running PostgreSQL with bench_encoded table and bench_idx index.
# Usage: PGDATABASE=tqvector_bench bash scripts/bench_storage.sh
set -euo pipefail

PGDATABASE="${PGDATABASE:-tqvector_bench}"

echo "=== tqvector Storage Accounting ==="
psql "$PGDATABASE" -c "
SELECT
  (SELECT count(*) FROM bench_encoded) AS row_count,
  pg_size_pretty(pg_relation_size('bench_encoded'::regclass)) AS table_size,
  pg_size_pretty(pg_relation_size('bench_idx'::regclass)) AS index_size,
  pg_size_pretty(pg_total_relation_size('bench_encoded'::regclass)) AS total_size,
  pg_relation_size('bench_encoded'::regclass) AS table_bytes,
  pg_relation_size('bench_idx'::regclass) AS index_bytes,
  pg_total_relation_size('bench_encoded'::regclass) AS total_bytes;
"

echo ""
echo "=== Per-Vector Stats ==="
psql "$PGDATABASE" -c "
SELECT
  avg(pg_column_size(vec))::numeric(10,1) AS avg_datum_bytes,
  min(pg_column_size(vec)) AS min_datum_bytes,
  max(pg_column_size(vec)) AS max_datum_bytes;
FROM bench_encoded;
"
