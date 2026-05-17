#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
INITDB="${INITDB:-$PGBIN/initdb}"
PG_UPGRADE="${PG_UPGRADE:-$PGBIN/pg_upgrade}"
PG_AMCHECK="${PG_AMCHECK:-$PGBIN/pg_amcheck}"
OLD_PORT="${OLD_PORT:-39420}"
NEW_PORT="${NEW_PORT:-39421}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
LOG_DIR_OVERRIDE="${LOG_DIR:-}"
SMOKE_LOG="${SMOKE_LOG:-}"
ARTIFACT_DIR=""

usage() {
  cat <<'USAGE'
Usage: scripts/run_pg_upgrade_smoke_pg18.sh [options]

Options:
  --artifact-dir DIR  Store smoke and PostgreSQL logs in DIR.
  --log-dir DIR       Store PostgreSQL logs in DIR.
  --new-port PORT     Upgraded PostgreSQL port. Default: 39421.
  --old-port PORT     Source PostgreSQL port. Default: 39420.
  --pgbin DIR         PostgreSQL bin directory. Default: $PGBIN.
  --run-dir DIR       Run directory. Default: target/pg-upgrade-smoke-pg18-$RUN_ID.
  --run-id ID         Run id used in the default run directory.
  --skip-install      Skip cargo pgrx install.
  --smoke-log FILE    Tee smoke output to FILE.
  -h, --help          Show this help.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir)
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    --log-dir)
      LOG_DIR_OVERRIDE="$2"
      shift 2
      ;;
    --new-port)
      NEW_PORT="$2"
      shift 2
      ;;
    --old-port)
      OLD_PORT="$2"
      shift 2
      ;;
    --pgbin)
      PGBIN="$2"
      PG_CTL="$PGBIN/pg_ctl"
      PSQL="$PGBIN/psql"
      INITDB="$PGBIN/initdb"
      PG_UPGRADE="$PGBIN/pg_upgrade"
      PG_AMCHECK="$PGBIN/pg_amcheck"
      shift 2
      ;;
    --run-dir)
      RUN_DIR_OVERRIDE="$2"
      shift 2
      ;;
    --run-id)
      RUN_ID="$2"
      shift 2
      ;;
    --skip-install)
      ECAZ_SKIP_INSTALL=1
      shift
      ;;
    --smoke-log)
      SMOKE_LOG="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/pg-upgrade-smoke-pg18-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/pg-upgrade-smoke-success.log}"
else
  LOG_DIR="${LOG_DIR_OVERRIDE:-$RUN_DIR/logs}"
fi
OLD_DATA="$RUN_DIR/old"
NEW_DATA="$RUN_DIR/new"
SOCKET_KEY="$(printf '%s' "$RUN_DIR" | cksum | awk '{print $1}')"
SOCKET_DIR="${SOCKET_DIR:-$ROOT_DIR/target/s-pg-upgrade-$SOCKET_KEY}"

if [[ -n "$SMOKE_LOG" && "${ECAZ_PG_UPGRADE_SMOKE_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_PG_UPGRADE_SMOKE_LOG_ACTIVE=1
  exec > >(tee "$SMOKE_LOG") 2>&1
fi

if [[ -e "$RUN_DIR" ]]; then
  echo "RUN_DIR already exists: $RUN_DIR" >&2
  exit 2
fi

mkdir -p "$LOG_DIR" "$SOCKET_DIR"
: > "$LOG_DIR/old-postgres.log"
: > "$LOG_DIR/new-postgres.log"

cleanup() {
  "$PG_CTL" -D "$NEW_DATA" -m fast stop >/dev/null 2>&1 || true
  "$PG_CTL" -D "$OLD_DATA" -m fast stop >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "run_dir=$RUN_DIR"
echo "socket_dir=$SOCKET_DIR"
echo "old_port=$OLD_PORT"
echo "new_port=$NEW_PORT"
echo "pgbin=$PGBIN"

if [[ "${ECAZ_SKIP_INSTALL:-0}" != "1" ]]; then
  (cd "$ROOT_DIR" && cargo pgrx install --test --pg-config "$PGBIN/pg_config" \
    --features "pg18 pg_test" --no-default-features)
fi

"$INITDB" -D "$OLD_DATA" -A trust -U postgres >/dev/null
"$INITDB" -D "$NEW_DATA" -A trust -U postgres >/dev/null

"$PG_CTL" -w -D "$OLD_DATA" -l "$LOG_DIR/old-postgres.log" \
  -o "-p $OLD_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null

old_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$OLD_PORT" -U postgres -d postgres)

"${old_psql[@]}" <<'SQL' >/dev/null
CREATE EXTENSION ecaz;
CREATE TABLE ecaz_pg_upgrade_smoke (
    id bigint primary key,
    embedding ecvector(4)
);
INSERT INTO ecaz_pg_upgrade_smoke (id, embedding) VALUES
    (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0]::real[], 4, 42)),
    (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.0, 0.0]::real[], 4, 42)),
    (3, encode_to_ecvector(ARRAY[-1.0, 0.0, 0.0, 0.0]::real[], 4, 42)),
    (4, encode_to_ecvector(ARRAY[0.0, -1.0, 0.0, 0.0]::real[], 4, 42));
