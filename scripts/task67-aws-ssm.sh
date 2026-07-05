#!/usr/bin/env bash
# Narrow AWS SSM control surface for Task 67 benchmark recovery/diagnostics.

set -euo pipefail

REGION="${AWS_REGION:-us-west-2}"
INSTANCE_ID="${TASK67_AWS_DB_INSTANCE_ID:-i-0056e46b981edbb17}"

usage() {
  cat >&2 <<'USAGE'
usage: scripts/task67-aws-ssm.sh <command> [args...]

commands:
  cancel-command <command-id> <output-json>
  describe-instance <output-json>
  get-invocation <command-id> <output-json>
  list-commands <output-json>
  pg-diagnose <send-json> <invocation-json>
  pg-restart <send-json> <invocation-json>
  pg-sql <sql> <send-json> <invocation-json>
  repo-head <send-json> <invocation-json>

environment:
  AWS_REGION                 default: us-west-2
  TASK67_AWS_DB_INSTANCE_ID  default: i-0056e46b981edbb17
USAGE
}

send_shell_command() {
  local output_json="$1"
  shift
  local payload
  payload="$(python3 - "$@" <<'PY'
import json
import sys

print(json.dumps({"commands": list(sys.argv[1:])}))
PY
)"
  aws ssm send-command \
    --region "$REGION" \
    --instance-ids "$INSTANCE_ID" \
    --document-name AWS-RunShellScript \
    --parameters "$payload" \
    --output json > "$output_json"
}

command_id_from_send() {
  python3 - "$1" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as f:
    print(json.load(f)["Command"]["CommandId"])
PY
}

get_invocation() {
  local command_id="$1"
  local output_json="$2"
  aws ssm get-command-invocation \
    --region "$REGION" \
    --command-id "$command_id" \
    --instance-id "$INSTANCE_ID" \
    --output json > "$output_json"
}

wait_and_get_invocation() {
  local send_json="$1"
  local invocation_json="$2"
  local command_id status
  command_id="$(command_id_from_send "$send_json")"
  for _ in $(seq 1 120); do
    get_invocation "$command_id" "$invocation_json" || true
    status="$(python3 - "$invocation_json" <<'PY' 2>/dev/null || true
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as f:
    print(json.load(f).get("Status", ""))
PY
)"
    case "$status" in
      Success|Failed|Cancelled|TimedOut)
        return 0
        ;;
    esac
    sleep 2
  done
  echo "timed out waiting for SSM command $command_id" >&2
  return 124
}

cmd="${1:-}"
if [[ -z "$cmd" ]]; then
  usage
  exit 2
fi
shift

case "$cmd" in
  cancel-command)
    [[ $# -eq 2 ]] || { usage; exit 2; }
    aws ssm cancel-command \
      --region "$REGION" \
      --command-id "$1" \
      --instance-ids "$INSTANCE_ID" \
      --output json > "$2"
    ;;
  describe-instance)
    [[ $# -eq 1 ]] || { usage; exit 2; }
    aws ssm describe-instance-information \
      --region "$REGION" \
      --filters "Key=InstanceIds,Values=$INSTANCE_ID" \
      --output json > "$1"
    ;;
  get-invocation)
    [[ $# -eq 2 ]] || { usage; exit 2; }
    get_invocation "$1" "$2"
    ;;
  list-commands)
    [[ $# -eq 1 ]] || { usage; exit 2; }
    aws ssm list-command-invocations \
      --region "$REGION" \
      --instance-id "$INSTANCE_ID" \
      --details \
      --output json > "$1"
    ;;
  pg-diagnose)
    [[ $# -eq 2 ]] || { usage; exit 2; }
    send_shell_command "$1" \
      "set -eux" \
      "systemctl list-units --type=service --all --no-pager | grep -Ei 'postgres|pgsql|pgrx' || true" \
      "ps -ef | grep -Ei 'postgres|pgsql' | grep -v grep || true" \
      "ls -la /var/lib/pgsql/18 || true" \
      "find /var/lib/pgsql -maxdepth 3 -name postgresql.conf -o -name PG_VERSION"
    wait_and_get_invocation "$1" "$2"
    ;;
  pg-restart)
    [[ $# -eq 2 ]] || { usage; exit 2; }
    send_shell_command "$1" \
      "set -eux" \
      "sudo systemctl restart postgresql.service" \
      "sudo systemctl status postgresql.service --no-pager" \
      "sudo -u postgres psql -h /var/run/postgresql -d postgres -c 'select version();'"
    wait_and_get_invocation "$1" "$2"
    ;;
  pg-sql)
    [[ $# -eq 3 ]] || { usage; exit 2; }
    sql="$1"
    send_json="$2"
    invocation_json="$3"
    send_shell_command "$send_json" \
      "set -eux" \
      "sudo -u postgres psql -h /var/run/postgresql -d postgres -c $(printf '%q' "$sql")"
    wait_and_get_invocation "$send_json" "$invocation_json"
    ;;
  repo-head)
    [[ $# -eq 2 ]] || { usage; exit 2; }
    send_shell_command "$1" \
      "set -eux" \
      "sudo -u postgres bash -lc 'cd /var/lib/pgsql/build/ecaz && git rev-parse HEAD && git log --oneline -1'" \
      "/usr/local/bin/ecaz --version || true"
    wait_and_get_invocation "$1" "$2"
    ;;
  *)
    usage
    exit 2
    ;;
esac
