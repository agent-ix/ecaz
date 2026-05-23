#!/usr/bin/env bash
set -euo pipefail

ROOT=/home/peter/dev/ecaz
RUN_DIR="$ROOT/target/task51-local-pg18-bound-prune-smoke-$(date +%s)"
DATA="$RUN_DIR/data"
SOCK="$RUN_DIR/socket"
LOG="$RUN_DIR/postgres.log"
PORT=$((29000 + ($$ % 1000)))
PG_CTL=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl
PSQL=/home/peter/.pgrx/18.3/pgrx-install/bin/psql

mkdir -p "$SOCK"
"$PG_CTL" initdb -D "$DATA" -o "-A trust -U postgres" >/dev/null

cleanup() {
    "$PG_CTL" -D "$DATA" -m fast stop >/dev/null 2>&1 || true
}
trap cleanup EXIT

"$PG_CTL" -w -D "$DATA" -l "$LOG" \
    -o "-p $PORT -k $SOCK -c listen_addresses='' -c shared_preload_libraries=ecaz" \
    start >/dev/null

"$PSQL" -h "$SOCK" -p "$PORT" -U postgres -d postgres -v ON_ERROR_STOP=1 <<'SQL'
SHOW shared_preload_libraries;
CREATE EXTENSION ecaz;

CREATE TABLE task51_ivf_bound_prune_smoke (
    id bigint primary key,
    embedding ecvector
);

INSERT INTO task51_ivf_bound_prune_smoke
SELECT
    g,
    ARRAY[((256 - g)::real / 256.0), ((g % 17)::real / 1000.0)]::real[]::ecvector
FROM generate_series(1, 256) AS g;

CREATE INDEX task51_ivf_bound_prune_off_idx
    ON task51_ivf_bound_prune_smoke USING ec_ivf (embedding ecvector_ip_ops)
    WITH (
        nlists = 1,
        nprobe = 1,
        training_sample_rows = 256,
        storage_format = 'rabitq',
        quant_bits = 4,
        rerank = 'off',
        rerank_width = 250
    );

SET enable_seqscan = off;
SET ec_ivf.nprobe = 1;
SET ec_ivf.rerank_width = 250;

\echo === no-rerank-rabitq-limit-220 ===
EXPLAIN (ecaz, ANALYZE, COSTS OFF, VERBOSE)
    SELECT id
    FROM task51_ivf_bound_prune_smoke
    ORDER BY embedding <#> ARRAY[1.0,0.0]::real[]
    LIMIT 220;

SELECT count(*) AS no_rerank_limit_220_count
FROM (
    SELECT id
    FROM task51_ivf_bound_prune_smoke
    ORDER BY embedding <#> ARRAY[1.0,0.0]::real[]
    LIMIT 220
) AS q;

DROP INDEX task51_ivf_bound_prune_off_idx;

CREATE INDEX task51_ivf_bound_prune_heap_idx
    ON task51_ivf_bound_prune_smoke USING ec_ivf (embedding ecvector_ip_ops)
    WITH (
        nlists = 1,
        nprobe = 1,
        training_sample_rows = 256,
        storage_format = 'rabitq',
        quant_bits = 4,
        rerank = 'heap_f32',
        rerank_width = 3
    );

SET ec_ivf.rerank_width = 3;

\echo === heap-f32-rabitq-limit-3 ===
EXPLAIN (ecaz, ANALYZE, COSTS OFF, VERBOSE)
    SELECT id
    FROM task51_ivf_bound_prune_smoke
    ORDER BY embedding <#> ARRAY[1.0,0.0]::real[]
    LIMIT 3;

SELECT count(*) AS heap_f32_limit_3_count
FROM (
    SELECT id
    FROM task51_ivf_bound_prune_smoke
    ORDER BY embedding <#> ARRAY[1.0,0.0]::real[]
    LIMIT 3
) AS q;
SQL

echo "run_dir=$RUN_DIR"
