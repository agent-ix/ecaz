#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
REMOTE_READY_PORT="${REMOTE_READY_PORT:-39318}"
COORD_PORT="${COORD_PORT:-39319}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
LOG_DIR_OVERRIDE="${LOG_DIR:-}"
ARTIFACT_DIR=""
SMOKE_LOG="${SMOKE_LOG:-}"

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_multicluster_stage_e_network_partition_pg18.sh [options]

Options:
  --artifact-dir DIR       Store fixture and PostgreSQL logs in DIR.
  --coord-port PORT        Coordinator PostgreSQL port. Default: 39319.
  --log-dir DIR            Store PostgreSQL logs in DIR.
  --pgbin DIR              PostgreSQL bin directory. Default: $PGBIN.
  --remote-ready-port PORT Ready remote PostgreSQL port. Default: 39318.
  --run-dir DIR            Run directory. Default: target/spire-stage-e-network-partition-pg18-$RUN_ID.
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

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-stage-e-network-partition-pg18-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/stage_e_fault_simulated_network_partition.log}"
else
  LOG_DIR="${LOG_DIR_OVERRIDE:-$RUN_DIR/logs}"
fi
REMOTE_READY_DATA="$RUN_DIR/remote-ready"
COORD_DATA="$RUN_DIR/coord"
SOCKET_DIR="$RUN_DIR/sockets"
MISSING_SOCKET_DIR="$RUN_DIR/missing-socket"
STRICT_LOG="${ARTIFACT_DIR:-$LOG_DIR}/stage_e_fault_simulated_network_partition_strict.log"
DEGRADED_LOG="${ARTIFACT_DIR:-$LOG_DIR}/stage_e_fault_simulated_network_partition_degraded.log"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_STAGE_E_NETPART_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_STAGE_E_NETPART_LOG_ACTIVE=1
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
echo "remote_ready_port=$REMOTE_READY_PORT"
echo "coord_port=$COORD_PORT"
echo "missing_socket_dir=$MISSING_SOCKET_DIR"

if [[ "${ECAZ_SKIP_INSTALL:-0}" != "1" ]]; then
  (cd "$ROOT_DIR" && cargo pgrx install --test --pg-config "$PGBIN/pg_config" \
    --features "pg18 pg_test" --no-default-features)
fi

"$PG_CTL" initdb -D "$REMOTE_READY_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$COORD_DATA" -o "-A trust -U postgres" >/dev/null

export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_READY="host=$SOCKET_DIR port=$REMOTE_READY_PORT dbname=postgres user=postgres connect_timeout=1"
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_MISSING="host=$MISSING_SOCKET_DIR port=6543 dbname=postgres user=postgres connect_timeout=1"

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

"${coord_psql[@]}" <<'SQL' >/dev/null
CREATE TABLE ec_spire_stage_e_coord_sql
    (id bigint primary key, embedding ecvector);
INSERT INTO ec_spire_stage_e_coord_sql (id, embedding) VALUES
    (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
    (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)),
    (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)),
    (4, encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42));
CREATE INDEX ec_spire_stage_e_coord_idx
    ON ec_spire_stage_e_coord_sql USING ec_spire
    (embedding ecvector_spire_ip_ops) WITH (nlists = 2, nprobe = 2);
SQL

