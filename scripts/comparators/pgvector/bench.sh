#!/usr/bin/env bash
# Latency bench for pgvector (HNSW and IVFFlat indexes).
# Writes per-(index) latency.log under <out>/<size>/pgv/<idx>/.
#
# Per-index isolation via replicated corpus tables (see
# load_pgvector.sh). No drop+rebuild swap dance — each variant has its
# own table with only its own index, so the planner has nothing to
# pick wrong.
set -euo pipefail

COMPARATOR_NAME="pgvector"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../_common.sh"
source "$SCRIPT_DIR/../_bench_lib.sh"

OUT="" SIZE="" DB="${PGDATABASE:-tqvector_bench}" ITERATIONS=200 K=10

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --size) SIZE="$2"; shift 2 ;;
    --db) DB="$2"; shift 2 ;;
    --iterations) ITERATIONS="$2"; shift 2 ;;
    --k) K="$2"; shift 2 ;;
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -10; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

[[ -z "$OUT" || -z "$SIZE" ]] && { echo "Usage: $0 --out <dir> --size <S>"; exit 1; }
export PGDATABASE="$DB" PGHOST="${PGHOST:-/tmp}" PGUSER="${PGUSER:-postgres}"
base="real_${SIZE}_pgv"
queries_tbl="${base}_queries"

for kind in hnsw ivfflat; do
  corpus_tbl="${base}_${kind}_corpus"
  comparator_bench_latency \
    --corpus-table "$corpus_tbl" \
    --queries-table "$queries_tbl" \
    --op "<#>" \
    --outdir "$OUT/$SIZE/pgv/$kind" \
    --iterations "$ITERATIONS" \
    --k "$K"
done
