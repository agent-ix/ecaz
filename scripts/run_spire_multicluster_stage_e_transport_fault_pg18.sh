#!/usr/bin/env bash
set -euo pipefail

# Transport Stage E fixture family.
#
# Supported cases:
#   connection_reset_mid_batch
#   local_cancel
#   local_statement_timeout
#   remote_backend_termination
#   remote_statement_timeout
#
# These rows fail in the production transport adapter after a remote connection
# has opened. The fixture drives the pg-test production transport probe helper,
# which runs real libpq work and then summarizes strict/degraded executor state.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
REMOTE_READY_PORT="${REMOTE_READY_PORT:-39324}"
COORD_PORT="${COORD_PORT:-39325}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
LOG_DIR_OVERRIDE="${LOG_DIR:-}"
ARTIFACT_DIR=""
SMOKE_LOG="${SMOKE_LOG:-}"
FAULT_CASE=""

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh --case CASE [options]

Cases:
  connection_reset_mid_batch
  local_cancel
  local_statement_timeout
  remote_backend_termination
  remote_statement_timeout

Options:
  --artifact-dir DIR       Store fixture and PostgreSQL logs in DIR.
  --case CASE              Stage E transport fault case.
  --coord-port PORT        Coordinator PostgreSQL port. Default: 39325.
  --log-dir DIR            Store PostgreSQL logs in DIR.
  --pgbin DIR              PostgreSQL bin directory. Default: $PGBIN.
  --remote-ready-port PORT Ready remote PostgreSQL port. Default: 39324.
  --run-dir DIR            Run directory. Default: target/spire-stage-e-transport-fault-pg18-$RUN_ID.
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

if [[ "$FAULT_CASE" != "connection_reset_mid_batch" \
  && "$FAULT_CASE" != "local_cancel" \
  && "$FAULT_CASE" != "local_statement_timeout" \
  && "$FAULT_CASE" != "remote_backend_termination" \
  && "$FAULT_CASE" != "remote_statement_timeout" ]]; then
  echo "unsupported or missing --case: ${FAULT_CASE:-<none>}" >&2
  usage >&2
  exit 2
fi

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-stage-e-transport-fault-pg18-$RUN_ID}"
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

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_STAGE_E_TRANSPORT_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_STAGE_E_TRANSPORT_LOG_ACTIVE=1
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

export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_TRANSPORT_FAULT="host=$SOCKET_DIR port=$REMOTE_READY_PORT dbname=postgres user=postgres connect_timeout=1"
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_TRANSPORT_READY="$EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_TRANSPORT_FAULT"

"$PG_CTL" -w -D "$REMOTE_READY_DATA" -l "$LOG_DIR/remote-ready-postgres.log" \
  -o "-p $REMOTE_READY_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null
"$PG_CTL" -w -D "$COORD_DATA" -l "$LOG_DIR/coord-postgres.log" \
  -o "-p $COORD_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null

remote_ready_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE_READY_PORT" -U postgres -d postgres)
coord_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$COORD_PORT" -U postgres -d postgres)

"${remote_ready_psql[@]}" -c "SELECT 1" >/dev/null
"${coord_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null

matrix_row="$("${coord_psql[@]}" -At -F ',' -c "SELECT fault_case, failure_category, strict_action, strict_status, degraded_action, degraded_status, counter_delta FROM ec_spire_remote_search_stage_e_fault_matrix() WHERE fault_case = '$FAULT_CASE'")"

fault_failure_category="remote_statement_timeout"
fault_injection="ec_spire.remote_search_statement_timeout_ms=25 fault_node_sql=pg_sleep(0.30)"
pgoptions=""
if [[ "$FAULT_CASE" == "remote_statement_timeout" ]]; then
  pgoptions="-c ec_spire.remote_search_statement_timeout_ms=25"
elif [[ "$FAULT_CASE" == "remote_backend_termination" ]]; then
  fault_failure_category="remote_backend_terminated"
  fault_injection="fault_node_sql=pg_terminate_backend(pg_backend_pid())"
elif [[ "$FAULT_CASE" == "connection_reset_mid_batch" ]]; then
  fault_failure_category="remote_backend_terminated"
  fault_injection="fault_node_sql=generate_series_first_row_then_pg_terminate_backend"