CREATE INDEX ecaz_pg_upgrade_smoke_hnsw_idx
    ON ecaz_pg_upgrade_smoke USING ec_hnsw (embedding ecvector_ip_ops)
    WITH (m = 4, ef_construction = 32);
ANALYZE ecaz_pg_upgrade_smoke;
SQL

pre_top2="$("${old_psql[@]}" -At -c "SELECT string_agg(id::text, ',' ORDER BY rn) FROM (SELECT id, row_number() OVER () AS rn FROM (SELECT id FROM ecaz_pg_upgrade_smoke ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.0, 0.0]::real[], id LIMIT 2) q) ranked")"
pre_index_count="$("${old_psql[@]}" -At -c "SELECT count(*) FROM pg_class c JOIN pg_am a ON a.oid = c.relam WHERE a.amname = 'ec_hnsw' AND c.relname = 'ecaz_pg_upgrade_smoke_hnsw_idx'")"
pre_heap_count="$("${old_psql[@]}" -At -c "SELECT count(*) FROM ecaz_pg_upgrade_smoke")"

echo "pre_top2=$pre_top2"
echo "pre_index_count=$pre_index_count"
echo "pre_heap_count=$pre_heap_count"
[[ "$pre_top2" == "1,2" ]]
[[ "$pre_index_count" == "1" ]]
[[ "$pre_heap_count" == "4" ]]

"$PG_CTL" -w -D "$OLD_DATA" -m fast stop >/dev/null

"$PG_UPGRADE" \
  --old-bindir="$PGBIN" \
  --new-bindir="$PGBIN" \
  --old-datadir="$OLD_DATA" \
  --new-datadir="$NEW_DATA" \
  --old-port="$OLD_PORT" \
  --new-port="$NEW_PORT" \
  --socketdir="$SOCKET_DIR" \
  --username=postgres \
  --retain

"$PG_CTL" -w -D "$NEW_DATA" -l "$LOG_DIR/new-postgres.log" \
  -o "-p $NEW_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null

new_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$NEW_PORT" -U postgres -d postgres)

post_top2="$("${new_psql[@]}" -At -c "SELECT string_agg(id::text, ',' ORDER BY rn) FROM (SELECT id, row_number() OVER () AS rn FROM (SELECT id FROM ecaz_pg_upgrade_smoke ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.0, 0.0]::real[], id LIMIT 2) q) ranked")"
post_index_count="$("${new_psql[@]}" -At -c "SELECT count(*) FROM pg_class c JOIN pg_am a ON a.oid = c.relam WHERE a.amname = 'ec_hnsw' AND c.relname = 'ecaz_pg_upgrade_smoke_hnsw_idx'")"
post_heap_count="$("${new_psql[@]}" -At -c "SELECT count(*) FROM ecaz_pg_upgrade_smoke")"
extversion="$("${new_psql[@]}" -At -c "SELECT extversion FROM pg_extension WHERE extname = 'ecaz'")"

echo "post_top2=$post_top2"
echo "post_index_count=$post_index_count"
echo "post_heap_count=$post_heap_count"
echo "extension_version=$extversion"
[[ "$post_top2" == "$pre_top2" ]]
[[ "$post_index_count" == "$pre_index_count" ]]
[[ "$post_heap_count" == "$pre_heap_count" ]]

"$PG_AMCHECK" -h "$SOCKET_DIR" -p "$NEW_PORT" -U postgres -d postgres --no-password --install-missing --database=postgres >/dev/null
echo "pg_amcheck=passed"
echo "PG18 pg_upgrade smoke passed"
