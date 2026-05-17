#!/usr/bin/env bash
set -euo pipefail

# Lifecycle Stage E fixture family.
#
# Supported cases:
#   create_index_concurrently_missing_descriptor
#   create_index_concurrently_new_descriptor
#   drop_remote_index_before_fanout
#   drop_remote_index_in_flight
#   reindex_remote_index_before_fanout
#   reindex_remote_index_in_flight
#
# These rows exercise production libpq candidate receive after a remote DDL
# lifecycle event and prove strict/degraded handling matches the lifecycle
# matrix.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
REMOTE_READY_PORT="${REMOTE_READY_PORT:-39326}"
COORD_PORT="${COORD_PORT:-39327}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
LOG_DIR_OVERRIDE="${LOG_DIR:-}"
ARTIFACT_DIR=""
SMOKE_LOG="${SMOKE_LOG:-}"
LIFECYCLE_CASE=""

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh --case CASE [options]

Cases:
  create_index_concurrently_missing_descriptor
  create_index_concurrently_new_descriptor
  drop_remote_index_before_fanout
  drop_remote_index_in_flight
  reindex_remote_index_before_fanout
  reindex_remote_index_in_flight

Options:
  --artifact-dir DIR       Store fixture and PostgreSQL logs in DIR.
  --case CASE              Stage E lifecycle case.
  --coord-port PORT        Coordinator PostgreSQL port. Default: 39327.
  --log-dir DIR            Store PostgreSQL logs in DIR.
  --pgbin DIR              PostgreSQL bin directory. Default: $PGBIN.
  --remote-ready-port PORT Ready remote PostgreSQL port. Default: 39326.
  --run-dir DIR            Run directory. Default: target/spire-stage-e-lifecycle-pg18-$RUN_ID.
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
      LIFECYCLE_CASE="$2"
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

if [[ "$LIFECYCLE_CASE" != "create_index_concurrently_missing_descriptor" \
  && "$LIFECYCLE_CASE" != "create_index_concurrently_new_descriptor" \
  && "$LIFECYCLE_CASE" != "drop_remote_index_before_fanout" \
  && "$LIFECYCLE_CASE" != "drop_remote_index_in_flight" \
  && "$LIFECYCLE_CASE" != "reindex_remote_index_before_fanout" \
  && "$LIFECYCLE_CASE" != "reindex_remote_index_in_flight" ]]; then
  echo "unsupported or missing --case: ${LIFECYCLE_CASE:-<none>}" >&2
  usage >&2
  exit 2
fi

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-stage-e-lifecycle-pg18-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/stage_e_lifecycle_${LIFECYCLE_CASE}.log}"
else
  LOG_DIR="${LOG_DIR_OVERRIDE:-$RUN_DIR/logs}"
fi
REMOTE_READY_DATA="$RUN_DIR/remote-ready"
COORD_DATA="$RUN_DIR/coord"
SOCKET_KEY="$(printf '%s' "$RUN_DIR" | cksum | awk '{print $1}')"
SOCKET_DIR="${SOCKET_DIR:-$ROOT_DIR/target/s-$SOCKET_KEY}"
STRICT_LOG="${ARTIFACT_DIR:-$LOG_DIR}/stage_e_lifecycle_${LIFECYCLE_CASE}_strict.log"
DEGRADED_LOG="${ARTIFACT_DIR:-$LOG_DIR}/stage_e_lifecycle_${LIFECYCLE_CASE}_degraded.log"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_STAGE_E_LIFECYCLE_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_STAGE_E_LIFECYCLE_LOG_ACTIVE=1
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
echo "lifecycle_case=$LIFECYCLE_CASE"
echo "remote_ready_port=$REMOTE_READY_PORT"
echo "coord_port=$COORD_PORT"

if [[ "${ECAZ_SKIP_INSTALL:-0}" != "1" ]]; then
  (cd "$ROOT_DIR" && cargo pgrx install --test --pg-config "$PGBIN/pg_config" \
    --features "pg18 pg_test" --no-default-features)
fi

"$PG_CTL" initdb -D "$REMOTE_READY_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$COORD_DATA" -o "-A trust -U postgres" >/dev/null

