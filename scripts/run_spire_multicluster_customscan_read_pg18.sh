#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
REMOTE_PORT="${REMOTE_PORT:-39228}"
COORD_PORT="${COORD_PORT:-39229}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
LOG_DIR_OVERRIDE="${LOG_DIR:-}"
SMOKE_LOG="${SMOKE_LOG:-}"
ARTIFACT_DIR=""

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_multicluster_customscan_read_pg18.sh [options]

Options:
  --artifact-dir DIR  Store smoke and PostgreSQL logs in DIR.
  --coord-port PORT   Coordinator PostgreSQL port. Default: 39229.
  --log-dir DIR       Store PostgreSQL logs in DIR.
  --pgbin DIR         PostgreSQL bin directory. Default: $PGBIN.
  --remote-port PORT  Remote PostgreSQL port. Default: 39228.
  --run-dir DIR       Run directory. Default: target/spire-cscan-read-pg18-$RUN_ID.
  --run-id ID         Run id used in the default run directory.
  --skip-install      Skip cargo pgrx install.
  --smoke-log FILE    Tee smoke output to FILE.
  -h, --help          Show this help.
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
    --remote-port)
      REMOTE_PORT="$2"
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

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-cscan-read-pg18-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/multicluster-customscan-read.log}"
else
  LOG_DIR="${LOG_DIR_OVERRIDE:-$RUN_DIR/logs}"
fi
REMOTE_DATA="$RUN_DIR/remote"
COORD_DATA="$RUN_DIR/coord"
SOCKET_DIR="$RUN_DIR/s"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_CUSTOMSCAN_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_CUSTOMSCAN_LOG_ACTIVE=1
  exec > >(tee "$SMOKE_LOG") 2>&1
fi

if [[ -e "$RUN_DIR" ]]; then
  echo "RUN_DIR already exists: $RUN_DIR" >&2
  exit 2
fi

mkdir -p "$LOG_DIR" "$SOCKET_DIR"
: > "$LOG_DIR/remote-postgres.log"
: > "$LOG_DIR/coord-postgres.log"

cleanup() {
  "$PG_CTL" -D "$COORD_DATA" -m fast stop >/dev/null 2>&1 || true
  "$PG_CTL" -D "$REMOTE_DATA" -m fast stop >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "run_dir=$RUN_DIR"
echo "remote_port=$REMOTE_PORT"
echo "coord_port=$COORD_PORT"

if [[ "${ECAZ_SKIP_INSTALL:-0}" != "1" ]]; then
  (cd "$ROOT_DIR" && cargo pgrx install --test --pg-config "$PGBIN/pg_config" \
    --features "pg18 pg_test" --no-default-features)
fi

"$PG_CTL" initdb -D "$REMOTE_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$COORD_DATA" -o "-A trust -U postgres" >/dev/null

export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_CUSTOMSCAN_MULTICLUSTER="host=$SOCKET_DIR port=$REMOTE_PORT dbname=postgres user=postgres connect_timeout=1"

"$PG_CTL" -w -D "$REMOTE_DATA" -l "$LOG_DIR/remote-postgres.log" \
  -o "-p $REMOTE_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null
"$PG_CTL" -w -D "$COORD_DATA" -l "$LOG_DIR/coord-postgres.log" \
  -o "-p $COORD_PORT -k $SOCKET_DIR -c listen_addresses=''" start >/dev/null

remote_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE_PORT" -U postgres -d postgres)
coord_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$COORD_PORT" -U postgres -d postgres)

"${remote_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null
"${coord_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null

"${remote_psql[@]}" <<'SQL' >/dev/null
CREATE DOMAIN ec_spire_customscan_label_domain AS text
    CHECK (VALUE <> 'blocked');
CREATE TYPE ec_spire_customscan_pair AS (code int4, label text);
CREATE TABLE ec_spire_customscan_remote_sql
    (id bigint primary key,
     title text not null,
     tags text[] not null,
     label ec_spire_customscan_label_domain not null,
     pair ec_spire_customscan_pair not null,
     embedding ecvector);
INSERT INTO ec_spire_customscan_remote_sql
    (id, title, tags, label, pair, embedding) VALUES
    (10, 'remote alpha', ARRAY['red', 'blue']::text[],
     'domain alpha'::ec_spire_customscan_label_domain,
     ROW(7, 'left')::ec_spire_customscan_pair,
     encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
    (20, 'remote beta', ARRAY['green']::text[],
     'domain beta'::ec_spire_customscan_label_domain,
     ROW(9, 'right')::ec_spire_customscan_pair,
     encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42));
CREATE INDEX ec_spire_customscan_remote_idx
    ON ec_spire_customscan_remote_sql USING ec_spire
    (embedding ecvector_spire_ip_ops)
    WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq');
SQL

"${coord_psql[@]}" <<'SQL' >/dev/null
CREATE DOMAIN ec_spire_customscan_label_domain AS text
    CHECK (VALUE <> 'blocked');
CREATE TYPE ec_spire_customscan_pair AS (code int4, label text);
CREATE TABLE ec_spire_customscan_coord_sql
    (id bigint primary key,
     title text not null,
     tags text[] not null,
     label ec_spire_customscan_label_domain not null,
     pair ec_spire_customscan_pair not null,
     embedding ecvector);
INSERT INTO ec_spire_customscan_coord_sql
    (id, title, tags, label, pair, embedding) VALUES
    (1, 'coordinator alpha', ARRAY['local', 'red']::text[],
     'domain coord alpha'::ec_spire_customscan_label_domain,
     ROW(1, 'coord-left')::ec_spire_customscan_pair,
     encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)),
    (2, 'coordinator beta', ARRAY['local', 'green']::text[],
     'domain coord beta'::ec_spire_customscan_label_domain,
     ROW(2, 'coord-right')::ec_spire_customscan_pair,
     encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42));
