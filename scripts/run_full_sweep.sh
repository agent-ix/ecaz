#!/usr/bin/env bash
# Per-AM, per-size benchmark sweep using the real_<size>_<am>_corpus
# tables produced by scripts/load_multi_am.sh.
#
# For each (size × AM) combination, runs:
#   - ecaz bench latency
#   - ecaz bench recall
#   - ecaz bench storage
#   - EXPLAIN ANALYZE on one representative query
#
# For ec_ivf, sweeps the three colocated storage_format variants
# (auto/turboquant, rabitq, pq_fastscan) via the drop-the-others
# dance so the planner can only pick the target index for each pass.
# Recreates dropped indexes at exit so the EBS snapshot ends in a
# valid state.

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/run_full_sweep.sh --out <dir> [--db <database>]
    [--sizes "10k 50k 100k 1m"]
    [--ams "hnsw ivf diskann"]
    [--ivf-storage-formats "turboquant rabitq pq_fastscan"]
    [--iterations N] [--k K] [--concurrency C]
    [--profile-runner small|medium|large|local]
    [--ecaz <path>]

Required:
  --out <dir>            Artifact root. Each (size/am/storage_format)
                         pass writes under <out>/<size>/<am>/<sf>/.

Defaults:
  PGHOST=/tmp PGUSER=postgres PGDATABASE=tqvector_bench
  sizes = "10k 50k 100k 1m"
  ams = "hnsw ivf diskann"
  ivf-storage-formats = "turboquant rabitq pq_fastscan"
  iterations = 200, k = 10, concurrency = 1
  profile-runner = medium
  ecaz = /usr/local/bin/ecaz
EOF
}

OUT=""
DB="${PGDATABASE:-tqvector_bench}"
SIZES="10k 50k 100k 1m"
AMS="hnsw ivf diskann"
IVF_SFS="turboquant rabitq pq_fastscan"
ITERATIONS=200
K=10
CONCURRENCY=1
PROFILE_RUNNER="medium"
ECAZ="${ECAZ_BIN:-/usr/local/bin/ecaz}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --db) DB="$2"; shift 2 ;;
    --sizes) SIZES="$2"; shift 2 ;;
    --ams) AMS="$2"; shift 2 ;;
    --ivf-storage-formats) IVF_SFS="$2"; shift 2 ;;
    --iterations) ITERATIONS="$2"; shift 2 ;;
    --k) K="$2"; shift 2 ;;
    --concurrency) CONCURRENCY="$2"; shift 2 ;;
    --profile-runner) PROFILE_RUNNER="$2"; shift 2 ;;
    --ecaz) ECAZ="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

[[ -z "$OUT" ]] && { usage; exit 1; }

case "$PROFILE_RUNNER" in
  small)  NICE="taskset -c 0 nice -n 10 ionice -c 3 --"; INTER=10 ;;
  medium) NICE="nice -n 5 --"; INTER=5 ;;
  large)  NICE=""; INTER=2 ;;
  local)  NICE=""; INTER=0 ;;
  *) echo "bad --profile-runner: $PROFILE_RUNNER" >&2; exit 1 ;;
esac

export PGHOST="${PGHOST:-/tmp}" PGUSER="${PGUSER:-postgres}" PGDATABASE="$DB"
mkdir -p "$OUT"

log() { echo "[full-sweep] $(date '+%H:%M:%S') $*"; }
MANIFEST="$OUT/manifest.json"

# ec_ivf storage_format index naming convention:
#   real_<size>_ivf_idx          turboquant (default)
#   real_<size>_ivf_rabitq_idx   storage_format=rabitq
#   real_<size>_ivf_pqfs_idx     storage_format=pq_fastscan
ivf_idx_name() {
  case "$2" in
    auto|turboquant) echo "real_$1_ivf_idx" ;;
    rabitq)          echo "real_$1_ivf_rabitq_idx" ;;
    pq_fastscan|pqfs) echo "real_$1_ivf_pqfs_idx" ;;
    *) echo "real_$1_ivf_${2}_idx" ;;
  esac
}
ivf_idx_with_clause() {
  case "$1" in
    auto|turboquant) echo "" ;;
    *) echo "WITH (storage_format='$1')" ;;
  esac
}

drop_index_if_exists() {
  if psql -tAc "select 1 from pg_indexes where indexname='$1';" | grep -q 1; then
    psql -c "DROP INDEX $1;" >> "$OUT/index-events.log" 2>&1
  fi
}

ensure_ivf_index() {
  local size="$1" sf="$2"
  local idx=$(ivf_idx_name "$size" "$sf")
  local with=$(ivf_idx_with_clause "$sf")
  if ! psql -tAc "select 1 from pg_indexes where indexname='$idx';" | grep -q 1; then
    log "rebuilding $idx ($with) for snapshot-clean exit"
    psql -c "CREATE INDEX $idx ON real_${size}_ivf_corpus USING ec_ivf (embedding) $with;" \
      >> "$OUT/index-events.log" 2>&1
  fi
}

