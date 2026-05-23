#!/usr/bin/env bash
# Shared SSM port-forwarding helper for scripts/spire-aws/*.sh.
#
# AWS deploys coord+remotes with no public IPs; the laptop has no direct
# route into the VPC. Each script that needs to run `ecaz`/`psql` from
# the laptop against the coordinator must source this helper. It opens
# an `aws ssm start-session --document-name AWS-StartPortForwardingSession`
# in the background that forwards localhost:$COORD_PORT to coord:5432, sets
# COORD_HOST + COORD_PORT for the caller, and traps EXIT to terminate the
# tunnel.
#
# Required: TOPOLOGY env var pointing at the aws-topology.json (caller's
# already done this). Optional: COORD_PORT override (default: pick a free
# port in the 15432-15500 range).
#
# Usage:
#   . scripts/spire-aws/_ssm-tunnel.sh
#   ecaz dev sql --host "$COORD_HOST" --port "$COORD_PORT" ...

set -euo pipefail

: "${TOPOLOGY:?TOPOLOGY env var must be set before sourcing _ssm-tunnel.sh}"

REGION=$(jq -r '.region' "$TOPOLOGY")
COORD_INSTANCE_ID=$(jq -r '.coordinator.instance_id' "$TOPOLOGY")

# Pick a free local port in 15432..15500.
COORD_PORT="${COORD_PORT:-}"
if [ -z "$COORD_PORT" ]; then
  for p in $(seq 15432 15500); do
    if ! (echo -n > /dev/tcp/127.0.0.1/$p) >/dev/null 2>&1; then
      COORD_PORT=$p; break
    fi
  done
  test -n "$COORD_PORT"
fi
COORD_HOST=localhost
export COORD_HOST COORD_PORT

# Start the SSM tunnel in the background. The AWS Session Manager plugin
# must be installed locally (`session-manager-plugin`).
SSM_TUNNEL_LOG="${SSM_TUNNEL_LOG:-/tmp/ssm-tunnel-${COORD_INSTANCE_ID}.log}"
aws ssm start-session \
  --region "$REGION" \
  --target "$COORD_INSTANCE_ID" \
  --document-name AWS-StartPortForwardingSession \
  --parameters "{\"portNumber\":[\"5432\"],\"localPortNumber\":[\"${COORD_PORT}\"]}" \
  > "$SSM_TUNNEL_LOG" 2>&1 &
SSM_TUNNEL_PID=$!

# Wait until the tunnel is listening, with a 30 s timeout.
for _ in $(seq 1 60); do
  if (echo -n > /dev/tcp/127.0.0.1/"$COORD_PORT") >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$SSM_TUNNEL_PID" 2>/dev/null; then
    echo "ERROR: ssm tunnel died early; see $SSM_TUNNEL_LOG" >&2
    cat "$SSM_TUNNEL_LOG" >&2 || true
    exit 1
  fi
  sleep 0.5
done

if ! (echo -n > /dev/tcp/127.0.0.1/"$COORD_PORT") >/dev/null 2>&1; then
  echo "ERROR: ssm tunnel did not open localhost:${COORD_PORT} within 30s" >&2
  cat "$SSM_TUNNEL_LOG" >&2 || true
  kill "$SSM_TUNNEL_PID" 2>/dev/null || true
  exit 1
fi

trap '
  if [ -n "${SSM_TUNNEL_PID:-}" ] && kill -0 "$SSM_TUNNEL_PID" 2>/dev/null; then
    kill "$SSM_TUNNEL_PID" 2>/dev/null || true
    wait "$SSM_TUNNEL_PID" 2>/dev/null || true
  fi
' EXIT

echo "ssm-tunnel: localhost:${COORD_PORT} -> ${COORD_INSTANCE_ID}:5432 (pid=${SSM_TUNNEL_PID})"
