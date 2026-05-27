#!/usr/bin/env bash
# Restart one local SSM port forward owned by with-ssm-port-forwards.sh.

set -euo pipefail

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

if [[ -f "$PID_FILE" ]]; then
  OLD_PID="$(cat "$PID_FILE" 2>/dev/null || true)"
  if [[ -n "$OLD_PID" ]]; then
    kill "$OLD_PID" >/dev/null 2>&1 || true
  fi
fi

deadline=$((SECONDS + TIMEOUT_SECONDS))
attempt=0
while (( SECONDS < deadline )); do
  attempt=$((attempt + 1))
  LAST_LOG="$ARTIFACT_DIR/tunnel-${LABEL}-restart-${TIMESTAMP}-attempt-${attempt}.log"
  aws ssm start-session \
    --region "$REGION" \
    --target "$INSTANCE_ID" \
    --document-name AWS-StartPortForwardingSession \
    --parameters "{\"portNumber\":[\"5432\"],\"localPortNumber\":[\"${LOCAL_PORT}\"]}" \
    > "$LAST_LOG" 2>&1 &
  PID="$!"
  printf '%s\n' "$PID" > "$PID_FILE"

  attempt_deadline=$((SECONDS + 10))
  while (( SECONDS < attempt_deadline && SECONDS < deadline )); do
    if grep -Eq "Port ${LOCAL_PORT} opened for sessionId" "$LAST_LOG" 2>/dev/null; then
      echo "tunnel ${LABEL} restarted on ${LOCAL_HOST}:${LOCAL_PORT} after ${attempt} attempt(s)"
      exit 0
    fi
    if ! kill -0 "$PID" >/dev/null 2>&1; then
      break
    fi
    sleep 1
  done

  kill "$PID" >/dev/null 2>&1 || true
  sleep 1
done

echo "timed out waiting for tunnel ${LABEL} restart on ${LOCAL_HOST}:${LOCAL_PORT}; last log: ${LAST_LOG}" >&2
exit 1
