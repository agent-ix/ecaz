#!/usr/bin/env bash
# Load corpus into Lantern tables + build HNSW (USearch-backed) index.
set -euo pipefail

COMPARATOR_NAME="lantern"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/_common.sh"

SIZE="" CORPUS="" QUERIES="" DIM="" DB="${PGDATABASE:-tqvector_bench}"
HNSW_M=16 HNSW_EFC=128

while [[ $# -gt 0 ]]; do
  case "$1" in
    --size) SIZE="$2"; shift 2 ;;
    --corpus-file) CORPUS="$2"; shift 2 ;;
    --queries-file) QUERIES="$2"; shift 2 ;;
    --dim) DIM="$2"; shift 2 ;;
    --db) DB="$2"; shift 2 ;;
    --hnsw-m) HNSW_M="$2"; shift 2 ;;
    --hnsw-ef-construction) HNSW_EFC="$2"; shift 2 ;;
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -10; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

[[ -z "$SIZE" || -z "$CORPUS" || -z "$QUERIES" || -z "$DIM" ]] && {
  echo "Usage: $0 --size <S> --corpus-file <tsv> --queries-file <tsv> --dim <N>"; exit 1;
}

export PGDATABASE="$DB" PGHOST="${PGHOST:-/tmp}" PGUSER="${PGUSER:-postgres}"

if ! comparator_extension_available_in_pg lantern; then
  comparator_log "lantern ext not installed; run install_lantern.sh"; exit 1
fi
comparator_require_pgvector  # we use vector(N) for cross-comparator consistency
psql -c "CREATE EXTENSION IF NOT EXISTS lantern;" >/dev/null 2>&1

prefix="real_${SIZE}_lantern"
if ! comparator_table_loaded "${prefix}_corpus"; then
  comparator_load_vector_table "${prefix}_corpus" "$CORPUS" "$DIM"
fi
if ! comparator_table_loaded "${prefix}_queries"; then
  comparator_load_vector_table "${prefix}_queries" "$QUERIES" "$DIM"
fi

idx="${prefix}_hnsw_idx"
if ! psql -tAc "select 1 from pg_indexes where indexname='$idx';" | grep -q 1; then
  comparator_log "building $idx (lantern_hnsw m=$HNSW_M ef_construction=$HNSW_EFC)"
  psql -c "CREATE INDEX $idx ON ${prefix}_corpus USING lantern_hnsw (embedding dist_cos_ops) WITH (m = $HNSW_M, ef_construction = $HNSW_EFC);"
fi

comparator_log "done. tables: ${prefix}_corpus, ${prefix}_queries; index: $idx"
