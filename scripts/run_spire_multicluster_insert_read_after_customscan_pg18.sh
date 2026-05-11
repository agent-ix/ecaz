#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PGBIN="${PGBIN:-/home/peter/.pgrx/18.3/pgrx-install/bin}"
PG_CTL="${PG_CTL:-$PGBIN/pg_ctl}"
PSQL="${PSQL:-$PGBIN/psql}"
REMOTE_PORT="${REMOTE_PORT:-39238}"
COORD_PORT="${COORD_PORT:-39239}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR_OVERRIDE="${RUN_DIR:-}"
LOG_DIR_OVERRIDE="${LOG_DIR:-}"
SMOKE_LOG="${SMOKE_LOG:-}"
INSERT_MODE="${INSERT_MODE:-helper}"
ARTIFACT_DIR=""

usage() {
  cat <<'USAGE'
Usage: scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh [options]

Options:
  --artifact-dir DIR  Store smoke and PostgreSQL logs in DIR.
  --coord-port PORT   Coordinator PostgreSQL port. Default: 39239.
  --insert-mode MODE  Insert path to exercise: helper or trigger. Default: helper.
  --log-dir DIR       Store PostgreSQL logs in DIR.
  --pgbin DIR         PostgreSQL bin directory. Default: $PGBIN.
  --remote-port PORT  Remote PostgreSQL port. Default: 39238.
  --run-dir DIR       Run directory. Default: target/spire-insert-read-after-cscan-pg18-$RUN_ID.
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
    --insert-mode)
      INSERT_MODE="$2"
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

if [[ "$INSERT_MODE" != "helper" && "$INSERT_MODE" != "trigger" ]]; then
  echo "unsupported --insert-mode: $INSERT_MODE" >&2
  usage >&2
  exit 2
fi

RUN_DIR="${RUN_DIR_OVERRIDE:-$ROOT_DIR/target/spire-insert-read-after-cscan-pg18-$RUN_ID}"
if [[ -n "$ARTIFACT_DIR" ]]; then
  LOG_DIR="$ARTIFACT_DIR"
  SMOKE_LOG="${SMOKE_LOG:-$ARTIFACT_DIR/multicluster-insert-read-after-customscan.log}"
else
  LOG_DIR="${LOG_DIR_OVERRIDE:-$RUN_DIR/logs}"
fi
REMOTE_DATA="$RUN_DIR/remote"
COORD_DATA="$RUN_DIR/coord"
SOCKET_DIR="$RUN_DIR/s"

if [[ -n "$SMOKE_LOG" && "${ECAZ_SPIRE_INSERT_READ_AFTER_LOG_ACTIVE:-0}" != "1" ]]; then
  mkdir -p "${SMOKE_LOG%/*}"
  export ECAZ_SPIRE_INSERT_READ_AFTER_LOG_ACTIVE=1
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
echo "insert_mode=$INSERT_MODE"

if [[ "${ECAZ_SKIP_INSTALL:-0}" != "1" ]]; then
  (cd "$ROOT_DIR" && cargo pgrx install --test --pg-config "$PGBIN/pg_config" \
    --features "pg18 pg_test" --no-default-features)
fi

"$PG_CTL" initdb -D "$REMOTE_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$COORD_DATA" -o "-A trust -U postgres" >/dev/null

export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_INSERT_READ_AFTER_CUSTOMSCAN="host=$SOCKET_DIR port=$REMOTE_PORT dbname=postgres user=postgres connect_timeout=1"

"$PG_CTL" -w -D "$REMOTE_DATA" -l "$LOG_DIR/remote-postgres.log" \
  -o "-p $REMOTE_PORT -k $SOCKET_DIR -c listen_addresses='' -c max_prepared_transactions=10" start >/dev/null
"$PG_CTL" -w -D "$COORD_DATA" -l "$LOG_DIR/coord-postgres.log" \
  -o "-p $COORD_PORT -k $SOCKET_DIR -c listen_addresses='' -c max_prepared_transactions=10" start >/dev/null

remote_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$REMOTE_PORT" -U postgres -d postgres)
coord_psql=("$PSQL" -v ON_ERROR_STOP=1 -h "$SOCKET_DIR" -p "$COORD_PORT" -U postgres -d postgres)

"${remote_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null
"${coord_psql[@]}" -c "CREATE EXTENSION ecaz" >/dev/null

"${remote_psql[@]}" <<'SQL' >/dev/null
CREATE TABLE ec_spire_insert_read_remote_sql
    (id bigint primary key, title text not null, embedding ecvector, source_identity bytea);
INSERT INTO ec_spire_insert_read_remote_sql (id, title, embedding, source_identity) VALUES
    (10, 'remote seed positive', encode_to_ecvector(ARRAY[0.2, 0.8], 4, 42),
     decode('000102030405060708090a0b0c0d0e0f', 'hex')),
    (20, 'remote seed negative', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42),
     decode('101112131415161718191a1b1c1d1e1f', 'hex'));
