#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
REMOTE_FAST_PORT="${REMOTE_FAST_PORT:-39218}"
COORD_PORT="${COORD_PORT:-39219}"
REMOTE_SLOW_PORT="${REMOTE_SLOW_PORT:-39220}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
LOG_DIR_OVERRIDE="${LOG_DIR:-}"
SMOKE_LOG="${SMOKE_LOG:-}"
ARTIFACT_DIR=""

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_multicluster_transport_overlap_pg18.sh [options]

Options:
  --artifact-dir DIR       Store smoke and PostgreSQL logs in DIR.
  --coord-port PORT        Coordinator PostgreSQL port. Default: 39219.
  --log-dir DIR            Store PostgreSQL logs in DIR.
  --pgbin DIR              PostgreSQL bin directory. Default: $PGBIN.
  --remote-fast-port PORT  Fast remote PostgreSQL port. Default: 39218.
  --remote-slow-port PORT  Slow remote PostgreSQL port. Default: 39220.
  --run-dir DIR            Run directory. Default: target/spire-multicluster-transport-overlap-pg18-$RUN_ID.
  --run-id ID              Run id used in the default run directory.
  --skip-install           Skip cargo pgrx install.
  --smoke-log FILE         Tee smoke output to FILE.
  -h, --help               Show this help.
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
    --log-dir)
      LOG_DIR_OVERRIDE="$2"
      shift 2
      ;;
    --pgbin)
      PGBIN="$2"
      PG_CTL="$PGBIN/pg_ctl"
      PSQL="$PGBIN/psql"
      shift 2
      ;;
    --remote-fast-port)
      REMOTE_FAST_PORT="$2"
      shift 2
      ;;
    --remote-slow-port)
      REMOTE_SLOW_PORT="$2"
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

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-multicluster-transport-overlap-pg18-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/multicluster-transport-overlap.log}"
else
  LOG_DIR="${LOG_DIR_OVERRIDE:-$RUN_DIR/logs}"
fi
REMOTE_FAST_DATA="$RUN_DIR/remote-fast"
REMOTE_SLOW_DATA="$RUN_DIR/remote-slow"
COORD_DATA="$RUN_DIR/coord"
SOCKET_KEY="$(printf '%s' "$RUN_DIR" | cksum | awk '{print $1}')"
SOCKET_DIR="${SOCKET_DIR:-$ROOT_DIR/target/s-$SOCKET_KEY}"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_TRANSPORT_OVERLAP_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_TRANSPORT_OVERLAP_LOG_ACTIVE=1
  exec > >(tee "$SMOKE_LOG") 2>&1
fi

if [[ -e "$RUN_DIR" ]]; then
  echo "RUN_DIR already exists: $RUN_DIR" >&2
  exit 2
fi

mkdir -p "$LOG_DIR" "$SOCKET_DIR"
: > "$LOG_DIR/remote-fast-postgres.log"
: > "$LOG_DIR/remote-slow-postgres.log"
: > "$LOG_DIR/coord-postgres.log"

cleanup() {
  "$PG_CTL" -D "$COORD_DATA" -m fast stop >/dev/null 2>&1 || true
  "$PG_CTL" -D "$REMOTE_SLOW_DATA" -m fast stop >/dev/null 2>&1 || true
  "$PG_CTL" -D "$REMOTE_FAST_DATA" -m fast stop >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "run_dir=$RUN_DIR"
echo "remote_fast_port=$REMOTE_FAST_PORT"
echo "remote_slow_port=$REMOTE_SLOW_PORT"
echo "coord_port=$COORD_PORT"

if [[ "${ECAZ_SKIP_INSTALL:-0}" != "1" ]]; then
  (cd "$ROOT_DIR" && cargo pgrx install --test --pg-config "$PGBIN/pg_config" \
    --features "pg18 pg_test" --no-default-features)
fi

"$PG_CTL" initdb -D "$REMOTE_FAST_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$REMOTE_SLOW_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$COORD_DATA" -o "-A trust -U postgres" >/dev/null

export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_MULTICLUSTER_FAST="host=$SOCKET_DIR port=$REMOTE_FAST_PORT dbname=postgres user=postgres connect_timeout=1"
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_MULTICLUSTER_SLOW="host=$SOCKET_DIR port=$REMOTE_SLOW_PORT dbname=postgres user=postgres connect_timeout=1"

"$PG_CTL" -w -D "$REMOTE_FAST_DATA" -l "$LOG_DIR/remote-fast-postgres.log" \
  -o "-p $REMOTE_FAST_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null
"$PG_CTL" -w -D "$REMOTE_SLOW_DATA" -l "$LOG_DIR/remote-slow-postgres.log" \
  -o "-p $REMOTE_SLOW_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null
"$PG_CTL" -w -D "$COORD_DATA" -l "$LOG_DIR/coord-postgres.log" \
  -o "-p $COORD_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null

remote_fast_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE_FAST_PORT" -U postgres -d postgres)
remote_slow_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE_SLOW_PORT" -U postgres -d postgres)
coord_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$COORD_PORT" -U postgres -d postgres)

"${remote_fast_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null
"${remote_slow_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null
"${coord_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null

probe_rows="$("${coord_psql[@]}" -At -F ',' -c "
SELECT node_id::text
       || ',' || status
       || ',' || failure_category
       || ',' || started_after_ms::text
       || ',' || completed_after_ms::text
       || ',' || elapsed_ms::text
       || ',' || row_count::text
  FROM tests.ec_spire_test_production_transport_probe(
       ARRAY[2,3]::integer[],
       ARRAY['spire/remote/multicluster/slow','spire/remote/multicluster/fast']::text[],
       2
  )
 ORDER BY node_id
")"

slow_status=""
slow_failure=""
slow_completed=0
slow_elapsed=0
fast_status=""
fast_failure=""
fast_completed=0
fast_elapsed=0

while IFS=, read -r node_id status failure started completed elapsed row_count; do
  echo "transport_overlap_row=$node_id,$status,$failure,$started,$completed,$elapsed,$row_count"
  case "$node_id" in
    2)
      slow_status="$status"
      slow_failure="$failure"
      slow_completed="$completed"
      slow_elapsed="$elapsed"
      ;;
    3)
      fast_status="$status"
      fast_failure="$failure"
      fast_completed="$completed"
      fast_elapsed="$elapsed"
      ;;
    *)
      echo "unexpected node_id=$node_id" >&2
      exit 5
      ;;
  esac
done <<< "$probe_rows"

echo "slow_status=$slow_status"
echo "slow_failure_category=$slow_failure"
echo "slow_completed_after_ms=$slow_completed"
echo "slow_elapsed_ms=$slow_elapsed"
echo "fast_status=$fast_status"
echo "fast_failure_category=$fast_failure"
echo "fast_completed_after_ms=$fast_completed"
echo "fast_elapsed_ms=$fast_elapsed"

[[ "$slow_status" == "ready" ]]
[[ "$slow_failure" == "none" ]]
[[ "$fast_status" == "ready" ]]
[[ "$fast_failure" == "none" ]]
[[ "$fast_completed" -lt "$slow_completed" ]]
[[ "$slow_elapsed" -ge 250 ]]

echo "fast_completed_before_slow=true"
echo "SPIRE multicluster PG18 transport overlap passed"
