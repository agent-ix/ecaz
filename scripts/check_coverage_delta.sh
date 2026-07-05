#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage: scripts/check_coverage_delta.sh [--ratchet] SUMMARY BASELINE [CHANGED_FILES]

SUMMARY is cargo-llvm-cov's summary.txt.
BASELINE is a TSV with: path<TAB>line_coverage_percent<TAB>note.
When CHANGED_FILES is present, only matching changed baseline paths are checked.
Coverage may drop at most 2.00 percentage points from baseline.
With --ratchet, checked paths whose actual coverage improves by more than 2.00
percentage points are rewritten to the current actual value. This is intended
for explicit baseline-update commits, not routine PR checks.
EOF
}

ratchet=false
if [ "${1:-}" = "--ratchet" ]; then
  ratchet=true
  shift
fi

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
tmp_updates="$(mktemp)"
tmp_baseline="$(mktemp)"
trap 'rm -f "$tmp_summary" "$tmp_changed" "$tmp_updates" "$tmp_baseline"' EXIT

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
    if [ "$ratchet" = true ] && awk -v actual="$actual" -v expected="$expected" 'BEGIN { exit !((actual - expected) > 2.000001) }'; then
      printf '%s\t%.2f\n' "$path" "$actual" >> "$tmp_updates"
    fi
  fi
done < "$baseline"

if [ "$ratchet" = true ] && [ "$status" -eq 0 ] && [ -s "$tmp_updates" ]; then
  awk -F '\t' '
    NR == FNR {
      updates[$1] = $2;
      next;
    }
    /^#/ || NF < 2 {
      print;
      next;
    }
    $1 in updates {
      $2 = updates[$1];
    }
    { print $1 "\t" $2 "\t" $3 }
  ' "$tmp_updates" "$baseline" > "$tmp_baseline"
  mv "$tmp_baseline" "$baseline"
  echo "coverage baseline ratcheted: $baseline"
fi

exit "$status"
