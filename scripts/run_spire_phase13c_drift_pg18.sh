#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
PORT="${PORT:-39429}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
LOG_DIR_OVERRIDE="${LOG_DIR:-}"
SMOKE_LOG="${SMOKE_LOG:-}"
ARTIFACT_DIR=""

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_phase13c_drift_pg18.sh [options]

Runs the Phase 13c PK SELECT schema-drift smoke against a local PG18 cluster.

Options:
  --artifact-dir DIR  Store smoke and PostgreSQL logs in DIR.
  --log-dir DIR       Store PostgreSQL logs in DIR.
  --pgbin DIR         PostgreSQL bin directory. Default: $PGBIN.
  --port PORT         PostgreSQL port. Default: 39429.
  --run-dir DIR       Run directory. Default: target/spire-phase13c-drift-pg18-$RUN_ID.
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

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-phase13c-drift-pg18-$RUN_ID}"
if [[ "$RUN_DIR" != /* ]]; then
  RUN_DIR="$ROOT_DIR/$RUN_DIR"
fi
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/phase13c-drift-success.log}"
else
  LOG_DIR="${LOG_DIR_OVERRIDE:-$RUN_DIR/logs}"
fi
PGDATA="$RUN_DIR/pgdata"
SOCKET_DIR="$RUN_DIR/sockets"
SECRET_NAME="spire/remote/phase13c/drift"
SECRET_ENV="EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_PHASE13C_DRIFT"
SOURCE_IDENTITY="9192939495969798999a9b9c9d9e9fa0"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_DRIFT_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_DRIFT_LOG_ACTIVE=1
  exec > >(tee "$SMOKE_LOG") 2>&1
fi

if [[ -e "$RUN_DIR" ]]; then
  echo "RUN_DIR already exists: $RUN_DIR" >&2
  exit 2
fi

mkdir -p "$LOG_DIR" "$SOCKET_DIR"
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
export "$SECRET_ENV=host=localhost port=$PORT dbname=postgres user=postgres"

"$PG_CTL" -w -D "$PGDATA" -l "$LOG_DIR/postgres.log" \
  -o "-p $PORT -k $SOCKET_DIR -c listen_addresses='localhost'" start >/dev/null

psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$PORT" -U postgres -d postgres)
"${psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null
extversion="$("${psql[@]}" -At -c "SELECT extversion FROM pg_extension WHERE extname = 'ecaz'")"

register_descriptor() {
  local coord_index="$1"
  local remote_index="$2"
  local remote_identity_hex="$3"
  local active_epoch="$4"
  local node_id="$5"
  local generation="$6"

  "${psql[@]}" -c "
SELECT ec_spire_register_remote_node_descriptor(
    '${coord_index}'::regclass,
    ${node_id},
    ${generation},
    '${SECRET_NAME}',
    decode('${remote_identity_hex}', 'hex'),
    '${remote_index}',
    'active',
    ${active_epoch},
    ${active_epoch},
    '${extversion}',
    'none'
)" >/dev/null
}

run_pk_select_variant() {
  local variant="$1"
  local expected="$2"
  local pk="$3"
  local node_id="$4"
  local prefix="ec_spire_phase13c_pk_select_${variant}"
  local remote_table="${prefix}_remote"
  local remote_index="${prefix}_remote_idx"
  local coord_table="${prefix}_coord"
  local coord_index="${prefix}_coord_idx"
  local variant_log="$LOG_DIR/pk-select-schema-drift-${variant}.log"

  "${psql[@]}" <<SQL >/dev/null
DROP TABLE IF EXISTS ${remote_table};
DROP TABLE IF EXISTS ${coord_table};
CREATE TABLE ${remote_table}
    (id bigint primary key, title text not null, embedding ecvector,
     source_identity bytea not null);
INSERT INTO ${remote_table} (id, title, embedding, source_identity)
VALUES (${pk}, 'before drift', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42),
        decode('${SOURCE_IDENTITY}', 'hex'));
CREATE INDEX ${remote_index}
    ON ${remote_table} USING ec_spire (embedding ecvector_spire_ip_ops);
CREATE TABLE ${coord_table}
    (id bigint primary key, title text not null, embedding ecvector,
     source_identity bytea not null);
INSERT INTO ${coord_table} (id, title, embedding, source_identity)
VALUES (1, 'coordinator seed', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42),
        decode('a1a2a3a4a5a6a7a8a9aaabacadaeafb0', 'hex'));
CREATE INDEX ${coord_index}
    ON ${coord_table} USING ec_spire (embedding ecvector_spire_ip_ops);
SQL

  local active_epoch
  local remote_identity_hex
  active_epoch="$("${psql[@]}" -At -c "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('${coord_index}'::regclass)")"
  remote_identity_hex="$("${psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('${remote_index}'::regclass::oid)")"
  register_descriptor "$coord_index" "$remote_index" "$remote_identity_hex" "$active_epoch" "$node_id" 31
  "${psql[@]}" -c "
INSERT INTO ec_spire_placement
    (index_oid, pk_value, node_id, centroid_id, served_epoch, source_identity)
VALUES ('${coord_index}'::regclass, int8send(${pk}::bigint)::bytea,
        ${node_id}, 2, ${active_epoch}, decode('${SOURCE_IDENTITY}', 'hex'))" >/dev/null

  case "$variant" in
    coord_only)
      "${psql[@]}" -c "ALTER TABLE ${coord_table} ADD COLUMN coord_only text" >/dev/null
      ;;
    remote_only)
      "${psql[@]}" -c "ALTER TABLE ${remote_table} ADD COLUMN remote_only text" >/dev/null
      ;;
    both_sides)
      "${psql[@]}" -c "
ALTER TABLE ${coord_table} ADD COLUMN coord_side text;
ALTER TABLE ${remote_table} ADD COLUMN remote_side integer" >/dev/null
      ;;
    *)
      echo "unexpected PK SELECT drift variant: $variant" >&2
      exit 3
      ;;
  esac

  if "${psql[@]}" -c "
SELECT * FROM ec_spire_forward_coordinator_select_tuple_payload(
    '${coord_index}'::regclass,
    'id',
    int8send(${pk}::bigint)::bytea,
    ARRAY['id', 'title']::text[])" > "$variant_log" 2>&1; then
    echo "expected PK SELECT ${variant} drift to fail" >&2
    exit 4
  fi
  grep -q "schema_drift" "$variant_log"
  grep -q "$expected" "$variant_log"
  if grep -q "remote SQL failed" "$variant_log"; then
    echo "PK SELECT ${variant} drift reached remote SELECT instead of failing in guard" >&2
    exit 5
  fi

  local remote_summary
  remote_summary="$("${psql[@]}" -At -c "
SELECT title || '|' || count(*)::text
  FROM ${remote_table}
 WHERE id = ${pk}
 GROUP BY title")"
  [[ "$remote_summary" == "before drift|1" ]]
  echo "pk_select_schema_drift_variant=${variant},schema_drift,${expected}"
}

run_pk_select_variant "coord_only" "coordinator side drifted" 7901 91
run_pk_select_variant "remote_only" "remote side drifted" 7902 92
run_pk_select_variant "both_sides" "coordinator and remote schema fingerprints differ" 7903 93

echo "SPIRE Phase 13c PG18 PK SELECT drift smoke passed"
