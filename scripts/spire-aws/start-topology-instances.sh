#!/usr/bin/env bash
# Start exactly the EC2 instances named by a SPIRE AWS topology file.

set -euo pipefail

TOPOLOGY="${1:?usage: start-topology-instances.sh <topology-json> <artifact-dir>}"
ARTIFACT_DIR="${2:?usage: start-topology-instances.sh <topology-json> <artifact-dir>}"
EXPECTED_INSTANCE_TYPE="${SPIRE_AWS_EXPECT_INSTANCE_TYPE:-}"
EXPECTED_AZ="${SPIRE_AWS_EXPECT_AVAILABILITY_ZONE:-}"

mkdir -p "$ARTIFACT_DIR"

REGION="$(jq -r '.region' "$TOPOLOGY")"
LOG="$ARTIFACT_DIR/start-topology-instances.log"
STATE_LOG="$ARTIFACT_DIR/start-topology-instance-state.log"

mapfile -t INSTANCE_IDS < <(
  jq -r '
    [
      .coordinator.instance_id,
      (.remotes[]?.instance_id)
    ]
    | map(select(. != null and . != ""))
    | unique
    | .[]
  ' "$TOPOLOGY"
)

if ((${#INSTANCE_IDS[@]} == 0)); then
  echo "no instance ids found in topology: $TOPOLOGY" >&2
  exit 2
fi

{
  printf 'topology=%s\n' "$TOPOLOGY"
  printf 'region=%s\n' "$REGION"
  printf 'instance_ids=%s\n' "${INSTANCE_IDS[*]}"
  if [[ -n "$EXPECTED_INSTANCE_TYPE" ]]; then
    printf 'expected_instance_type=%s\n' "$EXPECTED_INSTANCE_TYPE"
  fi
  if [[ -n "$EXPECTED_AZ" ]]; then
    printf 'expected_availability_zone=%s\n' "$EXPECTED_AZ"
  fi
} > "$LOG"

aws ec2 describe-instances \
  --region "$REGION" \
  --instance-ids "${INSTANCE_IDS[@]}" \
  --query 'Reservations[].Instances[].[InstanceId,State.Name,InstanceType,Placement.AvailabilityZone,Tags[?Key==`Name`]|[0].Value]' \
  --output text | sort > "$STATE_LOG.before"

if [[ -n "$EXPECTED_INSTANCE_TYPE" ]] || [[ -n "$EXPECTED_AZ" ]]; then
  while IFS=$'\t' read -r instance_id state instance_type az name; do
    if [[ -n "$EXPECTED_INSTANCE_TYPE" && "$instance_type" != "$EXPECTED_INSTANCE_TYPE" ]]; then
      echo "instance ${instance_id} has type ${instance_type}, expected ${EXPECTED_INSTANCE_TYPE}" | tee -a "$LOG" >&2
      exit 2
    fi
    if [[ -n "$EXPECTED_AZ" && "$az" != "$EXPECTED_AZ" ]]; then
      echo "instance ${instance_id} is in ${az}, expected ${EXPECTED_AZ}" | tee -a "$LOG" >&2
      exit 2
    fi
    printf 'validated_instance=%s state=%s type=%s az=%s name=%s\n' "$instance_id" "$state" "$instance_type" "$az" "${name:-}" >> "$LOG"
  done < "$STATE_LOG.before"
fi

aws ec2 start-instances \
  --region "$REGION" \
  --instance-ids "${INSTANCE_IDS[@]}" \
  --output json > "$ARTIFACT_DIR/start-topology-instances.json"

aws ec2 wait instance-running \
  --region "$REGION" \
  --instance-ids "${INSTANCE_IDS[@]}"

aws ec2 wait instance-status-ok \
  --region "$REGION" \
  --instance-ids "${INSTANCE_IDS[@]}"

aws ec2 describe-instances \
  --region "$REGION" \
  --instance-ids "${INSTANCE_IDS[@]}" \
  --query 'Reservations[].Instances[].[InstanceId,State.Name,InstanceType,Placement.AvailabilityZone,Tags[?Key==`Name`]|[0].Value]' \
  --output text | sort > "$STATE_LOG.after"

cat "$STATE_LOG.after" > "$STATE_LOG"
echo "started topology instances: ${INSTANCE_IDS[*]}" | tee -a "$LOG"
