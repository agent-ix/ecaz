#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
REMOTE_PORT="${REMOTE_PORT:-39218}"
COORD_PORT="${COORD_PORT:-39219}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
LOG_DIR_OVERRIDE="${LOG_DIR:-}"
SMOKE_LOG="${SMOKE_LOG:-}"
ARTIFACT_DIR=""

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_multicluster_pg18_smoke.sh [options]

Options:
  --artifact-dir DIR  Store smoke and PostgreSQL logs in DIR.
  --coord-port PORT   Coordinator PostgreSQL port. Default: 39219.
  --log-dir DIR       Store PostgreSQL logs in DIR.
  --pgbin DIR         PostgreSQL bin directory. Default: $PGBIN.
  --remote-port PORT  Remote PostgreSQL port. Default: 39218.
  --run-dir DIR       Run directory. Default: target/spire-multicluster-pg18-$RUN_ID.
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
    --coord-port)
      COORD_PORT="$2"
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
    --remote-port)
      REMOTE_PORT="$2"
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

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-multicluster-pg18-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/multicluster-smoke-success.log}"
else
  LOG_DIR="${LOG_DIR_OVERRIDE:-$RUN_DIR/logs}"
fi
REMOTE_DATA="$RUN_DIR/remote"
COORD_DATA="$RUN_DIR/coord"
SOCKET_DIR="$RUN_DIR/sockets"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_SMOKE_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_SMOKE_LOG_ACTIVE=1
  exec > >(tee "$SMOKE_LOG") 2>&1
fi

if [[ -e "$RUN_DIR" ]]; then
  echo "RUN_DIR already exists: $RUN_DIR" >&2
  exit 2
fi

mkdir -p "$LOG_DIR" "$SOCKET_DIR"
: > "$LOG_DIR/remote-postgres.log"
: > "$LOG_DIR/coord-postgres.log"

