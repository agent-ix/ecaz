#!/usr/bin/env bash
# Configure optional coordinator EBS volumes as PostgreSQL tablespaces for
# SPIRE local-store benchmarks.

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
VOLUME_COUNT=$(jq '.coordinator.local_store_volumes // [] | length' "$TOPOLOGY")

if [[ "$VOLUME_COUNT" -eq 0 ]]; then
  echo "coordinator local-store volume count is 0; nothing to configure" \
    | tee "$ARTIFACT_DIR/setup-coordinator-store-volumes.log"
  exit 0
fi

COMMANDS_JSON=$(jq -c '
  def shquote:
    @sh;
  [
    "set -euo pipefail",
    "dnf -y install xfsprogs nvme-cli",
    "PG_BIN=/usr/pgsql-18/bin; if [ ! -x \"$PG_BIN/psql\" ]; then PG_BIN=$(dirname \"$(command -v psql)\"); fi",
    "if systemctl list-unit-files postgresql-18.service --no-legend 2>/dev/null | grep -q '\''^postgresql-18.service'\''; then PG_SERVICE=postgresql-18; else PG_SERVICE=postgresql; fi"
  ]
  + [
    .coordinator.local_store_volumes[] as $v
    | ($v.volume_id | gsub("-"; "")) as $volume_nodash
    | ($v.mount_path + "/pg_tblspc") as $location
    | [
        "echo configuring volume_id=" + $v.volume_id + " tablespace=" + $v.tablespace,
        "deadline=$((SECONDS + 300)); dev=\"\"; while [ $SECONDS -lt $deadline ]; do dev=$(readlink -f /dev/disk/by-id/nvme-Amazon_Elastic_Block_Store_" + $volume_nodash + " 2>/dev/null || true); if [ -n \"$dev\" ] && [ -b \"$dev\" ]; then break; fi; sleep 5; done; if [ -z \"$dev\" ] || [ ! -b \"$dev\" ]; then echo missing block device for " + $v.volume_id + " >&2; exit 2; fi",
        "if ! blkid \"$dev\" >/dev/null 2>&1; then mkfs.xfs -f \"$dev\"; fi",
        "uuid=$(blkid -s UUID -o value \"$dev\")",
        "mkdir -p " + ($v.mount_path | shquote),
        "grep -q \"UUID=$uuid \" /etc/fstab || echo \"UUID=$uuid " + $v.mount_path + " xfs defaults,nofail 0 2\" >> /etc/fstab",
        "mountpoint -q " + ($v.mount_path | shquote) + " || mount " + ($v.mount_path | shquote),
        "install -o postgres -g postgres -m 0700 -d " + ($location | shquote),
        "sudo -u postgres \"$PG_BIN/psql\" -v ON_ERROR_STOP=1 -d postgres -Atc \"SELECT 1 FROM pg_tablespace WHERE spcname = '\''" + $v.tablespace + "'\''\" | grep -qx 1 || sudo -u postgres \"$PG_BIN/psql\" -v ON_ERROR_STOP=1 -d postgres -c \"CREATE TABLESPACE " + $v.tablespace + " LOCATION '\''" + $location + "'\''\""
      ]
  ] | flatten
  + [
    "systemctl restart \"$PG_SERVICE\"",
    "df -h",
    "sudo -u postgres \"$PG_BIN/psql\" -d postgres -Atc \"SELECT spcname || E'\''\\t'\'' || pg_tablespace_location(oid) FROM pg_tablespace WHERE spcname LIKE '\''ecaz_spire_store_%'\'' ORDER BY spcname\""
  ]
' "$TOPOLOGY")
PARAMETERS_JSON=$(jq -cn --argjson commands "$COMMANDS_JSON" '{commands: $commands}')

CMD_ID=$(aws ssm send-command \
  --region "$REGION" \
  --document-name "AWS-RunShellScript" \
  --instance-ids "$COORD_ID" \
  --parameters "$PARAMETERS_JSON" \
  --output-s3-bucket-name "$BUCKET" \
  --output-s3-key-prefix "spire-aws/setup-coordinator-store-volumes" \
  --comment "ecaz Task 107 coordinator local-store tablespaces" \
  --query "Command.CommandId" \
  --output text)

echo "setup coordinator store volumes ssm command id: ${CMD_ID}" \
  | tee "$ARTIFACT_DIR/setup-coordinator-store-volumes.log"

STATUS="Pending"
DEADLINE=$((SECONDS + ${SPIRE_AWS_SSM_TIMEOUT_SECONDS:-1800}))
while (( SECONDS < DEADLINE )); do
  STATUS=$(aws ssm get-command-invocation \
    --region "$REGION" \
    --command-id "$CMD_ID" \
    --instance-id "$COORD_ID" \
    --query Status \
    --output text 2>/dev/null || echo "Pending")
  case "$STATUS" in
    Success) break ;;
    Failed|Cancelled|Cancelling|TimedOut) break ;;
    *) sleep 15 ;;
  esac
done

aws ssm get-command-invocation \
  --region "$REGION" \
  --command-id "$CMD_ID" \
  --instance-id "$COORD_ID" \
  > "$ARTIFACT_DIR/setup-coordinator-store-volumes.ssm.json" || true

if [[ "$STATUS" != "Success" ]]; then
  echo "setup coordinator store volumes failed with status=${STATUS}" >&2
  exit 1
fi
