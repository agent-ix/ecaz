#!/usr/bin/env bash
# Phase 13b.10 — one fault drill per invocation.

set -euo pipefail

DRILL="${1:?drill name required}"
TOPOLOGY="${2:?topology JSON path required}"
ARTIFACT_DIR="${3:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

REGION=$(jq -r '.region' "$TOPOLOGY")
COORD_HOST=$(jq -r '.coordinator.private_ip' "$TOPOLOGY")
COORD_INDEX="${COORD_INDEX:-ec_spire_aws_repr_1m_idx}"
TARGET_REMOTE_ID=$(jq -r '.remotes[0].instance_id' "$TOPOLOGY")
TARGET_NODE_ID=$(jq -r '.remotes[0].node_id' "$TOPOLOGY")
TARGET_SECRET=$(jq -r '.remotes[0].secret_arn' "$TOPOLOGY")
LOG="$ARTIFACT_DIR/fault-${DRILL}.log"

run_query() {
  ecaz bench latency \
    --host "$COORD_HOST" --user ecaz_coord --database postgres \
    --prefix ec_spire_aws_repr_1m --profile ec_spire \
    --k 10 --sweep 32 --concurrency 1 --iterations 100 \
    --log-output "$ARTIFACT_DIR/fault-${DRILL}-bench.log" || true
}

snapshot_diag() {
  ecaz dev sql \
    --host "$COORD_HOST" --user ecaz_coord --database postgres \
    --sql "SELECT * FROM ec_spire_remote_search_production_executor_session_summary('${COORD_INDEX}'::regclass, 1, ARRAY[]::real[], ARRAY[]::bigint[], 10)" \
    --log-output "$ARTIFACT_DIR/fault-${DRILL}-session-summary.log"
  ecaz dev sql \
    --host "$COORD_HOST" --user ecaz_coord --database postgres \
    --sql "SELECT * FROM ec_spire_index_active_snapshot_diagnostics('${COORD_INDEX}'::regclass)" \
    --log-output "$ARTIFACT_DIR/fault-${DRILL}-placement.log"
}

case "$DRILL" in
  degraded)
    aws ec2 stop-instances --region "$REGION" --instance-ids "$TARGET_REMOTE_ID" | tee -a "$LOG"
    aws ec2 wait instance-stopped --region "$REGION" --instance-ids "$TARGET_REMOTE_ID"
    ecaz dev sql --host "$COORD_HOST" --user ecaz_coord --database postgres \
      --sql "SET ec_spire.remote_search_consistency_mode = 'degraded'"
    run_query
    snapshot_diag
    aws ec2 start-instances --region "$REGION" --instance-ids "$TARGET_REMOTE_ID" | tee -a "$LOG"
    aws ec2 wait instance-running --region "$REGION" --instance-ids "$TARGET_REMOTE_ID"
    ;;
  strict)
    aws ec2 stop-instances --region "$REGION" --instance-ids "$TARGET_REMOTE_ID" | tee -a "$LOG"
    aws ec2 wait instance-stopped --region "$REGION" --instance-ids "$TARGET_REMOTE_ID"
    ecaz dev sql --host "$COORD_HOST" --user ecaz_coord --database postgres \
      --sql "SET ec_spire.remote_search_consistency_mode = 'strict'"
    run_query
    snapshot_diag
    aws ec2 start-instances --region "$REGION" --instance-ids "$TARGET_REMOTE_ID" | tee -a "$LOG"
    aws ec2 wait instance-running --region "$REGION" --instance-ids "$TARGET_REMOTE_ID"
    ;;
  orphaned-2pc)
    ecaz dev sql --host "$COORD_HOST" --user ecaz_coord --database postgres \
      --file scripts/spire-aws/inject-2pc-orphan.sql \
      --set "prefix=ec_spire_aws_repr_1m" \
      --log-output "$ARTIFACT_DIR/fault-${DRILL}-inject.log"
    ecaz dev sql --host "$COORD_HOST" --user ecaz_coord --database postgres \
      --sql "SELECT * FROM ec_spire_reap_orphaned_remote_prepared_xacts(${TARGET_NODE_ID})" \
      --log-output "$ARTIFACT_DIR/fault-${DRILL}-reap.log"
    snapshot_diag
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
    run_query
    snapshot_diag
    echo "operator step: restore the prior secret version via aws secretsmanager restore-secret" | tee -a "$LOG"
    ;;
  *)
    echo "unknown drill: $DRILL" >&2; exit 2 ;;
esac
