#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
COORD_PORT="${COORD_PORT:-39440}"
REMOTE1_PORT="${REMOTE1_PORT:-39441}"
REMOTE2_PORT="${REMOTE2_PORT:-39442}"
REMOTE3_PORT="${REMOTE3_PORT:-39443}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
ARTIFACT_DIR=""
SMOKE_LOG="${SMOKE_LOG:-}"

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_phase13e_static_remote_placement_pg18.sh [options]

Options:
  --artifact-dir DIR   Store smoke and PostgreSQL logs in DIR.
  --coord-port PORT    Coordinator PostgreSQL port. Default: 39440.
  --pgbin DIR          PostgreSQL bin directory. Default: $PGBIN.
  --remote1-port PORT  First remote PostgreSQL port. Default: 39441.
  --remote2-port PORT  Second remote PostgreSQL port. Default: 39442.
  --remote3-port PORT  Third remote PostgreSQL port. Default: 39443.
  --run-dir DIR        Run directory. Default: target/spire-phase13e-static-remote-$RUN_ID.
  --run-id ID          Run id used in the default run directory.
  --skip-install       Skip cargo pgrx install.
  --smoke-log FILE     Tee smoke output to FILE.
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

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-phase13e-static-remote-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/phase13e-static-remote-placement.log}"
else
  LOG_DIR="$RUN_DIR/logs"
fi
SOCKET_DIR="$RUN_DIR/s"
COORD_DATA="$RUN_DIR/coord"
REMOTE1_DATA="$RUN_DIR/remote1"
REMOTE2_DATA="$RUN_DIR/remote2"
REMOTE3_DATA="$RUN_DIR/remote3"
ASSIGNMENT_DIR="$RUN_DIR/assignments"
IDENTITY_DIR="$RUN_DIR/identities"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_PHASE13E_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_PHASE13E_LOG_ACTIVE=1
  exec > >(tee "$SMOKE_LOG") 2>&1
fi

if [[ -e "$RUN_DIR" ]]; then
  echo "RUN_DIR already exists: $RUN_DIR" >&2
  exit 2
fi

mkdir -p "$LOG_DIR" "$SOCKET_DIR" "$ASSIGNMENT_DIR" "$IDENTITY_DIR"
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

export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_PHASE13E_NODE2="host=$SOCKET_DIR port=$REMOTE1_PORT dbname=postgres user=postgres connect_timeout=1"
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_PHASE13E_NODE3="host=$SOCKET_DIR port=$REMOTE2_PORT dbname=postgres user=postgres connect_timeout=1"
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_PHASE13E_NODE4="host=$SOCKET_DIR port=$REMOTE3_PORT dbname=postgres user=postgres connect_timeout=1"

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

"${coord_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null
"${remote1_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null
"${remote2_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null
"${remote3_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null

"${coord_psql[@]}" <<'SQL' >/dev/null
CREATE TABLE ec_spire_phase13e_coord_sql
    (id bigint primary key, title text not null, embedding ecvector);
INSERT INTO ec_spire_phase13e_coord_sql (id, title, embedding)
SELECT id,
       format('doc %s', id),
       encode_to_ecvector(
         CASE (id % 4)
           WHEN 1 THEN ARRAY[1.0, 0.0]::real[]
           WHEN 2 THEN ARRAY[0.6, 0.4]::real[]
           WHEN 3 THEN ARRAY[-1.0, 0.0]::real[]
           ELSE ARRAY[0.0, 1.0]::real[]
         END,
         4,
         42
       )
  FROM generate_series(1, 12) AS id;
CREATE INDEX ec_spire_phase13e_coord_idx
    ON ec_spire_phase13e_coord_sql USING ec_spire
    (embedding ecvector_spire_ip_ops)
    WITH (nlists = 3, nprobe = 3, storage_format = 'rabitq');
SQL

