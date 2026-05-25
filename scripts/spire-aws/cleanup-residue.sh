#!/usr/bin/env bash
set -euo pipefail

execute=0
bucket_prefix="ecaz-spire-aws-"
secret_prefix="ecaz-spire-aws"

usage() {
  cat <<'EOF'
usage: cleanup-residue.sh [--execute] [--bucket-prefix PREFIX] [--secret-prefix PREFIX]

Dry-run by default. Lists SPIRE AWS residue buckets and secrets, and when
--execute is supplied deletes all object versions/delete markers before
deleting matching buckets and force-deletes matching pending secrets.
EOF
}

while (($#)); do
  case "$1" in
    --execute)
      execute=1
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
    printf 'ERROR: cannot list object versions for %s; grant s3:ListBucketVersions before cleanup can proceed\n' "$bucket" >&2
    failed=1
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

if ((failed)); then
  exit 2
fi
