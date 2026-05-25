#!/usr/bin/env bash
# Phase 13e.1 — register every remote on the coordinator via live endpoint
# identity JSON emitted from a distributed placement plan.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

TOPOLOGY="${1:?topology JSON path required}"
ARTIFACT_DIR="${2:?artifact directory required}"
PLAN_FILE="${3:?distributed-placement-plan.json path required}"
mkdir -p "$ARTIFACT_DIR"

COORD_HOST=$(jq -r '.coordinator.private_ip' "$TOPOLOGY")
PLAN_PREFIX=$(jq -r '.prefix' "$PLAN_FILE")
COORD_TABLE="${PLAN_PREFIX}_corpus"
IDENTITY_DIR="${ARTIFACT_DIR}/remote-identities"
LEAF_VERIFY_DIR="${ARTIFACT_DIR}/remote-leaf-materialization"
REGISTRATION_SQL="${ARTIFACT_DIR}/register-remotes-rendered.sql"
PLACEMENT_REMOTES_JSON="${ARTIFACT_DIR}/placement-remotes.json"
mkdir -p "$IDENTITY_DIR" "$LEAF_VERIFY_DIR"

if [[ ! -s "$PLAN_FILE" ]]; then
  echo "distributed placement plan not found or empty: $PLAN_FILE" >&2
  exit 2
fi

ecaz dev sql \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --file scripts/spire-aws/verify-required-gucs.sql \
  --log-output "$ARTIFACT_DIR/verify-gucs-coord.log"

jq -c '.remotes[]' "$PLAN_FILE" | while read -r remote_plan; do
  NODE_ID=$(jq -r '.node_id' <<< "$remote_plan")
  REMOTE_HOST=$(jq -r --argjson node_id "$NODE_ID" \
    '.remotes[] | select(.node_id == $node_id) | .private_ip' "$TOPOLOGY")
  IDENTITY_SQL=$(jq -r '.remote_identity_query_sql' <<< "$remote_plan")

  if [[ -z "$REMOTE_HOST" || "$REMOTE_HOST" == "null" ]]; then
    echo "topology does not contain remote node_id=${NODE_ID}" >&2
    exit 2
  fi
  if [[ -z "$IDENTITY_SQL" || "$IDENTITY_SQL" == "null" ]]; then
    echo "placement plan does not contain remote_identity_query_sql for node_id=${NODE_ID}" >&2
    exit 2
  fi

  IDENTITY_FILE="$IDENTITY_DIR/node-${NODE_ID}-identity.json"
  IDENTITY_STDERR="$IDENTITY_DIR/node-${NODE_ID}-identity.stderr.log"
  ecaz dev sql \
    --host "$REMOTE_HOST" --user ecaz_coord --database postgres \
    --env "PGOPTIONS=-c client_min_messages=error" \
    --sql "$IDENTITY_SQL" \
    > "$IDENTITY_FILE" \
    2> "$IDENTITY_STDERR"
done

ecaz corpus render-spire-registrations \
  --plan-file "$PLAN_FILE" \
  --identity-dir "$IDENTITY_DIR" \
  --output-file "$REGISTRATION_SQL" \
  --descriptor-generation "${DESCRIPTOR_GENERATION:-1}"

ecaz dev sql \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --file "$REGISTRATION_SQL" \
  --log-output "$ARTIFACT_DIR/register-remotes.log"

COORD_INDEX=$(jq -r '.coordinator_index_name' "$PLAN_FILE")
REMOTES_JSON=$(jq -c '.remotes | map({node_id})' "$PLAN_FILE")
printf '%s\n' "$REMOTES_JSON" > "$PLACEMENT_REMOTES_JSON"

ecaz dev sql \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --file scripts/spire-aws/publish-remote-placements.sql \
  --set "coord_index=$COORD_INDEX" \
  --set "remotes_json=$REMOTES_JSON" \
  --log-output "$ARTIFACT_DIR/publish-remote-placements.log"

