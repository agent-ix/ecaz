#!/usr/bin/env bash
# Restart one local SSM port forward owned by with-ssm-port-forwards.sh.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

LABEL="${1:?tunnel label required}"
INSTANCE_ID="${2:?instance id required}"
LOCAL_PORT="${3:?local port required}"
ARTIFACT_DIR="${4:?artifact directory required}"

REGION="${SPIRE_AWS_TUNNEL_REGION:?SPIRE_AWS_TUNNEL_REGION required}"
LOCAL_HOST="${SPIRE_AWS_TUNNEL_HOST:-127.0.0.1}"
TUNNEL_STATE_DIR="${SPIRE_AWS_TUNNEL_STATE_DIR:-$ARTIFACT_DIR/tunnel-state}"
TIMEOUT_SECONDS="${SPIRE_AWS_TUNNEL_READY_TIMEOUT_SECONDS:-60}"
TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
PID_FILE="$TUNNEL_STATE_DIR/${LABEL}.pid"
LAST_LOG=""

mkdir -p "$ARTIFACT_DIR" "$TUNNEL_STATE_DIR"

stop_tunnel_process() {
  local pid="$1"
  local pgid=""

  if [[ -z "$pid" ]]; then
    return
  fi

  pgid="$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ' || true)"
  if [[ -n "$pgid" && "$pgid" == "$pid" ]]; then
    kill -- "-$pgid" >/dev/null 2>&1 || true
  else
    kill "$pid" >/dev/null 2>&1 || true
  fi
  wait "$pid" >/dev/null 2>&1 || true
}

start_tunnel_process() {
  local log="$1"

  if command -v setsid >/dev/null 2>&1; then
    setsid aws ssm start-session \
      --region "$REGION" \
      --target "$INSTANCE_ID" \
      --document-name AWS-StartPortForwardingSession \
      --parameters "{\"portNumber\":[\"5432\"],\"localPortNumber\":[\"${LOCAL_PORT}\"]}" \
      > "$log" 2>&1 &
  else
    aws ssm start-session \
      --region "$REGION" \
      --target "$INSTANCE_ID" \
      --document-name AWS-StartPortForwardingSession \
      --parameters "{\"portNumber\":[\"5432\"],\"localPortNumber\":[\"${LOCAL_PORT}\"]}" \
      > "$log" 2>&1 &
  fi
  printf '%s\n' "$!"
}

if [[ -f "$PID_FILE" ]]; then
  OLD_PID="$(cat "$PID_FILE" 2>/dev/null || true)"
  stop_tunnel_process "$OLD_PID"
fi

deadline=$((SECONDS + TIMEOUT_SECONDS))
attempt=0
while (( SECONDS < deadline )); do
  attempt=$((attempt + 1))
  LAST_LOG="$ARTIFACT_DIR/tunnel-${LABEL}-restart-${TIMESTAMP}-attempt-${attempt}.log"
  PID="$(start_tunnel_process "$LAST_LOG")"
  printf '%s\n' "$PID" > "$PID_FILE"

  remaining=$((deadline - SECONDS))
  attempt_timeout="$remaining"
  if (( attempt_timeout > 0 )) && "$SCRIPT_DIR/wait-for-ssm-port-forward-ready.sh" "$LABEL" "$LOCAL_PORT" "$LAST_LOG" "$PID" "$attempt_timeout"; then
    echo "tunnel ${LABEL} restarted on ${LOCAL_HOST}:${LOCAL_PORT} after ${attempt} attempt(s)"
    exit 0
  fi

  stop_tunnel_process "$PID"
  sleep 1
done

echo "timed out waiting for tunnel ${LABEL} restart on ${LOCAL_HOST}:${LOCAL_PORT}; last log: ${LAST_LOG}" >&2
exit 1
