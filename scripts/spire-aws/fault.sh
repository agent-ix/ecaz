#!/usr/bin/env bash
# Phase 13b.10 — one fault drill per invocation.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

DRILL="${1:?drill name required}"
TOPOLOGY="${2:?topology JSON path required}"
ARTIFACT_DIR="${3:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

REGION=$(jq -r '.region' "$TOPOLOGY")
COORD_HOST=$(jq -r '.coordinator.operator_host // .coordinator.private_ip' "$TOPOLOGY")
COORD_PORT=$(jq -r '.coordinator.operator_port // 5432' "$TOPOLOGY")
PREFIX="${PREFIX:-ec_spire_aws_repr_1m}"
COORD_INDEX="${COORD_INDEX:-${PREFIX}_idx}"
REMOTE_INDEX="${REMOTE_INDEX:-${PREFIX}_remote_idx}"
TARGET_REMOTE_ID=$(jq -r '.remotes[0].instance_id' "$TOPOLOGY")
TARGET_NODE_ID=$(jq -r '.remotes[0].node_id' "$TOPOLOGY")
TARGET_SECRET=$(jq -r '.remotes[0].secret_arn' "$TOPOLOGY")
TARGET_REMOTE_HOST=$(jq -r '.remotes[0].operator_host // .remotes[0].private_ip' "$TOPOLOGY")
TARGET_REMOTE_PORT=$(jq -r '.remotes[0].operator_port // 5432' "$TOPOLOGY")
LOG="$ARTIFACT_DIR/fault-${DRILL}.log"
ECAZ_BIN="${ECAZ_BIN:-ecaz}"
FAULT_NPROBE="${SPIRE_AWS_FAULT_NPROBE:-100}"
REMOTE_SQL_READY_TIMEOUT_SECONDS="${SPIRE_AWS_REMOTE_SQL_READY_TIMEOUT_SECONDS:-300}"
REMOTE_STOPPED=0
RESTORE_STRICT_ON_EXIT=0
QUERY_VECTOR_LITERAL=""

wait_remote_sql_ready() {
  local deadline attempt status
  deadline=$((SECONDS + REMOTE_SQL_READY_TIMEOUT_SECONDS))
  attempt=0
  while (( SECONDS < deadline )); do
    attempt=$((attempt + 1))
    status=0
    "$ECAZ_BIN" dev sql \
      --host "$TARGET_REMOTE_HOST" --port "$TARGET_REMOTE_PORT" --user ecaz_coord --database postgres \
      --sql "SELECT 1" \
      --log-output "$ARTIFACT_DIR/fault-${DRILL}-remote-${TARGET_NODE_ID}-sql-ready-attempt-${attempt}.log" \
      >/dev/null 2>&1 || status=$?
    if [[ "$status" -eq 0 ]]; then
      echo "remote node ${TARGET_NODE_ID} SQL ready after ${attempt} attempt(s)" | tee -a "$LOG"
      return 0
    fi
    sleep 5
  done

  echo "remote node ${TARGET_NODE_ID} SQL did not become ready within ${REMOTE_SQL_READY_TIMEOUT_SECONDS}s" | tee -a "$LOG" >&2
  return 1
}

restart_remote_if_needed() {
  if [[ "$REMOTE_STOPPED" == "1" ]]; then
    aws ec2 start-instances --region "$REGION" --instance-ids "$TARGET_REMOTE_ID" | tee -a "$LOG"
    aws ec2 wait instance-running --region "$REGION" --instance-ids "$TARGET_REMOTE_ID"
    wait_remote_sql_ready
    REMOTE_STOPPED=0
  fi
}

publish_coord_consistency_mode() {
  local mode="${1:?mode required}"
  "$ECAZ_BIN" dev sql \
    --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
    --sql "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${COORD_INDEX}'::regclass::oid, '$mode')" \
    --log-output "$ARTIFACT_DIR/fault-${DRILL}-publish-${mode}.log"

  jq -c '.remotes[]' "$TOPOLOGY" | while read -r remote; do
    local remote_node_id remote_host remote_port
    remote_node_id="$(jq -r '.node_id' <<< "$remote")"
    remote_host="$(jq -r '.operator_host // .private_ip' <<< "$remote")"
    remote_port="$(jq -r '.operator_port // 5432' <<< "$remote")"
    "$ECAZ_BIN" dev sql \
      --host "$remote_host" --port "$remote_port" --user ecaz_coord --database postgres \
      --sql "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${REMOTE_INDEX}'::regclass::oid, '$mode')" \
      --log-output "$ARTIFACT_DIR/fault-${DRILL}-publish-node-${remote_node_id}-${mode}.log"
  done
}

cleanup() {
  local status=0
  restart_remote_if_needed || status=$?
  if [[ "$RESTORE_STRICT_ON_EXIT" == "1" ]]; then
    publish_coord_consistency_mode strict || status=$?
    RESTORE_STRICT_ON_EXIT=0
  fi
  return "$status"
}

trap cleanup EXIT

stop_remote() {
  aws ec2 stop-instances --region "$REGION" --instance-ids "$TARGET_REMOTE_ID" | tee -a "$LOG"
  aws ec2 wait instance-stopped --region "$REGION" --instance-ids "$TARGET_REMOTE_ID"
  REMOTE_STOPPED=1
}

