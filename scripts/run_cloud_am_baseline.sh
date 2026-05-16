#!/usr/bin/env bash
# Reproducible AM-level baseline for ec_ivf with two storage formats,
# on a host where the corpora are already loaded (e.g. snapshot restore).
#
# This is the "snapshot-and-bench" companion to the `ecaz bench suite`
# runner (FR-038): the suite runner assumes load+bench from scratch,
# whereas this script assumes the corpus and one storage-format index
# already exist from a restored snapshot, and builds the other storage
# format on demand. Both produce comparable artifact shapes per NFR-015.
#
# For each (corpus prefix x storage_format) combination it runs:
#   - latency  -> <out>/<sf>/<prefix>-latency.log
#   - recall   -> <out>/<sf>/<prefix>-recall.log
#   - storage  -> <out>/<sf>/<prefix>-storage.log
#   - EXPLAIN  -> <out>/<sf>/<prefix>-explain.log
# Plus a top-level manifest.json listing exact commands per artifact.
#
# Index management: builds the requested storage_format if missing.
# Drops the *other* index for the duration of each pass so the planner
# can't pick an off-target index. Restores both at exit so the EBS
# snapshot can be replayed with both formats present.

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/run_cloud_am_baseline.sh --out <dir> --db <database>
    [--host <pghost>] [--user <pguser>] [--port <pgport>]
    [--prefixes "ec_hnsw_real_10k ec_hnsw_real_50k"]
    [--storage-formats "turboquant rabitq"]
    [--iterations N] [--k K] [--concurrency C]
    [--profile-runner "small|large|local"]
    [--ecaz <path>]
    [--keep-other-index]

Required:
  --out <dir>           Artifact directory.
  --db  <name>          Postgres database to connect to.

Defaults:
  PGHOST=/tmp  PGUSER=postgres  PGPORT=5432
  prefixes = "ec_hnsw_real_10k ec_hnsw_real_50k"
  storage_formats = "turboquant rabitq"
  iterations = 200, k = 10, concurrency = 1
  profile-runner = local   (controls nice / sleeps)
  ecaz = /usr/local/bin/ecaz

Outputs:
  <out>/manifest.json              -- per-artifact commands + timing
  <out>/<sf>/<prefix>-latency.log
  <out>/<sf>/<prefix>-recall.log
  <out>/<sf>/<prefix>-storage.log
  <out>/<sf>/<prefix>-explain.log

Example (on m8g.large bench host, snapshot already restored):
  scripts/run_cloud_am_baseline.sh \
      --out /tmp/artifacts/am-baseline \
      --db tqvector_bench \
      --profile-runner small
EOF
}

OUT=""
DB=""
PGHOST_ARG="${PGHOST:-/tmp}"
PGUSER_ARG="${PGUSER:-postgres}"
PGPORT_ARG="${PGPORT:-5432}"
PREFIXES="ec_hnsw_real_10k ec_hnsw_real_50k"
STORAGE_FORMATS="turboquant rabitq"
ITERATIONS=200
K=10
CONCURRENCY=1
PROFILE_RUNNER="local"
ECAZ="${ECAZ_BIN:-/usr/local/bin/ecaz}"
KEEP_OTHER=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --db) DB="$2"; shift 2 ;;
    --host) PGHOST_ARG="$2"; shift 2 ;;
    --user) PGUSER_ARG="$2"; shift 2 ;;
    --port) PGPORT_ARG="$2"; shift 2 ;;
    --prefixes) PREFIXES="$2"; shift 2 ;;
    --storage-formats) STORAGE_FORMATS="$2"; shift 2 ;;
    --iterations) ITERATIONS="$2"; shift 2 ;;
    --k) K="$2"; shift 2 ;;
    --concurrency) CONCURRENCY="$2"; shift 2 ;;
    --profile-runner) PROFILE_RUNNER="$2"; shift 2 ;;
    --ecaz) ECAZ="$2"; shift 2 ;;
    --keep-other-index) KEEP_OTHER=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