else
  fault_failure_category="local_query_cancelled"
  fault_injection="local_cancel_after_ms=25 all_remote_sql=pg_sleep(0.30)"
fi
if [[ "$FAULT_CASE" == "local_statement_timeout" ]]; then
  fault_failure_category="local_statement_timeout"
  fault_injection="statement_timeout_after_ms=1 all_remote_sql=pg_sleep(0.30)"
fi

run_case() {
  local mode="$1"
  local output_log="$2"
  local expected_status="$3"
  local expected_transport_failed="$4"
  local expected_degraded_skipped="$5"
  local expected_next_step="$6"
  local expected_first_failure="$7"
  local expected_first_skip="$8"
  local expected_cancelled_dispatches="${9:-0}"
  local expected_first_cancellation="${10:-none}"
  local raw_rows
  local summary

  if [[ "$FAULT_CASE" == "local_cancel" ]]; then
    raw_rows="$("${coord_psql[@]}" -At -F ',' -c "SELECT node_id, status, failure_category, row_count FROM tests.ec_spire_test_production_transport_probe_local_cancel(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/transport_fault','spire/remote/stage_e/transport_ready']::text[], 25) ORDER BY node_id")"
    summary="$("${coord_psql[@]}" -At -F ',' -c "SELECT state_model, dispatch_count, transport_sent_dispatch_count, transport_ready_dispatch_count, transport_failed_dispatch_count, candidate_receive_pending_dispatch_count, cancelled_dispatch_count, first_cancellation_category, degraded_skipped_dispatch_count, first_degraded_skip_category, next_executor_step, status FROM tests.ec_spire_test_production_transport_probe_local_cancel_summary(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/transport_fault','spire/remote/stage_e/transport_ready']::text[], 25, '$mode')")"
  elif [[ "$FAULT_CASE" == "local_statement_timeout" ]]; then
    raw_rows="$("${coord_psql[@]}" -At -F ',' -c "SELECT node_id, status, failure_category, row_count FROM tests.ec_spire_test_prod_transport_stmt_timeout(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/transport_fault','spire/remote/stage_e/transport_ready']::text[], 1) ORDER BY node_id")"
    summary="$("${coord_psql[@]}" -At -F ',' -c "SELECT state_model, dispatch_count, transport_sent_dispatch_count, transport_ready_dispatch_count, transport_failed_dispatch_count, candidate_receive_pending_dispatch_count, cancelled_dispatch_count, first_cancellation_category, degraded_skipped_dispatch_count, first_degraded_skip_category, next_executor_step, status FROM tests.ec_spire_test_prod_transport_stmt_timeout_summary(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/transport_fault','spire/remote/stage_e/transport_ready']::text[], 1, '$mode')")"
  elif [[ -n "$pgoptions" ]]; then
    raw_rows="$(PGOPTIONS="$pgoptions" "${coord_psql[@]}" -At -F ',' -c "SELECT node_id, status, failure_category, row_count FROM tests.ec_spire_test_production_transport_probe_case(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/transport_fault','spire/remote/stage_e/transport_ready']::text[], 2, '$FAULT_CASE') ORDER BY node_id")"
    summary="$(PGOPTIONS="$pgoptions" "${coord_psql[@]}" -At -F ',' -c "SELECT state_model, dispatch_count, transport_sent_dispatch_count, transport_ready_dispatch_count, transport_failed_dispatch_count, first_transport_failure_category, candidate_receive_pending_dispatch_count, degraded_skipped_dispatch_count, first_degraded_skip_category, next_executor_step, status FROM tests.ec_spire_test_production_transport_probe_case_summary(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/transport_fault','spire/remote/stage_e/transport_ready']::text[], 2, '$FAULT_CASE', '$mode')")"
  else
    raw_rows="$("${coord_psql[@]}" -At -F ',' -c "SELECT node_id, status, failure_category, row_count FROM tests.ec_spire_test_production_transport_probe_case(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/transport_fault','spire/remote/stage_e/transport_ready']::text[], 2, '$FAULT_CASE') ORDER BY node_id")"
    summary="$("${coord_psql[@]}" -At -F ',' -c "SELECT state_model, dispatch_count, transport_sent_dispatch_count, transport_ready_dispatch_count, transport_failed_dispatch_count, first_transport_failure_category, candidate_receive_pending_dispatch_count, degraded_skipped_dispatch_count, first_degraded_skip_category, next_executor_step, status FROM tests.ec_spire_test_production_transport_probe_case_summary(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/transport_fault','spire/remote/stage_e/transport_ready']::text[], 2, '$FAULT_CASE', '$mode')")"
  fi

  {
    echo "matrix_row=$matrix_row"
    echo "injection=$fault_injection"
    echo "query_command=tests.ec_spire_test_production_transport_probe_case_summary(..., '$mode')"
    echo "expected_status=$expected_status"
    echo "expected_transport_failed_dispatch_count=$expected_transport_failed"
    echo "expected_degraded_skipped_dispatch_count=$expected_degraded_skipped"
    echo "expected_cancelled_dispatch_count=$expected_cancelled_dispatches"
    echo "expected_first_transport_failure_category=$expected_first_failure"
    echo "expected_first_cancellation_category=$expected_first_cancellation"
    echo "expected_first_degraded_skip_category=$expected_first_skip"
    echo "expected_next_executor_step=$expected_next_step"
    echo "observed_transport_rows=$raw_rows"
    echo "observed_summary=$summary"
  } | tee "$output_log"

  if [[ "$FAULT_CASE" == "local_cancel" || "$FAULT_CASE" == "local_statement_timeout" ]]; then
    IFS=, read -r state_model dispatch_count sent_count ready_count failed_count pending_count cancelled_count first_cancellation degraded_count first_skip next_step status <<< "$summary"
    first_failure="none"
  else
    IFS=, read -r state_model dispatch_count sent_count ready_count failed_count first_failure pending_count degraded_count first_skip next_step status <<< "$summary"
    cancelled_count="0"
    first_cancellation="none"
  fi
  [[ "$state_model" == "spire_remote_fanout_executor_v1" ]]
  [[ "$dispatch_count" == "2" ]]
  if [[ "$FAULT_CASE" == "local_cancel" || "$FAULT_CASE" == "local_statement_timeout" ]]; then
    [[ "$ready_count" == "0" ]]
  else
    [[ "$ready_count" == "1" ]]
  fi
  [[ "$failed_count" == "$expected_transport_failed" ]]
  [[ "$first_failure" == "$expected_first_failure" ]]
  if [[ "$FAULT_CASE" == "local_cancel" || "$FAULT_CASE" == "local_statement_timeout" ]]; then
    [[ "$pending_count" == "0" ]]
  else
    [[ "$pending_count" == "1" ]]
  fi
  [[ "$cancelled_count" == "$expected_cancelled_dispatches" ]]
  [[ "$first_cancellation" == "$expected_first_cancellation" ]]
  [[ "$degraded_count" == "$expected_degraded_skipped" ]]
  [[ "$first_skip" == "$expected_first_skip" ]]
  [[ "$next_step" == "$expected_next_step" ]]
  [[ "$status" == "$expected_status" ]]
  if [[ "$FAULT_CASE" == "local_cancel" || "$FAULT_CASE" == "local_statement_timeout" ]]; then
    [[ "$sent_count" == "0" ]]
  elif [[ "$mode" == "strict" ]]; then
    [[ "$sent_count" == "2" ]]
  else
    [[ "$sent_count" == "1" ]]
  fi
}

if [[ "$FAULT_CASE" == "local_cancel" || "$FAULT_CASE" == "local_statement_timeout" ]]; then
  run_case "strict" "$STRICT_LOG" "remote_executor_cancelled" "0" "0" "remote_executor_cancellation" "none" "none" "2" "$fault_failure_category"
  run_case "degraded" "$DEGRADED_LOG" "remote_executor_cancelled" "0" "0" "remote_executor_cancellation" "none" "none" "2" "$fault_failure_category"
else
  run_case "strict" "$STRICT_LOG" "remote_transport_failed" "1" "0" "production_transport_adapter" "$fault_failure_category" "none"
  run_case "degraded" "$DEGRADED_LOG" "requires_compact_candidate_receive" "0" "1" "compact_candidate_receive" "none" "$fault_failure_category"
fi

echo "strict_log=$STRICT_LOG"
echo "degraded_log=$DEGRADED_LOG"
echo "stage_e_fault_${FAULT_CASE}_passed=true"
echo "SPIRE Stage E $FAULT_CASE PG18 fixture passed"
