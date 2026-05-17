#!/usr/bin/env bash
set -euo pipefail

PGHOST="${PGHOST:-/home/peter/.pgrx}"
PGPORT="${PGPORT:-28818}"
PGDATABASE="${PGDATABASE:-postgres}"
PGUSER_ARG=()
if [[ -n "${PGUSER:-}" ]]; then
  PGUSER_ARG=(--user "$PGUSER")
fi

PSQL="${PSQL:-/home/peter/.pgrx/18.3/pgrx-install/bin/psql}"
PG_CONFIG="${PG_CONFIG:-/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config}"
ECAZ="${ECAZ:-target/debug/ecaz}"

ARTIFACT_DIR="${ARTIFACT_DIR:-review/30533-spire-local-placement-benchmark/artifacts}"
CORPUS_FILE="${CORPUS_FILE:-target/real-corpus/ec_real_10k/ec_real_10k_corpus.tsv}"
QUERIES_FILE="${QUERIES_FILE:-target/real-corpus/ec_real_10k/ec_real_10k_queries.tsv}"
MANIFEST_FILE="${MANIFEST_FILE:-target/real-corpus/ec_real_10k/ec_real_10k_manifest.json}"

TABLESPACE_NAME="${TABLESPACE_NAME:-ecaz_spire_e}"
TABLESPACE_PATH="${TABLESPACE_PATH:-/mnt/e/ecaz_pg_tblspc/spire_e}"

K="${K:-10}"
DIM="${DIM:-1536}"
SWEEP="${SWEEP:-8,24}"
ITERATIONS="${ITERATIONS:-100}"
MEMORY_SAMPLE_INTERVAL_MS="${MEMORY_SAMPLE_INTERVAL_MS:-25}"

RUN_SETUP="${RUN_SETUP:-1}"
RUN_INSTALL="${RUN_INSTALL:-0}"
RUN_LOAD="${RUN_LOAD:-1}"
RUN_LATENCY="${RUN_LATENCY:-1}"
RUN_RECALL="${RUN_RECALL:-1}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir)
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    --tablespace-name)
      TABLESPACE_NAME="$2"
      shift 2
      ;;
    --tablespace-path)
      TABLESPACE_PATH="$2"
      shift 2
      ;;
    --sweep)
      SWEEP="$2"
      shift 2
      ;;
    --iterations)
      ITERATIONS="$2"
      shift 2
      ;;
    --install-extension)
      RUN_INSTALL=1
      shift
      ;;
    --skip-setup)
      RUN_SETUP=0
      shift
      ;;
    --skip-load)
      RUN_LOAD=0
      shift
      ;;
    --skip-latency)
      RUN_LATENCY=0
      shift
      ;;
    --skip-recall)
      RUN_RECALL=0
      shift
      ;;
    -h|--help)
      cat <<'USAGE'
Usage: bash scripts/bench_spire_local_placement_pg18.sh [options]

Runs the Task 30 SPIRE local placement benchmark lanes against a local PG18
pgrx scratch cluster:
  - two stores on pg_default (same-device baseline)
  - two stores split across pg_default and /mnt/e
  - latency comparison against the pre-existing one-store prefix

Options:
  --artifact-dir DIR       Artifact directory to write logs into
  --tablespace-name NAME   PostgreSQL tablespace name for /mnt/e lane
  --tablespace-path PATH   Tablespace directory; must be under /mnt/e
  --sweep VALUES           nprobe sweep, default 8,24
  --iterations N           latency iterations per sweep value, default 100
  --install-extension      install current extension into PG18 before running
  --skip-setup             do not create/register the /mnt/e tablespace
  --skip-load              do not load/build the two multi-store indexes
  --skip-latency           do not run latency benches
  --skip-recall            do not run recall checks
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

psql_args=(
  -h "$PGHOST"
  -p "$PGPORT"
  -d "$PGDATABASE"
  -v ON_ERROR_STOP=1
)