[[ -z "$OUT" || -z "$DB" ]] && { usage; exit 1; }

case "$PROFILE_RUNNER" in
  # 2 vCPU / 8 GB (m8g.large class)
  small)  NICE="taskset -c 0 nice -n 10 ionice -c 3 --"; INTER=10 ;;
  # 4 vCPU / 16 GB (m8g.xlarge class) -- recommended default for cloud cycles
  medium) NICE="nice -n 5 --"; INTER=5 ;;
  # 8+ vCPU
  large)  NICE=""; INTER=2 ;;
  # developer workstation
  local)  NICE=""; INTER=0 ;;
  *) echo "bad --profile-runner: $PROFILE_RUNNER (small|medium|large|local)" >&2; exit 1 ;;
esac

mkdir -p "$OUT"
export PGHOST="$PGHOST_ARG" PGUSER="$PGUSER_ARG" PGPORT="$PGPORT_ARG" PGDATABASE="$DB"

MANIFEST="$OUT/manifest.json"
log() { echo "[am-baseline] $(date '+%H:%M:%S') $*"; }

# Map storage_format name -> index name suffix.
idx_name() {
  local prefix="$1" sf="$2"
  case "$sf" in
    turboquant|auto) echo "${prefix}_idx" ;;
    rabitq)          echo "${prefix}_rabitq_idx" ;;
    pq_fastscan)     echo "${prefix}_pqfs_idx" ;;
    *) echo "${prefix}_${sf}_idx" ;;
  esac
}

# Map storage_format -> WITH clause arg, or empty for default (auto).
idx_with() {
  case "$1" in
    turboquant|auto) echo "" ;;
    *) echo "WITH (storage_format='$1')" ;;
  esac
}

ensure_index() {
  local prefix="$1" sf="$2" idx with
  idx="$(idx_name "$prefix" "$sf")"
  with="$(idx_with "$sf")"
  if psql -tAc "select 1 from pg_indexes where indexname='$idx';" | grep -q 1; then
    log "index $idx already present"
  else
    log "building $idx ($with)"
    psql -c "CREATE INDEX $idx ON ${prefix}_corpus USING ec_ivf (embedding) $with;" \
      >> "$OUT/index-builds.log" 2>&1
  fi
}

drop_index_if_exists() {
  local idx="$1"
  if psql -tAc "select 1 from pg_indexes where indexname='$idx';" | grep -q 1; then
    log "dropping $idx (off-target for this pass)"
    psql -c "DROP INDEX $idx;" >> "$OUT/index-builds.log" 2>&1
  fi
}

