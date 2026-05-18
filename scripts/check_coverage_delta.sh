#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage: scripts/check_coverage_delta.sh SUMMARY BASELINE [CHANGED_FILES]

SUMMARY is cargo-llvm-cov's summary.txt.
BASELINE is a TSV with: path<TAB>line_coverage_percent<TAB>note.
When CHANGED_FILES is present, only matching changed baseline paths are checked.
Coverage may drop at most 2.00 percentage points from baseline.
EOF
}

summary="${1:-}"
baseline="${2:-}"
changed="${3:-}"
if [ -z "$summary" ] || [ -z "$baseline" ]; then
  usage >&2
  exit 2
fi
if [ ! -f "$summary" ]; then
  echo "missing coverage summary: $summary" >&2
  exit 2
fi
if [ ! -f "$baseline" ]; then
  echo "missing coverage baseline: $baseline" >&2
  exit 2
fi

tmp_summary="$(mktemp)"
tmp_changed="$(mktemp)"
trap 'rm -f "$tmp_summary" "$tmp_changed"' EXIT

awk '
  /^[^-[:space:]][^[:space:]]+[[:space:]]+[0-9]/ {
    gsub(/%/, "", $10);
    print $1 "\t" $10;
  }
' "$summary" > "$tmp_summary"

if [ -n "$changed" ]; then
  if [ ! -f "$changed" ]; then
    echo "missing changed-files list: $changed" >&2
    exit 2
  fi
  sed 's#^src/##' "$changed" > "$tmp_changed"
else
  : > "$tmp_changed"
fi

status=0
while IFS=$'\t' read -r path expected _note; do
  case "$path" in
    ""|\#*) continue ;;
  esac
  if [ -n "$changed" ] && ! grep -Fxq "$path" "$tmp_changed" && ! grep -Fxq "src/$path" "$changed"; then
    continue
  fi
  actual="$(awk -v p="$path" '$1 == p { print $2; found=1 } END { if (!found) exit 1 }' "$tmp_summary" || true)"
  if [ -z "$actual" ]; then
    echo "coverage baseline path missing from summary: $path" >&2
    status=1
    continue
  fi
  if ! awk -v actual="$actual" -v expected="$expected" 'BEGIN { exit !((expected - actual) <= 2.000001) }'; then
    printf 'coverage regression: %s actual=%.2f baseline=%.2f allowed_drop=2.00\n' \
      "$path" "$actual" "$expected" >&2
    status=1
  else
    printf 'coverage ok: %s actual=%.2f baseline=%.2f\n' "$path" "$actual" "$expected"
  fi
done < "$baseline"

exit "$status"
