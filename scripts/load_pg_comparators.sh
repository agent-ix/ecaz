#!/usr/bin/env bash
# Load a prepared corpus (TSV from `ecaz corpus prepare`) into a
# per-extension table with the pgvector `vector(N)` column type, then
# build that extension's recommended ANN index. Mirrors the layout of
# scripts/load_multi_am.sh (per-AM separate tables for cache isolation
# per ADR-050), extended to third-party Postgres extensions.
#
# Each extension gets its own table to avoid cross-extension buffer
# cache contamination:
#   real_<S>_pgv_corpus            (pgvector HNSW + IVFFlat)
#   real_<S>_pgvscale_corpus       (pgvectorscale StreamingDiskANN)
#   real_<S>_vchord_corpus         (VectorChord RaBitQ-on-IVF)
#   real_<S>_lantern_corpus        (Lantern HNSW)
#
# All use pgvector's vector(<dim>) column type since pgvectorscale,
# vchord, and lantern all build on top of pgvector's type system.
#
# Idempotent: skips load if corpus table already populated; skips
# index build if index already exists.

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/load_pg_comparators.sh --size <label> --corpus-file <tsv> --queries-file <tsv>
    --dim <N>
    [--db <database>] [--exts "pgv pgvscale vchord lantern"]
    [--index-configs <json>]

Required:
  --size <label>          Short label (10k/50k/100k/1m). Names tables real_<S>_<ext>_corpus.
  --corpus-file <tsv>     TSV with id<TAB>vector_json columns.
  --queries-file <tsv>    Same shape as corpus-file.
  --dim <N>               Embedding dimensionality (e.g. 1536 for DBpedia OpenAI-3-large).

Options:
  --db <name>             Default tqvector_bench.
  --exts <list>           Space-separated extension labels. Default
                          "pgv pgvscale vchord lantern".

  Index-config defaults (vary per extension; sensible mid-range
  recall settings):
    pgvector hnsw         m=16, ef_construction=64
    pgvector ivfflat      lists=sqrt(N) = ~32/72/100/315
    pgvectorscale         StreamingDiskANN defaults
    vchord                lists=sqrt(N), spherical_centroids=true
    lantern               hnsw m=16, ef_construction=128

Example:
  sudo -u postgres scripts/load_pg_comparators.sh \
    --size 1m --dim 1536 \
    --corpus-file /var/lib/pgsql/18/datasets/staged-1m/ec_real_ann_benchmarks_anchor_corpus.tsv \
    --queries-file /var/lib/pgsql/18/datasets/staged-1m/ec_real_ann_benchmarks_anchor_queries.tsv
EOF
}

SIZE=""
CORPUS=""
QUERIES=""
DIM=""
DB="${PGDATABASE:-tqvector_bench}"
EXTS="pgv pgvscale vchord lantern"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --size) SIZE="$2"; shift 2 ;;
    --corpus-file) CORPUS="$2"; shift 2 ;;
    --queries-file) QUERIES="$2"; shift 2 ;;
    --dim) DIM="$2"; shift 2 ;;
    --db) DB="$2"; shift 2 ;;
    --exts) EXTS="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 1 ;;
  esac
done

[[ -z "$SIZE" || -z "$CORPUS" || -z "$QUERIES" || -z "$DIM" ]] && { usage; exit 1; }

export PGDATABASE="$DB"
: "${PGHOST:=/tmp}"
: "${PGUSER:=postgres}"
export PGHOST PGUSER

log() { echo "[load-comparators] $(date '+%H:%M:%S') $*"; }

# nlists heuristic: sqrt(N) rounded up to a multiple of 16, capped at 4096.
nlists_for_size() {
  case "$1" in
    10k)  echo 100 ;;
    50k)  echo 224 ;;
    100k) echo 320 ;;
    1m)   echo 1024 ;;
    *)    echo 100 ;;
  esac
}

# Convert TSV (id<TAB>'[v1, v2, ...]') into pgvector format.
# pgvector accepts JSON-array-style strings directly, so the same TSV
# can be ingested with no transformation -- just declare the column as
# vector(N) and COPY will parse the bracketed list.
load_extension_table() {
  local ext="$1"
  local prefix="real_${SIZE}_${ext}"
  local count
  count=$(psql -tAc "select coalesce((select count(*) from ${prefix}_corpus),-1);" 2>/dev/null || echo -1)
  if [[ "$count" -gt 0 ]]; then
    log "${prefix}_corpus already loaded ($count rows); skipping load"
    return 0
  fi
  log "creating ${prefix}_corpus + ${prefix}_queries with vector($DIM)"
  psql -c "DROP TABLE IF EXISTS ${prefix}_corpus, ${prefix}_queries;"
  psql -c "CREATE TABLE ${prefix}_corpus (id bigint PRIMARY KEY, embedding vector($DIM));"
  psql -c "CREATE TABLE ${prefix}_queries (id bigint PRIMARY KEY, embedding vector($DIM));"
  log "COPY corpus from $CORPUS"
  psql -c "\\COPY ${prefix}_corpus(id, embedding) FROM '$CORPUS'"
  log "COPY queries from $QUERIES"
  psql -c "\\COPY ${prefix}_queries(id, embedding) FROM '$QUERIES'"
}

