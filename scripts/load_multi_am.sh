#!/usr/bin/env bash
# Load a prepared corpus into per-AM separate tables, build each AM's
# default index, plus the ec_ivf storage_format=rabitq and pq_fastscan
# variants colocated on the ec_ivf table.
#
# Per ADR-050: "Build surfaces SHOULD be isolated one-index-per-table
# by default". This script implements that for ec_hnsw, ec_ivf, and
# ec_diskann (with ec_ivf storage formats colocated for swap-on-pass
# bench discipline).
#
# After this script, for each size <S> you get:
#   real_<S>_hnsw_corpus    + real_<S>_hnsw_idx        (ec_hnsw default m+ef)
#   real_<S>_ivf_corpus     + real_<S>_ivf_idx         (ec_ivf TQ default)
#                           + real_<S>_ivf_rabitq_idx  (storage_format=rabitq)
#                           + real_<S>_ivf_pqfs_idx    (storage_format=pq_fastscan)
#   real_<S>_diskann_corpus + real_<S>_diskann_idx     (ec_diskann default)
#
# Idempotent: skips any (corpus, profile) load whose tables already
# exist with the expected row count.

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/load_multi_am.sh --size <label> --corpus-file <tsv> --queries-file <tsv>
    [--db <database>] [--manifest-file <json>]
    [--ams "hnsw ivf diskann"]
    [--ecaz <path>]

Required:
  --size <label>           Short label like 10k, 50k, 100k, 1m. Used in
                           table names (real_<label>_<am>_corpus).
  --corpus-file <tsv>      Prepared corpus TSV (output of `ecaz corpus prepare`).
  --queries-file <tsv>     Prepared queries TSV.

Optional:
  --db <name>              Default tqvector_bench.
  --manifest-file <json>   Prepared manifest sidecar.
  --ams <list>             Space-separated AMs (default: "hnsw ivf diskann").
  --ecaz <path>            ecaz binary path (default /usr/local/bin/ecaz).

Example:
  sudo -u postgres scripts/load_multi_am.sh \
    --size 1m \
    --corpus-file /var/lib/pgsql/18/datasets/staged-1m/ec_real_ann_benchmarks_anchor_corpus.tsv \
    --queries-file /var/lib/pgsql/18/datasets/staged-1m/ec_real_ann_benchmarks_anchor_queries.tsv \
    --manifest-file /var/lib/pgsql/18/datasets/staged-1m/ec_real_ann_benchmarks_anchor_manifest.json
EOF
}

SIZE=""
CORPUS=""
QUERIES=""
MANIFEST=""
AMS="hnsw ivf diskann"
ECAZ="${ECAZ_BIN:-/usr/local/bin/ecaz}"
DB="${PGDATABASE:-tqvector_bench}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --size) SIZE="$2"; shift 2 ;;
    --corpus-file) CORPUS="$2"; shift 2 ;;
    --queries-file) QUERIES="$2"; shift 2 ;;
    --manifest-file) MANIFEST="$2"; shift 2 ;;
    --db) DB="$2"; shift 2 ;;
    --ams) AMS="$2"; shift 2 ;;
    --ecaz) ECAZ="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

[[ -z "$SIZE" || -z "$CORPUS" || -z "$QUERIES" ]] && { usage; exit 1; }

export PGDATABASE="$DB"
: "${PGHOST:=/tmp}"
: "${PGUSER:=postgres}"
export PGHOST PGUSER

log() { echo "[load-multi-am] $(date '+%H:%M:%S') $*"; }

table_exists() {
  psql -tAc "select 1 from information_schema.tables where table_name='$1';" \
    | grep -q '^1$'
}

ensure_corpus_loaded() {
  local prefix="$1" profile="$2"
  if table_exists "${prefix}_corpus"; then
    log "${prefix}_corpus already loaded; skipping (re-create with DROP TABLE if needed)"
    return 0
  fi
  log "loading $prefix with profile $profile"
  local mf_arg=()
  [[ -n "$MANIFEST" ]] && mf_arg=(--manifest-file "$MANIFEST" --allow-manifest-mismatch)
  "$ECAZ" corpus load \
    --prefix "$prefix" \
    --profile "$profile" \
    --corpus-file "$CORPUS" \
    --queries-file "$QUERIES" \
    "${mf_arg[@]}"
}

# Per-AM loads
for am in $AMS; do
  case "$am" in
    hnsw)
      ensure_corpus_loaded "real_${SIZE}_hnsw" "ec_hnsw"
      ;;
    ivf)
      ensure_corpus_loaded "real_${SIZE}_ivf" "ec_ivf"
      # Extra storage_format variants colocated on the ec_ivf table.
      for sf in rabitq pq_fastscan; do
        idx="real_${SIZE}_ivf_${sf//pq_fastscan/pqfs}_idx"
        # collapsed alias: real_<size>_ivf_pqfs_idx, real_<size>_ivf_rabitq_idx
        if psql -tAc "select 1 from pg_indexes where indexname='$idx';" | grep -q 1; then
          log "$idx already exists; skipping"
        else
          log "building $idx (storage_format=$sf)"
          psql -c "CREATE INDEX $idx ON real_${SIZE}_ivf_corpus USING ec_ivf (embedding) WITH (storage_format='$sf');"
        fi
      done
      ;;
    diskann)
      ensure_corpus_loaded "real_${SIZE}_diskann" "ec_diskann"
      ;;
    spire)
      log "ec_spire load is deferred; not implemented in this script yet"
      ;;
    *)
      log "unknown AM: $am"
      ;;
  esac
done

log "done loading size=$SIZE ams=$AMS"
log "tables:"
psql -tAc "select tablename from pg_tables where tablename like 'real_${SIZE}_%' order by tablename;"
log "indexes:"
psql -tAc "select indexname from pg_indexes where indexname like 'real_${SIZE}_%' order by indexname;"
