#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
COORD_PORT="${COORD_PORT:-39700}"
REMOTE1_PORT="${REMOTE1_PORT:-39701}"
REMOTE2_PORT="${REMOTE2_PORT:-39702}"
REMOTE3_PORT="${REMOTE3_PORT:-39703}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
ARTIFACT_DIR=""
SMOKE_LOG="${SMOKE_LOG:-}"
TIER="${TIER:-correctness}"
PREFIX_OVERRIDE="${PREFIX:-}"
PREPARED_PREFIX_OVERRIDE="${SPIRE_AWS_REPRESENTATIVE_PREPARED_PREFIX:-}"
PREPARED_DIR_OVERRIDE="${SPIRE_AWS_REPRESENTATIVE_PREPARED_DIR:-}"
BENCH_TOP_K="${BENCH_TOP_K:-10}"
BENCH_QUERIES_LIMIT="${BENCH_QUERIES_LIMIT:-200}"
BENCH_SWEEP="${BENCH_SWEEP:-64,96}"
BENCH_ROWCAP_SWEEP="${BENCH_ROWCAP_SWEEP:-96}"
BENCH_TRUTH_CORPUS_FILE="${BENCH_TRUTH_CORPUS_FILE:-}"
RUN_BENCH_SUITE="${RUN_BENCH_SUITE:-1}"
RUN_FAULT_DRILLS="${RUN_FAULT_DRILLS:-1}"

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_phase13e_aws_harness_local_pg18.sh [options]

Runs the AWS Phase 13e correctness harness against four local PG18 instances.
This reuses scripts/spire-aws/load.sh, register.sh, and smoke.sh with a local
topology so AWS correctness failures can be reproduced without AWS spend.

Options:
  --artifact-dir DIR   Store harness, PostgreSQL, and SPIRE logs in DIR.
  --coord-port PORT    Coordinator PostgreSQL port. Default: 39700.
  --pgbin DIR          PostgreSQL bin directory. Default: $PGBIN.
  --remote1-port PORT  First remote PostgreSQL port. Default: 39701.
  --remote2-port PORT  Second remote PostgreSQL port. Default: 39702.
  --remote3-port PORT  Third remote PostgreSQL port. Default: 39703.
  --run-dir DIR        Run directory. Default: target/spire-phase13e-aws-local-$RUN_ID.
  --run-id ID          Run id used in the default run directory.
  --tier TIER          load.sh tier: correctness or representative. Default: correctness.
  --prefix PREFIX      Corpus prefix for representative/local-real runs.
  --prepared-prefix P  Prepared corpus basename prefix for representative tier.
  --prepared-dir DIR   Directory containing prepared corpus/query/manifest files.
  --bench-top-k K      Top-k for the packet-local bench suite. Default: 10.
  --bench-queries-limit N
                      Query count for the packet-local bench suite. Default: 200.
  --bench-sweep LIST   Comma-separated nprobe sweep. Default: 64,96.
  --bench-rowcap-sweep LIST
                      Comma-separated nprobe sweep for rowcap25k step. Default: 96.
  --bench-truth-corpus-file FILE
                      Local corpus TSV for exact truth in bench spire-pipeline.
  --skip-bench-suite   Skip the packet-local ecaz bench suite step.
  --skip-fault-drills  Skip correctness-only pooling/fault drills.
  --skip-install       Skip cargo pgrx install.
  --smoke-log FILE     Tee harness output to FILE.
  -h, --help           Show this help.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir)
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    --coord-port)
      COORD_PORT="$2"
      shift 2
      ;;
    --pgbin)
      PGBIN="$2"
      PG_CTL="$PGBIN/pg_ctl"
      PSQL="$PGBIN/psql"
      shift 2
      ;;
    --remote1-port)
      REMOTE1_PORT="$2"
      shift 2
      ;;
    --remote2-port)
      REMOTE2_PORT="$2"
      shift 2
      ;;
    --remote3-port)
      REMOTE3_PORT="$2"
      shift 2
      ;;
    --run-dir)
      RUN_DIR_OVERRIDE="$2"
      shift 2
      ;;
    --run-id)
      RUN_ID="$2"
      shift 2
      ;;
    --tier)
      TIER="$2"
      shift 2
      ;;
    --prefix)
      PREFIX_OVERRIDE="$2"
      shift 2
      ;;
    --prepared-prefix)
      PREPARED_PREFIX_OVERRIDE="$2"
      shift 2
      ;;
    --prepared-dir)
      PREPARED_DIR_OVERRIDE="$2"
      shift 2
      ;;
    --bench-top-k)
      BENCH_TOP_K="$2"
      shift 2
      ;;
    --bench-queries-limit)
      BENCH_QUERIES_LIMIT="$2"
      shift 2
      ;;
    --bench-sweep)
      BENCH_SWEEP="$2"
      shift 2
      ;;
    --bench-rowcap-sweep)
      BENCH_ROWCAP_SWEEP="$2"
      shift 2
      ;;
    --bench-truth-corpus-file)
      BENCH_TRUTH_CORPUS_FILE="$2"
      shift 2
      ;;
    --skip-bench-suite)
      RUN_BENCH_SUITE=0
      shift
      ;;
    --skip-fault-drills)
      RUN_FAULT_DRILLS=0
      shift
      ;;
    --skip-install)
      ECAZ_SKIP_INSTALL=1
      shift
      ;;
    --smoke-log)
      SMOKE_LOG="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-phase13e-aws-local-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/phase13e-aws-harness-local.log}"
