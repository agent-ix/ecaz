#!/usr/bin/env bash
# Load a prepared corpus TSV into pgvector tables + build HNSW and
# IVFFlat indexes (the two ANN indexes pgvector ships).
#
# Per-index isolation: HNSW and IVFFlat each live on their OWN
# replicated corpus table so bench passes don't need a drop+rebuild
# swap dance (the planner can't pick the wrong index if only one
# exists on the bench target table). Cost: one extra ~6 GB/M-row
# vector(1536) table per size. See ADR-050 for the analogous ec_*
# pattern.
set -euo pipefail

COMPARATOR_NAME="pgvector"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../_common.sh"

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
base="real_${SIZE}_pgv"

# Per-variant corpus tables. queries table stays shared.
hnsw_tbl="${base}_hnsw_corpus"
ivf_tbl="${base}_ivfflat_corpus"
queries_tbl="${base}_queries"

# Source-load shared queries first.
if ! comparator_table_loaded "$queries_tbl"; then
  comparator_load_vector_table "$queries_tbl" "$QUERIES" "$DIM"
fi

# Source-load HNSW corpus from TSV.
if ! comparator_table_loaded "$hnsw_tbl"; then
  comparator_load_vector_table "$hnsw_tbl" "$CORPUS" "$DIM"
fi

# IVFFlat corpus: clone from HNSW corpus via CTAS rather than re-reading
# the (potentially 6+ GB) TSV. Saves disk I/O and avoids needing the
# source file present for the second pass.
if ! comparator_table_loaded "$ivf_tbl"; then
  comparator_log "  CTAS $ivf_tbl FROM $hnsw_tbl"
  psql -c "DROP TABLE IF EXISTS $ivf_tbl CASCADE;"
  psql -c "CREATE TABLE $ivf_tbl AS TABLE $hnsw_tbl;"
  psql -c "ALTER TABLE $ivf_tbl ADD PRIMARY KEY (id);"
fi

# pgvector's HNSW build needs maintenance_work_mem big enough to hold
# the in-memory graph or it falls back to a much slower disk-based
# path. For 1M x 1536-dim, 4 GB is comfortable. PG default is only
# 64 MB which triggers the "hnsw graph no longer fits into
# maintenance_work_mem after N tuples" warning and a ~10-100x
# slowdown. IVFFlat also benefits but less dramatically.
MAINT_WORK_MEM="${MAINT_WORK_MEM:-4GB}"

# HNSW
hnsw_idx="${base}_hnsw_idx"
if ! psql -tAc "select 1 from pg_indexes where indexname='$hnsw_idx';" | grep -q 1; then
  comparator_log "building $hnsw_idx on $hnsw_tbl (hnsw m=$HNSW_M ef_construction=$HNSW_EFC, maintenance_work_mem=$MAINT_WORK_MEM)"
  psql -c "SET maintenance_work_mem = '$MAINT_WORK_MEM'; CREATE INDEX $hnsw_idx ON $hnsw_tbl USING hnsw (embedding vector_ip_ops) WITH (m = $HNSW_M, ef_construction = $HNSW_EFC);"
fi

# IVFFlat
ivf_idx="${base}_ivfflat_idx"
if ! psql -tAc "select 1 from pg_indexes where indexname='$ivf_idx';" | grep -q 1; then
  comparator_log "building $ivf_idx on $ivf_tbl (ivfflat lists=$IVFFLAT_LISTS, maintenance_work_mem=$MAINT_WORK_MEM)"
  psql -c "SET maintenance_work_mem = '$MAINT_WORK_MEM'; CREATE INDEX $ivf_idx ON $ivf_tbl USING ivfflat (embedding vector_ip_ops) WITH (lists = $IVFFLAT_LISTS);"
fi

comparator_log "done. tables: $hnsw_tbl, $ivf_tbl, $queries_tbl; indexes: $hnsw_idx, $ivf_idx"
