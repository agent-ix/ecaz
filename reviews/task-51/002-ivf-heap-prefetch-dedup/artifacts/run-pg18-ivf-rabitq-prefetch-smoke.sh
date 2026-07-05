#!/usr/bin/env bash
set -euo pipefail

ROOT=/home/peter/dev/ecaz
RUN_DIR="$ROOT/target/task51-local-pg18-prefetch-smoke-$(date +%s)"
DATA="$RUN_DIR/data"
SOCK="$RUN_DIR/socket"
LOG="$RUN_DIR/postgres.log"
PG_CTL=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl
PSQL=/home/peter/.pgrx/18.3/pgrx-install/bin/psql

mkdir -p "$SOCK"
"$PG_CTL" initdb -D "$DATA" -o "-A trust -U postgres" >/dev/null

cleanup() {
    "$PG_CTL" -D "$DATA" -m fast stop >/dev/null 2>&1 || true
}
trap cleanup EXIT

"$PG_CTL" -w -D "$DATA" -l "$LOG" \
    -o "-p 28952 -k $SOCK -c listen_addresses='' -c shared_preload_libraries=ecaz" \
    start >/dev/null

"$PSQL" -h "$SOCK" -p 28952 -U postgres -d postgres -v ON_ERROR_STOP=1 \
    -c "SHOW shared_preload_libraries" \
    -c "CREATE EXTENSION ecaz" \
    -c "CREATE TABLE task51_ivf_rabitq_prefetch_smoke (id bigint primary key, embedding ecvector)" \
    -c "INSERT INTO task51_ivf_rabitq_prefetch_smoke VALUES
        (0, '[1.0,0.0]'::ecvector),
        (1, '[0.8,0.1]'::ecvector),
        (2, '[0.0,1.0]'::ecvector),
        (3, '[-1.0,0.0]'::ecvector),
        (4, '[0.9,0.2]'::ecvector),
        (5, '[0.2,0.9]'::ecvector)" \
    -c "CREATE INDEX task51_ivf_rabitq_prefetch_smoke_idx
        ON task51_ivf_rabitq_prefetch_smoke USING ec_ivf (embedding ecvector_ip_ops)
        WITH (
            nlists = 2,
            nprobe = 2,
            training_sample_rows = 6,
            storage_format = 'rabitq',
            rerank = 'heap_f32',
            rerank_width = 3
        )" \
    -c "SET enable_seqscan = off" \
    -c "SET ec_ivf.nprobe = 2" \
    -c "SET ec_ivf.rerank_width = 3" \
    -c "EXPLAIN (ecaz, ANALYZE, COSTS OFF, VERBOSE)
        SELECT id
        FROM task51_ivf_rabitq_prefetch_smoke
        ORDER BY embedding <#> ARRAY[1.0,0.0]::real[]
        LIMIT 3"

echo "run_dir=$RUN_DIR"
