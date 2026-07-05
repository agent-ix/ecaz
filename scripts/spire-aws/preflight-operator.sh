#!/usr/bin/env bash
set -euo pipefail

tfvars="${1:?usage: preflight-operator.sh <terraform.tfvars>}"

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 2
}

read_tfvar() {
  local key="$1"
  awk -F= -v key="$key" '
    $0 ~ /^[[:space:]]*#/ { next }
    $1 ~ "^[[:space:]]*" key "[[:space:]]*$" {
      value = $2
      sub(/^[[:space:]]+/, "", value)
      sub(/[[:space:]]+$/, "", value)
      sub(/^"/, "", value)
      sub(/"$/, "", value)
      print value
      exit
    }
  ' "$tfvars"
}

require_tfvar() {
  local key="$1"
  local value
  value="$(read_tfvar "$key")"
  [[ -n "$value" ]] || die "missing required terraform.tfvars key: $key"
  printf '%s' "$value"
}

require_graviton_family() {
  local key="$1"
  local value="$2"
  [[ "$value" =~ ^(m7g|m8g|r7g|c7g|c8g)\. ]] ||
    die "$key must use the established Graviton/aarch64 lane, got: $value"
}

require_expected_lane_value() {
  local key="$1"
  local value="$2"
  local expected="$3"
  [[ "$value" == "$expected" ]] ||
    die "$key must match the established Phase 13e Graviton lane (${expected}); got: $value. Amend the task/runbook before changing AWS lane."
}

[[ -f "$tfvars" ]] || die "missing $tfvars; create it from infra/spire-aws/terraform.tfvars.example before provisioning"

region="$(require_tfvar region)"
availability_zone="$(require_tfvar availability_zone)"
ami_id="$(require_tfvar ami_id)"
owner="$(require_tfvar owner)"
auto_stop_at="$(require_tfvar auto_stop_at)"

coordinator_instance_type="$(read_tfvar coordinator_instance_type)"
remote_instance_type="$(read_tfvar remote_instance_type)"
remote_count="$(read_tfvar remote_count)"

coordinator_instance_type="${coordinator_instance_type:-m7g.large}"
remote_instance_type="${remote_instance_type:-m7g.large}"
remote_count="${remote_count:-3}"

[[ "$region" =~ ^[a-z]{2}-[a-z]+-[0-9]$ ]] || die "region does not look like an AWS region: $region"
[[ "$availability_zone" == "$region"* ]] || die "availability_zone must be in region $region, got: $availability_zone"
[[ "$ami_id" =~ ^ami-[0-9a-f]+$ ]] || die "ami_id must be an AMI id, got: $ami_id"
[[ "$remote_count" =~ ^[0-9]+$ ]] || die "remote_count must be an integer, got: $remote_count"
[[ "$owner" != "your-gh-handle" ]] || die "owner must be set to the operator/reviewer-visible handle"
[[ "$auto_stop_at" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$ ]] ||
  die "auto_stop_at must be UTC ISO-8601 like 2026-05-25T23:00:00Z"
auto_stop_epoch="$(date -u -d "$auto_stop_at" +%s 2>/dev/null)" ||
  die "auto_stop_at is not parseable by date: $auto_stop_at"
now_epoch="${SPIRE_AWS_PREFLIGHT_NOW_EPOCH:-$(date -u +%s)}"
[[ "$now_epoch" =~ ^[0-9]+$ ]] || die "SPIRE_AWS_PREFLIGHT_NOW_EPOCH must be an epoch-second integer, got: $now_epoch"
((auto_stop_epoch > now_epoch)) ||
  die "auto_stop_at must be in the future before provisioning, got: $auto_stop_at"
min_auto_stop_lead_seconds="${SPIRE_AWS_MIN_AUTO_STOP_LEAD_SECONDS:-18000}"
[[ "$min_auto_stop_lead_seconds" =~ ^[0-9]+$ ]] ||
  die "SPIRE_AWS_MIN_AUTO_STOP_LEAD_SECONDS must be an integer second count, got: $min_auto_stop_lead_seconds"
((auto_stop_epoch - now_epoch >= min_auto_stop_lead_seconds)) ||
  die "auto_stop_at must be at least ${min_auto_stop_lead_seconds}s after preflight time for the representative pass watchdog budget, got: $auto_stop_at"

require_graviton_family coordinator_instance_type "$coordinator_instance_type"
require_graviton_family remote_instance_type "$remote_instance_type"
require_expected_lane_value region "$region" "us-west-2"
require_expected_lane_value availability_zone "$availability_zone" "us-west-2a"
if [[ "${SPIRE_AWS_ALLOW_NONDEFAULT_GRAVITON_LANE:-0}" == "1" ]]; then
  printf 'SPIRE AWS nondefault Graviton lane override accepted: coordinator=%s remote=%s\n' \
    "$coordinator_instance_type" "$remote_instance_type"
else
  require_expected_lane_value coordinator_instance_type "$coordinator_instance_type" "m7g.large"
  require_expected_lane_value remote_instance_type "$remote_instance_type" "m7g.large"
fi

architecture="$(aws ec2 describe-images \
  --region "$region" \
  --image-ids "$ami_id" \
  --query 'Images[0].Architecture' \
  --output text)"

[[ "$architecture" == "arm64" ]] || die "ami_id must resolve to an arm64 AMI, got architecture: $architecture"

printf 'SPIRE AWS operator preflight passed: region=%s az=%s ami=%s coordinator=%s remote=%s remote_count=%s\n' \
  "$region" \
  "$availability_zone" \
  "$ami_id" \
  "$coordinator_instance_type" \
  "$remote_instance_type" \
  "$remote_count"
