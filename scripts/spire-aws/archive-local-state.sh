#!/usr/bin/env bash
set -euo pipefail

state_file="${1:?usage: archive-local-state.sh <terraform.tfstate> <artifact-dir>}"
artifact_dir="${2:?usage: archive-local-state.sh <terraform.tfstate> <artifact-dir>}"

if [[ ! -f "$state_file" ]]; then
  printf 'No local Terraform state exists at %s; nothing to archive\n' "$state_file"
  exit 0
fi

mkdir -p "$artifact_dir"
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
base="$(basename "$state_file")"
archive_path="${artifact_dir}/${base}.${timestamp}.archived"
backup_path="${state_file}.${timestamp}.archived"

cp "$state_file" "$archive_path"
mv "$state_file" "$backup_path"

if [[ -f "${state_file}.backup" ]]; then
  cp "${state_file}.backup" "${artifact_dir}/${base}.backup.${timestamp}.archived"
  mv "${state_file}.backup" "${state_file}.backup.${timestamp}.archived"
fi

printf 'Archived local Terraform state to %s\n' "$archive_path"
printf 'Moved local Terraform state aside to %s\n' "$backup_path"
