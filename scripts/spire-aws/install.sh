#!/usr/bin/env bash
# Phase 13b.5 — install PostgreSQL 18 and the ecaz extension on every node.
# Args:
#   $1  Path to topology JSON (from `terraform output -json topology`)
#   $2  Artifact directory for logs
#
# Uses AWS Session Manager (`aws ssm send-command`) to run the bootstrap
# script on every instance in parallel. Each node receives the ecaz tarball
# from S3 and writes its install transcript back to the artifact bucket.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

TOPOLOGY="${1:?topology JSON path required}"
ARTIFACT_DIR="${2:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

REGION=$(jq -r '.region' "$TOPOLOGY")
BUCKET=$(jq -r '.artifact_bucket' "$TOPOLOGY")
COORD_ID=$(jq -r '.coordinator.instance_id' "$TOPOLOGY")
REMOTE_IDS=$(jq -r '.remotes[].instance_id' "$TOPOLOGY")
ECAZ_URL="${ECAZ_GIT_URL:-https://github.com/agent-ix/ecaz.git}"
ECAZ_REF="${ECAZ_GIT_REF:?ECAZ_GIT_REF must be set (export from Makefile)}"

ALL_IDS=("$COORD_ID")
while IFS= read -r id; do ALL_IDS+=("$id"); done <<< "$REMOTE_IDS"

aws s3 cp \
  "$REPO_ROOT/scripts/spire-aws/bootstrap-node.sh" \
  "s3://${BUCKET}/bootstrap-node.sh" \
  --region "$REGION" \
  > "$ARTIFACT_DIR/bootstrap-upload.log"

# Each node downloads bootstrap-node.sh from S3 and runs it. The script
# clones ecaz, builds the extension via cargo-pgrx, and CREATEs it. The
# build is ~10 min per node on r8g hardware; SSM wait timeout is 30 min.
CMD_ID=$(aws ssm send-command \
  --region "$REGION" \
  --document-name "AWS-RunShellScript" \
  --instance-ids "${ALL_IDS[@]}" \
  --timeout-seconds 1800 \
  --parameters "commands=[\"sudo aws s3 cp s3://${BUCKET}/bootstrap-node.sh /tmp/bootstrap-node.sh\",\"sudo ECAZ_SPIRE_AWS_BUCKET=${BUCKET} ECAZ_GIT_URL=${ECAZ_URL} ECAZ_GIT_REF=${ECAZ_REF} bash /tmp/bootstrap-node.sh\"]" \
  --output-s3-bucket-name "$BUCKET" \
  --output-s3-key-prefix "spire-aws/install" \
  --comment "ecaz Phase 13b.5 install (git ref ${ECAZ_REF})" \
  --query "Command.CommandId" --output text)

echo "ssm command id: $CMD_ID" | tee "$ARTIFACT_DIR/install.log"

for id in "${ALL_IDS[@]}"; do
  aws ssm wait command-executed --region "$REGION" --command-id "$CMD_ID" --instance-id "$id" --cli-read-timeout 2000 --cli-connect-timeout 60
  aws ssm get-command-invocation \
    --region "$REGION" --command-id "$CMD_ID" --instance-id "$id" \
    > "$ARTIFACT_DIR/install-${id}.log"
done
