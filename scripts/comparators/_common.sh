# Shared helpers for per-comparator install/load/bench scripts.
# Source this from each comparator script:
#   source "$(dirname "$0")/_common.sh"
#
# Each comparator (pgvector, pgvectorscale, vchord, lantern, ...) gets
# its OWN install/load/bench script files in this directory so they
# can be added, removed, or rerun fully independently.

# Build dir for source-built extensions (per [[feedback_aws_bench_sizing]]:
# use the data EBS volume so root volume doesn't fill).
COMPARATORS_BUILD_DIR_DEFAULT="/var/lib/pgsql/build/exts"
PG_CONFIG_DEFAULT="/usr/bin/pg_config"

comparator_log() {
  echo "[comparator:${COMPARATOR_NAME:-?}] $(date '+%H:%M:%S') $*"
}

# Returns 0 if the extension is already installed (has a .control file).
comparator_extension_installed() {
  local control_name="$1"
  local sharedir
  sharedir="$("${PG_CONFIG:-$PG_CONFIG_DEFAULT}" --sharedir 2>/dev/null)"
  [[ -n "$sharedir" ]] && ls "$sharedir/extension/${control_name}.control" >/dev/null 2>&1
}

# Returns 0 if the extension is available in pg_available_extensions.
comparator_extension_available_in_pg() {
  psql -tAc "select 1 from pg_available_extensions where name='$1';" 2>/dev/null | grep -q 1
}

# nlists heuristic for IVF-style indexes: sqrt(N) rounded up.
comparator_nlists_for_size() {
  case "$1" in
    10k)  echo 100 ;;
    50k)  echo 224 ;;
    100k) echo 320 ;;
    1m)   echo 1024 ;;
    *)    echo 100 ;;
  esac
}

# Standard table-loaded check used by every load script.
comparator_table_loaded() {
  local table="$1"
  local count
  count=$(psql -tAc "select coalesce((select count(*) from $table),-1);" 2>/dev/null || echo -1)
  [[ "$count" -gt 0 ]]
}

# Common pgvector-format COPY: takes a TSV of <id>\t<json-array>.
# All comparator extensions (pgvector, pgvectorscale, vchord, lantern)
# build on pgvector's vector(N) column type, so the COPY format is
# the same; only the index syntax differs per extension.
comparator_load_vector_table() {
  local table="$1" tsv="$2" dim="$3"
  comparator_log "  COPY $table from $tsv (dim=$dim)"
  psql -c "DROP TABLE IF EXISTS $table CASCADE;"
  psql -c "CREATE TABLE $table (id bigint PRIMARY KEY, embedding vector($dim));"
  psql -c "\\COPY $table(id, embedding) FROM '$tsv'"
}

# Standard "ensure pgvector extension exists" guard. All comparators
# need pgvector's vector type.
comparator_require_pgvector() {
  if ! comparator_extension_available_in_pg vector; then
    comparator_log "pgvector (vector ext) not installed; cannot proceed"
    exit 1
  fi
  psql -c "CREATE EXTENSION IF NOT EXISTS vector;" >/dev/null 2>&1
}
