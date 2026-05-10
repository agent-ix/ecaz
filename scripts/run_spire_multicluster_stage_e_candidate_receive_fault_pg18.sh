#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
REMOTE_READY_PORT="${REMOTE_READY_PORT:-39322}"
COORD_PORT="${COORD_PORT:-39323}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
LOG_DIR_OVERRIDE="${LOG_DIR:-}"
ARTIFACT_DIR=""
SMOKE_LOG="${SMOKE_LOG:-}"
FAULT_CASE=""

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_multicluster_stage_e_candidate_receive_fault_pg18.sh --case CASE [options]

Cases:
  missing_or_reindexed_remote_index

Options:
  --artifact-dir DIR       Store fixture and PostgreSQL logs in DIR.
  --case CASE              Stage E candidate-receive fault case.
  --coord-port PORT        Coordinator PostgreSQL port. Default: 39323.
  --log-dir DIR            Store PostgreSQL logs in DIR.
  --pgbin DIR              PostgreSQL bin directory. Default: $PGBIN.
  --remote-ready-port PORT Ready remote PostgreSQL port. Default: 39322.
  --run-dir DIR            Run directory. Default: target/spire-stage-e-candidate-receive-fault-pg18-$RUN_ID.
  --run-id ID              Run id used in the default run directory.
  --skip-install           Skip cargo pgrx install.
  --smoke-log FILE         Tee fixture output to FILE.
  -h, --help               Show this help.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir)
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    --case)
      FAULT_CASE="$2"
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
    --remote-ready-port)
      REMOTE_READY_PORT="$2"
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

if [[ "$FAULT_CASE" != "missing_or_reindexed_remote_index" ]]; then
  echo "unsupported or missing --case: ${FAULT_CASE:-<none>}" >&2
  usage >&2
  exit 2
fi

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-stage-e-candidate-receive-fault-pg18-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/stage_e_fault_${FAULT_CASE}.log}"
else
  LOG_DIR="${LOG_DIR_OVERRIDE:-$RUN_DIR/logs}"
fi
REMOTE_READY_DATA="$RUN_DIR/remote-ready"
COORD_DATA="$RUN_DIR/coord"
SOCKET_DIR="$RUN_DIR/sockets"
STRICT_LOG="${ARTIFACT_DIR:-$LOG_DIR}/stage_e_fault_${FAULT_CASE}_strict.log"
DEGRADED_LOG="${ARTIFACT_DIR:-$LOG_DIR}/stage_e_fault_${FAULT_CASE}_degraded.log"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_STAGE_E_CANDIDATE_RECEIVE_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_STAGE_E_CANDIDATE_RECEIVE_LOG_ACTIVE=1
  exec > >(tee "$SMOKE_LOG") 2>&1
fi

if [[ -e "$RUN_DIR" ]]; then
  echo "RUN_DIR already exists: $RUN_DIR" >&2
  exit 2
fi

mkdir -p "$LOG_DIR" "$SOCKET_DIR"
: > "$LOG_DIR/remote-ready-postgres.log"
: > "$LOG_DIR/coord-postgres.log"

cleanup() {
  "$PG_CTL" -D "$COORD_DATA" -m fast stop >/dev/null 2>&1 || true
  "$PG_CTL" -D "$REMOTE_READY_DATA" -m fast stop >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "run_dir=$RUN_DIR"
echo "fault_case=$FAULT_CASE"
echo "remote_ready_port=$REMOTE_READY_PORT"
echo "coord_port=$COORD_PORT"

if [[ "${ECAZ_SKIP_INSTALL:-0}" != "1" ]]; then
  (cd "$ROOT_DIR" && cargo pgrx install --test --pg-config "$PGBIN/pg_config" \
    --features "pg18 pg_test" --no-default-features)
fi

"$PG_CTL" initdb -D "$REMOTE_READY_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$COORD_DATA" -o "-A trust -U postgres" >/dev/null

export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_READY="host=$SOCKET_DIR port=$REMOTE_READY_PORT dbname=postgres user=postgres connect_timeout=1"
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_MISSING_INDEX="$EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_READY"
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_COORD_READY="host=$SOCKET_DIR port=$COORD_PORT dbname=postgres user=postgres connect_timeout=1"

"$PG_CTL" -w -D "$REMOTE_READY_DATA" -l "$LOG_DIR/remote-ready-postgres.log" \
  -o "-p $REMOTE_READY_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null
"$PG_CTL" -w -D "$COORD_DATA" -l "$LOG_DIR/coord-postgres.log" \
  -o "-p $COORD_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null

remote_ready_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE_READY_PORT" -U postgres -d postgres)
coord_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$COORD_PORT" -U postgres -d postgres)

"${remote_ready_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null
"${coord_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null

"${remote_ready_psql[@]}" <<'SQL' >/dev/null
CREATE TABLE ec_spire_stage_e_ready_remote_sql
    (id bigint primary key, embedding ecvector);
INSERT INTO ec_spire_stage_e_ready_remote_sql (id, embedding) VALUES
    (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
    (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)),
    (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)),
    (4, encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42));