# Restore all ec_ivf storage_format indexes at exit so EBS snapshot
# is left in a usable state for the next cycle.
restore_ivf_indexes() {
  for s in $SIZES; do
    for sf in $IVF_SFS; do
      ensure_ivf_index "$s" "$sf" || true
    done
  done
}
trap restore_ivf_indexes EXIT

run_bench_quad() {
  local prefix="$1" profile="$2" sf_label="$3" outdir="$4"
  mkdir -p "$outdir"
  log "  bench $prefix profile=$profile sf=$sf_label"

  # latency
  $NICE "$ECAZ" bench latency --prefix "$prefix" --profile "$profile" \
    --k "$K" --iterations "$ITERATIONS" --concurrency "$CONCURRENCY" \
    --log-output "$outdir/latency.log" >> "$outdir/latency-stdout.log" 2>&1 || true
  sleep "$INTER"

  # recall
  $NICE "$ECAZ" bench recall --prefix "$prefix" --profile "$profile" \
    --k "$K" --log-output "$outdir/recall.log" >> "$outdir/recall-stdout.log" 2>&1 || true
  sleep "$INTER"

  # storage
  $NICE "$ECAZ" bench storage --prefix "$prefix" > "$outdir/storage.log" 2>&1 || true
  sleep "$INTER"

  # explain (one representative query)
  $NICE psql -c "EXPLAIN (ANALYZE, BUFFERS) SELECT id FROM ${prefix}_corpus ORDER BY embedding <#> (SELECT embedding FROM ${prefix}_queries LIMIT 1) LIMIT $K;" \
    > "$outdir/explain.log" 2>&1 || true

  printf '  {"size":"%s","am":"%s","sf":"%s","prefix":"%s","ts":"%s"}\n' \
    "${prefix##real_}" "$profile" "$sf_label" "$prefix" "$(date -u -Iseconds)" \
    >> "$MANIFEST.tmp"
}

# Begin manifest
: > "$MANIFEST.tmp"
{
  echo '{'
  printf '  "suite": "full-sweep",\n'
  printf '  "started_utc": "%s",\n' "$(date -u -Iseconds)"
  printf '  "host": "%s",\n' "$(uname -n)"
  printf '  "arch": "%s",\n' "$(uname -m)"
  printf '  "kernel": "%s",\n' "$(uname -r)"
  printf '  "db": "%s",\n' "$DB"
  printf '  "ecaz": "%s",\n' "$($ECAZ --version 2>&1 | head -1)"
  printf '  "sizes": "%s",\n' "$SIZES"
  printf '  "ams": "%s",\n' "$AMS"
  printf '  "ivf_storage_formats": "%s",\n' "$IVF_SFS"
  printf '  "iterations": %d,\n' "$ITERATIONS"
  printf '  "k": %d,\n' "$K"
  printf '  "concurrency": %d,\n' "$CONCURRENCY"
  printf '  "profile_runner": "%s",\n' "$PROFILE_RUNNER"
  printf '  "steps": [\n'
} > "$MANIFEST"

# Sweep loop
for size in $SIZES; do
  for am in $AMS; do
    case "$am" in
      hnsw)
        # ec_hnsw: one prefix per size, default index already built via
        # m=8 + m=16 sweep in load_multi_am.sh. The bench harness chooses
        # ef_search via profile defaults.
        run_bench_quad "real_${size}_hnsw" "ec_hnsw" "default" "$OUT/${size}/hnsw/default"
        ;;
      diskann)
        run_bench_quad "real_${size}_diskann" "ec_diskann" "default" "$OUT/${size}/diskann/default"
        ;;
      ivf)
        # Three storage_format passes. For each: drop the others, bench,
        # next pass will see only its target index. Final trap restores
        # all three.
        for sf in $IVF_SFS; do
          log "ivf pass: size=$size sf=$sf"
          # Drop off-target indexes
          for other_sf in $IVF_SFS; do
            [[ "$other_sf" == "$sf" ]] && continue
            drop_index_if_exists "$(ivf_idx_name "$size" "$other_sf")"
          done
          # Ensure target is present
          ensure_ivf_index "$size" "$sf"
          run_bench_quad "real_${size}_ivf" "ec_ivf" "$sf" "$OUT/${size}/ivf/$sf"
        done
        ;;
    esac
  done
done

# Finalize manifest
{
  sed '$!s/$/,/' "$MANIFEST.tmp"
  echo '  ],'
  printf '  "finished_utc": "%s"\n' "$(date -u -Iseconds)"
  echo '}'
} >> "$MANIFEST"
rm -f "$MANIFEST.tmp"

log "sweep complete; artifacts in $OUT; manifest at $MANIFEST"
