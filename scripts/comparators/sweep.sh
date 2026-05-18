#!/usr/bin/env bash
# Run a recall + latency operating-point sweep against all loaded
# comparators at a single corpus size, plus a brute-force ground-truth
# pass. Runs locally against the PG socket on this host.
#
# Output layout under --out:
#   <out>/<size>/_groundtruth.out
#   <out>/<size>/<ext>/<variant>/<setting>.out
#
# Each .out is the raw psql log containing one
#   "INFO:  SAMPLE qid=N ms=F ids={...}"
# line per query, in order. Feed the directory to compute_recall.py to
# derive latency.log + recall.txt per cell + a Pareto table.
set -euo pipefail

COMPARATOR_NAME="sweep"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/_common.sh"

OUT="" SIZE="" DB="${PGDATABASE:-tqvector_bench}" K=10 ITERS=200

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="$2"; shift 2 ;;
    --size) SIZE="$2"; shift 2 ;;
    --db) DB="$2"; shift 2 ;;
    --k) K="$2"; shift 2 ;;
    --iterations) ITERS="$2"; shift 2 ;;
    -h|--help) sed -n '2,$ s/^# \?//p' "$0" | head -20; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

[[ -z "$OUT" || -z "$SIZE" ]] && { echo "Usage: $0 --out <dir> --size <S>"; exit 1; }
export PGDATABASE="$DB" PGHOST="${PGHOST:-/tmp}" PGUSER="${PGUSER:-postgres}"

QUERIES="real_${SIZE}_pgv_queries"
OUTDIR="$OUT/$SIZE"
mkdir -p "$OUTDIR"

run_cell() {
  # args: outfile corpus guc value
  local outfile="$1" corpus="$2" guc="$3" value="$4"
  comparator_log "  cell $(basename "$outfile") corpus=$corpus $guc=$value"
  psql -v ON_ERROR_STOP=0 > "$outfile" 2>&1 <<SQL
SET statement_timeout = '300s';
SET ${guc} = '${value}';
DO \$\$
DECLARE
  qid bigint;
  qids bigint[];
  ids bigint[];
  t0 timestamptz;
  t1 timestamptz;
BEGIN
  SELECT array_agg(id ORDER BY id)
    FROM (SELECT id FROM ${QUERIES} ORDER BY id LIMIT ${ITERS}) s
    INTO qids;
  FOREACH qid IN ARRAY qids LOOP
    t0 := clock_timestamp();
    EXECUTE format(
      'SELECT array_agg(id ORDER BY embedding <#> (SELECT embedding FROM %I WHERE id = %s)) '
      'FROM (SELECT id, embedding FROM %I '
            'ORDER BY embedding <#> (SELECT embedding FROM %I WHERE id = %s) '
            'LIMIT %s) s',
      '${QUERIES}', qid, '${corpus}', '${QUERIES}', qid, ${K})
      INTO ids;
    t1 := clock_timestamp();
    RAISE INFO 'SAMPLE qid=% ms=% ids=%',
      qid, extract(epoch FROM (t1 - t0)) * 1000.0, ids;
  END LOOP;
END
\$\$;
SQL
}

run_groundtruth() {
  local outfile="$1" corpus="$2"
  comparator_log "  ground truth (seqscan top-$K, $ITERS queries)"
  psql -v ON_ERROR_STOP=0 > "$outfile" 2>&1 <<SQL
SET enable_indexscan = off;
SET enable_bitmapscan = off;
SET max_parallel_workers_per_gather = 4;
SET statement_timeout = '900s';
DO \$\$
DECLARE
  qid bigint;
  qids bigint[];
  ids bigint[];
  t0 timestamptz;
  t1 timestamptz;
BEGIN
  SELECT array_agg(id ORDER BY id)
    FROM (SELECT id FROM ${QUERIES} ORDER BY id LIMIT ${ITERS}) s
    INTO qids;
  FOREACH qid IN ARRAY qids LOOP
    t0 := clock_timestamp();
    EXECUTE format(
      'SELECT array_agg(id ORDER BY embedding <#> (SELECT embedding FROM %I WHERE id = %s)) '
      'FROM (SELECT id, embedding FROM %I '
            'ORDER BY embedding <#> (SELECT embedding FROM %I WHERE id = %s) '
            'LIMIT %s) s',
      '${QUERIES}', qid, '${corpus}', '${QUERIES}', qid, ${K})
      INTO ids;
    t1 := clock_timestamp();
    RAISE INFO 'SAMPLE qid=% ms=% ids=%',
      qid, extract(epoch FROM (t1 - t0)) * 1000.0, ids;
  END LOOP;
END
\$\$;
SQL
}

run_groundtruth "$OUTDIR/_groundtruth.out" "real_${SIZE}_pgv_hnsw_corpus"

# pgvector HNSW
mkdir -p "$OUTDIR/pgv/hnsw"
for v in 16 40 100 400; do
  run_cell "$OUTDIR/pgv/hnsw/ef${v}.out" "real_${SIZE}_pgv_hnsw_corpus" "hnsw.ef_search" "$v"
done

# pgvector IVFFlat
mkdir -p "$OUTDIR/pgv/ivfflat"
for v in 1 8 32 100; do
  run_cell "$OUTDIR/pgv/ivfflat/p${v}.out" "real_${SIZE}_pgv_ivfflat_corpus" "ivfflat.probes" "$v"
done

# pgvectorscale DiskANN
mkdir -p "$OUTDIR/pgvscale/diskann"
for v in 40 100 400 1000; do
  run_cell "$OUTDIR/pgvscale/diskann/sl${v}.out" "real_${SIZE}_pgvscale_corpus" "diskann.query_search_list_size" "$v"
done

# vchord RaBitQ-on-IVF
mkdir -p "$OUTDIR/vchord/rabitq"
LISTS="$(comparator_nlists_for_size "$SIZE")"
for v in 1 4 16 64; do
  # vchord.probes is the number of IVF lists scanned at the single level.
  [[ "$v" -gt "$LISTS" ]] && continue
  run_cell "$OUTDIR/vchord/rabitq/p${v}.out" "real_${SIZE}_vchord_corpus" "vchordrq.probes" "$v"
done

comparator_log "done. results under $OUTDIR"