coord_epoch="$("${coord_psql[@]}" -At -c "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('ec_spire_stage_e_coord_idx'::regclass)")"
coord_pids="$("${coord_psql[@]}" -At -F ',' -c "SELECT string_agg(leaf_pid::text, ',' ORDER BY leaf_pid) FROM ec_spire_index_leaf_snapshot('ec_spire_stage_e_coord_idx'::regclass)")"
ready_identity_hex="$("${remote_ready_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_ready_remote_idx'::regclass)")"
extversion="$("${coord_psql[@]}" -At -c "SELECT extversion FROM pg_extension WHERE extname = 'ecaz'")"

IFS=, read -r missing_pid ready_pid extra_pid <<< "$coord_pids"
if [[ -z "$missing_pid" || -z "$ready_pid" || -n "${extra_pid:-}" ]]; then
  echo "expected exactly two coordinator leaf PIDs, got: $coord_pids" >&2
  exit 3
fi

"${coord_psql[@]}" -v coord_epoch="$coord_epoch" -v missing_pid="$missing_pid" \
  -v ready_pid="$ready_pid" -v extversion="$extversion" -v ready_identity_hex="$ready_identity_hex" <<'SQL' >/dev/null
SELECT tests.ec_spire_test_rewrite_placement_node(
    'ec_spire_stage_e_coord_idx'::regclass::oid,
    :missing_pid::bigint,
    2
);
SELECT tests.ec_spire_test_rewrite_placement_node(
    'ec_spire_stage_e_coord_idx'::regclass::oid,
    :ready_pid::bigint,
    3
);
SELECT ec_spire_register_remote_node_descriptor(
    'ec_spire_stage_e_coord_idx'::regclass,
    2,
    1,
    'spire/remote/stage_e/missing',
    decode('02', 'hex'),
    'ec_spire_stage_e_ready_remote_idx',
    'active',
    :coord_epoch::bigint,
    :coord_epoch::bigint,
    :'extversion',
    'none'
);
SELECT ec_spire_register_remote_node_descriptor(
    'ec_spire_stage_e_coord_idx'::regclass,
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

matrix_row="$("${coord_psql[@]}" -At -F ',' -c "SELECT fault_case, failure_category, strict_action, strict_status, degraded_action, degraded_status, counter_delta FROM ec_spire_remote_search_stage_e_fault_matrix() WHERE fault_case = 'simulated_network_partition'")"

run_case() {
  local mode="$1"
  local output_log="$2"
  local expected_status="$3"
  local expected_transport_failed="$4"
  local expected_degraded_skipped="$5"
  local expected_next_step="$6"

  if [[ "$mode" == "degraded" ]]; then
    "${coord_psql[@]}" -c "SELECT tests.ec_spire_test_rewrite_consistency_mode('ec_spire_stage_e_coord_idx'::regclass::oid, 'degraded')" >/dev/null
  fi

  local summary
  summary="$("${coord_psql[@]}" -At -F ',' -c "SELECT state_model, dispatch_count, transport_sent_dispatch_count, transport_ready_dispatch_count, transport_failed_dispatch_count, first_transport_failure_category, candidate_receive_pending_dispatch_count, degraded_skipped_dispatch_count, first_degraded_skip_category, next_executor_step, status FROM tests.ec_spire_test_production_transport_probe_summary(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/missing','spire/remote/stage_e/ready']::text[], 0, '$mode')")"
  local diagnostics
  diagnostics="$("${coord_psql[@]}" -At -F ',' -c "WITH _ AS (SELECT set_config('ec_spire.remote_search_consistency_mode', '$mode', false)) SELECT consistency_mode, remote_node_count, ready_remote_node_count, blocked_remote_node_count, remote_fanout_count, candidate_batch_count, candidate_row_count, status, next_blocker FROM ec_spire_remote_search_operator_diagnostics('ec_spire_stage_e_coord_idx'::regclass, ARRAY[1.0, 0.0]::real[], 1), _")"

  {
    echo "matrix_row=$matrix_row"
    echo "injection_command=EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_MISSING=$EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_MISSING"
    echo "query_command=tests.ec_spire_test_production_transport_probe_summary(..., '$mode')"
    echo "operator_diagnostic_row=$diagnostics"
    echo "expected_status=$expected_status"
    echo "expected_transport_failed_dispatch_count=$expected_transport_failed"
    echo "expected_degraded_skipped_dispatch_count=$expected_degraded_skipped"
    echo "expected_next_executor_step=$expected_next_step"
    echo "observed_summary=$summary"
  } | tee "$output_log"

  IFS=, read -r state_model dispatch_count sent_count ready_count failed_count first_failure pending_count degraded_count first_skip next_step status <<< "$summary"
  [[ "$dispatch_count" == "2" ]]
  [[ "$ready_count" == "1" ]]
  [[ "$failed_count" == "$expected_transport_failed" ]]
  [[ "$degraded_count" == "$expected_degraded_skipped" ]]
  [[ "$next_step" == "$expected_next_step" ]]
  [[ "$status" == "$expected_status" ]]
  if [[ "$mode" == "strict" ]]; then
    [[ "$sent_count" == "2" ]]
    [[ "$first_failure" == "connect_failed" ]]
    [[ "$first_skip" == "none" ]]
  else
    [[ "$sent_count" == "1" ]]
    [[ "$first_failure" == "none" ]]
    [[ "$first_skip" == "connect_failed" ]]
  fi
}

run_case "strict" "$STRICT_LOG" "remote_transport_failed" "1" "0" "production_transport_adapter"
run_case "degraded" "$DEGRADED_LOG" "requires_compact_candidate_receive" "0" "1" "compact_candidate_receive"

echo "strict_log=$STRICT_LOG"
echo "degraded_log=$DEGRADED_LOG"
echo "stage_e_fault_simulated_network_partition_passed=true"
echo "SPIRE Stage E simulated network partition PG18 fixture passed"