else
  LOG_DIR="$RUN_DIR/logs"
fi
SOCKET_DIR="$ROOT_DIR/target/spire-phase13e-aws-local-sockets-$RUN_ID"
COORD_DATA="$RUN_DIR/coord"
REMOTE1_DATA="$RUN_DIR/remote1"
REMOTE2_DATA="$RUN_DIR/remote2"
REMOTE3_DATA="$RUN_DIR/remote3"
TOPOLOGY="$RUN_DIR/topology.local.json"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_PHASE13E_AWS_LOCAL_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_PHASE13E_AWS_LOCAL_LOG_ACTIVE=1
  exec > >(tee "$SMOKE_LOG") 2>&1
fi

if [[ -e "$RUN_DIR" ]]; then
  echo "RUN_DIR already exists: $RUN_DIR" >&2
  exit 2
fi

mkdir -p "$LOG_DIR" "$SOCKET_DIR" "$RUN_DIR"
: > "$LOG_DIR/coord-postgres.log"
: > "$LOG_DIR/remote1-postgres.log"
: > "$LOG_DIR/remote2-postgres.log"
: > "$LOG_DIR/remote3-postgres.log"

cleanup() {
  "$PG_CTL" -D "$COORD_DATA" -m fast stop >/dev/null 2>&1 || true
  "$PG_CTL" -D "$REMOTE1_DATA" -m fast stop >/dev/null 2>&1 || true
  "$PG_CTL" -D "$REMOTE2_DATA" -m fast stop >/dev/null 2>&1 || true
  "$PG_CTL" -D "$REMOTE3_DATA" -m fast stop >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "run_dir=$RUN_DIR"
echo "coord_port=$COORD_PORT"
echo "remote1_port=$REMOTE1_PORT"
echo "remote2_port=$REMOTE2_PORT"
echo "remote3_port=$REMOTE3_PORT"
echo "tier=$TIER"
echo "bench_top_k=$BENCH_TOP_K"
echo "bench_queries_limit=$BENCH_QUERIES_LIMIT"
echo "bench_sweep=$BENCH_SWEEP"
echo "bench_rowcap_sweep=$BENCH_ROWCAP_SWEEP"

if [[ "${ECAZ_SKIP_INSTALL:-0}" != "1" ]]; then
  (cd "$ROOT_DIR" && cargo pgrx install --test --pg-config "$PGBIN/pg_config" \
    --features "pg18 pg_test" --no-default-features)
fi

"$PG_CTL" initdb -D "$COORD_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$REMOTE1_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$REMOTE2_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$REMOTE3_DATA" -o "-A trust -U postgres" >/dev/null

export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_AWS_LOCAL_NODE2="host=$SOCKET_DIR port=$REMOTE1_PORT dbname=postgres user=ecaz_coord connect_timeout=1"
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_AWS_LOCAL_NODE3="host=$SOCKET_DIR port=$REMOTE2_PORT dbname=postgres user=ecaz_coord connect_timeout=1"
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_AWS_LOCAL_NODE4="host=$SOCKET_DIR port=$REMOTE3_PORT dbname=postgres user=ecaz_coord connect_timeout=1"

"$PG_CTL" -w -D "$REMOTE1_DATA" -l "$LOG_DIR/remote1-postgres.log" \
  -o "-p $REMOTE1_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null
"$PG_CTL" -w -D "$REMOTE2_DATA" -l "$LOG_DIR/remote2-postgres.log" \
  -o "-p $REMOTE2_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null
"$PG_CTL" -w -D "$REMOTE3_DATA" -l "$LOG_DIR/remote3-postgres.log" \
  -o "-p $REMOTE3_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null
"$PG_CTL" -w -D "$COORD_DATA" -l "$LOG_DIR/coord-postgres.log" \
  -o "-p $COORD_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null

coord_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$COORD_PORT" -U postgres -d postgres)
remote1_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE1_PORT" -U postgres -d postgres)
remote2_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE2_PORT" -U postgres -d postgres)
remote3_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE3_PORT" -U postgres -d postgres)

for psql_cmd in coord_psql remote1_psql remote2_psql remote3_psql; do
  declare -n psql_ref="$psql_cmd"
  "${psql_ref[@]}" <<'SQL' >/dev/null
CREATE ROLE ecaz_coord LOGIN SUPERUSER;
CREATE EXTENSION ecaz;
SQL
done

cat > "$TOPOLOGY" <<JSON
{
  "coordinator": {
    "node_id": 1,
    "operator_host": "$SOCKET_DIR",
    "operator_port": $COORD_PORT
  },
  "remotes": [
    {
      "node_id": 2,
      "operator_host": "$SOCKET_DIR",
      "operator_port": $REMOTE1_PORT,
      "secret_name": "spire/remote/aws-local/node2"
    },
    {
      "node_id": 3,
      "operator_host": "$SOCKET_DIR",
      "operator_port": $REMOTE2_PORT,
      "secret_name": "spire/remote/aws-local/node3"
    },
    {
      "node_id": 4,
      "operator_host": "$SOCKET_DIR",
      "operator_port": $REMOTE3_PORT,
      "secret_name": "spire/remote/aws-local/node4"
    }
  ]
}
JSON

ECAZ_BIN="${ECAZ_BIN:-$ROOT_DIR/target/release/ecaz}"
if [[ ! -x "$ECAZ_BIN" ]]; then
  ECAZ_BIN="ecaz"
fi
export ECAZ_BIN
export WORK_DIR="$RUN_DIR/work"
if [[ -n "$PREFIX_OVERRIDE" ]]; then
  export PREFIX="$PREFIX_OVERRIDE"
fi
if [[ -n "$PREPARED_PREFIX_OVERRIDE" ]]; then
  export SPIRE_AWS_REPRESENTATIVE_PREPARED_PREFIX="$PREPARED_PREFIX_OVERRIDE"
fi
if [[ -n "$PREPARED_DIR_OVERRIDE" ]]; then
  export SPIRE_AWS_REPRESENTATIVE_PREPARED_DIR="$PREPARED_DIR_OVERRIDE"
fi

effective_prefix="${PREFIX_OVERRIDE:-${PREFIX:-}}"
if [[ -z "$effective_prefix" ]]; then
  case "$TIER" in
    correctness)
      effective_prefix="ec_spire_aws_synth_10k"
      ;;
    representative)
      effective_prefix="ec_spire_aws_repr_1m"
      ;;
    *)
      echo "unknown tier: $TIER" >&2
      exit 2
      ;;
  esac
