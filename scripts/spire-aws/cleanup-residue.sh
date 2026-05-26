#!/usr/bin/env bash
set -euo pipefail

execute=0
bucket_prefix="ecaz-spire-aws-"
secret_prefix="ecaz-spire-aws"
phase_label="13-spire-aws-verification"
vpc_name="ecaz-spire-aws"
iam_role_name="ecaz-spire-aws-node"
iam_profile_name="ecaz-spire-aws-node"
allow_preexisting_residue="${SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE:-0}"

usage() {
  cat <<'EOF'
usage: cleanup-residue.sh [--execute] [--allow-preexisting-residue]
                          [--bucket-prefix PREFIX] [--secret-prefix PREFIX]
                          [--phase-label PHASE] [--vpc-name NAME]
                          [--iam-role-name NAME] [--iam-profile-name NAME]

Dry-run by default. Lists SPIRE AWS residue buckets, secrets, Phase 13 VPCs,
and the static node IAM role/profile. When --execute is supplied, deletes all
object versions/delete markers before deleting matching buckets, force-deletes
matching secrets, removes matching Phase 13 VPC resources, and deletes the
static node IAM role/profile.

By default, missing S3 version-list permission on matching buckets fails. Use
--allow-preexisting-residue only when a packet has documented that the buckets
are old residue outside the new run and local state has been archived.
EOF
}

while (($#)); do
  case "$1" in
    --execute)
      execute=1
      shift
      ;;
    --allow-preexisting-residue)
      allow_preexisting_residue=1
      shift
      ;;
    --bucket-prefix)
      bucket_prefix="${2:?missing value for --bucket-prefix}"
      shift 2
      ;;
    --secret-prefix)
      secret_prefix="${2:?missing value for --secret-prefix}"
      shift 2
      ;;
    --phase-label)
      phase_label="${2:?missing value for --phase-label}"
      shift 2
      ;;
    --vpc-name)
      vpc_name="${2:?missing value for --vpc-name}"
      shift 2
      ;;
    --iam-role-name)
      iam_role_name="${2:?missing value for --iam-role-name}"
      shift 2
      ;;
    --iam-profile-name)
      iam_profile_name="${2:?missing value for --iam-profile-name}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'ERROR: unknown argument: %s\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

mode="dry-run"
if ((execute)); then
  mode="execute"
fi
printf 'SPIRE AWS residue cleanup mode: %s\n' "$mode"
failed=0

delete_json_file() {
  local path="$1"
  if [[ -f "$path" ]]; then
    rm -f "$path"
  fi
}

list_vpcs() {
  aws ec2 describe-vpcs \
    --filters "Name=tag:Phase,Values=${phase_label}" "Name=tag:Name,Values=${vpc_name}" \
    --query 'Vpcs[].VpcId' \
    --output text | tr '\t' '\n' | sed '/^$/d'
}

json_array_length() {
  jq 'length' "$1"
}

delete_security_group_rules() {
  local sg_id="$1"
  local ingress_json
  local egress_json

  ingress_json="$(mktemp)"
  egress_json="$(mktemp)"
  aws ec2 describe-security-groups \
    --group-ids "$sg_id" \
    --query 'SecurityGroups[0].IpPermissions' \
    --output json >"$ingress_json"
  aws ec2 describe-security-groups \
    --group-ids "$sg_id" \
    --query 'SecurityGroups[0].IpPermissionsEgress' \
    --output json >"$egress_json"

  if (($(json_array_length "$ingress_json") > 0)); then
    aws ec2 revoke-security-group-ingress \
      --group-id "$sg_id" \
      --ip-permissions "file://${ingress_json}" >/dev/null || true
  fi
  if (($(json_array_length "$egress_json") > 0)); then
    aws ec2 revoke-security-group-egress \
      --group-id "$sg_id" \
      --ip-permissions "file://${egress_json}" >/dev/null || true
  fi

  delete_json_file "$ingress_json"
  delete_json_file "$egress_json"
}

wait_for_vpc_endpoints_deleted() {
  local vpc_id="$1"
  local remaining

  for _ in $(seq 1 60); do
    remaining="$(aws ec2 describe-vpc-endpoints \
      --filters "Name=vpc-id,Values=${vpc_id}" \
      --query 'length(VpcEndpoints[])' \
      --output text)"
    if [[ "$remaining" == "0" ]]; then
      return 0
    fi
    sleep 5
  done

  printf 'ERROR: timed out waiting for VPC endpoints to delete in %s\n' "$vpc_id" >&2
  return 1
}

mapfile -t buckets < <(aws s3api list-buckets \
  --query "Buckets[?starts_with(Name, \`${bucket_prefix}\`)].Name" \
  --output text | tr '\t' '\n' | sed '/^$/d')