CREATE INDEX ec_spire_customscan_coord_idx
    ON ec_spire_customscan_coord_sql USING ec_spire
    (embedding ecvector_spire_ip_ops)
    WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq');
SQL

remote_epoch="$("${remote_psql[@]}" -At -c "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('ec_spire_customscan_remote_idx'::regclass)")"
coord_epoch="$("${coord_psql[@]}" -At -c "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('ec_spire_customscan_coord_idx'::regclass)")"
remote_pids="$("${remote_psql[@]}" -At -c "SELECT array_to_string(array_agg(leaf_pid ORDER BY leaf_pid), ',') FROM ec_spire_index_leaf_snapshot('ec_spire_customscan_remote_idx'::regclass)")"
coord_pids="$("${coord_psql[@]}" -At -c "SELECT array_to_string(array_agg(leaf_pid ORDER BY leaf_pid), ',') FROM ec_spire_index_leaf_snapshot('ec_spire_customscan_coord_idx'::regclass)")"
remote_identity_hex="$("${remote_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_customscan_remote_idx'::regclass::oid)")"
extversion="$("${coord_psql[@]}" -At -c "SELECT extversion FROM pg_extension WHERE extname = 'ecaz'")"

if [[ "$remote_epoch" != "$coord_epoch" ]]; then
  echo "epoch mismatch remote=$remote_epoch coord=$coord_epoch" >&2
  exit 3
fi
if [[ "$remote_pids" != "$coord_pids" ]]; then
  echo "leaf pid mismatch remote=$remote_pids coord=$coord_pids" >&2
  exit 4
fi

"${coord_psql[@]}" -v coord_epoch="$coord_epoch" -v extversion="$extversion" \
  -v remote_identity_hex="$remote_identity_hex" <<'SQL' >/dev/null
WITH rewritten AS (
    SELECT tests.ec_spire_test_rewrite_placement_node(
        'ec_spire_customscan_coord_idx'::regclass::oid,
        leaf_pid,
        2
    )
    FROM ec_spire_index_leaf_snapshot('ec_spire_customscan_coord_idx'::regclass)
)
SELECT count(*) FROM rewritten;

SELECT ec_spire_register_remote_node_descriptor(
    'ec_spire_customscan_coord_idx'::regclass,
    2,
    91,
    'spire/remote/customscan/multicluster',
    decode(:'remote_identity_hex', 'hex'),
    'ec_spire_customscan_remote_idx',
    'active',
    :coord_epoch::bigint,
    :coord_epoch::bigint,
    :'extversion',
    'none'
);
SQL

plan="$(PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off" "${coord_psql[@]}" -At -c "EXPLAIN (COSTS OFF) SELECT id, title FROM ec_spire_customscan_coord_sql ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1")"
read_row="$(PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off" "${coord_psql[@]}" -At -F '|' -c "SELECT id, title, tags, label::text, pair::text FROM ec_spire_customscan_coord_sql ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1")"
payload_probe="$("${remote_psql[@]}" -At -F ',' -c "SELECT status, payload_column_count::text, tuple_payload::text FROM ec_spire_remote_search_tuple_payload('ec_spire_customscan_remote_idx'::regclass::oid, ${remote_epoch}::bigint, ARRAY[1.0, 0.0]::real[], ARRAY[${remote_pids}]::bigint[], 1, 'strict', ARRAY['id','title']::text[]) LIMIT 1")"
typed_payload_probe="$("${remote_psql[@]}" -At -F ',' -c "SELECT status, tuple_transport, payload_formats = ARRAY['pg_binary_attr_v1','pg_binary_attr_v1','pg_binary_attr_v1']::text[], payload_values[3] = array_send(ARRAY['red','blue']::text[])::bytea FROM ec_spire_remote_search_tuple_payload_typed('ec_spire_customscan_remote_idx'::regclass::oid, ${remote_epoch}::bigint, ARRAY[1.0, 0.0]::real[], ARRAY[${remote_pids}]::bigint[], 1, 'strict', ARRAY['id','title','tags']::text[]) LIMIT 1")"
profile_summary="$("${coord_psql[@]}" -At -F '|' -c "
WITH profile AS (
    SELECT metric, value
    FROM ec_spire_remote_search_production_read_profile(
        'ec_spire_customscan_coord_idx'::regclass::oid,
        ARRAY[1.0, 0.0]::real[],
        1
    )
)
SELECT max(value) FILTER (WHERE metric = 'status') || '|'
       || max(value) FILTER (WHERE metric = 'final_heap_fetch_status') || '|'
       || max(value) FILTER (WHERE metric = 'socket_open_count') || '|'
       || max(value) FILTER (WHERE metric = 'conninfo_secret_lookup_count') || '|'
       || max(value) FILTER (WHERE metric = 'regclass_probe_count') || '|'
       || max(value) FILTER (WHERE metric = 'endpoint_identity_query_count') || '|'
       || max(value) FILTER (WHERE metric = 'candidate_receive_query_count') || '|'
       || max(value) FILTER (WHERE metric = 'heap_receive_query_count') || '|'
       || max(value) FILTER (WHERE metric = 'returned_candidate_count')
