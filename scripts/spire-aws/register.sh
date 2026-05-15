#!/usr/bin/env bash
# Phase 13b.6 — register every remote on the coordinator via
# `ec_spire_register_remote_node_descriptor`.

set -euo pipefail

TOPOLOGY="${1:?topology JSON path required}"
ARTIFACT_DIR="${2:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

COORD_HOST=$(jq -r '.coordinator.private_ip' "$TOPOLOGY")
COORD_INDEX="${COORD_INDEX:-ec_spire_aws_repr_1m_idx}"
REMOTE_INDEX="${REMOTE_INDEX:-ec_spire_aws_repr_1m_remote_idx}"
EXTVERSION="${EXTVERSION:-0.1.2}"

ecaz dev sql \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --file scripts/spire-aws/verify-required-gucs.sql \
  --log-output "$ARTIFACT_DIR/verify-gucs-coord.log"

jq -c '.remotes[]' "$TOPOLOGY" | while read -r remote; do
  NODE_ID=$(jq -r '.node_id' <<< "$remote")
  SECRET=$(jq -r '.secret_name' <<< "$remote")
  ecaz dev sql \
    --host "$COORD_HOST" --user ecaz_coord --database postgres \
    --file scripts/spire-aws/register-remotes.sql \
    --set "coord_index=$COORD_INDEX" \
    --set "node_id=$NODE_ID" \
    --set "descriptor_generation=1" \
    --set "conninfo_secret=$SECRET" \
    --set "remote_index=$REMOTE_INDEX" \
    --set "state=active" \
    --set "served_epoch=1" \
    --set "min_retained_epoch=1" \
    --set "extversion=$EXTVERSION" \
    --log-output "$ARTIFACT_DIR/register-remote-${NODE_ID}.log"
done

ecaz dev sql \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --sql "SELECT * FROM ec_spire_remote_node_snapshot('${COORD_INDEX}'::regclass)" \
  --log-output "$ARTIFACT_DIR/remote-node-snapshot-baseline.log"