run_step() {
  local kind="$1" prefix="$2" sf="$3" outdir="$4"
  local logfile cmd
  mkdir -p "$outdir"
  logfile="$outdir/${prefix}-${kind}.log"
  case "$kind" in
    latency)
      cmd="$ECAZ bench latency --prefix $prefix --profile ec_ivf --k $K --iterations $ITERATIONS --concurrency $CONCURRENCY --log-output $logfile"
      ;;
    recall)
      cmd="$ECAZ bench recall --prefix $prefix --profile ec_ivf --k $K --log-output $logfile"
      ;;
    storage)
      cmd="$ECAZ bench storage --prefix $prefix"
      ;;
    explain)
      cmd="psql -c \"EXPLAIN (ANALYZE, BUFFERS) SELECT id FROM ${prefix}_corpus ORDER BY embedding <#> (SELECT embedding FROM ${prefix}_queries LIMIT 1) LIMIT $K;\""
      ;;
    *) echo "unknown step kind $kind"; return 1 ;;
  esac
  log "$kind $prefix sf=$sf"
  {
    echo "# storage_format: $sf"
    echo "# command: $cmd"
    echo "# date:    $(date -Is)"
    echo "# host:    $(uname -a)"
    echo "# ---"
  } > "$logfile.tmp"
  local start end rc
  start=$(date +%s.%N)
  if [[ "$kind" == "storage" ]]; then
    $NICE $ECAZ bench storage --prefix "$prefix" >> "$logfile.tmp" 2>&1 || true
  elif [[ "$kind" == "explain" ]]; then
    $NICE psql -c "EXPLAIN (ANALYZE, BUFFERS) SELECT id FROM ${prefix}_corpus ORDER BY embedding <#> (SELECT embedding FROM ${prefix}_queries LIMIT 1) LIMIT $K;" >> "$logfile.tmp" 2>&1 || true
  else
    eval "$NICE $cmd" >> "$logfile.tmp" 2>&1 || true
    # The ecaz bench latency/recall already writes <logfile>; append its output.
    if [[ -f "$logfile" && "$logfile" != "$logfile.tmp" ]]; then
      cat "$logfile" >> "$logfile.tmp"
    fi
  fi
  end=$(date +%s.%N)
  mv "$logfile.tmp" "$logfile"
  rc=$?
  printf '  {"kind":"%s","prefix":"%s","storage_format":"%s","artifact":"%s","duration_s":%.2f,"exit":%d}\n' \
    "$kind" "$prefix" "$sf" "$logfile" "$(echo "$end - $start" | bc)" "$rc" >> "$MANIFEST.tmp"
  sleep "$INTER"
}

# Restore any missing indexes on exit so the box is left in a known
# state ready for an EBS snapshot.
restore_all_indexes() {
  for prefix in $PREFIXES; do
    for sf in $STORAGE_FORMATS; do
      ensure_index "$prefix" "$sf" || true
    done
  done
}
trap restore_all_indexes EXIT

# Begin manifest
{
  echo '{'
  printf '  "suite": "cloud-am-baseline",\n'
  printf '  "started_utc": "%s",\n' "$(date -u -Iseconds)"
  printf '  "host": "%s",\n' "$(uname -n)"
  printf '  "arch": "%s",\n' "$(uname -m)"
  printf '  "kernel": "%s",\n' "$(uname -r)"
  printf '  "db": "%s",\n' "$DB"
  printf '  "ecaz_bin": "%s",\n' "$ECAZ"
  printf '  "ecaz_version": "%s",\n' "$($ECAZ --version 2>&1 | head -1)"
  printf '  "prefixes": "%s",\n' "$PREFIXES"
  printf '  "storage_formats": "%s",\n' "$STORAGE_FORMATS"
  printf '  "iterations": %d,\n' "$ITERATIONS"
  printf '  "k": %d,\n' "$K"
  printf '  "concurrency": %d,\n' "$CONCURRENCY"
  printf '  "profile_runner": "%s",\n' "$PROFILE_RUNNER"
  printf '  "steps": [\n'
} > "$MANIFEST"
: > "$MANIFEST.tmp"

# Iterate storage formats; for each, ensure target index, drop others.
for sf in $STORAGE_FORMATS; do
  SF_DIR="$OUT/$sf"
  mkdir -p "$SF_DIR"
  for prefix in $PREFIXES; do
    ensure_index "$prefix" "$sf"
    if [[ $KEEP_OTHER -eq 0 ]]; then
      for other_sf in $STORAGE_FORMATS; do
        [[ "$other_sf" == "$sf" ]] && continue
        drop_index_if_exists "$(idx_name "$prefix" "$other_sf")"
      done
    fi
    for kind in latency recall storage explain; do
      run_step "$kind" "$prefix" "$sf" "$SF_DIR"
    done
  done
done

# Finalize manifest
{
  # commas between entries
  sed '$!s/$/,/' "$MANIFEST.tmp"
  echo '  ],'
  printf '  "finished_utc": "%s"\n' "$(date -u -Iseconds)"
  echo '}'
} >> "$MANIFEST"
rm -f "$MANIFEST.tmp"

log "done. manifest at $MANIFEST"
