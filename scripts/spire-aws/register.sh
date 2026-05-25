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
IDENTITY_DIR="${ARTIFACT_DIR}/remote-identities"
REGISTRATION_SQL="${ARTIFACT_DIR}/register-remotes-rendered.sql"
PLACEMENT_REMOTES_JSON="${ARTIFACT_DIR}/placement-remotes.json"
mkdir -p "$IDENTITY_DIR"

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

  ecaz dev sql \
    --host "$REMOTE_HOST" --user ecaz_coord --database postgres \
    --sql "$IDENTITY_SQL" \
    --log-output "$IDENTITY_DIR/node-${NODE_ID}-identity.json"
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

ecaz dev sql \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --sql "SELECT * FROM ec_spire_remote_node_snapshot('${COORD_INDEX}'::regclass)" \
  --log-output "$ARTIFACT_DIR/remote-node-snapshot-baseline.log"

ecaz dev sql \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --sql "SELECT * FROM ec_spire_index_placement_snapshot('${COORD_INDEX}'::regclass)" \
  --log-output "$ARTIFACT_DIR/coordinator-placement-snapshot-after-remote-publish.log"