jq -c '.remotes[]' "$PLAN_FILE" | while read -r remote_plan; do
  NODE_ID=$(jq -r '.node_id' <<< "$remote_plan")
  REMOTE_INDEX=$(jq -r '.remote_index_regclass' <<< "$remote_plan")
  REMOTE_PREFIX=$(jq -r '.remote_prefix' <<< "$remote_plan")
  REMOTE_TABLE="${REMOTE_PREFIX}_corpus"
  REMOTE_HOST=$(jq -r --argjson node_id "$NODE_ID" \
    '.remotes[] | select(.node_id == $node_id) | .private_ip' "$TOPOLOGY")

  if [[ -z "$REMOTE_HOST" || "$REMOTE_HOST" == "null" ]]; then
    echo "topology does not contain remote node_id=${NODE_ID}" >&2
    exit 2
  fi
  if [[ -z "$REMOTE_INDEX" || "$REMOTE_INDEX" == "null" ]]; then
    echo "placement plan does not contain remote_index_regclass for node_id=${NODE_ID}" >&2
    exit 2
  fi
  if [[ -z "$REMOTE_PREFIX" || "$REMOTE_PREFIX" == "null" ]]; then
    echo "placement plan does not contain remote_prefix for node_id=${NODE_ID}" >&2
    exit 2
  fi

  REQUIRED_PIDS="$LEAF_VERIFY_DIR/node-${NODE_ID}-coordinator-required-leaves.txt"
  OBSERVED_PIDS="$LEAF_VERIFY_DIR/node-${NODE_ID}-remote-observed-leaves.txt"
  MISSING_PIDS="$LEAF_VERIFY_DIR/node-${NODE_ID}-missing-or-mismatched-leaves.txt"
  COORD_BASE_ASSIGNMENTS="$LEAF_VERIFY_DIR/node-${NODE_ID}-coordinator-base-assignments.tsv"

  ecaz dev sql \
    --host "$COORD_HOST" --user ecaz_coord --database postgres \
    --set "coord_index=$COORD_INDEX" \
    --set "node_id=$NODE_ID" \
    --sql "SELECT leaf_pid::text || E'\t' || effective_assignment_count::text FROM ec_spire_index_leaf_snapshot(:'coord_index'::regclass::oid) WHERE placement_state = 'available' AND node_id = :node_id::int ORDER BY leaf_pid" \
    > "$REQUIRED_PIDS" \
    2> "$LEAF_VERIFY_DIR/node-${NODE_ID}-coordinator-required-leaves.stderr.log"

  ecaz dev sql \
    --host "$COORD_HOST" --user ecaz_coord --database postgres \
    --set "coord_index=$COORD_INDEX" \
    --set "coord_table=$COORD_TABLE" \
    --set "node_id=$NODE_ID" \
    --file scripts/spire-aws/export-coordinator-leaf-base-assignments.sql \
    > "$COORD_BASE_ASSIGNMENTS" \
    2> "$LEAF_VERIFY_DIR/node-${NODE_ID}-coordinator-base-assignments.stderr.log"

  ecaz dev sql \
    --host "$REMOTE_HOST" --user ecaz_coord --database postgres \
    --set "remote_index=$REMOTE_INDEX" \
    --set "remote_table=$REMOTE_TABLE" \
    --set "assignment_file=$COORD_BASE_ASSIGNMENTS" \
    --file scripts/spire-aws/materialize-remote-leaf-base-assignments.sql \
    > "$LEAF_VERIFY_DIR/node-${NODE_ID}-remote-materialize.log" \
    2> "$LEAF_VERIFY_DIR/node-${NODE_ID}-remote-materialize.stderr.log"

  ecaz dev sql \
    --host "$REMOTE_HOST" --user ecaz_coord --database postgres \
    --set "remote_index=$REMOTE_INDEX" \
    --sql "SELECT leaf_pid::text || E'\t' || effective_assignment_count::text FROM ec_spire_index_leaf_snapshot(:'remote_index'::regclass::oid) WHERE placement_state = 'available' ORDER BY leaf_pid" \
    > "$OBSERVED_PIDS" \
    2> "$LEAF_VERIFY_DIR/node-${NODE_ID}-remote-observed-leaves.stderr.log"

  sort "$REQUIRED_PIDS" -o "$REQUIRED_PIDS"
  sort "$OBSERVED_PIDS" -o "$OBSERVED_PIDS"
  comm -23 "$REQUIRED_PIDS" "$OBSERVED_PIDS" > "$MISSING_PIDS"
  if [[ -s "$MISSING_PIDS" ]]; then
    echo "remote node_id=${NODE_ID} index ${REMOTE_INDEX} is missing coordinator-assigned leaf PID/count entries; leaf-owned materialization is required before distributed SPIRE reads are valid" >&2
    echo "missing or mismatched leaf list: $MISSING_PIDS" >&2
    exit 2
  fi
done

ecaz dev sql \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --sql "SELECT * FROM ec_spire_remote_node_snapshot('${COORD_INDEX}'::regclass)" \
  --log-output "$ARTIFACT_DIR/remote-node-snapshot-baseline.log"

ecaz dev sql \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --sql "SELECT * FROM ec_spire_index_placement_snapshot('${COORD_INDEX}'::regclass)" \
  --log-output "$ARTIFACT_DIR/coordinator-placement-snapshot-after-remote-publish.log"