export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_LIFECYCLE_DROPPED="host=$SOCKET_DIR port=$REMOTE_READY_PORT dbname=postgres user=postgres connect_timeout=1"
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_STAGE_E_LIFECYCLE_COORD_READY="host=$SOCKET_DIR port=$COORD_PORT dbname=postgres user=postgres connect_timeout=1"

"$PG_CTL" -w -D "$REMOTE_READY_DATA" -l "$LOG_DIR/remote-ready-postgres.log" \
  -o "-p $REMOTE_READY_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null
"$PG_CTL" -w -D "$COORD_DATA" -l "$LOG_DIR/coord-postgres.log" \
  -o "-p $COORD_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null

remote_ready_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE_READY_PORT" -U postgres -d postgres)
coord_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$COORD_PORT" -U postgres -d postgres)

"${remote_ready_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null
"${coord_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null

"${remote_ready_psql[@]}" <<'SQL' >/dev/null
CREATE TABLE ec_spire_stage_e_lifecycle_remote_sql
    (id bigint primary key, embedding ecvector);
INSERT INTO ec_spire_stage_e_lifecycle_remote_sql (id, embedding) VALUES
    (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
    (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)),
    (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)),
    (4, encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42));
SQL

"${coord_psql[@]}" <<'SQL' >/dev/null
CREATE TABLE ec_spire_stage_e_lifecycle_coord_sql
    (id bigint primary key, embedding ecvector);
INSERT INTO ec_spire_stage_e_lifecycle_coord_sql (id, embedding) VALUES
    (1, encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
    (2, encode_to_ecvector(ARRAY[0.0, 1.0], 4, 42)),
    (3, encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42)),
    (4, encode_to_ecvector(ARRAY[0.0, -1.0], 4, 42));
CREATE INDEX ec_spire_stage_e_lifecycle_coord_idx
    ON ec_spire_stage_e_lifecycle_coord_sql USING ec_spire
    (embedding ecvector_spire_ip_ops) WITH (nlists = 2, storage_format = 'rabitq');
SQL

remote_dropped_identity_hex=""
remote_reindexed_identity_hex=""
drop_regclass="f"

