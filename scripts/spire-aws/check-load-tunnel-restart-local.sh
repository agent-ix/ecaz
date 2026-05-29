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

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/ecaz-spire-tunnel-restart-local.XXXXXX")"
cleanup() {
  local pid_file pid
  if [[ -d "$tmp_dir/artifacts/tunnel-state" ]]; then
    while IFS= read -r pid_file; do
      pid="$(cat "$pid_file" 2>/dev/null || true)"
      if [[ -n "$pid" ]]; then
        kill "$pid" >/dev/null 2>&1 || true
        wait "$pid" >/dev/null 2>&1 || true
      fi
    done < <(find "$tmp_dir/artifacts/tunnel-state" -type f -name '*.pid')
  fi
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

mkdir -p "$tmp_dir/fake-bin" "$tmp_dir/artifacts"
cat > "$tmp_dir/fake-bin/aws" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "ssm" && "${2:-}" == "start-session" ]]; then
  sleep "${FAKE_AWS_READY_DELAY_SECONDS:-0}"
  printf 'Starting session with SessionId: fake-session\n'
  printf 'Port %s opened for sessionId fake-session.\n' "${FAKE_AWS_LOCAL_PORT:?}"
  printf 'Waiting for connections...\n'
  while true; do sleep 60; done
fi

printf 'unexpected fake aws invocation: %s\n' "$*" >&2
exit 2
EOF
chmod +x "$tmp_dir/fake-bin/aws"

env \
  PATH="$tmp_dir/fake-bin:$PATH" \
  SPIRE_AWS_TUNNEL_REGION=us-west-2 \
  SPIRE_AWS_TUNNEL_READY_TIMEOUT_SECONDS=25 \
  FAKE_AWS_READY_DELAY_SECONDS=12 \
  FAKE_AWS_LOCAL_PORT=15432 \
  "$REPO_ROOT/scripts/spire-aws/restart-ssm-port-forward.sh" \
    coordinator i-local-test 15432 "$tmp_dir/artifacts" \
  > "$tmp_dir/restart.log"

grep -Fq 'after 1 attempt(s)' "$tmp_dir/restart.log" ||
  die "restart command should allow one slow SSM startup to use the full ready timeout"

printf 'SPIRE AWS load tunnel restart local self-check passed\n'
