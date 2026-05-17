#!/usr/bin/env bash
# Latency bench for pgvector (HNSW and IVFFlat indexes).
# Writes per-(index) latency.log under <out>/<size>/pgv/<idx>/.
set -euo pipefail

COMPARATOR_NAME="pgvector"
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
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -8; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

[[ -z "$OUT" || -z "$SIZE" ]] && { echo "Usage: $0 --out <dir> --size <S>"; exit 1; }
export PGDATABASE="$DB" PGHOST="${PGHOST:-/tmp}" PGUSER="${PGUSER:-postgres}"
prefix="real_${SIZE}_pgv"

# pgvector ships hnsw + ivfflat. Run each in its own pass; drop the
# other so planner can't pick it.
for kind in hnsw ivfflat; do
  this="${prefix}_${kind}_idx"
  other_kind=$([[ "$kind" == "hnsw" ]] && echo ivfflat || echo hnsw)
  other="${prefix}_${other_kind}_idx"
  psql -tAc "select 1 from pg_indexes where indexname='$other';" | grep -q 1 && \
    psql -c "DROP INDEX $other;"

  comparator_bench_latency \
    --prefix "$prefix" \
    --op "<#>" \
    --outdir "$OUT/$SIZE/pgv/$kind" \
    --iterations "$ITERATIONS" \
    --k "$K"
done

# Rebuild dropped indexes so the EBS snapshot is left in a usable state.
"$SCRIPT_DIR/load_pgvector.sh" --size "$SIZE" \
  --corpus-file /dev/null --queries-file /dev/null --dim 1 2>/dev/null || true
# (The load script is idempotent and won't reload tables; it just
#  re-creates missing indexes via its CREATE INDEX guards.)
