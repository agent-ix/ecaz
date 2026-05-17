#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
REMOTE_READY_PORT="${REMOTE_READY_PORT:-39320}"
COORD_PORT="${COORD_PORT:-39321}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
LOG_DIR_OVERRIDE="${LOG_DIR:-}"
ARTIFACT_DIR=""
SMOKE_LOG="${SMOKE_LOG:-}"
FAULT_CASE=""

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_multicluster_stage_e_predispatch_fault_pg18.sh --case CASE [options]

Cases:
  epoch_mismatch
  version_skew

Options:
  --artifact-dir DIR       Store fixture and PostgreSQL logs in DIR.
  --case CASE              Stage E pre-dispatch fault case.
  --coord-port PORT        Coordinator PostgreSQL port. Default: 39321.
  --log-dir DIR            Store PostgreSQL logs in DIR.
  --pgbin DIR              PostgreSQL bin directory. Default: $PGBIN.
  --remote-ready-port PORT Ready remote PostgreSQL port. Default: 39320.
  --run-dir DIR            Run directory. Default: target/spire-stage-e-predispatch-fault-pg18-$RUN_ID.
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

if [[ "$FAULT_CASE" != "epoch_mismatch" && "$FAULT_CASE" != "version_skew" ]]; then
  echo "unsupported or missing --case: ${FAULT_CASE:-<none>}" >&2
  usage >&2
  exit 2
fi

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-stage-e-predispatch-fault-pg18-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/stage_e_fault_${FAULT_CASE}.log}"
else
  LOG_DIR="${LOG_DIR_OVERRIDE:-$RUN_DIR/logs}"
fi
REMOTE_READY_DATA="$RUN_DIR/remote-ready"
COORD_DATA="$RUN_DIR/coord"
SOCKET_KEY="$(printf '%s' "$RUN_DIR" | cksum | awk '{print $1}')"
SOCKET_DIR="${SOCKET_DIR:-$ROOT_DIR/target/s-$SOCKET_KEY}"
STRICT_LOG="${ARTIFACT_DIR:-$LOG_DIR}/stage_e_fault_${FAULT_CASE}_strict.log"
DEGRADED_LOG="${ARTIFACT_DIR:-$LOG_DIR}/stage_e_fault_${FAULT_CASE}_degraded.log"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_STAGE_E_PREDISPATCH_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_STAGE_E_PREDISPATCH_LOG_ACTIVE=1
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
    (embedding ecvector_spire_ip_ops) WITH (nlists = 2);
SQL

ready_identity_hex="$("${remote_ready_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_ready_remote_idx'::regclass)")"
extversion="$("${coord_psql[@]}" -At -c "SELECT extversion FROM pg_extension WHERE extname = 'ecaz'")"
matrix_row="$("${coord_psql[@]}" -At -F ',' -c "SELECT fault_case, failure_category, strict_action, strict_status, degraded_action, degraded_status, counter_delta FROM ec_spire_remote_search_stage_e_fault_matrix() WHERE fault_case = '$FAULT_CASE'")"

create_case_index() {
  local mode="$1"
  local table_name="ec_spire_stage_e_${FAULT_CASE}_${mode}_coord_sql"
  local index_name="ec_spire_stage_e_${FAULT_CASE}_${mode}_coord_idx"

  "${coord_psql[@]}" -v table_name="$table_name" -v index_name="$index_name" <<'SQL' >/dev/null
CREATE TABLE :table_name
    (id bigint primary key, embedding ecvector);
INSERT INTO :table_name (id, embedding) VALUES
    (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
    (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)),
    (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)),
    (4, encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42));
CREATE INDEX :index_name
    ON :table_name USING ec_spire
    (embedding ecvector_spire_ip_ops) WITH (nlists = 2, nprobe = 2);
SQL

  local coord_epoch
  local coord_pids
  local bad_pid
  local ready_pid
  local extra_pid
  local fault_extension_version="$extversion"
  local fault_last_served_epoch
  local fault_min_retained_epoch
  coord_epoch="$("${coord_psql[@]}" -At -c "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('$index_name'::regclass)")"
  coord_pids="$("${coord_psql[@]}" -At -F ',' -c "SELECT string_agg(leaf_pid::text, ',' ORDER BY leaf_pid) FROM ec_spire_index_leaf_snapshot('$index_name'::regclass)")"
  IFS=, read -r bad_pid ready_pid extra_pid <<< "$coord_pids"
  if [[ -z "$bad_pid" || -z "$ready_pid" || -n "${extra_pid:-}" ]]; then
    echo "expected exactly two coordinator leaf PIDs for $index_name, got: $coord_pids" >&2
    exit 3
  fi

  if [[ "$mode" == "degraded" ]]; then
    "${coord_psql[@]}" -v index_name="$index_name" <<'SQL' >/dev/null
SELECT tests.ec_spire_test_rewrite_consistency_mode(
    :'index_name'::regclass::oid,
      'degraded'
);
SQL
  fi
  fault_last_served_epoch="$coord_epoch"
  fault_min_retained_epoch="$coord_epoch"
  if [[ "$FAULT_CASE" == "epoch_mismatch" ]]; then
    fault_last_served_epoch=0
  else
    fault_extension_version="0.0.0-test-skew"
  fi

  "${coord_psql[@]}" -v index_name="$index_name" -v coord_epoch="$coord_epoch" \
    -v bad_pid="$bad_pid" -v ready_pid="$ready_pid" -v extversion="$extversion" \
    -v fault_extension_version="$fault_extension_version" \
    -v fault_last_served_epoch="$fault_last_served_epoch" \
    -v fault_min_retained_epoch="$fault_min_retained_epoch" \
    -v ready_identity_hex="$ready_identity_hex" <<'SQL' >/dev/null