create_remote_shard() {
  local node_id="$1"
  local -n remote_psql_ref="$2"
  local assignment_file="$ASSIGNMENT_DIR/node-${node_id}-assignments.tsv"

  "${coord_psql[@]}" -At -F $'\t' \
    -v coord_index=ec_spire_phase13e_coord_idx \
    -v coord_table=ec_spire_phase13e_coord_sql \
    -v node_id="$node_id" \
    -v remotes_json='[{"node_id":2},{"node_id":3},{"node_id":4}]' \
    -f "$ROOT_DIR/scripts/spire-aws/export-coordinator-leaf-base-assignments.sql" \
    > "$assignment_file"

  if [[ ! -s "$assignment_file" ]]; then
    echo "node_id=${node_id} received no coordinator leaf assignments" >&2
    exit 3
  fi

  "${remote_psql_ref[@]}" <<SQL >/dev/null
CREATE TEMP TABLE ec_spire_phase13e_assignment_import (
  active_epoch bigint NOT NULL,
  leaf_pid bigint NOT NULL,
  parent_pid bigint NOT NULL,
  object_version bigint NOT NULL,
  row_index bigint NOT NULL,
  assignment_flags int NOT NULL,
  vec_id_hex text NOT NULL,
  row_locator_hex text NOT NULL,
  coordinator_heap_block bigint NOT NULL,
  coordinator_heap_offset int NOT NULL,
  coordinator_heap_ctid text NOT NULL,
  row_id bigint NOT NULL,
  payload_format int NOT NULL,
  gamma real NOT NULL,
  encoded_payload_hex text NOT NULL
);
\copy ec_spire_phase13e_assignment_import FROM '$assignment_file' WITH (FORMAT text, DELIMITER E'\t')
CREATE TABLE ec_spire_phase13e_remote_sql
    (id bigint primary key, title text not null, embedding ecvector);
INSERT INTO ec_spire_phase13e_remote_sql (id, title, embedding)
WITH shard_rows AS (
  SELECT DISTINCT row_id
  FROM ec_spire_phase13e_assignment_import
)
SELECT row_id,
       format('doc %s', row_id),
       encode_to_ecvector(
         CASE (row_id % 4)
           WHEN 1 THEN ARRAY[1.0, 0.0]::real[]
           WHEN 2 THEN ARRAY[0.6, 0.4]::real[]
           WHEN 3 THEN ARRAY[-1.0, 0.0]::real[]
           ELSE ARRAY[0.0, 1.0]::real[]
         END,
         4,
         42
       )
  FROM shard_rows
 ORDER BY row_id;
CREATE INDEX ec_spire_phase13e_remote_idx
    ON ec_spire_phase13e_remote_sql USING ec_spire
    (embedding ecvector_spire_ip_ops)
    WITH (nlists = 3, nprobe = 3, storage_format = 'rabitq');
SQL

  local materialize_sql="$RUN_DIR/node-${node_id}-materialize-rendered.sql"
  local assignment_file_sql="${assignment_file//\'/\'\'}"
  sed "s|__ECAZ_ASSIGNMENT_FILE__|$assignment_file_sql|g" \
    "$ROOT_DIR/scripts/spire-aws/materialize-remote-leaf-base-assignments.sql" \
    > "$materialize_sql"

  "${remote_psql_ref[@]}" \
    -v remote_index=ec_spire_phase13e_remote_idx \
    -v remote_table=ec_spire_phase13e_remote_sql \
    -f "$materialize_sql" \
    > "$LOG_DIR/node-${node_id}-materialize.log"

  "${remote_psql_ref[@]}" -At -v remote_index=ec_spire_phase13e_remote_idx \
    > "$IDENTITY_DIR/node-${node_id}-identity.json" <<'SQL'
SELECT jsonb_build_object(
  'remote_index_regclass', 'ec_spire_phase13e_remote_idx',
  'last_served_epoch', a.active_epoch,
  'min_retained_epoch', a.active_epoch,
  'extension_version', e.extension_version,
  'remote_index_identity_hex', e.profile_fingerprint,
  'endpoint_status', e.status,
  'tuple_transport_status', e.tuple_transport_status
)::text
FROM ec_spire_remote_search_endpoint_identity(:'remote_index'::regclass::oid) e
CROSS JOIN ec_spire_index_active_snapshot_diagnostics(:'remote_index'::regclass::oid) a;
SQL
}

create_remote_shard 2 remote1_psql
create_remote_shard 3 remote2_psql
create_remote_shard 4 remote3_psql

"${coord_psql[@]}" \
  -v coord_index=ec_spire_phase13e_coord_idx \
  -v remotes_json='[{"node_id":2},{"node_id":3},{"node_id":4}]' \
  -f "$ROOT_DIR/scripts/spire-aws/publish-remote-placements.sql" >/dev/null

