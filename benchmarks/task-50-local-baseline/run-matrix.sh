#!/bin/bash
# Task 50 local baseline AM/storage matrix runner.
#
# For each (profile, surface) pair, this script loads the corpus under an
# isolated PG prefix (one corpus table per AM variant, per the
# index-isolation rule), then captures recall, latency, and storage logs
# into the packet's artifacts/ directory.
#
# Idempotent: skips a row when all four logs (load+recall+latency+storage)
# exist and are non-empty. Per-row failures are recorded in
# artifacts/matrix-status.tsv and do not abort the script.
#
# Re-run from the repo root:
#   benchmarks/task-50-local-baseline/run-matrix.sh
#
# Override which profiles or surfaces to run with PROFILES/SURFACES env vars.
set -u
set -o pipefail

REPO_ROOT="${REPO_ROOT:-/home/peter/dev/ecaz}"
STAGE="${STAGE:-$REPO_ROOT/target/real-corpus/staged-task50}"
ART="${ART:-$REPO_ROOT/benchmarks/task-50-local-baseline/artifacts}"
ECAZ="${ECAZ:-$REPO_ROOT/target/release/ecaz}"
STATUS_FILE="$ART/matrix-status.tsv"

export PGHOST="${PGHOST:-/home/peter/.pgrx}"
export PGPORT="${PGPORT:-28818}"

DEFAULT_PROFILES="ec_real_10k ec_real_25k ec_real_50k ec_real_100k ec_real_ann_benchmarks_anchor"
PROFILES_LIST="${PROFILES:-$DEFAULT_PROFILES}"

# Each surface entry: "label am storage_format". Empty storage_format means none.
# HNSW is intentionally last: on the 990k anchor it may not build, or may take
# very long. Putting it after the others means the higher-priority lanes land
# evidence first and an HNSW timeout doesn't gate the rest.
DEFAULT_SURFACES=$'ivfrabitq\tec_ivf\trabitq\nspirerabitq\tec_spire\trabitq\ndiskann\tec_diskann\t\nhnsw\tec_hnsw\t'
SURFACES_LIST="${SURFACES:-$DEFAULT_SURFACES}"

mkdir -p "$ART"
if [ ! -s "$STATUS_FILE" ]; then
  printf "timestamp\tprofile\tlabel\tstep\tstatus\tnotes\n" >"$STATUS_FILE"
fi

record() {
  printf "%s\t%s\t%s\t%s\t%s\t%s\n" "$(date -Iseconds)" "$1" "$2" "$3" "$4" "$5" >>"$STATUS_FILE"
}

run_step() {
  local profile="$1" label="$2" step="$3" log="$4"
  shift 4
  echo "=== [$profile / $label] $step"
  # Also tee to the log; ecaz --log-file is best-effort but some commands
  # have been observed to leave it empty. Use PIPESTATUS to recover the
  # ecaz exit code through the pipe.
  ("$@" 2>&1; echo "__ecaz_exit__=$?") | tee -a "$log"
  local rc
  rc=$(tail -n 5 "$log" | grep -oE '__ecaz_exit__=[0-9]+' | tail -n 1 | cut -d= -f2)
  rc=${rc:-1}
  if [ "$rc" = "0" ]; then
    record "$profile" "$label" "$step" "ok" "$log"
    return 0
  else
    record "$profile" "$label" "$step" "FAIL(rc=$rc)" "$log"
    return "$rc"
  fi
}

run_one() {
  local profile="$1" label="$2" am="$3" fmt="$4"
  local pgprefix="${profile}_${label}"
  local load_log="$ART/corpus-load-${profile}-${label}.log"
  local recall_log="$ART/recall-${profile}-${label}.log"
  local latency_log="$ART/latency-${profile}-${label}.log"
  local storage_log="$ART/storage-${profile}-${label}.log"

  if [ -s "$load_log" ] && [ -s "$recall_log" ] && [ -s "$latency_log" ] && [ -s "$storage_log" ]; then
    echo "--- [$profile / $label] all four logs present and non-empty; skipping"
    return 0
  fi

  local fmt_args=()
  [ -n "$fmt" ] && fmt_args=(--storage-format "$fmt")

  if [ ! -s "$load_log" ]; then
    run_step "$profile" "$label" load "$load_log" \
      "$ECAZ" corpus load \
        --prefix "$pgprefix" \
        --profile "$am" \
        "${fmt_args[@]}" \
        --corpus-file   "$STAGE/${profile}_corpus.tsv" \
        --queries-file  "$STAGE/${profile}_queries.tsv" \
        --manifest-file "$STAGE/${profile}_manifest.json" \
        --allow-manifest-mismatch \
        --log-file "$load_log" \
      || { echo "    load failed; skipping the rest of this row"; return 0; }
  else
    echo "--- load log already present; assuming corpus loaded"
  fi

  [ -s "$recall_log" ] || run_step "$profile" "$label" recall "$recall_log" \
    "$ECAZ" bench recall  --prefix "$pgprefix" --profile "$am" --log-file "$recall_log" || true

  [ -s "$latency_log" ] || run_step "$profile" "$label" latency "$latency_log" \
    "$ECAZ" bench latency --prefix "$pgprefix" --profile "$am" --log-file "$latency_log" || true

  [ -s "$storage_log" ] || run_step "$profile" "$label" storage "$storage_log" \
    "$ECAZ" bench storage --prefix "$pgprefix" --log-file "$storage_log" || true
}

for profile in $PROFILES_LIST; do
  while IFS=$'\t' read -r label am fmt; do
    [ -z "${label:-}" ] && continue
    run_one "$profile" "$label" "$am" "$fmt"
  done <<<"$SURFACES_LIST"
done

echo "matrix complete; per-row status in $STATUS_FILE"