build_index_pgv() {
  local prefix="real_${SIZE}_pgv"
  local nlists
  nlists=$(nlists_for_size "$SIZE")
  # HNSW + IVFFlat side by side. Bench script swaps via DROP/CREATE.
  for idx_kind in hnsw ivfflat; do
    local idx="${prefix}_${idx_kind}_idx"
    if psql -tAc "select 1 from pg_indexes where indexname='$idx';" | grep -q 1; then
      log "$idx exists; skipping"
      continue
    fi
    case "$idx_kind" in
      hnsw)
        log "building $idx hnsw (m=16, ef_construction=64) on ${prefix}_corpus"
        psql -c "CREATE INDEX $idx ON ${prefix}_corpus USING hnsw (embedding vector_ip_ops) WITH (m = 16, ef_construction = 64);"
        ;;
      ivfflat)
        log "building $idx ivfflat (lists=$nlists) on ${prefix}_corpus"
        psql -c "CREATE INDEX $idx ON ${prefix}_corpus USING ivfflat (embedding vector_ip_ops) WITH (lists = $nlists);"
        ;;
    esac
  done
}

build_index_pgvscale() {
  local prefix="real_${SIZE}_pgvscale"
  local idx="${prefix}_diskann_idx"
  if psql -tAc "select 1 from pg_indexes where indexname='$idx';" | grep -q 1; then
    log "$idx exists; skipping"
    return 0
  fi
  log "building $idx pgvectorscale StreamingDiskANN on ${prefix}_corpus"
  psql -c "CREATE INDEX $idx ON ${prefix}_corpus USING diskann (embedding vector_ip_ops);"
}

build_index_vchord() {
  local prefix="real_${SIZE}_vchord"
  local idx="${prefix}_rabitq_idx"
  if psql -tAc "select 1 from pg_indexes where indexname='$idx';" | grep -q 1; then
    log "$idx exists; skipping"
    return 0
  fi
  log "building $idx VectorChord RaBitQ-on-IVF on ${prefix}_corpus"
  # VectorChord uses a CREATE INDEX with a config blob. Default RaBitQ.
  psql -c "CREATE INDEX $idx ON ${prefix}_corpus USING vchordrq (embedding vector_ip_ops);"
}

build_index_lantern() {
  local prefix="real_${SIZE}_lantern"
  local idx="${prefix}_hnsw_idx"
  if psql -tAc "select 1 from pg_indexes where indexname='$idx';" | grep -q 1; then
    log "$idx exists; skipping"
    return 0
  fi
  log "building $idx Lantern HNSW on ${prefix}_corpus"
  psql -c "CREATE INDEX $idx ON ${prefix}_corpus USING lantern_hnsw (embedding dist_cos_ops) WITH (m = 16, ef_construction = 128);"
}

# Verify required extensions exist before attempting their tables
extension_exists() {
  psql -tAc "select 1 from pg_available_extensions where name='$1';" | grep -q 1
}

for ext in $EXTS; do
  case "$ext" in
    pgv)
      extension_exists vector || { log "pgvector not installed; skipping pgv"; continue; }
      psql -c "CREATE EXTENSION IF NOT EXISTS vector;"
      load_extension_table pgv
      build_index_pgv
      ;;
    pgvscale)
      extension_exists vectorscale || { log "pgvectorscale not installed; skipping pgvscale"; continue; }
      psql -c "CREATE EXTENSION IF NOT EXISTS vector;"
      psql -c "CREATE EXTENSION IF NOT EXISTS vectorscale CASCADE;"
      load_extension_table pgvscale
      build_index_pgvscale
      ;;
    vchord)
      extension_exists vchord || { log "vchord not installed; skipping vchord"; continue; }
      psql -c "CREATE EXTENSION IF NOT EXISTS vector;"
      psql -c "CREATE EXTENSION IF NOT EXISTS vchord CASCADE;"
      load_extension_table vchord
      build_index_vchord
      ;;
    lantern)
      extension_exists lantern || { log "lantern not installed; skipping lantern"; continue; }
      psql -c "CREATE EXTENSION IF NOT EXISTS lantern;"
      load_extension_table lantern
      build_index_lantern
      ;;
    *)
      log "unknown ext: $ext"
      ;;
  esac
done

log "summary:"
psql -tAc "select tablename from pg_tables where tablename like 'real_${SIZE}_%_corpus' order by tablename;"
psql -tAc "select indexname from pg_indexes where indexname like 'real_${SIZE}_%_idx' order by indexname;"