CREATE INDEX ec_spire_insert_read_remote_idx
    ON ec_spire_insert_read_remote_sql USING ec_spire
    (embedding ecvector_spire_ip_ops)
    WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq');
SQL

"${coord_psql[@]}" <<'SQL' >/dev/null
CREATE TABLE ec_spire_insert_read_coord_sql
    (id bigint primary key, title text not null, embedding ecvector, source_identity bytea);
INSERT INTO ec_spire_insert_read_coord_sql (id, title, embedding, source_identity) VALUES
    (1, 'coordinator positive', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42),
     decode('202122232425262728292a2b2c2d2e2f', 'hex')),
    (2, 'coordinator mixed', encode_to_ecvector(ARRAY[0.8, 0.2], 4, 42),
     decode('303132333435363738393a3b3c3d3e3f', 'hex')),
    (3, 'coordinator negative', encode_to_ecvector(ARRAY[-1.0, 0.0], 4, 42),
     decode('404142434445464748494a4b4c4d4e4f', 'hex')),
    (4, 'coordinator other', encode_to_ecvector(ARRAY[-0.8, 0.2], 4, 42),
     decode('505152535455565758595a5b5c5d5e5f', 'hex'));
CREATE INDEX ec_spire_insert_read_coord_idx
    ON ec_spire_insert_read_coord_sql USING ec_spire
    (embedding ecvector_spire_ip_ops)
    WITH (nlists = 2, nprobe = 2, storage_format = 'rabitq');
-- Advance the coordinator index to the epoch that the remote index will reach
-- after the routed INSERT. The helper classifies/stages placement at the
-- coordinator active epoch, and the remote endpoint accepts only its active
-- epoch during the subsequent CustomScan request.
INSERT INTO ec_spire_insert_read_coord_sql (id, title, embedding, source_identity) VALUES
    (999, 'coordinator epoch alignment row', encode_to_ecvector(ARRAY[-1.0, -1.0], 4, 42),
     decode('606162636465666768696a6b6c6d6e6f', 'hex'));
SQL

remote_epoch="$("${remote_psql[@]}" -At -c "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('ec_spire_insert_read_remote_idx'::regclass)")"
coord_epoch="$("${coord_psql[@]}" -At -c "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('ec_spire_insert_read_coord_idx'::regclass)")"
remote_pids="$("${remote_psql[@]}" -At -c "SELECT array_to_string(array_agg(leaf_pid ORDER BY leaf_pid), ',') FROM ec_spire_index_leaf_snapshot('ec_spire_insert_read_remote_idx'::regclass)")"
coord_pids="$("${coord_psql[@]}" -At -c "SELECT array_to_string(array_agg(leaf_pid ORDER BY leaf_pid), ',') FROM ec_spire_index_leaf_snapshot('ec_spire_insert_read_coord_idx'::regclass)")"
remote_identity_hex="$("${remote_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_insert_read_remote_idx'::regclass::oid)")"
extversion="$("${coord_psql[@]}" -At -c "SELECT extversion FROM pg_extension WHERE extname = 'ecaz'")"

if [[ "$coord_epoch" != "$((remote_epoch + 1))" ]]; then
  echo "epoch alignment mismatch remote=$remote_epoch coord=$coord_epoch" >&2
  exit 3
fi
if [[ "$remote_pids" != "$coord_pids" ]]; then
  echo "leaf pid mismatch remote=$remote_pids coord=$coord_pids" >&2
  exit 4
fi

"${coord_psql[@]}" -v coord_epoch="$coord_epoch" -v remote_epoch="$remote_epoch" \
  -v extversion="$extversion" \
  -v remote_identity_hex="$remote_identity_hex" <<'SQL' >/dev/null
WITH rewritten AS (
    SELECT tests.ec_spire_test_rewrite_placement_node(
        'ec_spire_insert_read_coord_idx'::regclass::oid,
        leaf_pid,
        2
    )
    FROM ec_spire_index_leaf_snapshot('ec_spire_insert_read_coord_idx'::regclass)
)
SELECT count(*) FROM rewritten;

SELECT ec_spire_register_remote_node_descriptor(
    'ec_spire_insert_read_coord_idx'::regclass,
    2,
    92,
    'spire/remote/insert/read_after_customscan',
    decode(:'remote_identity_hex', 'hex'),
    'ec_spire_insert_read_remote_idx',
    'active',
    :coord_epoch::bigint,
    :remote_epoch::bigint,
    :'extversion',
    'none'
);
SQL

if [[ "$INSERT_MODE" == "helper" ]]; then
  insert_result="$("${coord_psql[@]}" -At -F ',' <<'SQL'
SELECT node_id::text,
       status,
       next_step,
       placement_staged::text,
       remote_prepared::text
  FROM ec_spire_prepare_coordinator_insert_tuple_payload(
       'ec_spire_insert_read_coord_idx'::regclass,
       decode('0303', 'hex'),
       ARRAY[1.0, 0.0]::real[],
       decode('303132333435363738393a3b3c3d3e3f', 'hex'),
       jsonb_build_object(
           'id', 303,
           'title', 'remote inserted via coordinator',
           'embedding', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)::text
       ),
       ARRAY['id','title','embedding']::text[]
  );
