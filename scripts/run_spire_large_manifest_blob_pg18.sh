#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
PORT="${PORT:-39820}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
ARTIFACT_DIR=""
SMOKE_LOG="${SMOKE_LOG:-}"

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_large_manifest_blob_pg18.sh [options]

Runs a single-node PG18 SPIRE regression that builds enough placements to force
the epoch manifest bundle above one PostgreSQL page, then verifies diagnostics
and an index-backed read.

Options:
  --artifact-dir DIR  Store smoke and PostgreSQL logs in DIR.
  --pgbin DIR         PostgreSQL bin directory. Default: $PGBIN.
  --port PORT         PostgreSQL port. Default: 39820.
  --run-dir DIR       Run directory. Default: target/spire-large-manifest-$RUN_ID.
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
    --pgbin)
      PGBIN="$2"
      PG_CTL="$PGBIN/pg_ctl"
      PSQL="$PGBIN/psql"
      shift 2
      ;;
    --port)
      PORT="$2"
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

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-large-manifest-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/spire-large-manifest-blob-pg18.log}"
else
  LOG_DIR="$RUN_DIR/logs"
fi
SOCKET_DIR="$ROOT_DIR/target/spire-large-manifest-sockets-$RUN_ID"
PGDATA="$RUN_DIR/pgdata"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_LARGE_MANIFEST_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_LARGE_MANIFEST_LOG_ACTIVE=1
  exec > >(tee "$SMOKE_LOG") 2>&1
fi

if [[ -e "$RUN_DIR" ]]; then
  echo "RUN_DIR already exists: $RUN_DIR" >&2
  exit 2
fi

mkdir -p "$LOG_DIR" "$SOCKET_DIR" "$RUN_DIR"
: > "$LOG_DIR/postgres.log"

cleanup() {
  "$PG_CTL" -D "$PGDATA" -m fast stop >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "run_dir=$RUN_DIR"
echo "port=$PORT"

if [[ "${ECAZ_SKIP_INSTALL:-0}" != "1" ]]; then
  (cd "$ROOT_DIR" && cargo pgrx install --test --pg-config "$PGBIN/pg_config" \
    --features "pg18 pg_test" --no-default-features)
fi

"$PG_CTL" initdb -D "$PGDATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" -w -D "$PGDATA" -l "$LOG_DIR/postgres.log" \
  -o "-p $PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null

psql_cmd=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$PORT" -U postgres -d postgres)
"${psql_cmd[@]}" -c "CREATE EXTENSION ecaz" >/dev/null

"${psql_cmd[@]}" <<'SQL'
\timing on
CREATE TABLE ec_spire_large_manifest_sql
    (id bigint primary key, embedding ecvector);
INSERT INTO ec_spire_large_manifest_sql (id, embedding)
SELECT i,
       encode_to_ecvector(
         ARRAY(SELECT (((i * d + d * d) % 997)::real / 997.0)::real
                 FROM generate_series(1, 16) AS d),
         4,
         42)
  FROM generate_series(1, 256) AS i;
CREATE INDEX ec_spire_large_manifest_idx ON ec_spire_large_manifest_sql
    USING ec_spire (embedding ecvector_spire_ip_ops)
    WITH (nlists = 256, nprobe = 16, rerank_width = 25);
SELECT object_count, placement_count
  FROM ec_spire_index_active_snapshot_diagnostics(
    'ec_spire_large_manifest_idx'::regclass);
SELECT count(*) AS indexed_rows
  FROM (
    SELECT id
      FROM ec_spire_large_manifest_sql
     ORDER BY embedding <#> encode_to_ecvector(
       ARRAY(SELECT (((17 * d + d * d) % 997)::real / 997.0)::real
               FROM generate_series(1, 16) AS d),
       4,
       42)
     LIMIT 10
  ) AS ranked;
SQL

summary="$("${psql_cmd[@]}" -At -F '|' <<'SQL'
WITH diag AS (
  SELECT object_count, placement_count
    FROM ec_spire_index_active_snapshot_diagnostics(
      'ec_spire_large_manifest_idx'::regclass)
),
ranked AS (
  SELECT count(*) AS indexed_rows
    FROM (
      SELECT id
        FROM ec_spire_large_manifest_sql
       ORDER BY embedding <#> encode_to_ecvector(
         ARRAY(SELECT (((17 * d + d * d) % 997)::real / 997.0)::real
                 FROM generate_series(1, 16) AS d),
         4,
         42)
       LIMIT 10
    ) AS q
)
SELECT object_count, placement_count, indexed_rows
  FROM diag CROSS JOIN ranked;
SQL
)"

IFS='|' read -r object_count placement_count indexed_rows <<<"$summary"
if (( object_count <= 240 )); then
  echo "object_count too low: $object_count" >&2
  exit 1
fi
if (( placement_count <= 240 )); then
  echo "placement_count too low: $placement_count" >&2
  exit 1
fi
if (( indexed_rows != 10 )); then
  echo "indexed row count mismatch: $indexed_rows" >&2
  exit 1
fi

echo "spire_large_manifest_blob_pg18_pass object_count=$object_count placement_count=$placement_count indexed_rows=$indexed_rows"
