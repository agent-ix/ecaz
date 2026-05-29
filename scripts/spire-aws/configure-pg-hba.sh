#!/usr/bin/env bash
# Re-apply Phase 13 AWS pg_hba rules to existing nodes without rebuilding ecaz.
# Args:
#   $1  Topology JSON
#   $2  Artifact directory for SSM invocation logs

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

TOPOLOGY="${1:?topology JSON path required}"
ARTIFACT_DIR="${2:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

REGION=$(jq -r '.region' "$TOPOLOGY")
mapfile -t INSTANCE_IDS < <(jq -r '[.coordinator.instance_id] + [.remotes[].instance_id] | .[]' "$TOPOLOGY")

commands_json=$(jq -cn '{
  commands: [
    "set -euo pipefail",
    "PGDATA=/var/lib/pgsql/data",
    "PG_SERVICE=postgresql",
    "tmp_hba=$(mktemp)",
    "cat > \"$tmp_hba\" <<'\''EOF'\''\nhost all ecaz_coord 127.0.0.1/32 trust\nhost all ecaz_coord ::1/128 trust\nhostssl all ecaz_coord 10.42.0.0/16 scram-sha-256\nEOF",
    "grep -vE '\''ecaz_coord (127\\.0\\.0\\.1/32|::1/128|10\\.42\\.0\\.0/16)'\'' \"$PGDATA/pg_hba.conf\" >> \"$tmp_hba\"",
    "install -o postgres -g postgres -m 0600 \"$tmp_hba\" \"$PGDATA/pg_hba.conf\"",
    "rm -f \"$tmp_hba\"",
    "systemctl reload \"$PG_SERVICE\"",
    "sudo -u postgres psql -Atc \"select usename from pg_user where usename='\''ecaz_coord'\'';\""
  ]
}')

cmd_id=$(aws ssm send-command \
  --region "$REGION" \
  --document-name "AWS-RunShellScript" \
  --instance-ids "${INSTANCE_IDS[@]}" \
  --parameters "$commands_json" \
  --comment "ecaz Phase 13 pg_hba repair" \
  --query "Command.CommandId" \
  --output text)

echo "$cmd_id" > "$ARTIFACT_DIR/configure-pg-hba-command-id.txt"
for instance_id in "${INSTANCE_IDS[@]}"; do
  status="Pending"
  deadline=$((SECONDS + 300))
  while (( SECONDS < deadline )); do
    status=$(aws ssm get-command-invocation \
      --region "$REGION" --command-id "$cmd_id" --instance-id "$instance_id" \
      --query Status --output text 2>/dev/null || echo "Pending")
    case "$status" in
      Success|Failed|Cancelled|Cancelling|TimedOut)
        break
        ;;
      *)
        sleep 5
        ;;
    esac
  done
  aws ssm get-command-invocation \
    --region "$REGION" --command-id "$cmd_id" --instance-id "$instance_id" \
    > "$ARTIFACT_DIR/configure-pg-hba-${instance_id}.json"
  if [[ "$status" != "Success" ]]; then
    echo "pg_hba repair failed for ${instance_id}: ${status}" >&2
    exit 1
  fi
done
