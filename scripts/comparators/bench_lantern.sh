#!/usr/bin/env bash
# Latency bench for Lantern HNSW.
set -euo pipefail

COMPARATOR_NAME="lantern"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/_common.sh"
source "$SCRIPT_DIR/_bench_lib.sh"

OUT="" SIZE="" DB="${PGDATABASE:-tqvector_bench}" ITERATIONS=200 K=10
while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --size) SIZE="$2"; shift 2 ;;
    --db) DB="$2"; shift 2 ;;
    --iterations) ITERATIONS="$2"; shift 2 ;;
    --k) K="$2"; shift 2 ;;
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -6; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

[[ -z "$OUT" || -z "$SIZE" ]] && { echo "Usage: $0 --out <dir> --size <S>"; exit 1; }
export PGDATABASE="$DB" PGHOST="${PGHOST:-/tmp}" PGUSER="${PGUSER:-postgres}"

# Lantern HNSW uses cosine distance by default (dist_cos_ops).
comparator_bench_latency \
  --prefix "real_${SIZE}_lantern" \
  --op "<=>" \
  --outdir "$OUT/$SIZE/lantern/hnsw" \
  --iterations "$ITERATIONS" --k "$K"