if ((${#buckets[@]} == 0)); then
  printf 'No S3 buckets matched prefix %s\n' "$bucket_prefix"
else
  printf 'S3 buckets matched prefix %s:\n' "$bucket_prefix"
  printf '  %s\n' "${buckets[@]}"
fi

for bucket in "${buckets[@]}"; do
  versions_json="$(mktemp)"
  delete_json="$(mktemp)"
  if ! aws s3api list-object-versions --bucket "$bucket" --output json >"$versions_json"; then
    if [[ "$allow_preexisting_residue" == "1" ]]; then
      printf 'WARNING: cannot list object versions for pre-existing residue bucket %s; leaving bucket in place because override is set\n' "$bucket" >&2
    else
      printf 'ERROR: cannot list object versions for %s; grant s3:ListBucketVersions before cleanup can proceed\n' "$bucket" >&2
      failed=1
    fi
    delete_json_file "$versions_json"
    delete_json_file "$delete_json"
    continue
  fi

  jq '[
    (.Versions // [])[] | {Key: .Key, VersionId: .VersionId},
    (.DeleteMarkers // [])[] | {Key: .Key, VersionId: .VersionId}
  ] | {Objects: .}' "$versions_json" >"$delete_json"

  object_count="$(jq '.Objects | length' "$delete_json")"
  printf 'Bucket %s has %s object versions/delete markers\n' "$bucket" "$object_count"

  if ((execute)); then
    if ((object_count > 0)); then
      aws s3api delete-objects --bucket "$bucket" --delete "file://${delete_json}" >/dev/null
    fi
    aws s3api delete-bucket --bucket "$bucket"
    printf 'Deleted bucket %s\n' "$bucket"
  fi
done

mapfile -t secrets < <(aws secretsmanager list-secrets --include-planned-deletion \
  --query "SecretList[?starts_with(Name, \`${secret_prefix}\`)].ARN" \
  --output text | tr '\t' '\n' | sed '/^$/d')

if ((${#secrets[@]} == 0)); then
  printf 'No Secrets Manager secrets matched prefix %s\n' "$secret_prefix"
else
  printf 'Secrets Manager secrets matched prefix %s:\n' "$secret_prefix"
  printf '  %s\n' "${secrets[@]}"
fi

if ((execute)); then
  for secret_arn in "${secrets[@]}"; do
    aws secretsmanager delete-secret \
      --secret-id "$secret_arn" \
      --force-delete-without-recovery >/dev/null
    printf 'Force-deleted secret %s\n' "$secret_arn"
  done
fi

mapfile -t vpcs < <(list_vpcs)

if ((${#vpcs[@]} == 0)); then
  printf 'No VPCs matched Phase=%s and Name=%s\n' "$phase_label" "$vpc_name"
else
  printf 'VPCs matched Phase=%s and Name=%s:\n' "$phase_label" "$vpc_name"
  printf '  %s\n' "${vpcs[@]}"
fi

for vpc_id in "${vpcs[@]}"; do
  mapfile -t instance_ids < <(aws ec2 describe-instances \
    --filters "Name=vpc-id,Values=${vpc_id}" "Name=instance-state-name,Values=pending,running,stopping,stopped" \
    --query 'Reservations[].Instances[].InstanceId' \
    --output text | tr '\t' '\n' | sed '/^$/d')
  mapfile -t endpoint_ids < <(aws ec2 describe-vpc-endpoints \
    --filters "Name=vpc-id,Values=${vpc_id}" \
    --query 'VpcEndpoints[].VpcEndpointId' \
    --output text | tr '\t' '\n' | sed '/^$/d')
  mapfile -t route_assoc_ids < <(aws ec2 describe-route-tables \
    --filters "Name=vpc-id,Values=${vpc_id}" \
    --query 'RouteTables[].Associations[?Main==`false`].RouteTableAssociationId[]' \
    --output text | tr '\t' '\n' | sed '/^$/d')
  mapfile -t route_table_ids < <(aws ec2 describe-route-tables \
    --filters "Name=vpc-id,Values=${vpc_id}" \
    --output json | jq -r '.RouteTables[] | select(any(.Associations[]?; .Main == true) | not) | .RouteTableId')
  mapfile -t subnet_ids < <(aws ec2 describe-subnets \
    --filters "Name=vpc-id,Values=${vpc_id}" \
    --query 'Subnets[].SubnetId' \
    --output text | tr '\t' '\n' | sed '/^$/d')
  mapfile -t security_group_ids < <(aws ec2 describe-security-groups \
    --filters "Name=vpc-id,Values=${vpc_id}" \
    --query 'SecurityGroups[?GroupName!=`default`].GroupId' \
    --output text | tr '\t' '\n' | sed '/^$/d')

  printf 'VPC %s residue: instances=%s endpoints=%s subnets=%s route_tables=%s security_groups=%s\n' \
    "$vpc_id" \
    "${#instance_ids[@]}" \
    "${#endpoint_ids[@]}" \
    "${#subnet_ids[@]}" \
    "${#route_table_ids[@]}" \
    "${#security_group_ids[@]}"

  if ((execute)); then
    if ((${#instance_ids[@]} > 0)); then
      aws ec2 terminate-instances --instance-ids "${instance_ids[@]}" >/dev/null
      aws ec2 wait instance-terminated --instance-ids "${instance_ids[@]}"
      printf 'Terminated instances in VPC %s\n' "$vpc_id"
    fi

    if ((${#endpoint_ids[@]} > 0)); then
      aws ec2 delete-vpc-endpoints --vpc-endpoint-ids "${endpoint_ids[@]}" >/dev/null
      wait_for_vpc_endpoints_deleted "$vpc_id"
      printf 'Deleted VPC endpoints in VPC %s\n' "$vpc_id"
    fi

    for assoc_id in "${route_assoc_ids[@]}"; do
      aws ec2 disassociate-route-table --association-id "$assoc_id" >/dev/null || true
    done

    for sg_id in "${security_group_ids[@]}"; do
      delete_security_group_rules "$sg_id"
    done
    for sg_id in "${security_group_ids[@]}"; do
      aws ec2 delete-security-group --group-id "$sg_id" >/dev/null
      printf 'Deleted security group %s\n' "$sg_id"
    done

    for subnet_id in "${subnet_ids[@]}"; do
      aws ec2 delete-subnet --subnet-id "$subnet_id" >/dev/null
      printf 'Deleted subnet %s\n' "$subnet_id"
    done

    for route_table_id in "${route_table_ids[@]}"; do
      aws ec2 delete-route-table --route-table-id "$route_table_id" >/dev/null
      printf 'Deleted route table %s\n' "$route_table_id"
    done

    aws ec2 delete-vpc --vpc-id "$vpc_id" >/dev/null
    printf 'Deleted VPC %s\n' "$vpc_id"
  fi
done

if aws iam get-role --role-name "$iam_role_name" >/dev/null 2>&1; then
  printf 'IAM role matched: %s\n' "$iam_role_name"
  if ((execute)); then
    mapfile -t instance_profiles < <(aws iam list-instance-profiles-for-role \
      --role-name "$iam_role_name" \
      --query 'InstanceProfiles[].InstanceProfileName' \
      --output text | tr '\t' '\n' | sed '/^$/d')
    mapfile -t attached_policy_arns < <(aws iam list-attached-role-policies \
      --role-name "$iam_role_name" \
      --query 'AttachedPolicies[].PolicyArn' \
      --output text | tr '\t' '\n' | sed '/^$/d')
    mapfile -t inline_policy_names < <(aws iam list-role-policies \
      --role-name "$iam_role_name" \
      --query 'PolicyNames[]' \
      --output text | tr '\t' '\n' | sed '/^$/d')

    for profile_name in "${instance_profiles[@]}"; do
      aws iam remove-role-from-instance-profile \
        --instance-profile-name "$profile_name" \
        --role-name "$iam_role_name" >/dev/null || true
      printf 'Removed role %s from instance profile %s\n' "$iam_role_name" "$profile_name"
    done

    for policy_arn in "${attached_policy_arns[@]}"; do
      aws iam detach-role-policy \
        --role-name "$iam_role_name" \
        --policy-arn "$policy_arn" >/dev/null
      printf 'Detached policy %s from role %s\n' "$policy_arn" "$iam_role_name"
    done

    for policy_name in "${inline_policy_names[@]}"; do
      aws iam delete-role-policy \
        --role-name "$iam_role_name" \
        --policy-name "$policy_name" >/dev/null
      printf 'Deleted inline policy %s from role %s\n' "$policy_name" "$iam_role_name"
    done

    aws iam delete-role --role-name "$iam_role_name" >/dev/null
    printf 'Deleted IAM role %s\n' "$iam_role_name"
  fi
else
  printf 'No IAM role matched: %s\n' "$iam_role_name"
fi

if aws iam get-instance-profile --instance-profile-name "$iam_profile_name" >/dev/null 2>&1; then
  printf 'IAM instance profile matched: %s\n' "$iam_profile_name"
  if ((execute)); then
    aws iam delete-instance-profile --instance-profile-name "$iam_profile_name" >/dev/null
    printf 'Deleted IAM instance profile %s\n' "$iam_profile_name"
  fi
else
  printf 'No IAM instance profile matched: %s\n' "$iam_profile_name"
fi

if ((failed)); then
  exit 2
fi
