#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -eq 0 ]]; then
  set -- src
fi

pattern_rg='unsafe\s*\{'
pattern_grep='unsafe[[:space:]]*\{'

if command -v rg >/dev/null 2>&1; then
  status=0
  output=$(rg --count-matches --with-filename "${pattern_rg}" "$@" || status=$?)
  if [[ "${status}" -gt 1 ]]; then
    exit "${status}"
  fi
else
  status=0
  output=$(grep -R -E -H -c "${pattern_grep}" "$@" || status=$?)
  if [[ "${status}" -gt 1 ]]; then
    exit "${status}"
  fi
fi

printf '%s\n' "${output}" |
  awk -F: '
    NF >= 2 && $NF > 0 {
      count = $NF
      file = $1
      for (i = 2; i < NF; i++) {
        file = file ":" $i
      }
      printf "%4d %s\n", count, file
    }
  ' |
  sort -k1,1nr -k2,2