fi

run_bench_suite() {
  local suite_artifact_dir="$LOG_DIR/bench-suite"
  local suite_config="$suite_artifact_dir/local-real-production-read-suite.json"
  local bench_sweep_json
  local rowcap_sweep_json

  mkdir -p "$suite_artifact_dir"
  bench_sweep_json="$(printf '%s' "$BENCH_SWEEP" | jq -R 'split(",") | map(tonumber)')"
  rowcap_sweep_json="$(printf '%s' "$BENCH_ROWCAP_SWEEP" | jq -R 'split(",") | map(tonumber)')"
  jq -n \
    --arg artifact_dir "$suite_artifact_dir" \
    --arg prefix "$effective_prefix" \
    --arg storage_log "$suite_artifact_dir/storage.log" \
    --arg default_log "$suite_artifact_dir/production-read-k10-default.log" \
    --arg rowcap_log "$suite_artifact_dir/production-read-k10-rowcap25k.log" \
    --arg truth_corpus_file "$BENCH_TRUTH_CORPUS_FILE" \
    --argjson bench_sweep "$bench_sweep_json" \
    --argjson rowcap_sweep "$rowcap_sweep_json" \
    --argjson bench_top_k "$BENCH_TOP_K" \
    --argjson bench_queries_limit "$BENCH_QUERIES_LIMIT" \
    '
    def maybe_truth($path):
      if $path == "" then . else . + {truth_corpus_file: $path} end;
    {
      name: "phase5-local-multinode-production-read",
      schema_version: 1,
      artifact_dir: $artifact_dir,
      defaults: {
        queries_limit: $bench_queries_limit,
        pg: 18
      },
      steps: [
        {
          kind: "storage",
          name: "storage-local-coordinator",
          tags: ["phase5", "local", "multinode", "storage"],
          prefix: $prefix,
          log_file: $storage_log
        },
        ({
          kind: "spire-pipeline",
          name: "production-read-k10-default",
          tags: ["phase5", "local", "multinode", "production-read", "default-cap"],
          prefix: $prefix,
          queries_limit: $bench_queries_limit,
          sweep: $bench_sweep,
          top_k: $bench_top_k,
          include_remote: true,
          require_remote_placements: true,
          include_cost_snapshot: true,
          include_query_metrics: true,
          include_recall: true,
          include_production_read_profile: true,
          production_read_only: true,
          query_metric_k: $bench_top_k,
          query_metric_projection_columns: ["id"],
          log_output: $default_log
        } | maybe_truth($truth_corpus_file)),
        ({
          kind: "spire-pipeline",
          name: "production-read-k10-rowcap25k",
          tags: ["phase5", "local", "multinode", "production-read", "rowcap25k"],
          prefix: $prefix,
          queries_limit: $bench_queries_limit,
          sweep: $rowcap_sweep,
          top_k: $bench_top_k,
          max_routed_candidate_rows: 25000,
          include_remote: true,
          require_remote_placements: true,
          include_cost_snapshot: true,
          include_query_metrics: true,
          include_recall: true,
          include_production_read_profile: true,
          production_read_only: true,
          query_metric_k: $bench_top_k,
          query_metric_projection_columns: ["id"],
          log_output: $rowcap_log
        } | maybe_truth($truth_corpus_file))
      ]
    }' > "$suite_config"

  "$ECAZ_BIN" bench suite run \
    --host "$SOCKET_DIR" --port "$COORD_PORT" --user ecaz_coord --database postgres \
    --config "$suite_config" \
    --manifest-output "$suite_artifact_dir/suite-manifest.json" \
    --results-output "$suite_artifact_dir/results.jsonl" \
    > "$suite_artifact_dir/suite-run.log" 2>&1
  echo "bench_suite_summary=passed|$suite_config|$suite_artifact_dir/suite-manifest.json|$suite_artifact_dir/results.jsonl"
}

