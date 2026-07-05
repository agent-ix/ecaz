#!/usr/bin/env bash
set -euo pipefail

baseline_file="${1:-scripts/unsafe_comment_baseline.txt}"

if [[ ! -f "$baseline_file" ]]; then
  echo "missing unsafe comment baseline: ${baseline_file}" >&2
  exit 1
fi

total=$(wc -l < "$baseline_file" | tr -d ' ')
files=$(awk -F: '{ seen[$1] = 1 } END { print length(seen) }' "$baseline_file")

echo "unsafe comment baseline"
echo "file: ${baseline_file}"
echo "entries: ${total}"
echo "files: ${files}"
echo

echo "top directories"
awk -F/ '
  {
    if (NF >= 3) {
      key = $1 "/" $2;
    } else {
      key = $1;
    }
    count[key]++;
  }
  END {
    for (key in count) {
      print count[key], key;
    }
  }
' "$baseline_file" | sort -nr | head -20

echo
echo "top files"
awk -F: '
  { count[$1]++ }
  END {
    for (file in count) {
      print count[file], file;
    }
  }
' "$baseline_file" | sort -nr | head -40