query_vector_literal() {
  if [[ -n "$QUERY_VECTOR_LITERAL" ]]; then
    printf '%s\n' "$QUERY_VECTOR_LITERAL"
    return 0
  fi

  local raw literal
  raw="$("$ECAZ_BIN" dev sql \
    --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
    --sql "SELECT 'ARRAY[' || array_to_string(source, ',') || ']::real[]' FROM ${PREFIX}_queries WHERE id = 0" \
    --log-output "$ARTIFACT_DIR/fault-${DRILL}-query-vector.log")"
  literal="$(printf '%s\n' "$raw" | tr -d '\r' | sed -n '/^ARRAY\[/p' | head -n 1)"
  if [[ -z "$literal" ]]; then
    echo "failed to render finite query vector literal for ${PREFIX}_queries id=0" | tee -a "$LOG" >&2
    return 1
  fi
  QUERY_VECTOR_LITERAL="$literal"
  printf '%s\n' "$QUERY_VECTOR_LITERAL"
}

run_knn_query() {
  local mode="${1:?mode required}"
  local query_vector
  query_vector="$(query_vector_literal)"
  "$ECAZ_BIN" dev sql \
    --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
    --env "PGOPTIONS=-c enable_seqscan=off -c enable_indexscan=off -c ec_spire.nprobe=$FAULT_NPROBE -c ec_spire.remote_search_consistency_mode=$mode" \
    --sql "SELECT id FROM ${PREFIX}_corpus ORDER BY embedding <#> ${query_vector} LIMIT 10" \
    --log-output "$ARTIFACT_DIR/fault-${DRILL}-knn-${mode}.log"
}

snapshot_diag() {
  local mode="${1:?mode required}"
  local query_vector
  query_vector="$(query_vector_literal)"
  "$ECAZ_BIN" dev sql \
    --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
    --env "PGOPTIONS=-c enable_seqscan=off -c enable_indexscan=off -c ec_spire.nprobe=$FAULT_NPROBE -c ec_spire.remote_search_consistency_mode=$mode" \
    --sql "SELECT * FROM ec_spire_remote_search_production_read_profile('${COORD_INDEX}'::regclass, ${query_vector}, 10)" \
    --log-output "$ARTIFACT_DIR/fault-${DRILL}-session-summary.log"
  "$ECAZ_BIN" dev sql \
    --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
    --sql "SELECT * FROM ec_spire_index_active_snapshot_diagnostics('${COORD_INDEX}'::regclass)" \
    --log-output "$ARTIFACT_DIR/fault-${DRILL}-placement.log"
}

assert_degraded_ready() {
  local query_vector
  query_vector="$(query_vector_literal)"
  "$ECAZ_BIN" dev sql \
    --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
    --env "PGOPTIONS=-c enable_seqscan=off -c enable_indexscan=off -c ec_spire.nprobe=$FAULT_NPROBE -c ec_spire.remote_search_consistency_mode=degraded" \
    --sql "WITH profile AS (SELECT metric, value FROM ec_spire_remote_search_production_read_profile('${COORD_INDEX}'::regclass, ${query_vector}, 10)), summary AS (SELECT max(value) FILTER (WHERE metric = 'status') AS status, max(value) FILTER (WHERE metric = 'degraded_skipped_dispatch_count')::int AS skipped, max(value) FILTER (WHERE metric = 'returned_candidate_count')::int AS returned, max(value) FILTER (WHERE metric = 'next_blocker') AS next_blocker FROM profile) SELECT CASE WHEN status = 'degraded_ready' AND skipped > 0 AND returned > 0 AND next_blocker = 'none' THEN 'degraded_ok' ELSE (1 / COALESCE(NULLIF(skipped - skipped, 0), 0))::text END FROM summary" \
    --log-output "$ARTIFACT_DIR/fault-${DRILL}-assertion.log"
}

assert_strict_fails() {
  local status=0
  run_knn_query strict || status=$?
  if [[ "$status" -eq 0 ]]; then
    echo "strict fault drill unexpectedly succeeded with remote node ${TARGET_NODE_ID} stopped" | tee -a "$LOG" >&2
    return 1
  fi
  echo "strict fault drill failed closed as expected for node_id=${TARGET_NODE_ID}" | tee -a "$LOG"
}

case "$DRILL" in
  degraded)
    publish_coord_consistency_mode degraded
    RESTORE_STRICT_ON_EXIT=1
    stop_remote
    run_knn_query degraded
    snapshot_diag degraded
    assert_degraded_ready
    restart_remote_if_needed
    publish_coord_consistency_mode strict
    RESTORE_STRICT_ON_EXIT=0
    ;;
  strict)
    publish_coord_consistency_mode strict
    stop_remote
    assert_strict_fails
    snapshot_diag strict || true
    restart_remote_if_needed
    ;;
  orphaned-2pc)
    "$ECAZ_BIN" dev sql --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
      --file scripts/spire-aws/inject-2pc-orphan.sql \
      --set "prefix=ec_spire_aws_repr_1m" \
      --log-output "$ARTIFACT_DIR/fault-${DRILL}-inject.log"
    "$ECAZ_BIN" dev sql --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
      --sql "SELECT * FROM ec_spire_reap_orphaned_remote_prepared_xacts(${TARGET_NODE_ID})" \
      --log-output "$ARTIFACT_DIR/fault-${DRILL}-reap.log"
    snapshot_diag strict
    ;;
  missing-guc)
    echo "operator step: SSM into remote, set max_prepared_transactions=0, restart PG, retry INSERT, restore" | tee "$LOG"
    ;;
  schema-drift)
    echo "operator step: ALTER non-embedding column on one side, re-run write, observe fingerprint guard category, revert" | tee "$LOG"
    ;;
  auth-failure)
    aws secretsmanager put-secret-value --region "$REGION" \
      --secret-id "$TARGET_SECRET" --secret-string '{"password":"INVALID"}' | tee -a "$LOG"
    run_knn_query strict || true
    snapshot_diag strict || true
    echo "operator step: restore the prior secret version via aws secretsmanager restore-secret" | tee -a "$LOG"
    ;;
  *)
    echo "unknown drill: $DRILL" >&2; exit 2 ;;
esac