CREATE INDEX ec_spire_stage_e_ready_remote_idx
    ON ec_spire_stage_e_ready_remote_sql USING ec_spire
    (embedding ecvector_spire_ip_ops) WITH (nlists = 2, storage_format = 'rabitq');
SQL

"${coord_psql[@]}" <<'SQL' >/dev/null
CREATE TABLE ec_spire_stage_e_coord_ready_sql
    (id bigint primary key, embedding ecvector);
INSERT INTO ec_spire_stage_e_coord_ready_sql (id, embedding) VALUES
    (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
    (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)),
    (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)),
    (4, encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42));
CREATE INDEX ec_spire_stage_e_coord_ready_idx
    ON ec_spire_stage_e_coord_ready_sql USING ec_spire
    (embedding ecvector_spire_ip_ops) WITH (nlists = 2, storage_format = 'rabitq');
SQL

remote_ready_identity_hex="$("${remote_ready_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_ready_remote_idx'::regclass)")"
coord_ready_identity_hex="$("${coord_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_coord_ready_idx'::regclass)")"
coord_ready_epoch="$("${coord_psql[@]}" -At -c "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('ec_spire_stage_e_coord_ready_idx'::regclass)")"
coord_ready_pids="$("${coord_psql[@]}" -At -F ',' -c "SELECT string_agg(leaf_pid::text, ',' ORDER BY leaf_pid) FROM ec_spire_index_leaf_snapshot('ec_spire_stage_e_coord_ready_idx'::regclass)")"
matrix_row="$("${coord_psql[@]}" -At -F ',' -c "SELECT fault_case, failure_category, strict_action, strict_status, degraded_action, degraded_status, counter_delta FROM ec_spire_remote_search_stage_e_fault_matrix() WHERE fault_case = '$FAULT_CASE'")"

IFS=, read -r missing_pid ready_pid extra_pid <<< "$coord_ready_pids"
if [[ -z "$missing_pid" || -z "$ready_pid" || -n "${extra_pid:-}" ]]; then
  echo "expected exactly two coordinator ready leaf PIDs, got: $coord_ready_pids" >&2
  exit 3
fi

run_case() {
  local mode="$1"
  local output_log="$2"
  local expected_status="$3"
  local expected_failed_receive="$4"
  local expected_degraded_skipped="$5"
  local expected_next_step="$6"
  local expected_first_failure="$7"
  local expected_first_skip="$8"
  local case_coord_identity="$coord_ready_identity_hex"

  if [[ "$mode" == "degraded" ]]; then
    "${coord_psql[@]}" <<'SQL' >/dev/null
SELECT tests.ec_spire_test_rewrite_consistency_mode(
    'ec_spire_stage_e_coord_ready_idx'::regclass::oid,
    'degraded'
);
SQL
    case_coord_identity="$("${coord_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_coord_ready_idx'::regclass)")"
  fi

  local raw_rows
  raw_rows="$("${coord_psql[@]}" -At -F ',' -c "SELECT node_id, status, failure_category, candidate_count FROM tests.ec_spire_test_production_candidate_receive(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/missing_index','spire/remote/stage_e/coord_ready']::text[], ARRAY['ec_spire_stage_e_missing_remote_idx','ec_spire_stage_e_coord_ready_idx']::text[], ARRAY['$remote_ready_identity_hex','$case_coord_identity']::text[], ARRAY[$missing_pid,$ready_pid]::bigint[], $coord_ready_epoch, ARRAY[1.0, 0.0]::real[], 1, '$mode') ORDER BY node_id")"
  local direct_ready_rows
  direct_ready_rows="$("${coord_psql[@]}" -At -F ',' -c "SELECT served_epoch, node_id, pid, object_version, row_index, assignment_flags, octet_length(vec_id), octet_length(row_locator), score, protocol_version, extension_version, profile_fingerprint, endpoint_status FROM ec_spire_remote_search('ec_spire_stage_e_coord_ready_idx'::regclass, $coord_ready_epoch, ARRAY[1.0, 0.0]::real[], ARRAY[$missing_pid,$ready_pid]::bigint[], 1, '$mode')")"
  local summary
  summary="$("${coord_psql[@]}" -At -F ',' -c "SELECT state_model, dispatch_count, candidate_receive_sent_dispatch_count, candidate_receive_ready_dispatch_count, candidate_receive_failed_dispatch_count, first_candidate_receive_failure_category, candidate_row_count, degraded_skipped_dispatch_count, first_degraded_skip_category, next_executor_step, status FROM tests.ec_spire_test_production_candidate_receive_summary(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/missing_index','spire/remote/stage_e/coord_ready']::text[], ARRAY['ec_spire_stage_e_missing_remote_idx','ec_spire_stage_e_coord_ready_idx']::text[], ARRAY['$remote_ready_identity_hex','$case_coord_identity']::text[], ARRAY[$missing_pid,$ready_pid]::bigint[], $coord_ready_epoch, ARRAY[1.0, 0.0]::real[], 1, '$mode')")"

  {
    echo "matrix_row=$matrix_row"
    echo "injection=remote_index_regclass=ec_spire_stage_e_missing_remote_idx"
    echo "missing_remote_identity=$remote_ready_identity_hex"
    echo "coord_ready_identity=$case_coord_identity"
    echo "coord_ready_epoch=$coord_ready_epoch"
    echo "coord_ready_pids=$coord_ready_pids"
    echo "query_command=tests.ec_spire_test_production_candidate_receive_summary(..., '$mode')"
    echo "expected_status=$expected_status"
    echo "expected_candidate_receive_failed_dispatch_count=$expected_failed_receive"
    echo "expected_degraded_skipped_dispatch_count=$expected_degraded_skipped"
    echo "expected_first_candidate_receive_failure_category=$expected_first_failure"
    echo "expected_first_degraded_skip_category=$expected_first_skip"
    echo "expected_next_executor_step=$expected_next_step"
    echo "direct_ready_rows=$direct_ready_rows"
    echo "observed_candidate_receive_rows=$raw_rows"
    echo "observed_summary=$summary"
  } | tee "$output_log"

  IFS=, read -r state_model dispatch_count sent_count ready_receive_count failed_receive_count first_failure candidate_row_count degraded_count first_skip next_step status <<< "$summary"
  [[ "$state_model" == "spire_remote_fanout_executor_v1" ]]
  [[ "$dispatch_count" == "2" ]]
  [[ "$ready_receive_count" == "1" ]]
  [[ "$failed_receive_count" == "$expected_failed_receive" ]]
  [[ "$first_failure" == "$expected_first_failure" ]]
  [[ "$degraded_count" == "$expected_degraded_skipped" ]]
  [[ "$first_skip" == "$expected_first_skip" ]]
  [[ "$candidate_row_count" == "1" ]]
  [[ "$next_step" == "$expected_next_step" ]]
  [[ "$status" == "$expected_status" ]]
  if [[ "$mode" == "strict" ]]; then
    [[ "$sent_count" == "2" ]]
  else
    [[ "$sent_count" == "1" ]]
  fi
}

run_case "strict" "$STRICT_LOG" "remote_candidate_receive_failed" "1" "0" "compact_candidate_receive" "remote_index_unavailable" "none"
run_case "degraded" "$DEGRADED_LOG" "degraded_ready" "0" "1" "remote_heap_resolution" "none" "remote_index_unavailable"

echo "strict_log=$STRICT_LOG"
echo "degraded_log=$DEGRADED_LOG"
echo "stage_e_fault_${FAULT_CASE}_passed=true"
echo "SPIRE Stage E $FAULT_CASE PG18 fixture passed"