SELECT tests.ec_spire_test_rewrite_placement_nodes(
    :'index_name'::regclass::oid,
    ARRAY[:bad_pid::bigint, :ready_pid::bigint],
    ARRAY[2, 3]
);
SELECT ec_spire_register_remote_node_descriptor(
    :'index_name'::regclass,
    2,
    1,
    'spire/remote/stage_e/version_skew',
    decode('02', 'hex'),
    'ec_spire_stage_e_ready_remote_idx',
    'active',
    :fault_last_served_epoch::bigint,
    :fault_min_retained_epoch::bigint,
    :'fault_extension_version',
    'none'
);
SELECT ec_spire_register_remote_node_descriptor(
    :'index_name'::regclass,
    3,
    1,
    'spire/remote/stage_e/ready',
    decode(:'ready_identity_hex', 'hex'),
    'ec_spire_stage_e_ready_remote_idx',
    'active',
    :coord_epoch::bigint,
    :coord_epoch::bigint,
    :'extversion',
    'none'
);
SQL

  echo "$index_name,$coord_epoch,$bad_pid,$ready_pid,$fault_extension_version,$fault_last_served_epoch,$fault_min_retained_epoch"
}

run_case() {
  local mode="$1"
  local output_log="$2"
  local expected_status="$3"
  local expected_blocked="$4"
  local expected_degraded_skipped="$5"
  local expected_next_step="$6"
  local expected_first_skip="$7"

  local case_setup
  local index_name
  local coord_epoch
  local bad_pid
  local ready_pid
  local fault_extension_version
  local fault_last_served_epoch
  local fault_min_retained_epoch
  case_setup="$(create_case_index "$mode")"
  IFS=, read -r index_name coord_epoch bad_pid ready_pid fault_extension_version fault_last_served_epoch fault_min_retained_epoch <<< "$case_setup"

  local summary
  summary="$("${coord_psql[@]}" -At -F ',' -c "SELECT state_model, dispatch_count, planned_dispatch_count, blocked_before_dispatch_count, transport_pending_dispatch_count, degraded_skipped_dispatch_count, first_degraded_skip_category, next_executor_step, status FROM ec_spire_remote_search_production_executor_state_summary('$index_name'::regclass, $coord_epoch, ARRAY[1.0, 0.0]::real[], ARRAY[$bad_pid,$ready_pid]::bigint[], 1, '$mode')")"

  {
    echo "matrix_row=$matrix_row"
    echo "injection=fault_node_extension_version=$fault_extension_version"
    echo "injection=fault_node_epoch_window=$fault_last_served_epoch/$fault_min_retained_epoch requested=$coord_epoch"
    echo "ready_remote_identity=$ready_identity_hex"
    echo "query_command=ec_spire_remote_search_production_executor_state_summary('$index_name', ..., '$mode')"
    echo "expected_status=$expected_status"
    echo "expected_blocked_before_dispatch_count=$expected_blocked"
    echo "expected_degraded_skipped_dispatch_count=$expected_degraded_skipped"
    echo "expected_first_degraded_skip_category=$expected_first_skip"
    echo "expected_next_executor_step=$expected_next_step"
    echo "observed_summary=$summary"
  } | tee "$output_log"

  IFS=, read -r state_model dispatch_count planned_count blocked_count transport_pending_count degraded_count first_skip next_step status <<< "$summary"
  [[ "$state_model" == "spire_remote_fanout_executor_v1" ]]
  [[ "$dispatch_count" == "2" ]]
  [[ "$blocked_count" == "$expected_blocked" ]]
  [[ "$degraded_count" == "$expected_degraded_skipped" ]]
  [[ "$first_skip" == "$expected_first_skip" ]]
  [[ "$next_step" == "$expected_next_step" ]]
  [[ "$status" == "$expected_status" ]]
  if [[ "$mode" == "strict" ]]; then
    [[ "$planned_count" == "1" ]]
    [[ "$transport_pending_count" == "1" ]]
  else
    [[ "$planned_count" == "2" ]]
    [[ "$transport_pending_count" == "1" ]]
  fi
}

if [[ "$FAULT_CASE" == "epoch_mismatch" ]]; then
  run_case "strict" "$STRICT_LOG" "stale_epoch" "1" "0" "remote_epoch_window" "none"
  run_case "degraded" "$DEGRADED_LOG" "requires_production_transport_adapter" "0" "1" "production_transport_adapter" "stale_epoch"
else
  run_case "strict" "$STRICT_LOG" "incompatible_extension_version" "1" "0" "remote_extension_version" "none"
  run_case "degraded" "$DEGRADED_LOG" "requires_production_transport_adapter" "0" "1" "production_transport_adapter" "incompatible_extension_version"
fi

echo "strict_log=$STRICT_LOG"
echo "degraded_log=$DEGRADED_LOG"
echo "stage_e_fault_${FAULT_CASE}_passed=true"
echo "SPIRE Stage E $FAULT_CASE PG18 fixture passed"
