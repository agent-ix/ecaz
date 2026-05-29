#!/usr/bin/env bash
set -euo pipefail

state_file="${1:?usage: preflight-state.sh <terraform.tfstate>}"

if [[ ! -f "$state_file" ]]; then
  printf 'SPIRE AWS state preflight passed: no local Terraform state file\n'
  exit 0
fi

mapfile -t resources < <(jq -r '
  (.resources // [])
  | map(select(.mode == "managed"))
  | .[]
  | "\(.type).\(.name)"
' "$state_file")

if ((${#resources[@]} == 0)); then
  printf 'SPIRE AWS state preflight passed: local Terraform state has no managed resources\n'
  exit 0
fi

printf 'ERROR: local SPIRE AWS Terraform state is not clean; refusing to provision over existing managed resources:\n' >&2
printf '  %s\n' "${resources[@]}" >&2
printf 'Clean up or move aside the prior state with packet-local evidence before provisioning a new SPIRE AWS run.\n' >&2
exit 2
