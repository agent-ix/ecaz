#!/usr/bin/env bash
# Load corpus into pgvectorscale tables + build StreamingDiskANN index.
set -euo pipefail

COMPARATOR_NAME="pgvectorscale"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/_common.sh"

SIZE="" CORPUS="" QUERIES="" DIM="" DB="${PGDATABASE:-tqvector_bench}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --size) SIZE="$2"; shift 2 ;;
    --corpus-file) CORPUS="$2"; shift 2 ;;
    --queries-file) QUERIES="$2"; shift 2 ;;
    --dim) DIM="$2"; shift 2 ;;
    --db) DB="$2"; shift 2 ;;
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -10; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

[[ -z "$SIZE" || -z "$CORPUS" || -z "$QUERIES" || -z "$DIM" ]] && {
  echo "Usage: $0 --size <S> --corpus-file <tsv> --queries-file <tsv> --dim <N>"; exit 1;
}

export PGDATABASE="$DB" PGHOST="${PGHOST:-/tmp}" PGUSER="${PGUSER:-postgres}"

if ! comparator_extension_available_in_pg vectorscale; then
  comparator_log "vectorscale ext not installed; run install_pgvectorscale.sh"; exit 1
fi
comparator_require_pgvector
psql -c "CREATE EXTENSION IF NOT EXISTS vectorscale CASCADE;" >/dev/null 2>&1

prefix="real_${SIZE}_pgvscale"
if ! comparator_table_loaded "${prefix}_corpus"; then
  comparator_load_vector_table "${prefix}_corpus" "$CORPUS" "$DIM"
fi
if ! comparator_table_loaded "${prefix}_queries"; then
  comparator_load_vector_table "${prefix}_queries" "$QUERIES" "$DIM"
fi

idx="${prefix}_diskann_idx"
if ! psql -tAc "select 1 from pg_indexes where indexname='$idx';" | grep -q 1; then
  comparator_log "building $idx (pgvectorscale StreamingDiskANN, default config)"
  psql -c "CREATE INDEX $idx ON ${prefix}_corpus USING diskann (embedding vector_ip_ops);"
fi

comparator_log "done. tables: ${prefix}_corpus, ${prefix}_queries; index: $idx"
