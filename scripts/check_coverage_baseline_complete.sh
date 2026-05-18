#!/usr/bin/env bash
set -euo pipefail

baseline="${1:-fixtures/quality/coverage-baseline.tsv}"
if [ ! -f "$baseline" ]; then
  echo "missing coverage baseline: $baseline" >&2
  exit 2
fi

tmp_expected="$(mktemp)"
tmp_actual="$(mktemp)"
trap 'rm -f "$tmp_expected" "$tmp_actual"' EXIT

{
  find src/quant -maxdepth 1 -type f -name '*.rs' ! -name traits.rs
  find src/storage -maxdepth 1 -type f \( -name 'page.rs' -o -name '*_guard.rs' \)
  find src/am -mindepth 2 -maxdepth 2 -type f -name 'page.rs'
  find src/am/ec_spire/storage -maxdepth 1 -type f -name '*.rs' ! -name 'tests.rs'
  printf '%s\n' \
    src/am/ec_spire/coordinator/diagnostics.rs \
    src/am/ec_diskann/build.rs \
    src/am/ec_diskann/routine.rs \
    src/am/ec_diskann/scan.rs \
    src/am/common/cost.rs
} | sed 's#^src/##' | sort -u > "$tmp_expected"

awk -F '\t' '!/^#/ && NF >= 2 { print $1 }' "$baseline" | sort -u > "$tmp_actual"

missing=0
while IFS= read -r path; do
  if ! grep -Fxq "$path" "$tmp_actual"; then
    echo "coverage baseline missing critical path: $path" >&2
    missing=1
  fi
done < "$tmp_expected"

if [ "$missing" -ne 0 ]; then
  exit 1
fi

echo "coverage baseline complete for $(wc -l < "$tmp_expected" | tr -d ' ') critical paths"
