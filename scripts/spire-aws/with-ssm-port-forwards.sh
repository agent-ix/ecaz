#!/usr/bin/env bash
# Start local SSM port forwards for every PostgreSQL node in a Phase 13
# topology and write an operator topology that scripts/spire-aws/*.sh can use.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

TOPOLOGY="${1:?topology JSON path required}"
ARTIFACT_DIR="${2:?artifact directory required}"
TUNNELED_TOPOLOGY="${3:?output topology JSON path required}"
shift 3

REGION=$(jq -r '.region' "$TOPOLOGY")
BASE_PORT="${SPIRE_AWS_TUNNEL_BASE_PORT:-15432}"
LOCAL_HOST="${SPIRE_AWS_TUNNEL_HOST:-127.0.0.1}"
TUNNEL_STATE_DIR="${SPIRE_AWS_TUNNEL_STATE_DIR:-$ARTIFACT_DIR/tunnel-state}"
TUNNEL_READY_TIMEOUT_SECONDS="${SPIRE_AWS_TUNNEL_READY_TIMEOUT_SECONDS:-60}"

mkdir -p "$ARTIFACT_DIR" "$TUNNEL_STATE_DIR"
export SPIRE_AWS_TUNNEL_REGION="$REGION"
export SPIRE_AWS_TUNNEL_HOST="$LOCAL_HOST"
export SPIRE_AWS_TUNNEL_STATE_DIR="$TUNNEL_STATE_DIR"
export SPIRE_AWS_TUNNEL_RESTART_COMMAND="$REPO_ROOT/scripts/spire-aws/restart-ssm-port-forward.sh"

if ! command -v session-manager-plugin >/dev/null 2>&1; then
  echo "session-manager-plugin is required for SSM port forwarding" >&2
  exit 2
fi

jq \
  --arg host "$LOCAL_HOST" \
  --argjson base_port "$BASE_PORT" \
  '.coordinator.operator_host = $host
   | .coordinator.operator_port = $base_port
   | .remotes |= to_entries
     | .remotes = (.remotes | map(.value + {
         operator_host: $host,
         operator_port: ($base_port + 1 + .key)
       }))' \
  "$TOPOLOGY" > "$TUNNELED_TOPOLOGY"

PIDS=()
cleanup() {
  local pid pid_file
  for pid in "${PIDS[@]:-}"; do
    kill "$pid" >/dev/null 2>&1 || true
  done
  if [[ -d "$TUNNEL_STATE_DIR" ]]; then
    while IFS= read -r pid_file; do
      pid="$(cat "$pid_file" 2>/dev/null || true)"
      if [[ -n "$pid" ]]; then
        kill "$pid" >/dev/null 2>&1 || true
      fi
    done < <(find "$TUNNEL_STATE_DIR" -type f -name '*.pid')
  fi
}
trap cleanup EXIT

start_tunnel() {
  local label="$1"
  local instance_id="$2"
  local local_port="$3"
  local log="$ARTIFACT_DIR/tunnel-${label}.log"

  aws ssm start-session \
    --region "$REGION" \
    --target "$instance_id" \
    --document-name AWS-StartPortForwardingSession \
    --parameters "{\"portNumber\":[\"5432\"],\"localPortNumber\":[\"${local_port}\"]}" \
    > "$log" 2>&1 &
  local pid="$!"
  PIDS+=("$pid")
  printf '%s\n' "$pid" > "$TUNNEL_STATE_DIR/${label}.pid"
}

wait_for_tunnel_ready() {
  local label="$1"
  local port="$2"
  local log="$ARTIFACT_DIR/tunnel-${label}.log"
  local pid_file="$TUNNEL_STATE_DIR/${label}.pid"
  local pid

  pid="$(cat "$pid_file")"
  "$SCRIPT_DIR/wait-for-ssm-port-forward-ready.sh" "$label" "$port" "$log" "$pid" "$TUNNEL_READY_TIMEOUT_SECONDS"
}

COORD_ID=$(jq -r '.coordinator.instance_id' "$TOPOLOGY")
COORD_PORT=$(jq -r '.coordinator.operator_port' "$TUNNELED_TOPOLOGY")
start_tunnel coordinator "$COORD_ID" "$COORD_PORT"

mapfile -t REMOTE_ROWS < <(jq -c '.remotes[]' "$TUNNELED_TOPOLOGY")
for remote in "${REMOTE_ROWS[@]}"; do
  label="remote-$(jq -r '.node_id' <<< "$remote")"
  instance_id=$(jq -r '.instance_id' <<< "$remote")
  port=$(jq -r '.operator_port' <<< "$remote")
  start_tunnel "$label" "$instance_id" "$port"
done

wait_for_tunnel_ready coordinator "$COORD_PORT"
for remote in "${REMOTE_ROWS[@]}"; do
  label="remote-$(jq -r '.node_id' <<< "$remote")"
  port=$(jq -r '.operator_port' <<< "$remote")
  wait_for_tunnel_ready "$label" "$port"
done

if [[ "${1:-}" == "--" ]]; then
  shift
  "$@"
else
  echo "tunneled topology: $TUNNELED_TOPOLOGY"
  wait
fi
