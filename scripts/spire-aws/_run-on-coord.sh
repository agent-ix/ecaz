#!/usr/bin/env bash
# Run a shell command on the coordinator via AWS SSM send-command and
# capture stdout/stderr back to the laptop. Centralizes the
# `aws ssm send-command` pattern used by register / load / smoke / bench
# / fault Makefile targets.
#
# The original scripts/spire-aws/{register,load,smoke,bench,fault}.sh
# were written assuming the laptop has direct PG connectivity to the
# coordinator's private IP. The AWS topology has no public IPs and no
# laptop-side VPN, so those scripts cannot run from the laptop. This
# helper invokes the same scripts (or any shell payload) ON the
# coordinator, where PG is reachable via localhost / loopback and
# `ecaz` CLI lives at /usr/local/bin/ecaz (installed by
# bootstrap-node.sh).
#
# Critical transport pitfall (F10 in the manifest): never feed
# multi-line shell commands to `jq -Rs .` via `<<<` here-string. The
# here-string appends a trailing newline that AWS-RunShellScript
# transports as a literal `\n` over the SSM transport, which bash on
# the remote interprets as a redirect token and produces:
#   line 1: 1n: ambiguous redirect
# Always use `printf '%s' "$cmd" | jq -Rs .` to keep the command body
# newline-clean.
#
# Args:
#   $1  label (e.g. "register", "load-correctness") — used to name
#       per-step artifact files
#   $2  shell command body to execute on the coord (single-line; this
#       script does not split or transform it)
#   $3  topology JSON path on the laptop (used once to set up
#       /tmp/topology.json on coord and to resolve coord instance id)
#   $4  artifact dir on the laptop where to write
#       <label>.{log,err,invocation.json}
#
# Side effect: on first invocation per chain (detected via the
# /tmp/topology.json marker check), uploads the topology JSON to the
# coord. Subsequent invocations skip the upload.

set -euo pipefail

LABEL="${1:?label required}"
CMD_BODY="${2:?command body required}"
TOPOLOGY="${3:?topology JSON path required}"
ARTIFACT_DIR="${4:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

REGION=$(jq -r '.region' "$TOPOLOGY")
COORD_ID=$(jq -r '.coordinator.instance_id' "$TOPOLOGY")

ssm_send_and_wait() {
  local label=$1 cmd=$2
  local CMD
  CMD=$(aws ssm send-command --region "$REGION" \
    --instance-ids "$COORD_ID" \
    --document-name AWS-RunShellScript \
    --timeout-seconds 3600 \
    --parameters "commands=[$(printf '%s' "$cmd" | jq -Rs .)]" \
    --query Command.CommandId --output text)
  echo "[$label] ssm cmd: $CMD" >&2

  while :; do
    local STATUS
    STATUS=$(aws ssm get-command-invocation --region "$REGION" \
      --command-id "$CMD" --instance-id "$COORD_ID" \
      --query Status --output text 2>/dev/null || echo Pending)
    case "$STATUS" in
      Pending|InProgress|Delayed) sleep 10 ;;
      *) break ;;
    esac
  done

  local OUT_JSON="$ARTIFACT_DIR/${label}.invocation.json"
  aws ssm get-command-invocation --region "$REGION" \
    --command-id "$CMD" --instance-id "$COORD_ID" > "$OUT_JSON"
  jq -r '.StandardOutputContent' "$OUT_JSON" > "$ARTIFACT_DIR/${label}.log"
  jq -r '.StandardErrorContent'  "$OUT_JSON" > "$ARTIFACT_DIR/${label}.err"
  local FINAL
  FINAL=$(jq -r '.Status' "$OUT_JSON")
  echo "[$label] final status: $FINAL" >&2
  if [ "$FINAL" != "Success" ]; then
    echo "[$label] STDOUT tail:" >&2
    tail -25 "$ARTIFACT_DIR/${label}.log" >&2 || true
    echo "[$label] STDERR tail:" >&2
    tail -25 "$ARTIFACT_DIR/${label}.err" >&2 || true
    return 1
  fi
  return 0
}

# Step 1: ensure /tmp/topology.json exists on coord. Idempotent — we
# always upload because it's cheap and avoids stale state.
B64_TOPO=$(base64 -w0 < "$TOPOLOGY")
# F24: /tmp/artifacts must be writable by the postgres user since the
# script bodies run via `sudo -u postgres`. The first SSM call creates
# it as root; chmod 1777 (sticky world-writable, like /tmp itself).
UPLOAD_CMD="echo ${B64_TOPO} | base64 -d > /tmp/topology.json && chmod 644 /tmp/topology.json && mkdir -p /tmp/artifacts && chmod 1777 /tmp/artifacts"
ssm_send_and_wait "_topology-upload-${LABEL}" "$UPLOAD_CMD"

# Step 2: run the requested command body. Wrap the user's payload in
# a small prologue that puts ecaz on PATH and exports a default
# ARTIFACT_DIR for the on-coord script if it needs one.
WRAPPED_CMD="export PATH=/usr/local/bin:/usr/pgsql-18/bin:/usr/bin:\$PATH; export ARTIFACT_DIR=/tmp/artifacts; mkdir -p /tmp/artifacts; chmod 1777 /tmp/artifacts; ${CMD_BODY}"
ssm_send_and_wait "$LABEL" "$WRAPPED_CMD"