scripts/spire-aws/load.sh "$TIER" "$TOPOLOGY" "$LOG_DIR"
plan_file="$(cat "$LOG_DIR/distributed-placement-plan-${TIER}.path")"
scripts/spire-aws/register.sh "$TOPOLOGY" "$LOG_DIR" "$plan_file"
PREFIX="$effective_prefix" scripts/spire-aws/smoke.sh "$TOPOLOGY" "$LOG_DIR"

if [[ "$RUN_BENCH_SUITE" == "1" ]]; then
  run_bench_suite
fi

if [[ "$TIER" != "correctness" || "$RUN_FAULT_DRILLS" != "1" ]]; then
  echo "SPIRE Phase 13e AWS harness local PG18 fixture passed"
  echo "HARNESS PASSED"
  exit 0
fi

corpus_table="${effective_prefix}_corpus"
queries_table="${effective_prefix}_queries"
coord_index="${effective_prefix}_idx"
remote_index="${effective_prefix}_remote_idx"
query_vector="$("${coord_psql[@]}" -At -c "SELECT 'ARRAY[' || array_to_string(source, ',') || ']::real[]' FROM ${queries_table} WHERE id = 0")"
fault_nprobe="${SPIRE_AWS_FAULT_NPROBE:-100}"

pooling_log="$LOG_DIR/pooling-socket-open-comparison.tsv"
pooling_sql="$RUN_DIR/pooling-socket-open-comparison.sql"
pooling_stop_remote1="$RUN_DIR/pooling-stop-remote1.sh"
pooling_start_remote1="$RUN_DIR/pooling-start-remote1.sh"
pooling_set_degraded="$RUN_DIR/pooling-set-degraded.sh"
pooling_set_strict="$RUN_DIR/pooling-set-strict.sh"

