#!/usr/bin/env bash
# Load corpus into vchord (VectorChord) tables + build RaBitQ-on-IVF
# index. The most relevant comparator for ecaz's RaBitQ-on-IVF work.
set -euo pipefail

COMPARATOR_NAME="vchord"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../_common.sh"

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

if ! comparator_extension_available_in_pg vchord; then
  comparator_log "vchord ext not installed; run install_vchord.sh"; exit 1
fi
comparator_require_pgvector
psql -c "CREATE EXTENSION IF NOT EXISTS vchord CASCADE;" >/dev/null 2>&1

prefix="real_${SIZE}_vchord"
if ! comparator_table_loaded "${prefix}_corpus"; then
  comparator_load_vector_table "${prefix}_corpus" "$CORPUS" "$DIM"
fi
if ! comparator_table_loaded "${prefix}_queries"; then
  comparator_load_vector_table "${prefix}_queries" "$QUERIES" "$DIM"
fi

MAINT_WORK_MEM="${MAINT_WORK_MEM:-4GB}"

# IVF list count: sqrt(N) heuristic — matches the pgvector IVFFlat
# defaults so the two IVF-family comparators run on equal footing.
LISTS="$(comparator_nlists_for_size "$SIZE")"

idx="${prefix}_rabitq_idx"
if ! psql -tAc "select 1 from pg_indexes where indexname='$idx';" | grep -q 1; then
  comparator_log "building $idx (vchordrq RaBitQ-on-IVF, lists=$LISTS, maintenance_work_mem=$MAINT_WORK_MEM)"
  psql -v ON_ERROR_STOP=1 <<SQL
SET maintenance_work_mem = '$MAINT_WORK_MEM';
CREATE INDEX $idx ON ${prefix}_corpus USING vchordrq (embedding vector_ip_ops)
WITH (options = \$vco\$
residual_quantization = true
[build.internal]
lists = [$LISTS]
\$vco\$);
SQL
fi

comparator_log "done. tables: ${prefix}_corpus, ${prefix}_queries; index: $idx"
