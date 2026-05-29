#!/usr/bin/env bash
# Wait until the Session Manager plugin reports a PostgreSQL port-forward open.

set -euo pipefail

LABEL="${1:?tunnel label required}"
LOCAL_PORT="${2:?local port required}"
LOG_FILE="${3:?tunnel log path required}"
PID="${4:?session-manager process pid required}"
TIMEOUT_SECONDS="${5:-60}"

if [[ ! "$TIMEOUT_SECONDS" =~ ^[0-9]+$ ]] || ((TIMEOUT_SECONDS <= 0)); then
  printf 'ERROR: timeout must be a positive integer, got: %s\n' "$TIMEOUT_SECONDS" >&2
  exit 2
fi

deadline=$((SECONDS + TIMEOUT_SECONDS))
while (( SECONDS < deadline )); do
  if grep -Eq "Port ${LOCAL_PORT} opened for sessionId" "$LOG_FILE" 2>/dev/null; then
    printf 'tunnel %s ready on port %s\n' "$LABEL" "$LOCAL_PORT"
    exit 0
  fi

  if ! kill -0 "$PID" >/dev/null 2>&1; then
    printf 'tunnel %s process exited before port %s opened; log: %s\n' "$LABEL" "$LOCAL_PORT" "$LOG_FILE" >&2
    exit 1
  fi

  sleep 1
done

printf 'timed out waiting for tunnel %s port %s opened log; log: %s\n' "$LABEL" "$LOCAL_PORT" "$LOG_FILE" >&2
exit 1
