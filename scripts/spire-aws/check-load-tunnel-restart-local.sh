#!/usr/bin/env bash
# Local static guard for representative load tunnel restart argument wiring.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
LOAD_SCRIPT="$REPO_ROOT/scripts/spire-aws/load.sh"

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 2
}

grep -Fq 'local instance_id="$2"' "$LOAD_SCRIPT" ||
  die "restart_operator_tunnel_if_available must accept instance_id as its second argument"
grep -Fq '"$restart_command" "$label" "$instance_id" "$port" "$ARTIFACT_DIR"' "$LOAD_SCRIPT" ||
  die "restart_operator_tunnel_if_available must call restart command with label instance_id port artifact_dir"
grep -Fq 'coord_id=$(jq -r '\''.coordinator.instance_id'\'' "$TOPOLOGY")' "$LOAD_SCRIPT" ||
  die "restart_all_operator_tunnels_if_available must read coordinator instance_id"
grep -Fq 'instance_id=$(jq -r '\''.instance_id'\'' <<< "$remote")' "$LOAD_SCRIPT" ||
  die "restart_all_operator_tunnels_if_available must read remote instance_id"
grep -Fq 'restart_operator_tunnel_if_available coordinator "$(jq -r '\''.coordinator.instance_id'\'' "$TOPOLOGY")" "$COORD_PORT"' "$LOAD_SCRIPT" ||
  die "representative coordinator reload must restart coordinator tunnel with instance_id and port"

printf 'SPIRE AWS load tunnel restart local self-check passed\n'