FROM profile
")"
IFS='|' read -r profile_status profile_heap_status profile_socket_count \
  profile_secret_count profile_regclass_count profile_identity_count \
  profile_candidate_count profile_heap_count profile_returned_count \
  <<< "$profile_summary"

echo "remote_epoch=$remote_epoch"
echo "coord_epoch=$coord_epoch"
echo "remote_pids=$remote_pids"
echo "coord_pids=$coord_pids"
echo "remote_identity_hex=$remote_identity_hex"
echo "plan=$plan"
echo "read_row=$read_row"
echo "payload_probe=$payload_probe"
echo "typed_payload_probe=$typed_payload_probe"
echo "profile_summary=$profile_summary"

[[ "$plan" == *"Custom Scan (EcSpireDistributedScan)"* ]]
[[ "$read_row" == '10|remote alpha|{red,blue}|domain alpha|(7,left)' ]]
[[ "$payload_probe" == ready,2,*'"id": 10'* ]]
[[ "$payload_probe" == ready,2,*'"title": "remote alpha"'* ]]
[[ "$typed_payload_probe" == "ready,pg_binary_attr_v1,t,t" ]]
[[ "$profile_status" == "ready" ]]
[[ "$profile_heap_status" == "remote_ready" ]]
[[ "$profile_socket_count" == "1" ]]
[[ "$profile_secret_count" == "1" ]]
[[ "$profile_regclass_count" == "1" ]]
[[ "$profile_identity_count" == "1" ]]
[[ "$profile_candidate_count" == "1" ]]
[[ "$profile_heap_count" == "1" ]]
[[ "$profile_returned_count" == "1" ]]

echo "SPIRE multicluster CustomScan read passed"
