#!/usr/bin/env bash
# Empty the current Terraform artifact bucket without requiring S3 version APIs.

set -euo pipefail

tf_dir="${1:?usage: empty-artifact-bucket.sh <terraform-dir>}"

bucket="$(terraform -chdir="$tf_dir" output -raw artifact_bucket 2>/dev/null || true)"
if [[ -z "$bucket" || "$bucket" == "null" ]]; then
  exit 0
fi

while true; do
  objects_json="$(aws s3api list-objects-v2 --bucket "$bucket" --output json 2>/dev/null || true)"
  if [[ -z "$objects_json" ]]; then
    exit 0
  fi

  object_count="$(jq '(.Contents // []) | length' <<<"$objects_json")"
  if [[ "$object_count" == "0" ]]; then
    exit 0
  fi

  delete_json="$(jq '{Objects: [(.Contents // [])[] | {Key}], Quiet: true}' <<<"$objects_json")"
  aws s3api delete-objects --bucket "$bucket" --delete "$delete_json" >/dev/null
done
