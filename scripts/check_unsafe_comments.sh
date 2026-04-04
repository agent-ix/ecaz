#!/usr/bin/env bash
set -euo pipefail

mapfile -t unsafe_lines < <(rg -n 'unsafe\s*\{' src || true)

if [[ ${#unsafe_lines[@]} -eq 0 ]]; then
  exit 0
fi

for entry in "${unsafe_lines[@]}"; do
  file=${entry%%:*}
  rest=${entry#*:}
  line=${rest%%:*}

  start=$(( line > 3 ? line - 3 : 1 ))
  if ! sed -n "${start},${line}p" "$file" | rg -q '// SAFETY:'; then
    echo "missing SAFETY comment near ${file}:${line}" >&2
    exit 1
  fi
done
