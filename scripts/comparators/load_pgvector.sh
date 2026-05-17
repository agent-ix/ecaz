#!/usr/bin/env bash
# Load a prepared corpus TSV into pgvector tables + build HNSW and
# IVFFlat indexes (the two ANN indexes pgvector ships). Independent
# of other comparator load scripts.
set -euo pipefail

COMPARATOR_NAME="pgvector"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/_common.sh"

SIZE="" CORPUS="" QUERIES="" DIM="" DB="${PGDATABASE:-tqvector_bench}"
HNSW_M=16 HNSW_EFC=64 IVFFLAT_LISTS=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --size) SIZE="$2"; shift 2 ;;
    --corpus-file) CORPUS="$2"; shift 2 ;;
    --queries-file) QUERIES="$2"; shift 2 ;;
    --dim) DIM="$2"; shift 2 ;;
    --db) DB="$2"; shift 2 ;;
    --hnsw-m) HNSW_M="$2"; shift 2 ;;
    --hnsw-ef-construction) HNSW_EFC="$2"; shift 2 ;;
    --ivfflat-lists) IVFFLAT_LISTS="$2"; shift 2 ;;
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -15; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

[[ -z "$SIZE" || -z "$CORPUS" || -z "$QUERIES" || -z "$DIM" ]] && {
  echo "Usage: $0 --size <S> --corpus-file <tsv> --queries-file <tsv> --dim <N>"; exit 1;
}

[[ -z "$IVFFLAT_LISTS" ]] && IVFFLAT_LISTS="$(comparator_nlists_for_size "$SIZE")"

export PGDATABASE="$DB" PGHOST="${PGHOST:-/tmp}" PGUSER="${PGUSER:-postgres}"

comparator_require_pgvector
prefix="real_${SIZE}_pgv"

if ! comparator_table_loaded "${prefix}_corpus"; then
  comparator_load_vector_table "${prefix}_corpus" "$CORPUS" "$DIM"
fi
if ! comparator_table_loaded "${prefix}_queries"; then
  comparator_load_vector_table "${prefix}_queries" "$QUERIES" "$DIM"
fi

# HNSW
hnsw_idx="${prefix}_hnsw_idx"
if ! psql -tAc "select 1 from pg_indexes where indexname='$hnsw_idx';" | grep -q 1; then
  comparator_log "building $hnsw_idx (hnsw m=$HNSW_M ef_construction=$HNSW_EFC)"
  psql -c "CREATE INDEX $hnsw_idx ON ${prefix}_corpus USING hnsw (embedding vector_ip_ops) WITH (m = $HNSW_M, ef_construction = $HNSW_EFC);"
fi

# IVFFlat
ivf_idx="${prefix}_ivfflat_idx"
if ! psql -tAc "select 1 from pg_indexes where indexname='$ivf_idx';" | grep -q 1; then
  comparator_log "building $ivf_idx (ivfflat lists=$IVFFLAT_LISTS)"
  psql -c "CREATE INDEX $ivf_idx ON ${prefix}_corpus USING ivfflat (embedding vector_ip_ops) WITH (lists = $IVFFLAT_LISTS);"
fi

comparator_log "done. tables: ${prefix}_corpus, ${prefix}_queries; indexes: $hnsw_idx, $ivf_idx"