cleanup() {
  "$PG_CTL" -D "$COORD_DATA" -m fast stop >/dev/null 2>&1 || true
  "$PG_CTL" -D "$REMOTE_DATA" -m fast stop >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "run_dir=$RUN_DIR"
echo "remote_port=$REMOTE_PORT"
echo "coord_port=$COORD_PORT"

if [[ "${ECAZ_SKIP_INSTALL:-0}" != "1" ]]; then
  (cd "$ROOT_DIR" && cargo pgrx install --test --pg-config "$PGBIN/pg_config" \
    --features "pg18 pg_test" --no-default-features)
fi

"$PG_CTL" initdb -D "$REMOTE_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$COORD_DATA" -o "-A trust -U postgres" >/dev/null

export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_MULTICLUSTER="host=$SOCKET_DIR port=$REMOTE_PORT dbname=postgres user=postgres"

"$PG_CTL" -w -D "$REMOTE_DATA" -l "$LOG_DIR/remote-postgres.log" \
  -o "-p $REMOTE_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null
"$PG_CTL" -w -D "$COORD_DATA" -l "$LOG_DIR/coord-postgres.log" \
  -o "-p $COORD_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null

remote_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE_PORT" -U postgres -d postgres)
coord_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$COORD_PORT" -U postgres -d postgres)

"${remote_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null
"${coord_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null

"${remote_psql[@]}" <<'SQL' >/dev/null
CREATE TABLE ec_spire_multicluster_remote_sql
    (id bigint primary key, embedding ecvector);
INSERT INTO ec_spire_multicluster_remote_sql (id, embedding) VALUES
    (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
    (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42));
CREATE INDEX ec_spire_multicluster_remote_idx
    ON ec_spire_multicluster_remote_sql USING ec_spire
    (embedding ecvector_spire_ip_ops) WITH (nlists = 1);
SQL

"${coord_psql[@]}" <<'SQL' >/dev/null
CREATE TABLE ec_spire_multicluster_coord_sql
    (id bigint primary key, embedding ecvector);
INSERT INTO ec_spire_multicluster_coord_sql (id, embedding) VALUES
    (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
    (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42));
CREATE INDEX ec_spire_multicluster_coord_idx
    ON ec_spire_multicluster_coord_sql USING ec_spire
    (embedding ecvector_spire_ip_ops) WITH (nlists = 1);
SQL

remote_epoch="$("${remote_psql[@]}" -At -c "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('ec_spire_multicluster_remote_idx'::regclass)")"
coord_epoch="$("${coord_psql[@]}" -At -c "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('ec_spire_multicluster_coord_idx'::regclass)")"
remote_pid="$("${remote_psql[@]}" -At -c "SELECT min(leaf_pid) FROM ec_spire_index_leaf_snapshot('ec_spire_multicluster_remote_idx'::regclass)")"
coord_pid="$("${coord_psql[@]}" -At -c "SELECT min(leaf_pid) FROM ec_spire_index_leaf_snapshot('ec_spire_multicluster_coord_idx'::regclass)")"
extversion="$("${coord_psql[@]}" -At -c "SELECT extversion FROM pg_extension WHERE extname = 'ecaz'")"

if [[ "$remote_epoch" != "$coord_epoch" ]]; then
  echo "epoch mismatch remote=$remote_epoch coord=$coord_epoch" >&2
  exit 3
fi
if [[ "$remote_pid" != "$coord_pid" ]]; then
  echo "leaf pid mismatch remote=$remote_pid coord=$coord_pid" >&2
  exit 4
fi

"${coord_psql[@]}" -v coord_epoch="$coord_epoch" -v coord_pid="$coord_pid" -v extversion="$extversion" <<'SQL' >/dev/null
SELECT tests.ec_spire_test_rewrite_placement_node(
    'ec_spire_multicluster_coord_idx'::regclass::oid,
    :coord_pid::bigint,
    2
);
SELECT ec_spire_register_remote_node_descriptor(
    'ec_spire_multicluster_coord_idx'::regclass,
    2,
    1,
    'spire/remote/multicluster',
    decode('01', 'hex'),
    'ec_spire_multicluster_remote_idx',
    'active',
    :coord_epoch::bigint,
    :coord_epoch::bigint,
    :'extversion',
    'none'
);
SQL

conn_status="$("${coord_psql[@]}" -At -c "SELECT connection_status || ',' || conninfo_lookup_kind FROM ec_spire_remote_search_libpq_executor_connection_check('ec_spire_multicluster_coord_idx'::regclass, $coord_epoch::bigint, ARRAY[1.0, 0.0]::real[], ARRAY[$coord_pid::bigint], 1, 'strict')")"
candidate_count="$("${coord_psql[@]}" -At -c "SELECT count(*) FROM ec_spire_remote_search_libpq_executor_candidates('ec_spire_multicluster_coord_idx'::regclass, $coord_epoch::bigint, ARRAY[1.0, 0.0]::real[], ARRAY[$coord_pid::bigint], 1, 'strict')")"
heap_summary="$("${coord_psql[@]}" -At -c "SELECT result_source || ',' || status || ',' || returned_candidate_count::text FROM ec_spire_remote_search_libpq_executor_heap_candidate_summary('ec_spire_multicluster_coord_idx'::regclass, $coord_epoch::bigint, ARRAY[1.0, 0.0]::real[], ARRAY[$coord_pid::bigint], 1, 'strict')")"
heap_row="$("${coord_psql[@]}" -At -c "SELECT node_id::text || ',' || heap_lookup_owner || ',' || (heap_offset > 0)::text FROM ec_spire_remote_search_libpq_executor_heap_candidates('ec_spire_multicluster_coord_idx'::regclass, $coord_epoch::bigint, ARRAY[1.0, 0.0]::real[], ARRAY[$coord_pid::bigint], 1, 'strict') LIMIT 1")"
coordinator_result="$("${coord_psql[@]}" -At -c "SELECT result_source || ',' || status || ',' || final_heap_fetch_status || ',' || returned_candidate_count::text FROM ec_spire_remote_search_coordinator_result_summary('ec_spire_multicluster_coord_idx'::regclass, $coord_epoch::bigint, ARRAY[1.0, 0.0]::real[], ARRAY[$coord_pid::bigint], 1, 'strict')")"

"${coord_psql[@]}" -c "SELECT ec_spire_persist_remote_epoch_manifest('ec_spire_multicluster_coord_idx'::regclass)" >/dev/null
manifest_executor="$("${coord_psql[@]}" -At -c "SELECT connection_status || ',' || validation_result_status || ',' || status FROM ec_spire_remote_epoch_manifest_libpq_executor_results('ec_spire_multicluster_coord_idx'::regclass)")"
remote_manifest_applied="$("${remote_psql[@]}" -At -c "SELECT count(*)::text || ',' || coalesce(sum(included_remote_node_count), 0)::text FROM ec_spire_remote_epoch_manifest_applied WHERE remote_index_oid = 'ec_spire_multicluster_remote_idx'::regclass AND active_epoch = $remote_epoch::bigint")"
remote_manifest_entries="$("${remote_psql[@]}" -At -c "SELECT count(*)::text || ',' || coalesce(sum(placement_count), 0)::text FROM ec_spire_remote_epoch_manifest_applied_entry WHERE remote_index_oid = 'ec_spire_multicluster_remote_idx'::regclass AND active_epoch = $remote_epoch::bigint")"

echo "connection_status=$conn_status"
echo "candidate_count=$candidate_count"
echo "heap_summary=$heap_summary"
echo "heap_row=$heap_row"
echo "coordinator_result=$coordinator_result"
echo "manifest_executor=$manifest_executor"
echo "remote_manifest_applied=$remote_manifest_applied"
echo "remote_manifest_entries=$remote_manifest_entries"

[[ "$conn_status" == "libpq_connection_opened,secret_provider" ]]
[[ "$candidate_count" == "1" ]]
[[ "$heap_summary" == "remote_heap_candidates,ready,1" ]]
[[ "$heap_row" == "2,origin_node_row_locator,true" ]]
[[ "$coordinator_result" == "remote_heap_candidates,ready,remote_ready,1" ]]
[[ "$manifest_executor" == "libpq_connection_opened,ready,ready" ]]
[[ "$remote_manifest_applied" == "1,1" ]]
[[ "$remote_manifest_entries" == "1,1" ]]

echo "SPIRE multicluster PG18 smoke passed"
