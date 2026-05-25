#!/usr/bin/env bash
set -euo pipefail

bucket_prefix="ecaz-spire-aws-"
secret_prefix="ecaz-spire-aws"

usage() {
  cat <<'EOF'
usage: preflight-permissions.sh [--bucket-prefix PREFIX] [--secret-prefix PREFIX]

Read-only AWS permission preflight for SPIRE AWS runs. This does not create,
modify, or delete resources. It verifies the operator identity can enumerate
SPIRE cleanup-sensitive resources before provisioning starts.
EOF
}

while (($#)); do
  case "$1" in
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

failed=0

identity_arn="$(aws sts get-caller-identity --query Arn --output text)"
printf 'AWS identity: %s\n' "$identity_arn"

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
  if aws s3api list-object-versions \
    --bucket "$bucket" \
    --max-items 1 \
    --query '{Versions:length(Versions || `[]`),DeleteMarkers:length(DeleteMarkers || `[]`)}' \
    --output json >/dev/null; then
    printf 'S3 version-list permission ok for bucket %s\n' "$bucket"
  else
    printf 'ERROR: missing s3:ListBucketVersions for bucket %s\n' "$bucket" >&2
    failed=1
  fi
done

if aws secretsmanager list-secrets \
  --include-planned-deletion \
  --query "length(SecretList[?starts_with(Name, \`${secret_prefix}\`)])" \
  --output text >/dev/null; then
  printf 'Secrets Manager list permission ok for prefix %s\n' "$secret_prefix"
else
  printf 'ERROR: missing Secrets Manager list permission for prefix %s\n' "$secret_prefix" >&2
  failed=1
fi

if ((failed)); then
  exit 2
fi

printf 'SPIRE AWS permission preflight passed\n'