SQL
  )"
  placement_pk_predicate="pk_value = decode('0303', 'hex')"
  expected_insert_result="2,remote_insert_prepared_pending_local_commit,await_local_commit,true,true"
  coordinator_row_count="not_applicable"
else
  "${coord_psql[@]}" <<'SQL' >/dev/null
SELECT ec_spire_enable_coordinator_insert(
    'ec_spire_insert_read_coord_sql'::regclass,
    'ec_spire_insert_read_coord_idx'::regclass,
    'id',
    'embedding',
    'source_identity'
);

INSERT INTO ec_spire_insert_read_coord_sql (id, title, embedding, source_identity) VALUES
    (303, 'remote inserted via coordinator',
     encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42),
     decode('303132333435363738393a3b3c3d3e3f', 'hex'));
SQL
  insert_result="trigger_insert_committed"
  placement_pk_predicate="pk_value = int8send(303::bigint)::bytea"
  expected_insert_result="trigger_insert_committed"
  coordinator_row_count="$("${coord_psql[@]}" -At -c "SELECT count(*) FROM ec_spire_insert_read_coord_sql WHERE id = 303")"
fi

remote_epoch_after="$("${remote_psql[@]}" -At -c "SELECT active_epoch FROM ec_spire_index_hierarchy_snapshot('ec_spire_insert_read_remote_idx'::regclass)")"
remote_identity_hex_after="$("${remote_psql[@]}" -At -c "SELECT profile_fingerprint FROM ec_spire_remote_search_endpoint_identity('ec_spire_insert_read_remote_idx'::regclass::oid)")"
if [[ "$remote_epoch_after" != "$coord_epoch" ]]; then
  echo "post-insert epoch mismatch remote_after=$remote_epoch_after coord=$coord_epoch" >&2
  exit 5
fi
descriptor_row="$("${coord_psql[@]}" -At -F ',' -c "SELECT descriptor_generation::text, last_served_epoch::text, min_retained_epoch::text, encode(remote_index_identity, 'hex') FROM ec_spire_remote_node_descriptor WHERE coordinator_index_oid = 'ec_spire_insert_read_coord_idx'::regclass AND node_id = 2")"

remote_row="$("${remote_psql[@]}" -At -F ',' -c "SELECT id, title FROM ec_spire_insert_read_remote_sql WHERE id = 303")"
placement_row="$("${coord_psql[@]}" -At -F ',' -c "SELECT node_id::text, centroid_id::text, served_epoch::text FROM ec_spire_placement WHERE index_oid = 'ec_spire_insert_read_coord_idx'::regclass AND $placement_pk_predicate")"
plan="$(PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off" "${coord_psql[@]}" -At -c "EXPLAIN (COSTS OFF) SELECT id, title FROM ec_spire_insert_read_coord_sql ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1")"
read_row="$(PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off" "${coord_psql[@]}" -At -F ',' -c "SELECT id, title FROM ec_spire_insert_read_coord_sql ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[] LIMIT 1")"

echo "remote_epoch=$remote_epoch"
echo "coord_epoch=$coord_epoch"
echo "remote_pids=$remote_pids"
echo "coord_pids=$coord_pids"
echo "remote_identity_hex=$remote_identity_hex"
echo "remote_epoch_after_insert=$remote_epoch_after"
echo "remote_identity_hex_after_insert=$remote_identity_hex_after"
echo "descriptor_row=$descriptor_row"
echo "insert_result=$insert_result"
echo "coordinator_row_count=$coordinator_row_count"
echo "remote_row=$remote_row"
echo "placement_row=$placement_row"
echo "plan=$plan"
echo "read_row=$read_row"

[[ "$insert_result" == "$expected_insert_result" ]]
if [[ "$INSERT_MODE" == "trigger" ]]; then
  [[ "$coordinator_row_count" == "0" ]]
fi
[[ "$descriptor_row" == "93,$coord_epoch,$coord_epoch,$remote_identity_hex_after" ]]
[[ "$remote_row" == "303,remote inserted via coordinator" ]]
[[ "$placement_row" == 2,*",$coord_epoch" ]]
[[ "$plan" == *"Custom Scan (EcSpireDistributedScan)"* ]]
[[ "$read_row" == "303,remote inserted via coordinator" ]]

echo "SPIRE multicluster coordinator insert read-after-CustomScan passed"