cat > "$pooling_stop_remote1" <<SCRIPT
#!/usr/bin/env bash
set -euo pipefail
"$PG_CTL" -D "$REMOTE1_DATA" -m fast stop >/dev/null
SCRIPT
cat > "$pooling_start_remote1" <<SCRIPT
#!/usr/bin/env bash
set -euo pipefail
"$PG_CTL" -w -D "$REMOTE1_DATA" -l "$LOG_DIR/remote1-postgres.log" -o "-p $REMOTE1_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null
SCRIPT
cat > "$pooling_set_degraded" <<SCRIPT
#!/usr/bin/env bash
set -euo pipefail
"$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$COORD_PORT" -U postgres -d postgres -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${coord_index}'::regclass::oid, 'degraded')" >/dev/null
"$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE1_PORT" -U postgres -d postgres -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${remote_index}'::regclass::oid, 'degraded')" >/dev/null
"$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE2_PORT" -U postgres -d postgres -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${remote_index}'::regclass::oid, 'degraded')" >/dev/null
"$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE3_PORT" -U postgres -d postgres -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${remote_index}'::regclass::oid, 'degraded')" >/dev/null
SCRIPT
cat > "$pooling_set_strict" <<SCRIPT
#!/usr/bin/env bash
set -euo pipefail
"$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$COORD_PORT" -U postgres -d postgres -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${coord_index}'::regclass::oid, 'strict')" >/dev/null
"$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE1_PORT" -U postgres -d postgres -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${remote_index}'::regclass::oid, 'strict')" >/dev/null
"$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE2_PORT" -U postgres -d postgres -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${remote_index}'::regclass::oid, 'strict')" >/dev/null
"$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE3_PORT" -U postgres -d postgres -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${remote_index}'::regclass::oid, 'strict')" >/dev/null
SCRIPT
chmod +x "$pooling_stop_remote1" "$pooling_start_remote1" "$pooling_set_degraded" "$pooling_set_strict"

cat > "$pooling_sql" <<SQL
SET enable_seqscan = off;
SET enable_indexscan = off;
SET ec_spire.nprobe = $fault_nprobe;
SET ec_spire.remote_search_consistency_mode = strict;
SET ec_spire.remote_search_connection_pool_size = 0;

WITH profile AS (
  SELECT metric, value
  FROM ec_spire_remote_search_production_read_profile(
      '${coord_index}'::regclass,
      $query_vector,
      10
  )
)
SELECT 'pool_disabled_1',
       max(value) FILTER (WHERE metric = 'status'),
       max(value) FILTER (WHERE metric = 'result_source'),
       max(value) FILTER (WHERE metric = 'dispatch_count'),
       max(value) FILTER (WHERE metric = 'socket_open_count'),
       max(value) FILTER (WHERE metric = 'candidate_receive_query_count'),
       max(value) FILTER (WHERE metric = 'heap_receive_query_count'),
       max(value) FILTER (WHERE metric = 'degraded_skipped_dispatch_count'),
       max(value) FILTER (WHERE metric = 'returned_candidate_count'),
       max(value) FILTER (WHERE metric = 'next_blocker')
