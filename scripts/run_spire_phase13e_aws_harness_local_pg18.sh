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

scripts/spire-aws/load.sh correctness "$TOPOLOGY" "$LOG_DIR"
plan_file="$(cat "$LOG_DIR/distributed-placement-plan-correctness.path")"
scripts/spire-aws/register.sh "$TOPOLOGY" "$LOG_DIR" "$plan_file"
PREFIX=ec_spire_aws_synth_10k scripts/spire-aws/smoke.sh "$TOPOLOGY" "$LOG_DIR"

query_vector="$("${coord_psql[@]}" -At -c "SELECT 'ARRAY[' || array_to_string(source, ',') || ']::real[]' FROM ec_spire_aws_synth_10k_queries WHERE id = 0")"
fault_nprobe="${SPIRE_AWS_FAULT_NPROBE:-100}"

publish_coord_mode() {
  local mode="${1:?mode required}"
  "${coord_psql[@]}" \
    -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('ec_spire_aws_synth_10k_idx'::regclass::oid, '$mode')" \
    > "$LOG_DIR/aws-local-fault-publish-${mode}.log"
  "${remote1_psql[@]}" \
    -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('ec_spire_aws_synth_10k_remote_idx'::regclass::oid, '$mode')" \
    > "$LOG_DIR/aws-local-fault-publish-node-2-${mode}.log"
  "${remote2_psql[@]}" \
    -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('ec_spire_aws_synth_10k_remote_idx'::regclass::oid, '$mode')" \
    > "$LOG_DIR/aws-local-fault-publish-node-3-${mode}.log"
  "${remote3_psql[@]}" \
    -c "SELECT * FROM ec_spire_set_static_remote_placement_consistency_mode('ec_spire_aws_synth_10k_remote_idx'::regclass::oid, '$mode')" \
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
  -c "SELECT id FROM ec_spire_aws_synth_10k_corpus ORDER BY embedding <#> ${query_vector} LIMIT 10" \
  > "$LOG_DIR/aws-local-fault-degraded-knn.log"

degraded_summary="$(PGOPTIONS="$degraded_pgoptions" "${coord_psql[@]}" -At -F '|' -c "WITH profile AS (SELECT metric, value FROM ec_spire_remote_search_production_read_profile('ec_spire_aws_synth_10k_idx'::regclass, ${query_vector}, 10)), summary AS (SELECT max(value) FILTER (WHERE metric = 'status') AS status, max(value) FILTER (WHERE metric = 'degraded_skipped_dispatch_count')::int AS skipped, max(value) FILTER (WHERE metric = 'returned_candidate_count')::int AS returned, max(value) FILTER (WHERE metric = 'next_blocker') AS next_blocker FROM profile) SELECT status, skipped, returned, next_blocker FROM summary")"
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
  -c "SELECT id FROM ec_spire_aws_synth_10k_corpus ORDER BY embedding <#> ${query_vector} LIMIT 10" \
  > "$LOG_DIR/aws-local-fault-strict-knn.log" 2> "$LOG_DIR/aws-local-fault-strict-knn.stderr.log" || strict_status=$?
if [[ "$strict_status" -eq 0 ]]; then
  echo "AWS local strict fault drill unexpectedly succeeded with remote1 stopped" >&2
  exit 1
fi
restart_remote1

echo "SPIRE Phase 13e AWS harness local PG18 fixture passed"
echo "HARNESS PASSED"