ecaz_args=(
  --database "$PGDATABASE"
  --host "$PGHOST"
  --port "$PGPORT"
)
if [[ ${#PGUSER_ARG[@]} -gt 0 ]]; then
  ecaz_args+=("${PGUSER_ARG[@]}")
fi

ensure_cli() {
  if [[ ! -x "$ECAZ" ]]; then
    cargo build -p ecaz-cli
  fi
}

install_extension() {
  cargo pgrx install \
    --pg-config "$PG_CONFIG" \
    --no-default-features \
    --features pg18
}

ensure_tablespace() {
  if [[ "$TABLESPACE_PATH" != /mnt/e/* ]]; then
    echo "refusing to create SPIRE benchmark tablespace outside /mnt/e: $TABLESPACE_PATH" >&2
    exit 1
  fi

  mkdir -p "$TABLESPACE_PATH"
  if ! "$PSQL" "${psql_args[@]}" -Atc \
    "SELECT 1 FROM pg_tablespace WHERE spcname = '$TABLESPACE_NAME'" | grep -qx 1; then
    "$PSQL" "${psql_args[@]}" -c \
      "CREATE TABLESPACE $TABLESPACE_NAME LOCATION '$TABLESPACE_PATH'"
  fi
}

load_lane() {
  local prefix="$1"
  local label="$2"
  local tablespaces="$3"
  "$ECAZ" "${ecaz_args[@]}" corpus load \
    --prefix "$prefix" \
    --profile ec_spire \
    --corpus-file "$CORPUS_FILE" \
    --queries-file "$QUERIES_FILE" \
    --manifest-file "$MANIFEST_FILE" \
    --allow-manifest-mismatch \
    --dim "$DIM" \
    --storage-format turboquant \
    --reloption nlists=32 \
    --reloption nprobe=24 \
    --reloption rerank_width=25 \
    --reloption local_store_count=2 \
    --reloption "local_store_tablespaces=$tablespaces" \
    --log-file "$ARTIFACT_DIR/load_real10k_${label}.log"
}

latency_lane() {
  local prefix="$1"
  local label="$2"
  "$ECAZ" "${ecaz_args[@]}" bench latency \
    --prefix "$prefix" \
    --profile ec_spire \
    --k "$K" \
    --iterations "$ITERATIONS" \
    --sweep "$SWEEP" \
    --force-index \
    --sample-backend-memory \
    --memory-sample-interval-ms "$MEMORY_SAMPLE_INTERVAL_MS" \
    --log-file "$ARTIFACT_DIR/latency_real10k_${label}_cli.log" \
    --log-output "$ARTIFACT_DIR/latency_real10k_${label}_table.log"
}

recall_lane() {
  local prefix="$1"
  local label="$2"
  "$ECAZ" "${ecaz_args[@]}" bench recall \
    --prefix "$prefix" \
    --profile ec_spire \
    --k "$K" \
    --sweep "$SWEEP" \
    --force-index \
    --truth-cache-file "$ARTIFACT_DIR/real10k_truth_k10.json" \
    --log-file "$ARTIFACT_DIR/recall_real10k_${label}_cli.log" \
    --log-output "$ARTIFACT_DIR/recall_real10k_${label}_table.log"
}

record_store_tablespaces() {
  "$PSQL" "${psql_args[@]}" -A -F $'\t' \
    -o "$ARTIFACT_DIR/store_relation_tablespaces.tsv" \
    -c "WITH indexes(lane, index_name) AS (
          VALUES
            ('one_store_pgdefault', 'task30_spire_real10k_tq_turboquant_idx'),
            ('two_store_same_pgdefault', 'task30_spire_real10k_tq_2same_turboquant_idx'),
            ('two_store_pgdefault_e', 'task30_spire_real10k_tq_2e_turboquant_idx')
        ),
        index_oids AS (
          SELECT lane, index_name, index_name::regclass AS index_oid
          FROM indexes
        ),
        store_rels AS (
          SELECT lane, 0 AS local_store_id, index_oid::oid AS relid
          FROM index_oids
          UNION ALL
          SELECT lane, 1 AS local_store_id,
                 format('ec_spire_store_%s_1', index_oid::oid)::regclass::oid AS relid
          FROM index_oids
          WHERE lane <> 'one_store_pgdefault'
        )
        SELECT
          lane,
          local_store_id,
          relid::regclass AS relation_name,
          COALESCE(NULLIF(t.spcname, ''), 'pg_default') AS tablespace,
          pg_tablespace_location(
            COALESCE(NULLIF(c.reltablespace, 0),
            (SELECT dattablespace FROM pg_database WHERE datname = current_database()))
          ) AS tablespace_location,
          pg_relation_filepath(relid) AS relation_filepath
        FROM store_rels
        JOIN pg_class c ON c.oid = relid
        LEFT JOIN pg_tablespace t
          ON t.oid = COALESCE(NULLIF(c.reltablespace, 0),
            (SELECT dattablespace FROM pg_database WHERE datname = current_database()))
        ORDER BY lane, local_store_id;"
}

mkdir -p "$ARTIFACT_DIR"
ensure_cli

if [[ "$RUN_INSTALL" == "1" ]]; then
  install_extension
fi

if [[ "$RUN_SETUP" == "1" ]]; then
  ensure_tablespace
fi

if [[ "$RUN_LOAD" == "1" ]]; then
  load_lane "task30_spire_real10k_tq_2same" "2same_pgdefault" "pg_default,pg_default"
  load_lane "task30_spire_real10k_tq_2e" "2store_pgdefault_e" "pg_default,$TABLESPACE_NAME"
fi

if [[ "$RUN_LATENCY" == "1" ]]; then
  latency_lane "task30_spire_real10k_tq" "1store_pgdefault"
  latency_lane "task30_spire_real10k_tq_2same" "2same_pgdefault"
  latency_lane "task30_spire_real10k_tq_2e" "2store_pgdefault_e"
fi

if [[ "$RUN_RECALL" == "1" ]]; then
  recall_lane "task30_spire_real10k_tq_2same" "2same_pgdefault"
  recall_lane "task30_spire_real10k_tq_2e" "2store_pgdefault_e"
fi

record_store_tablespaces