FROM profile;

WITH profile AS (
  SELECT metric, value
  FROM ec_spire_remote_search_production_read_profile(
      '${coord_index}'::regclass,
      $query_vector,
      10
  )
)
SELECT 'pool_disabled_2',
       max(value) FILTER (WHERE metric = 'status'),
       max(value) FILTER (WHERE metric = 'result_source'),
       max(value) FILTER (WHERE metric = 'dispatch_count'),
       max(value) FILTER (WHERE metric = 'socket_open_count'),
       max(value) FILTER (WHERE metric = 'candidate_receive_query_count'),
       max(value) FILTER (WHERE metric = 'heap_receive_query_count'),
       max(value) FILTER (WHERE metric = 'degraded_skipped_dispatch_count'),
       max(value) FILTER (WHERE metric = 'returned_candidate_count'),
       max(value) FILTER (WHERE metric = 'next_blocker')
FROM profile;

SET ec_spire.remote_search_connection_pool_size = 16;

WITH profile AS (
  SELECT metric, value
  FROM ec_spire_remote_search_production_read_profile(
      '${coord_index}'::regclass,
      $query_vector,
      10
  )
)
SELECT 'pooled_warmup',
       max(value) FILTER (WHERE metric = 'status'),
       max(value) FILTER (WHERE metric = 'result_source'),
       max(value) FILTER (WHERE metric = 'dispatch_count'),
       max(value) FILTER (WHERE metric = 'socket_open_count'),
       max(value) FILTER (WHERE metric = 'candidate_receive_query_count'),
       max(value) FILTER (WHERE metric = 'heap_receive_query_count'),
       max(value) FILTER (WHERE metric = 'degraded_skipped_dispatch_count'),
       max(value) FILTER (WHERE metric = 'returned_candidate_count'),
       max(value) FILTER (WHERE metric = 'next_blocker')
FROM profile;

WITH profile AS (
  SELECT metric, value
  FROM ec_spire_remote_search_production_read_profile(
      '${coord_index}'::regclass,
      $query_vector,
      10
  )
)
SELECT 'pooled_followup_1',
       max(value) FILTER (WHERE metric = 'status'),
       max(value) FILTER (WHERE metric = 'result_source'),
       max(value) FILTER (WHERE metric = 'dispatch_count'),
       max(value) FILTER (WHERE metric = 'socket_open_count'),
       max(value) FILTER (WHERE metric = 'candidate_receive_query_count'),
       max(value) FILTER (WHERE metric = 'heap_receive_query_count'),
       max(value) FILTER (WHERE metric = 'degraded_skipped_dispatch_count'),
       max(value) FILTER (WHERE metric = 'returned_candidate_count'),
       max(value) FILTER (WHERE metric = 'next_blocker')
FROM profile;

WITH profile AS (
  SELECT metric, value
  FROM ec_spire_remote_search_production_read_profile(
      '${coord_index}'::regclass,
      $query_vector,
      10
  )
)
SELECT 'pooled_followup_2',
       max(value) FILTER (WHERE metric = 'status'),
       max(value) FILTER (WHERE metric = 'result_source'),
       max(value) FILTER (WHERE metric = 'dispatch_count'),
       max(value) FILTER (WHERE metric = 'socket_open_count'),
       max(value) FILTER (WHERE metric = 'candidate_receive_query_count'),
       max(value) FILTER (WHERE metric = 'heap_receive_query_count'),
       max(value) FILTER (WHERE metric = 'degraded_skipped_dispatch_count'),
       max(value) FILTER (WHERE metric = 'returned_candidate_count'),
       max(value) FILTER (WHERE metric = 'next_blocker')
FROM profile;

\! "$pooling_set_degraded"
\! "$pooling_stop_remote1"
SET ec_spire.remote_search_consistency_mode = degraded;

