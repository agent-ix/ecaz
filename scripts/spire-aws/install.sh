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

# Wait for the SSM agent on every node to register with the SSM service
# (post-provision, the EC2 instance is up before its SSM agent has called
# home; send-command on such an instance fails with InvalidInstanceId).
echo "waiting for SSM agents to register on $(echo "${ALL_IDS[@]}" | wc -w) instances..."
EXPECTED_COUNT=$(echo "${ALL_IDS[@]}" | wc -w)
INSTANCE_LIST=$(echo "${ALL_IDS[@]}" | tr ' ' ',')
for _ in $(seq 1 60); do
  COUNT=$(aws ssm describe-instance-information --region "$REGION" \
    --query "length(InstanceInformationList[?contains(\`${INSTANCE_LIST}\`,InstanceId) && PingStatus=='Online'])" \
    --output text 2>/dev/null || echo 0)
  if [ "$COUNT" = "$EXPECTED_COUNT" ]; then
    echo "all SSM agents online ($COUNT/$EXPECTED_COUNT)"
    break
  fi
  echo "  $COUNT/$EXPECTED_COUNT online; sleeping 10s..."
  sleep 10
done

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
  # F27: `aws ssm wait command-executed` defaults to ~200s total
  # (40 attempts × 5s) before timing out. Cargo build takes 10-15 min
  # on these instances, so the waiter fails before the work finishes.
  # Manual poll loop with no fixed cap (capped only by SSM's own
  # --timeout-seconds 1800 on send-command).
  while :; do
    STATUS=$(aws ssm get-command-invocation --region "$REGION" \
      --command-id "$CMD_ID" --instance-id "$id" \
      --query Status --output text 2>/dev/null || echo Pending)
    case "$STATUS" in
      Pending|InProgress|Delayed) sleep 30 ;;
      *) break ;;
    esac
  done
  aws ssm get-command-invocation \
    --region "$REGION" --command-id "$CMD_ID" --instance-id "$id" \
    > "$ARTIFACT_DIR/install-${id}.log"
  # Fail loudly if a node didn't reach Success — make chain stops.
  test "$STATUS" = "Success" || { echo "install on $id failed: $STATUS"; exit 1; }
done