extversion="$("${coord_psql[@]}" -At -c "SELECT extversion FROM pg_extension WHERE extname = 'ecaz'")"

register_remote() {
  local node_id="$1"
  local secret_name="spire/remote/phase13e/node${node_id}"
  local identity_file="$IDENTITY_DIR/node-${node_id}-identity.json"
  local identity_hex
  local served_epoch
  local retained_epoch
  identity_hex="$(jq -r '.remote_index_identity_hex' "$identity_file")"
  served_epoch="$(jq -r '.last_served_epoch' "$identity_file")"
  retained_epoch="$(jq -r '.min_retained_epoch' "$identity_file")"
  "${coord_psql[@]}" \
    -v node_id="$node_id" \
    -v secret_name="$secret_name" \
    -v identity_hex="$identity_hex" \
    -v served_epoch="$served_epoch" \
    -v retained_epoch="$retained_epoch" \
    -v extversion="$extversion" <<'SQL' >/dev/null
SELECT ec_spire_register_remote_node_descriptor(
    'ec_spire_phase13e_coord_idx'::regclass::oid,
    :node_id::int,
    1301,
    :'secret_name',
    decode(:'identity_hex', 'hex'),
    'ec_spire_phase13e_remote_idx',
    'active',
    :served_epoch::bigint,
    :retained_epoch::bigint,
    :'extversion',
    'none'
);
SQL
}

register_remote 2
register_remote 3
register_remote 4

placement_summary="$("${coord_psql[@]}" -At -F '|' <<'SQL'
SELECT string_agg(node_id::text || ':' || placement_count::text, ',' ORDER BY node_id)
FROM ec_spire_remote_node_snapshot('ec_spire_phase13e_coord_idx'::regclass)
WHERE node_id IN (2, 3, 4);
SQL
)"
profile_summary="$(PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off" "${coord_psql[@]}" -At -F '|' <<'SQL'
WITH profile AS (
    SELECT metric, value
    FROM ec_spire_remote_search_production_read_profile(
        'ec_spire_phase13e_coord_idx'::regclass::oid,
        ARRAY[1.0, 0.0]::real[],
        6
    )
)
SELECT max(value) FILTER (WHERE metric = 'status') || '|'
       || max(value) FILTER (WHERE metric = 'dispatch_count') || '|'
       || max(value) FILTER (WHERE metric = 'socket_open_count') || '|'
       || max(value) FILTER (WHERE metric = 'candidate_receive_query_count') || '|'
       || max(value) FILTER (WHERE metric = 'heap_receive_query_count') || '|'
       || max(value) FILTER (WHERE metric = 'returned_candidate_count')
FROM profile;
SQL
)"
plan="$(PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off" "${coord_psql[@]}" -At <<'SQL'
EXPLAIN (COSTS OFF)
SELECT id, title
FROM ec_spire_phase13e_coord_sql
ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[]
LIMIT 6;
SQL
)"
read_rows="$(PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off" "${coord_psql[@]}" -At -F ',' <<'SQL'
SELECT id, title
FROM ec_spire_phase13e_coord_sql
ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[]
LIMIT 6;
SQL
)"

IFS='|' read -r profile_status profile_dispatch_count profile_socket_count \
  profile_candidate_count profile_heap_count profile_returned_count \
  <<< "$profile_summary"

echo "placement_summary=$placement_summary"
echo "profile_summary=$profile_summary"
echo "plan=$plan"
echo "read_rows=$read_rows"

[[ "$placement_summary" == *"2:"* ]]
[[ "$placement_summary" == *"3:"* ]]
[[ "$placement_summary" == *"4:"* ]]
[[ "$plan" == *"Custom Scan (EcSpireDistributedScan)"* ]]
[[ "$profile_status" == "ready" ]]
[[ "$profile_dispatch_count" == "3" ]]
[[ "$profile_socket_count" == "3" ]]
[[ "$profile_candidate_count" == "3" ]]
[[ "$profile_heap_count" == "3" ]]
[[ "$profile_returned_count" != "0" ]]
[[ "$read_rows" == *"doc "* ]]

echo "SPIRE Phase 13e static remote placement PG18 fixture passed"
