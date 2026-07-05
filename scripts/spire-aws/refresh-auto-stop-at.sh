#!/usr/bin/env bash
# Refresh the ignored local SPIRE AWS terraform.tfvars auto-stop deadline.

set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/../.." && pwd)"

tfvars="${1:-$repo_root/infra/spire-aws/terraform.tfvars}"
hours="${2:-${SPIRE_AWS_AUTO_STOP_HOURS:-8}}"

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 2
}

[[ -f "$tfvars" ]] || die "missing terraform tfvars file: $tfvars"
[[ "$hours" =~ ^[0-9]+$ ]] || die "hours must be an integer, got: $hours"
((hours >= 6 && hours <= 168)) || die "hours must be between 6 and 168, got: $hours"

new_auto_stop_at="$(date -u -d "+${hours} hours" '+%Y-%m-%dT%H:%M:%SZ')" ||
  die "failed to compute auto_stop_at"
tmp="$(mktemp "${tfvars}.XXXXXX")" || die "failed to create temp file next to $tfvars"

if ! awk -v new_auto_stop_at="$new_auto_stop_at" '
  /^[[:space:]]*auto_stop_at[[:space:]]*=/ {
    sub(/=.*/, "= \"" new_auto_stop_at "\"")
    updated = 1
  }
  { print }
  END { exit(updated ? 0 : 3) }
' "$tfvars" > "$tmp"; then
  rm -f "$tmp"
  die "failed to update auto_stop_at in $tfvars"
fi

mv "$tmp" "$tfvars"
printf 'Updated %s auto_stop_at=%s\n' "$tfvars" "$new_auto_stop_at"