WITH profile AS (
  SELECT metric, value
  FROM ec_spire_remote_search_production_read_profile(
      '${coord_index}'::regclass,
      $query_vector,
      10
  )
)
SELECT 'pooled_remote_down_degraded',
       max(value) FILTER (WHERE metric = 'status'),
       max(value) FILTER (WHERE metric = 'result_source'),
       max(value) FILTER (WHERE metric = 'dispatch_count'),
       max(value) FILTER (WHERE metric = 'socket_open_count'),
       max(value) FILTER (WHERE metric = 'candidate_receive_query_count'),
       max(value) FILTER (WHERE metric = 'heap_receive_query_count'),
       max(value) FILTER (WHERE metric = 'degraded_skipped_dispatch_count'),
       max(value) FILTER (WHERE metric = 'returned_candidate_count'),
       max(value) FILTER (WHERE metric = 'next_blocker')
FROM profile;

\! "$pooling_start_remote1"
\! "$pooling_set_strict"
SET ec_spire.remote_search_consistency_mode = strict;

WITH profile AS (
  SELECT metric, value
  FROM ec_spire_remote_search_production_read_profile(
      '${coord_index}'::regclass,
      $query_vector,
      10
  )
)
SELECT 'pooled_after_restart',
       max(value) FILTER (WHERE metric = 'status'),
       max(value) FILTER (WHERE metric = 'result_source'),
       max(value) FILTER (WHERE metric = 'dispatch_count'),
       max(value) FILTER (WHERE metric = 'socket_open_count'),
       max(value) FILTER (WHERE metric = 'candidate_receive_query_count'),
       max(value) FILTER (WHERE metric = 'heap_receive_query_count'),
       max(value) FILTER (WHERE metric = 'degraded_skipped_dispatch_count'),
       max(value) FILTER (WHERE metric = 'returned_candidate_count'),
       max(value) FILTER (WHERE metric = 'next_blocker')
FROM profile;
SQL

printf 'phase\tstatus\tresult_source\tdispatch_count\tsocket_open_count\tcandidate_receive_query_count\theap_receive_query_count\tdegraded_skipped_dispatch_count\treturned_candidate_count\tnext_blocker\n' > "$pooling_log"
"${coord_psql[@]}" -At -F $'\t' -f "$pooling_sql" >> "$pooling_log"

pooling_summary="$(awk -F '\t' '
NR == 1 { next }
$1 ~ /^pool_disabled_/ {
  disabled_socket_sum += $5;
  disabled_rows++;
  if ($2 != "ready" || $5 != 3) bad++;
}
$1 == "pooled_warmup" {
  pooled_warmup_socket = $5;
  if ($2 != "ready") bad++;
}
$1 ~ /^pooled_followup_/ {
  pooled_followup_socket_sum += $5;
  pooled_followup_rows++;
  if ($2 != "ready" || $5 != 0) bad++;
}
$1 == "pooled_remote_down_degraded" {
  degraded_status = $2;
  degraded_socket = $5;
  degraded_skipped = $8;
  if ($2 != "degraded_ready" || $8 < 1) bad++;
}
$1 == "pooled_after_restart" {
  after_restart_status = $2;
  after_restart_socket = $5;
  if ($2 != "ready" || $5 != 1) bad++;
}
END {
  printf "disabled_rows=%d|disabled_socket_sum=%d|pooled_warmup_socket=%d|pooled_followup_rows=%d|pooled_followup_socket_sum=%d|degraded_status=%s|degraded_socket=%d|degraded_skipped=%d|after_restart_status=%s|after_restart_socket=%d|bad=%d\n",
    disabled_rows,
    disabled_socket_sum,
    pooled_warmup_socket,
    pooled_followup_rows,
    pooled_followup_socket_sum,
    degraded_status,
    degraded_socket,
    degraded_skipped,
    after_restart_status,
    after_restart_socket,
    bad;
}' "$pooling_log")"
echo "pooling_socket_open_comparison=$pooling_summary"
[[ "$pooling_summary" == *"disabled_rows=2"* ]]
[[ "$pooling_summary" == *"disabled_socket_sum=6"* ]]
[[ "$pooling_summary" == *"pooled_warmup_socket=3"* ]]
[[ "$pooling_summary" == *"pooled_followup_rows=2"* ]]
[[ "$pooling_summary" == *"pooled_followup_socket_sum=0"* ]]
[[ "$pooling_summary" == *"degraded_status=degraded_ready"* ]]
[[ "$pooling_summary" == *"after_restart_status=ready"* ]]
[[ "$pooling_summary" == *"after_restart_socket=1"* ]]
[[ "$pooling_summary" == *"bad=0"* ]]