prepare_remote_dropped_index() {
  "${remote_ready_psql[@]}" <<'SQL' >/dev/null
DROP INDEX IF EXISTS ec_spire_stage_e_lifecycle_dropped_idx;
CREATE INDEX ec_spire_stage_e_lifecycle_dropped_idx
    ON ec_spire_stage_e_lifecycle_remote_sql USING ec_spire
    (embedding ecvector_spire_ip_ops) WITH (nlists = 2, storage_format = 'rabitq');
SQL
  remote_dropped_identity_hex="$("${remote_ready_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_lifecycle_dropped_idx'::regclass)")"
  drop_regclass="$("${remote_ready_psql[@]}" -At -c "SELECT to_regclass('ec_spire_stage_e_lifecycle_dropped_idx') IS NULL")"
  if [[ "$drop_regclass" != "f" ]]; then
    echo "expected prepared remote index to exist, got to_regclass null=$drop_regclass" >&2
    exit 3
  fi
}

prepare_remote_dropped_index_for_mode() {
  local mode="$1"
  prepare_remote_dropped_index
  if [[ "$mode" == "degraded" && "$LIFECYCLE_CASE" == "reindex_remote_index_in_flight" ]]; then
    "${remote_ready_psql[@]}" <<'SQL' >/dev/null
SELECT tests.ec_spire_test_rewrite_consistency_mode(
    'ec_spire_stage_e_lifecycle_dropped_idx'::regclass::oid,
    'degraded'
);
SQL
    remote_dropped_identity_hex="$("${remote_ready_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_lifecycle_dropped_idx'::regclass)")"
  fi
}

prepare_remote_dropped_index
coord_ready_identity_hex="$("${coord_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_lifecycle_coord_idx'::regclass)")"
coord_ready_epoch="$("${coord_psql[@]}" -At -c "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('ec_spire_stage_e_lifecycle_coord_idx'::regclass)")"
extversion="$("${coord_psql[@]}" -At -c "SELECT extversion FROM pg_extension WHERE extname = 'ecaz'")"
coord_ready_pids="$("${coord_psql[@]}" -At -F ',' -c "SELECT string_agg(leaf_pid::text, ',' ORDER BY leaf_pid) FROM ec_spire_index_leaf_snapshot('ec_spire_stage_e_lifecycle_coord_idx'::regclass)")"
lifecycle_row="$("${coord_psql[@]}" -At -F ',' -c "SELECT lifecycle_case, ddl_event, fanout_timing, strict_action, strict_status, degraded_action, degraded_status, required_detection, next_executor_step FROM ec_spire_remote_search_stage_e_lifecycle_matrix() WHERE lifecycle_case = '$LIFECYCLE_CASE'")"

IFS=, read -r dropped_pid ready_pid extra_pid <<< "$coord_ready_pids"
if [[ -z "$dropped_pid" || -z "$ready_pid" || -n "${extra_pid:-}" ]]; then
  echo "expected exactly two coordinator ready leaf PIDs, got: $coord_ready_pids" >&2
  exit 3
fi

if [[ "$LIFECYCLE_CASE" == "drop_remote_index_before_fanout" ]]; then
  "${remote_ready_psql[@]}" -c "DROP INDEX ec_spire_stage_e_lifecycle_dropped_idx" >/dev/null
  drop_regclass="$("${remote_ready_psql[@]}" -At -c "SELECT to_regclass('ec_spire_stage_e_lifecycle_dropped_idx') IS NULL")"
  if [[ "$drop_regclass" != "t" ]]; then
    echo "expected dropped remote index to be absent, got to_regclass null=$drop_regclass" >&2
    exit 3
  fi
elif [[ "$LIFECYCLE_CASE" == "reindex_remote_index_before_fanout" ]]; then
  planned_reindex_identity_hex="$remote_dropped_identity_hex"
  "${remote_ready_psql[@]}" -c "REINDEX INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_dropped_idx" >/dev/null
  remote_reindexed_identity_hex="$("${remote_ready_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_lifecycle_dropped_idx'::regclass)")"
  drop_regclass="$("${remote_ready_psql[@]}" -At -c "SELECT to_regclass('ec_spire_stage_e_lifecycle_dropped_idx') IS NULL")"
  if [[ "$drop_regclass" != "f" ]]; then
    echo "expected reindexed remote index to exist, got to_regclass null=$drop_regclass" >&2
    exit 3
  fi
  if [[ "$remote_reindexed_identity_hex" == "$planned_reindex_identity_hex" ]]; then
    echo "expected REINDEX CONCURRENTLY to change endpoint identity, still got $remote_reindexed_identity_hex" >&2
    exit 3
  fi
  remote_dropped_identity_hex="$planned_reindex_identity_hex"
fi

run_missing_descriptor_case() {
  local mode="$1"
  local output_log="$2"
  local expected_status="$3"
  local expected_planned_dispatches="$4"
  local expected_blocked_before_dispatches="$5"
  local expected_degraded_skipped="$6"
  local expected_first_skip="$7"
  local expected_next_step="$8"

  if [[ "$mode" == "degraded" ]]; then
    "${coord_psql[@]}" -c "SELECT tests.ec_spire_test_rewrite_placement_node('ec_spire_stage_e_lifecycle_coord_idx'::regclass::oid, $dropped_pid, 0)" >/dev/null
    "${coord_psql[@]}" <<'SQL' >/dev/null
SELECT tests.ec_spire_test_rewrite_consistency_mode(
    'ec_spire_stage_e_lifecycle_coord_idx'::regclass::oid,
    'degraded'
);
SQL
    "${coord_psql[@]}" -c "SELECT tests.ec_spire_test_rewrite_placement_node('ec_spire_stage_e_lifecycle_coord_idx'::regclass::oid, $dropped_pid, 2)" >/dev/null
  fi

  local summary
  summary="$("${coord_psql[@]}" -At -F ',' -c "SELECT state_model, dispatch_count, planned_dispatch_count, blocked_before_dispatch_count, remote_pid_count, planned_pid_count, blocked_pid_count, conninfo_secret_lookup_count, socket_open_count, endpoint_identity_query_count, degraded_skipped_dispatch_count, first_degraded_skip_category, next_executor_step, status FROM ec_spire_remote_search_production_executor_state_summary('ec_spire_stage_e_lifecycle_coord_idx'::regclass, $coord_ready_epoch, ARRAY[1.0, 0.0]::real[], ARRAY[$dropped_pid,$ready_pid]::bigint[], 1, '$mode')")"
  local readiness
  readiness="$("${coord_psql[@]}" -At -F ',' -c "SELECT target_kind, node_id, descriptor_state, node_status, status FROM ec_spire_remote_search_request_readiness('ec_spire_stage_e_lifecycle_coord_idx'::regclass, $coord_ready_epoch, ARRAY[1.0, 0.0]::real[], ARRAY[$dropped_pid,$ready_pid]::bigint[], 1, '$mode') ORDER BY node_id, target_kind")"

  {
    echo "lifecycle_row=$lifecycle_row"
    echo "injection=CREATE INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_missing_descriptor_idx before descriptor registration"
    echo "created_index_to_regclass_is_not_null=$missing_descriptor_index_exists"
    echo "coord_remote_pid=$dropped_pid"
    echo "coord_local_pid=$ready_pid"
    echo "coord_ready_epoch=$coord_ready_epoch"
    echo "query_command=ec_spire_remote_search_production_executor_state_summary(..., '$mode')"
    echo "expected_status=$expected_status"
    echo "expected_planned_dispatch_count=$expected_planned_dispatches"
    echo "expected_blocked_before_dispatch_count=$expected_blocked_before_dispatches"
    echo "expected_degraded_skipped_dispatch_count=$expected_degraded_skipped"
    echo "expected_first_degraded_skip_category=$expected_first_skip"
    echo "expected_next_executor_step=$expected_next_step"
    echo "observed_request_readiness_rows=$readiness"
    echo "observed_summary=$summary"
  } | tee "$output_log"

  IFS=, read -r state_model dispatch_count planned_count blocked_count remote_pid_count planned_pid_count blocked_pid_count secret_count socket_count identity_count degraded_count first_skip next_step status <<< "$summary"
  [[ "$state_model" == "spire_remote_fanout_executor_v1" ]]
  [[ "$dispatch_count" == "1" ]]
  [[ "$planned_count" == "$expected_planned_dispatches" ]]
  [[ "$blocked_count" == "$expected_blocked_before_dispatches" ]]
  [[ "$remote_pid_count" == "1" ]]
  [[ "$secret_count" == "0" ]]
  [[ "$socket_count" == "0" ]]
  [[ "$identity_count" == "0" ]]
  [[ "$degraded_count" == "$expected_degraded_skipped" ]]
  [[ "$first_skip" == "$expected_first_skip" ]]
  [[ "$next_step" == "$expected_next_step" ]]
  [[ "$status" == "$expected_status" ]]
}

if [[ "$LIFECYCLE_CASE" == "create_index_concurrently_missing_descriptor" ]]; then
  "${remote_ready_psql[@]}" -c "CREATE INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_missing_descriptor_idx ON ec_spire_stage_e_lifecycle_remote_sql USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2, storage_format = 'rabitq')" >/dev/null
  missing_descriptor_index_exists="$("${remote_ready_psql[@]}" -At -c "SELECT to_regclass('ec_spire_stage_e_lifecycle_missing_descriptor_idx') IS NOT NULL")"
  if [[ "$missing_descriptor_index_exists" != "t" ]]; then
    echo "expected CREATE INDEX CONCURRENTLY target to exist, got to_regclass not null=$missing_descriptor_index_exists" >&2
    exit 3
  fi
  "${coord_psql[@]}" -c "SELECT tests.ec_spire_test_rewrite_placement_node('ec_spire_stage_e_lifecycle_coord_idx'::regclass::oid, $dropped_pid, 2)" >/dev/null

  run_missing_descriptor_case "strict" "$STRICT_LOG" "requires_remote_node_descriptor" "0" "1" "0" "none" "remote_node_descriptor"
  run_missing_descriptor_case "degraded" "$DEGRADED_LOG" "degraded_skipped" "1" "0" "1" "requires_remote_node_descriptor" "remote_heap_resolution"

  echo "strict_log=$STRICT_LOG"
  echo "degraded_log=$DEGRADED_LOG"
  echo "stage_e_lifecycle_${LIFECYCLE_CASE}_passed=true"
  echo "SPIRE Stage E lifecycle $LIFECYCLE_CASE PG18 fixture passed"
  exit 0
fi

run_new_descriptor_case() {
  local mode="$1"
  local output_log="$2"
  local descriptor_generation="$3"
  local new_index_name="$4"
  local case_coord_identity="$coord_ready_identity_hex"

  prepare_remote_dropped_index
  local case_old_identity_hex="$remote_dropped_identity_hex"
  if [[ "$mode" == "degraded" ]]; then
    "${remote_ready_psql[@]}" <<'SQL' >/dev/null
SELECT tests.ec_spire_test_rewrite_consistency_mode(
    'ec_spire_stage_e_lifecycle_dropped_idx'::regclass::oid,
    'degraded'
);
SQL
    case_old_identity_hex="$("${remote_ready_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_lifecycle_dropped_idx'::regclass)")"
    "${coord_psql[@]}" <<'SQL' >/dev/null
SELECT tests.ec_spire_test_rewrite_consistency_mode(
    'ec_spire_stage_e_lifecycle_coord_idx'::regclass::oid,
    'degraded'
);
SQL
    case_coord_identity="$("${coord_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_lifecycle_coord_idx'::regclass)")"
  fi

  "${coord_psql[@]}" -c "SELECT ec_spire_register_remote_node_descriptor('ec_spire_stage_e_lifecycle_coord_idx'::regclass::oid, 2, $((descriptor_generation - 1)), 'spire/remote/stage_e/lifecycle/dropped', decode('$case_old_identity_hex', 'hex'), 'ec_spire_stage_e_lifecycle_dropped_idx', 'active', $coord_ready_epoch, $coord_ready_epoch, '$extversion', 'none')" >/dev/null

  local remote_sql
  remote_sql="CREATE INDEX CONCURRENTLY $new_index_name ON ec_spire_stage_e_lifecycle_remote_sql USING ec_spire (embedding ecvector_spire_ip_ops) WITH (nlists = 2, storage_format = 'rabitq')"
  local summary
  summary="$("${coord_psql[@]}" -At -F ',' -c "SELECT state_model, dispatch_count, candidate_receive_sent_dispatch_count, candidate_receive_ready_dispatch_count, candidate_receive_failed_dispatch_count, first_candidate_receive_failure_category, candidate_row_count, degraded_skipped_dispatch_count, first_degraded_skip_category, next_executor_step, status FROM tests.ec_spire_test_prod_receive_after_remote_descriptor_summary(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/lifecycle/dropped','spire/remote/stage_e/lifecycle/coord_ready']::text[], ARRAY['ec_spire_stage_e_lifecycle_dropped_idx','ec_spire_stage_e_lifecycle_coord_idx']::text[], ARRAY['$case_old_identity_hex','$case_coord_identity']::text[], ARRAY[$dropped_pid,$ready_pid]::bigint[], $coord_ready_epoch, ARRAY[1.0, 0.0]::real[], 1, '$mode', 'spire/remote/stage_e/lifecycle/dropped', \$\$$remote_sql\$\$, 'ec_spire_stage_e_lifecycle_coord_idx'::regclass::oid, 2, $descriptor_generation, 'spire/remote/stage_e/lifecycle/dropped', '$new_index_name', 'active', $coord_ready_epoch, $coord_ready_epoch, '$extversion', 'none')")"
  local new_identity_hex
  new_identity_hex="$("${remote_ready_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('$new_index_name'::regclass)")"
  local descriptor_row
  descriptor_row="$("${coord_psql[@]}" -At -F ',' -c "SELECT node_id, descriptor_generation, remote_index_regclass, encode(remote_index_identity, 'hex'), descriptor_state FROM ec_spire_remote_node_descriptor WHERE coordinator_index_oid = 'ec_spire_stage_e_lifecycle_coord_idx'::regclass::oid AND node_id = 2")"

  {
    echo "lifecycle_row=$lifecycle_row"
    echo "injection=CREATE INDEX CONCURRENTLY $new_index_name after request construction before receive; register descriptor_generation=$descriptor_generation before receive"
    echo "old_descriptor_index=ec_spire_stage_e_lifecycle_dropped_idx"
    echo "old_descriptor_identity=$case_old_identity_hex"
    echo "new_descriptor_index=$new_index_name"
    echo "new_descriptor_identity=$new_identity_hex"
    echo "coord_ready_identity=$case_coord_identity"
    echo "coord_ready_epoch=$coord_ready_epoch"
    echo "coord_ready_pids=$coord_ready_pids"
    echo "query_command=tests.ec_spire_test_prod_receive_after_remote_descriptor_summary(..., '$mode')"
    echo "expected_status=requires_remote_heap_resolution"
    echo "expected_candidate_receive_ready_dispatch_count=2"
    echo "expected_candidate_receive_failed_dispatch_count=0"
    echo "expected_degraded_skipped_dispatch_count=0"
    echo "expected_next_executor_step=remote_heap_resolution"
    echo "observed_descriptor_row=$descriptor_row"
    echo "observed_summary=$summary"
  } | tee "$output_log"

  IFS=, read -r state_model dispatch_count sent_count ready_receive_count failed_receive_count first_failure candidate_row_count degraded_count first_skip next_step status <<< "$summary"
  [[ "$state_model" == "spire_remote_fanout_executor_v1" ]]
  [[ "$dispatch_count" == "2" ]]
  [[ "$sent_count" == "2" ]]
  [[ "$ready_receive_count" == "2" ]]
  [[ "$failed_receive_count" == "0" ]]
  [[ "$first_failure" == "none" ]]
  [[ "$candidate_row_count" == "2" ]]
  [[ "$degraded_count" == "0" ]]
  [[ "$first_skip" == "none" ]]
  [[ "$next_step" == "remote_heap_resolution" ]]
  [[ "$status" == "requires_remote_heap_resolution" ]]
  [[ "$new_identity_hex" != "$case_old_identity_hex" ]]
}

if [[ "$LIFECYCLE_CASE" == "create_index_concurrently_new_descriptor" ]]; then
  run_new_descriptor_case "strict" "$STRICT_LOG" "11" "ec_spire_stage_e_lifecycle_new_descriptor_strict_idx"
  run_new_descriptor_case "degraded" "$DEGRADED_LOG" "21" "ec_spire_stage_e_lifecycle_new_descriptor_degraded_idx"

  echo "strict_log=$STRICT_LOG"
  echo "degraded_log=$DEGRADED_LOG"
  echo "stage_e_lifecycle_${LIFECYCLE_CASE}_passed=true"
  echo "SPIRE Stage E lifecycle $LIFECYCLE_CASE PG18 fixture passed"
  exit 0
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
    if [[ "$LIFECYCLE_CASE" == "reindex_remote_index_before_fanout" ]]; then
      "${remote_ready_psql[@]}" <<'SQL' >/dev/null
SELECT tests.ec_spire_test_rewrite_consistency_mode(
    'ec_spire_stage_e_lifecycle_dropped_idx'::regclass::oid,
    'degraded'
);
SQL
      remote_reindexed_identity_hex="$("${remote_ready_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_lifecycle_dropped_idx'::regclass)")"
    fi
    "${coord_psql[@]}" <<'SQL' >/dev/null
SELECT tests.ec_spire_test_rewrite_consistency_mode(
    'ec_spire_stage_e_lifecycle_coord_idx'::regclass::oid,
    'degraded'
);
SQL
    case_coord_identity="$("${coord_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_lifecycle_coord_idx'::regclass)")"
  fi

  local injection
  local query_command
  local raw_rows
  local summary

  if [[ "$LIFECYCLE_CASE" == "drop_remote_index_in_flight" \
    || "$LIFECYCLE_CASE" == "reindex_remote_index_in_flight" ]]; then
    local remote_sql
    local remote_sql_label
    local expected_live_identity_changed="0"
    if [[ "$LIFECYCLE_CASE" == "reindex_remote_index_in_flight" ]]; then
      remote_sql="REINDEX INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_dropped_idx"
      remote_sql_label="$remote_sql after request construction before receive"
      expected_live_identity_changed="1"
    else
      remote_sql="DROP INDEX ec_spire_stage_e_lifecycle_dropped_idx"
      remote_sql_label="$remote_sql after request construction before receive"
    fi
    injection="$remote_sql_label"
    query_command="tests.ec_spire_test_prod_receive_after_remote_sql_summary(..., '$mode')"
    prepare_remote_dropped_index_for_mode "$mode"
    local raw_identity="$remote_dropped_identity_hex"
    raw_rows="$("${coord_psql[@]}" -At -F ',' -c "SELECT node_id, status, failure_category, candidate_count FROM tests.ec_spire_test_prod_receive_after_remote_sql(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/lifecycle/dropped','spire/remote/stage_e/lifecycle/coord_ready']::text[], ARRAY['ec_spire_stage_e_lifecycle_dropped_idx','ec_spire_stage_e_lifecycle_coord_idx']::text[], ARRAY['$raw_identity','$case_coord_identity']::text[], ARRAY[$dropped_pid,$ready_pid]::bigint[], $coord_ready_epoch, ARRAY[1.0, 0.0]::real[], 1, '$mode', 'spire/remote/stage_e/lifecycle/dropped', \$\$$remote_sql\$\$) ORDER BY node_id")"
    if [[ "$expected_live_identity_changed" == "1" ]]; then
      remote_reindexed_identity_hex="$("${remote_ready_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_lifecycle_dropped_idx'::regclass)")"
      if [[ "$remote_reindexed_identity_hex" == "$raw_identity" ]]; then
        echo "expected in-flight REINDEX CONCURRENTLY to change endpoint identity, still got $remote_reindexed_identity_hex" >&2
        exit 3
      fi
    fi
    prepare_remote_dropped_index_for_mode "$mode"
    local summary_identity="$remote_dropped_identity_hex"
    summary="$("${coord_psql[@]}" -At -F ',' -c "SELECT state_model, dispatch_count, candidate_receive_sent_dispatch_count, candidate_receive_ready_dispatch_count, candidate_receive_failed_dispatch_count, first_candidate_receive_failure_category, candidate_row_count, degraded_skipped_dispatch_count, first_degraded_skip_category, next_executor_step, status FROM tests.ec_spire_test_prod_receive_after_remote_sql_summary(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/lifecycle/dropped','spire/remote/stage_e/lifecycle/coord_ready']::text[], ARRAY['ec_spire_stage_e_lifecycle_dropped_idx','ec_spire_stage_e_lifecycle_coord_idx']::text[], ARRAY['$summary_identity','$case_coord_identity']::text[], ARRAY[$dropped_pid,$ready_pid]::bigint[], $coord_ready_epoch, ARRAY[1.0, 0.0]::real[], 1, '$mode', 'spire/remote/stage_e/lifecycle/dropped', \$\$$remote_sql\$\$)")"
    drop_regclass="$("${remote_ready_psql[@]}" -At -c "SELECT to_regclass('ec_spire_stage_e_lifecycle_dropped_idx') IS NULL")"
    if [[ "$expected_live_identity_changed" == "1" ]]; then
      remote_reindexed_identity_hex="$("${remote_ready_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_stage_e_lifecycle_dropped_idx'::regclass)")"
      if [[ "$remote_reindexed_identity_hex" == "$summary_identity" ]]; then
        echo "expected in-flight REINDEX CONCURRENTLY to change endpoint identity, still got $remote_reindexed_identity_hex" >&2
        exit 3
      fi
    elif [[ "$drop_regclass" != "t" ]]; then
      echo "expected in-flight dropped remote index to be absent, got to_regclass null=$drop_regclass" >&2
      exit 3
    fi
  else
    if [[ "$LIFECYCLE_CASE" == "reindex_remote_index_before_fanout" ]]; then
      injection="REINDEX INDEX CONCURRENTLY ec_spire_stage_e_lifecycle_dropped_idx before fanout"
    else
      injection="DROP INDEX ec_spire_stage_e_lifecycle_dropped_idx before fanout"
    fi
    query_command="tests.ec_spire_test_production_candidate_receive_summary(..., '$mode')"
    raw_rows="$("${coord_psql[@]}" -At -F ',' -c "SELECT node_id, status, failure_category, candidate_count FROM tests.ec_spire_test_production_candidate_receive(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/lifecycle/dropped','spire/remote/stage_e/lifecycle/coord_ready']::text[], ARRAY['ec_spire_stage_e_lifecycle_dropped_idx','ec_spire_stage_e_lifecycle_coord_idx']::text[], ARRAY['$remote_dropped_identity_hex','$case_coord_identity']::text[], ARRAY[$dropped_pid,$ready_pid]::bigint[], $coord_ready_epoch, ARRAY[1.0, 0.0]::real[], 1, '$mode') ORDER BY node_id")"
    summary="$("${coord_psql[@]}" -At -F ',' -c "SELECT state_model, dispatch_count, candidate_receive_sent_dispatch_count, candidate_receive_ready_dispatch_count, candidate_receive_failed_dispatch_count, first_candidate_receive_failure_category, candidate_row_count, degraded_skipped_dispatch_count, first_degraded_skip_category, next_executor_step, status FROM tests.ec_spire_test_production_candidate_receive_summary(ARRAY[2,3]::integer[], ARRAY['spire/remote/stage_e/lifecycle/dropped','spire/remote/stage_e/lifecycle/coord_ready']::text[], ARRAY['ec_spire_stage_e_lifecycle_dropped_idx','ec_spire_stage_e_lifecycle_coord_idx']::text[], ARRAY['$remote_dropped_identity_hex','$case_coord_identity']::text[], ARRAY[$dropped_pid,$ready_pid]::bigint[], $coord_ready_epoch, ARRAY[1.0, 0.0]::real[], 1, '$mode')")"
  fi

  {
    echo "lifecycle_row=$lifecycle_row"
    echo "injection=$injection"
    echo "dropped_index_to_regclass_is_null=$drop_regclass"
    echo "dropped_remote_identity_before_drop=$remote_dropped_identity_hex"
    echo "remote_reindexed_identity=$remote_reindexed_identity_hex"
    echo "coord_ready_identity=$case_coord_identity"
    echo "coord_ready_epoch=$coord_ready_epoch"
    echo "coord_ready_pids=$coord_ready_pids"
    echo "query_command=$query_command"
    echo "expected_status=$expected_status"
    echo "expected_candidate_receive_failed_dispatch_count=$expected_failed_receive"
    echo "expected_degraded_skipped_dispatch_count=$expected_degraded_skipped"
    echo "expected_first_candidate_receive_failure_category=$expected_first_failure"
    echo "expected_first_degraded_skip_category=$expected_first_skip"
    echo "expected_next_executor_step=$expected_next_step"
    echo "observed_candidate_receive_rows=$raw_rows"
    echo "observed_summary=$summary"
  } | tee "$output_log"

  IFS=, read -r state_model dispatch_count sent_count ready_receive_count failed_receive_count first_failure candidate_row_count degraded_count first_skip next_step status <<< "$summary"
  [[ "$state_model" == "spire_remote_fanout_executor_v1" ]]
  [[ "$dispatch_count" == "2" ]]
  [[ "$ready_receive_count" == "1" ]]
  [[ "$failed_receive_count" == "$expected_failed_receive" ]]
  [[ "$first_failure" == "$expected_first_failure" ]]
  [[ "$candidate_row_count" == "1" ]]
  [[ "$degraded_count" == "$expected_degraded_skipped" ]]
  [[ "$first_skip" == "$expected_first_skip" ]]
  [[ "$next_step" == "$expected_next_step" ]]
  [[ "$status" == "$expected_status" ]]
  if [[ "$mode" == "strict" ]]; then
    [[ "$sent_count" == "2" ]]
  else
    [[ "$sent_count" == "1" ]]
  fi
}

if [[ "$LIFECYCLE_CASE" == "reindex_remote_index_before_fanout" \
  || "$LIFECYCLE_CASE" == "reindex_remote_index_in_flight" ]]; then
  run_case "strict" "$STRICT_LOG" "remote_candidate_receive_failed" "1" "0" "compact_candidate_receive" "endpoint_identity_mismatch" "none"
  run_case "degraded" "$DEGRADED_LOG" "degraded_ready" "0" "1" "remote_heap_resolution" "none" "endpoint_identity_mismatch"
else
  run_case "strict" "$STRICT_LOG" "remote_candidate_receive_failed" "1" "0" "compact_candidate_receive" "remote_index_unavailable" "none"
  run_case "degraded" "$DEGRADED_LOG" "degraded_ready" "0" "1" "remote_heap_resolution" "none" "remote_index_unavailable"
fi

echo "strict_log=$STRICT_LOG"
echo "degraded_log=$DEGRADED_LOG"
echo "stage_e_lifecycle_${LIFECYCLE_CASE}_passed=true"
echo "SPIRE Stage E lifecycle $LIFECYCLE_CASE PG18 fixture passed"