publish_coord_mode() {
  local mode="${1:?mode required}"
  "${coord_psql[@]}" \
    -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${coord_index}'::regclass::oid, '$mode')" \
    > "$LOG_DIR/aws-local-fault-publish-${mode}.log"
  "${remote1_psql[@]}" \
    -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${remote_index}'::regclass::oid, '$mode')" \
    > "$LOG_DIR/aws-local-fault-publish-node-2-${mode}.log"
  "${remote2_psql[@]}" \
    -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${remote_index}'::regclass::oid, '$mode')" \
    > "$LOG_DIR/aws-local-fault-publish-node-3-${mode}.log"
  "${remote3_psql[@]}" \
    -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('${remote_index}'::regclass::oid, '$mode')" \
    > "$LOG_DIR/aws-local-fault-publish-node-4-${mode}.log"
}

restart_remote1() {
  "$PG_CTL" -w -D "$REMOTE1_DATA" -l "$LOG_DIR/remote1-postgres.log" \
    -o "-p $REMOTE1_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null
}

echo "aws_local_fault_drill=degraded"
publish_coord_mode degraded
"$PG_CTL" -D "$REMOTE1_DATA" -m fast stop >/dev/null

degraded_pgoptions="-c enable_seqscan=off -c enable_indexscan=off -c ec_spire.nprobe=$fault_nprobe -c ec_spire.remote_search_consistency_mode=degraded"
PGOPTIONS="$degraded_pgoptions" "${coord_psql[@]}" \
  -c "SELECT id FROM ${corpus_table} ORDER BY embedding <#> ${query_vector} LIMIT 10" \
  > "$LOG_DIR/aws-local-fault-degraded-knn.log"

degraded_summary="$(PGOPTIONS="$degraded_pgoptions" "${coord_psql[@]}" -At -F '|' -c "WITH profile AS (SELECT metric, value FROM ec_spire_remote_search_production_read_profile('${coord_index}'::regclass, ${query_vector}, 10)), summary AS (SELECT max(value) FILTER (WHERE metric = 'status') AS status, max(value) FILTER (WHERE metric = 'degraded_skipped_dispatch_count')::int AS skipped, max(value) FILTER (WHERE metric = 'returned_candidate_count')::int AS returned, max(value) FILTER (WHERE metric = 'next_blocker') AS next_blocker FROM profile) SELECT status, skipped, returned, next_blocker FROM summary")"
printf '%s\n' "$degraded_summary" > "$LOG_DIR/aws-local-fault-degraded-summary.log"
IFS='|' read -r degraded_status degraded_skipped degraded_returned degraded_next_blocker <<< "$degraded_summary"
if [[ "$degraded_status" != "degraded_ready" || "$degraded_skipped" -le 0 || "$degraded_returned" -le 0 || "$degraded_next_blocker" != "none" ]]; then
  echo "AWS local degraded fault drill failed: $degraded_summary" >&2
  exit 1
fi
restart_remote1

echo "aws_local_fault_drill=strict"
publish_coord_mode strict
"$PG_CTL" -D "$REMOTE1_DATA" -m fast stop >/dev/null
strict_pgoptions="-c enable_seqscan=off -c enable_indexscan=off -c ec_spire.nprobe=$fault_nprobe -c ec_spire.remote_search_consistency_mode=strict"
strict_status=0
PGOPTIONS="$strict_pgoptions" "${coord_psql[@]}" \
  -c "SELECT id FROM ${corpus_table} ORDER BY embedding <#> ${query_vector} LIMIT 10" \
  > "$LOG_DIR/aws-local-fault-strict-knn.log" 2> "$LOG_DIR/aws-local-fault-strict-knn.stderr.log" || strict_status=$?
if [[ "$strict_status" -eq 0 ]]; then
  echo "AWS local strict fault drill unexpectedly succeeded with remote1 stopped" >&2
  exit 1
fi
restart_remote1

echo "SPIRE Phase 13e AWS harness local PG18 fixture passed"
echo "HARNESS PASSED"
