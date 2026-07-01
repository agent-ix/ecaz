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
FIXTURE_ROWS="${FIXTURE_ROWS:-12}"
BENCH_TOP_K="${BENCH_TOP_K:-6}"
BENCH_QUERIES_LIMIT="${BENCH_QUERIES_LIMIT:-1}"
BENCH_SWEEP="${BENCH_SWEEP:-3}"
SLOW_CANDIDATE_NODE2_MS="${SLOW_CANDIDATE_NODE2_MS:-0}"

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
  --fixture-rows N     Coordinator fixture row count. Default: 12.
  --bench-top-k K      Top-k for the local ecaz bench suite gate. Default: 6.
  --bench-queries-limit N
                      Query count for the local ecaz bench suite gate. Default: 1.
  --bench-sweep LIST   Comma-separated nprobe sweep for the suite gate. Default: 3.
  --slow-candidate-node2-ms MS
                      Slow node 2 compact candidate receive through a search_path wrapper.
                      Default: 0.
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
    --fixture-rows)
      FIXTURE_ROWS="$2"
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
    --slow-candidate-node2-ms)
      SLOW_CANDIDATE_NODE2_MS="$2"
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
SOCKET_DIR="$ROOT_DIR/target/spire-phase13e-sockets-$RUN_ID"
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
echo "fixture_rows=$FIXTURE_ROWS"
echo "bench_top_k=$BENCH_TOP_K"
echo "bench_queries_limit=$BENCH_QUERIES_LIMIT"
echo "bench_sweep=$BENCH_SWEEP"
echo "slow_candidate_node2_ms=$SLOW_CANDIDATE_NODE2_MS"

if [[ "${ECAZ_SKIP_INSTALL:-0}" != "1" ]]; then
  (cd "$ROOT_DIR" && cargo pgrx install --test --pg-config "$PGBIN/pg_config" \
    --features "pg18 pg_test" --no-default-features)
fi

"$PG_CTL" initdb -D "$COORD_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$REMOTE1_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$REMOTE2_DATA" -o "-A trust -U postgres" >/dev/null
"$PG_CTL" initdb -D "$REMOTE3_DATA" -o "-A trust -U postgres" >/dev/null

node2_conninfo="host=$SOCKET_DIR port=$REMOTE1_PORT dbname=postgres user=postgres connect_timeout=1"
if [[ "$SLOW_CANDIDATE_NODE2_MS" != "0" ]]; then
  node2_conninfo="$node2_conninfo options='-c search_path=ec_spire_phase13e_slow_candidate,public'"
fi
export EC_SPIRE_REMOTE_CONNINFO_SPIRE_REMOTE_PHASE13E_NODE2="$node2_conninfo"
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

if [[ "$SLOW_CANDIDATE_NODE2_MS" != "0" ]]; then
  "${remote1_psql[@]}" -v slow_ms="$SLOW_CANDIDATE_NODE2_MS" <<'SQL' >/dev/null
CREATE SCHEMA ec_spire_phase13e_slow_candidate;
CREATE TABLE ec_spire_phase13e_slow_candidate.config (slow_ms double precision NOT NULL);
INSERT INTO ec_spire_phase13e_slow_candidate.config VALUES (:slow_ms::double precision);
CREATE FUNCTION ec_spire_phase13e_slow_candidate.ec_spire_remote_search(
    index_oid oid,
    requested_epoch bigint,
    query real[],
    selected_pids bigint[],
    top_k integer,
    consistency_mode text
) RETURNS TABLE (
    served_epoch bigint,
    node_id bigint,
    pid bigint,
    object_version bigint,
    row_index bigint,
    assignment_flags smallint,
    vec_id bytea,
    row_locator bytea,
    score real,
    protocol_version text,
    extension_version text,
    opclass_identity text,
    storage_format text,
    assignment_payload_format text,
    quantizer_profile text,
    scoring_profile text,
    profile_fingerprint text,
    endpoint_status text
)
LANGUAGE plpgsql
STABLE
AS $$
DECLARE
    delay_ms double precision;
BEGIN
    SELECT config.slow_ms
      INTO delay_ms
      FROM ec_spire_phase13e_slow_candidate.config
     LIMIT 1;
    PERFORM pg_sleep(delay_ms / 1000.0);
    RETURN QUERY
    SELECT *
      FROM public.ec_spire_remote_search(
          index_oid,
          requested_epoch,
          query,
          selected_pids,
          top_k,
          consistency_mode
      );
END;
$$;
SQL
fi

"${coord_psql[@]}" -v fixture_rows="$FIXTURE_ROWS" <<'SQL' >/dev/null
CREATE TABLE ec_spire_phase13e_coord_corpus
    (id bigint primary key, title text not null, embedding ecvector, source real[] not null);
INSERT INTO ec_spire_phase13e_coord_corpus (id, title, embedding, source)
SELECT id,
       format('doc %s', id),
       encode_to_ecvector(source, 4, 42),
       source
  FROM (
    SELECT id,
           CASE (id % 4)
             WHEN 1 THEN ARRAY[1.0, 0.0]::real[]
             WHEN 2 THEN ARRAY[0.6, 0.4]::real[]
             WHEN 3 THEN ARRAY[-1.0, 0.0]::real[]
             ELSE ARRAY[0.0, 1.0]::real[]
           END AS source
      FROM generate_series(1, :fixture_rows) AS id
  ) AS rows;
CREATE TABLE ec_spire_phase13e_coord_queries
    (id bigint primary key, source real[] not null);
INSERT INTO ec_spire_phase13e_coord_queries (id, source) VALUES
    (1, ARRAY[1.0, 0.0]::real[]);
CREATE INDEX ec_spire_phase13e_coord_idx
    ON ec_spire_phase13e_coord_corpus USING ec_spire
    (embedding ecvector_spire_ip_ops)
    WITH (nlists = 3, nprobe = 3, storage_format = 'rabitq');
SQL

materialize_remote_shard_mode() {
  local node_id="$1"
  local -n remote_psql_ref="$2"
  local consistency_mode="$3"
  local assignment_file="$ASSIGNMENT_DIR/node-${node_id}-assignments.tsv"
  local materialize_sql="$RUN_DIR/node-${node_id}-materialize-${consistency_mode}-rendered.sql"
  local assignment_file_sql="${assignment_file//\'/\'\'}"
  sed "s|__ECAZ_ASSIGNMENT_FILE__|$assignment_file_sql|g" \
    "$ROOT_DIR/scripts/spire-aws/materialize-remote-leaf-base-assignments.sql" \
    > "$materialize_sql"

  "${remote_psql_ref[@]}" \
    -v remote_index=ec_spire_phase13e_remote_idx \
    -v remote_table=ec_spire_phase13e_remote_sql \
    -v consistency_mode="$consistency_mode" \
    -f "$materialize_sql" \
    > "$LOG_DIR/node-${node_id}-materialize-${consistency_mode}.log"
}

write_remote_identity() {
  local node_id="$1"
  local -n remote_psql_ref="$2"

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

publish_remote_placements_from_assignment_files() {
  local consistency_mode="$1"
  local pids=()
  local node_ids=()
  local -A seen_leaf_pids=()

  for node_id in 2 3 4; do
    local assignment_file="$ASSIGNMENT_DIR/node-${node_id}-assignments.tsv"
    while IFS=$'\t' read -r _active_epoch leaf_pid _rest; do
      if [[ -z "${seen_leaf_pids[$leaf_pid]+x}" ]]; then
        seen_leaf_pids[$leaf_pid]=1
        pids+=("$leaf_pid")
        node_ids+=("$node_id")
      fi
    done < "$assignment_file"
  done

  if [[ "${#pids[@]}" -eq 0 ]]; then
    echo "no leaf pids available for static remote placement publish" >&2
    exit 3
  fi

  local pid_csv
  local node_id_csv
  pid_csv="$(IFS=,; echo "${pids[*]}")"
  node_id_csv="$(IFS=,; echo "${node_ids[*]}")"

  "${coord_psql[@]}" \
    -v pid_csv="$pid_csv" \
    -v node_id_csv="$node_id_csv" \
    -v consistency_mode="$consistency_mode" <<'SQL' >/dev/null
SELECT *
  FROM ec_spire_publish_static_remote_placement_nodes_with_mode(
       'ec_spire_phase13e_coord_idx'::regclass::oid,
       string_to_array(:'pid_csv', ',')::bigint[],
       string_to_array(:'node_id_csv', ',')::int[],
       :'consistency_mode'
  );
SQL
}

create_remote_shard() {
  local node_id="$1"
  local -n remote_psql_ref="$2"
  local assignment_file="$ASSIGNMENT_DIR/node-${node_id}-assignments.tsv"

  "${coord_psql[@]}" -At -F $'\t' \
    -v coord_index=ec_spire_phase13e_coord_idx \
    -v coord_table=ec_spire_phase13e_coord_corpus \
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
    (id bigint primary key, title text not null, embedding ecvector, source real[] not null);
INSERT INTO ec_spire_phase13e_remote_sql (id, title, embedding, source)
WITH shard_rows AS (
  SELECT DISTINCT row_id
  FROM ec_spire_phase13e_assignment_import
),
encoded AS (
SELECT row_id,
       format('doc %s', row_id) AS title,
       source,
       encode_to_ecvector(
         source,
         4,
         42
       ) AS embedding
  FROM (
    SELECT row_id,
           CASE (row_id % 4)
             WHEN 1 THEN ARRAY[1.0, 0.0]::real[]
             WHEN 2 THEN ARRAY[0.6, 0.4]::real[]
             WHEN 3 THEN ARRAY[-1.0, 0.0]::real[]
             ELSE ARRAY[0.0, 1.0]::real[]
           END AS source
      FROM shard_rows
  ) AS rows
)
SELECT row_id, title, embedding, source
  FROM encoded
 ORDER BY row_id;
CREATE INDEX ec_spire_phase13e_remote_idx
    ON ec_spire_phase13e_remote_sql USING ec_spire
    (embedding ecvector_spire_ip_ops)
    WITH (nlists = 3, nprobe = 3, storage_format = 'rabitq');
SQL

  materialize_remote_shard_mode "$node_id" "$2" strict
  write_remote_identity "$node_id" "$2"
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
  local descriptor_generation="${2:-1301}"
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
    -v descriptor_generation="$descriptor_generation" \
    -v secret_name="$secret_name" \
    -v identity_hex="$identity_hex" \
    -v served_epoch="$served_epoch" \
    -v retained_epoch="$retained_epoch" \
    -v extversion="$extversion" <<'SQL' >/dev/null
SELECT ec_spire_register_remote_node_descriptor(
    'ec_spire_phase13e_coord_idx'::regclass::oid,
    :node_id::int,
    :descriptor_generation::bigint,
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
FROM ec_spire_phase13e_coord_corpus
ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[]
LIMIT 6;
SQL
)"
read_rows="$(PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off" "${coord_psql[@]}" -At -F ',' <<'SQL'
SELECT id, title
FROM ec_spire_phase13e_coord_corpus
ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[]
LIMIT 6;
SQL
)"
exact_rows="$("${coord_psql[@]}" -At -F ',' <<'SQL'
SELECT id, title
FROM ec_spire_phase13e_coord_corpus
ORDER BY CASE (id % 4)
           WHEN 1 THEN 0
           WHEN 2 THEN 1
           WHEN 0 THEN 2
           ELSE 3
         END,
         id
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
echo "exact_rows=$exact_rows"

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
[[ "$read_rows" == "$exact_rows" ]]

suite_artifact_dir="$LOG_DIR/bench-suite"
suite_config="$suite_artifact_dir/phase13e-local-spire-pipeline-suite.json"
mkdir -p "$suite_artifact_dir"
remote_selected_pids="$(
  awk -F $'\t' '!seen[$2]++ { print $2 + 0 }' "$ASSIGNMENT_DIR"/node-*-assignments.tsv \
    | sort -n \
    | jq -Rsc 'split("\n") | map(select(length > 0) | tonumber)'
)"
if [[ -z "$remote_selected_pids" || "$remote_selected_pids" == "null" ]]; then
  echo "no remote selected pids available for bench suite gate" >&2
  exit 6
fi
bench_sweep_json="$(printf '%s' "$BENCH_SWEEP" | jq -R 'split(",") | map(tonumber)')"
jq -n \
  --arg artifact_dir "$suite_artifact_dir" \
  --arg log_output "$suite_artifact_dir/spire-pipeline.log" \
  --argjson bench_sweep "$bench_sweep_json" \
  --argjson bench_top_k "$BENCH_TOP_K" \
  --argjson bench_queries_limit "$BENCH_QUERIES_LIMIT" \
  --argjson remote_selected_pids "$remote_selected_pids" \
  '{
    name: "phase13e-local-spire-pipeline",
    schema_version: 1,
    artifact_dir: $artifact_dir,
    defaults: {
      queries_limit: $bench_queries_limit,
      pg: 18
    },
    steps: [
      {
        kind: "spire-pipeline",
        name: "phase13e-local-spire-pipeline-read",
        tags: ["phase13e", "local", "production-read"],
        prefix: "ec_spire_phase13e_coord",
        index: "ec_spire_phase13e_coord_idx",
        queries_limit: $bench_queries_limit,
        sweep: $bench_sweep,
        include_remote: true,
        require_remote_placements: true,
        include_query_metrics: true,
        include_recall: true,
        include_production_read_profile: true,
        production_read_only: true,
        query_metric_k: $bench_top_k,
        top_k: $bench_top_k,
        consistency_mode: "strict",
        remote_tuple_transport: "pg_binary_attr_v1",
        remote_selected_pids: $remote_selected_pids,
        log_output: $log_output
      }
    ]
  }' > "$suite_config"
ECAZ_BIN="${ECAZ_BIN:-$ROOT_DIR/target/release/ecaz}"
if [[ ! -x "$ECAZ_BIN" ]]; then
  ECAZ_BIN="ecaz"
fi
"$ECAZ_BIN" \
  --host "$SOCKET_DIR" --port "$COORD_PORT" --user postgres --database postgres \
  bench suite run --config "$suite_config" \
  --manifest-output "$suite_artifact_dir/suite-manifest.json" \
  --results-output "$suite_artifact_dir/results.jsonl" \
  > "$suite_artifact_dir/suite-run.log" 2>&1
echo "bench_suite_summary=passed|$suite_config|$suite_artifact_dir/suite-manifest.json|$suite_artifact_dir/results.jsonl"

slow_lock_log="$LOG_DIR/slow-remote-node2-lock.log"
timeline_log="$LOG_DIR/production-read-timeline.tsv"
(
  "${remote1_psql[@]}" <<'SQL'
BEGIN;
LOCK TABLE ec_spire_phase13e_remote_sql IN ACCESS EXCLUSIVE MODE;
SELECT pg_sleep(0.75);
COMMIT;
SQL
) >"$slow_lock_log" 2>&1 &
slow_lock_pid=$!
sleep 0.1
PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off" \
  "${coord_psql[@]}" -At -F '|' <<'SQL' >"$timeline_log"
SELECT requested_epoch,
       node_id,
       phase,
       started_after_ms,
       completed_after_ms,
       elapsed_ms,
       candidate_count,
       status,
       failure_category
FROM ec_spire_remote_search_production_read_timeline(
    'ec_spire_phase13e_coord_idx'::regclass::oid,
    ARRAY[1.0, 0.0]::real[],
    6,
    ARRAY['id', 'title']::text[]
)
ORDER BY phase, node_id;
SQL
wait "$slow_lock_pid"

timeline_summary="$(awk -F '|' '
BEGIN {
  candidate_count = 0;
  heap_count = 0;
  slow_heap_completed = -1;
  fastest_heap_completed = -1;
  slow_candidate_completed = -1;
  earliest_heap_started = -1;
  bad_status_count = 0;
}
$3 == "candidate_receive" {
  candidate_count++;
  if ($8 != "ready") {
    bad_status_count++;
  }
  if ($2 == "2") {
    slow_candidate_completed = $5 + 0;
  }
}
$3 == "heap_receive" {
  heap_count++;
  if ($8 != "ready") {
    bad_status_count++;
  }
  if (earliest_heap_started < 0 || ($4 + 0) < earliest_heap_started) {
    earliest_heap_started = $4 + 0;
  }
  if ($2 == "2") {
    slow_heap_completed = $5 + 0;
  } else if (fastest_heap_completed < 0 || ($5 + 0) < fastest_heap_completed) {
    fastest_heap_completed = $5 + 0;
  }
}
END {
  heap_started_before_slow_candidate = (earliest_heap_started >= 0 && slow_candidate_completed >= 0 && earliest_heap_started < slow_candidate_completed) ? 1 : 0;
  print candidate_count "|" heap_count "|" slow_heap_completed "|" fastest_heap_completed "|" bad_status_count "|" slow_candidate_completed "|" earliest_heap_started "|" heap_started_before_slow_candidate;
}
' "$timeline_log")"
timeline_rows="$(tr '\n' ';' < "$timeline_log")"
IFS='|' read -r timeline_candidate_count timeline_heap_count \
  timeline_slow_heap_completed timeline_fastest_heap_completed timeline_bad_status_count \
  timeline_slow_candidate_completed timeline_earliest_heap_started timeline_heap_before_slow_candidate \
  <<< "$timeline_summary"

echo "production_timeline_rows=$timeline_rows"
echo "production_timeline_summary=$timeline_summary"

[[ "$timeline_candidate_count" == "3" ]]
[[ "$timeline_heap_count" == "3" ]]
[[ "$timeline_bad_status_count" == "0" ]]
[[ "$timeline_slow_heap_completed" -ge 250 ]]
[[ "$timeline_fastest_heap_completed" -ge 0 ]]
[[ "$timeline_fastest_heap_completed" -lt "$timeline_slow_heap_completed" ]]
if [[ "$SLOW_CANDIDATE_NODE2_MS" != "0" ]]; then
  [[ "$timeline_slow_candidate_completed" -ge "$SLOW_CANDIDATE_NODE2_MS" ]]
  [[ "$timeline_heap_before_slow_candidate" == "1" ]]
fi

echo "stopping_remote_node=2"
"$PG_CTL" -D "$REMOTE1_DATA" -m fast stop >/dev/null

strict_failure_log="$LOG_DIR/strict-remote-node2-failure.log"
set +e
PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off -c ec_spire.remote_search_consistency_mode=strict" \
  "${coord_psql[@]}" -At -F ',' <<'SQL' >"$strict_failure_log" 2>&1
SELECT id, title
FROM ec_spire_phase13e_coord_corpus
ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[]
LIMIT 6;
SQL
strict_failure_exit_code=$?
set -e
strict_failure_text="$(tr '\n' ' ' < "$strict_failure_log")"
echo "strict_remote_failure_exit_code=$strict_failure_exit_code"
echo "strict_remote_failure_text=$strict_failure_text"
[[ "$strict_failure_exit_code" -ne 0 ]]
[[ "$strict_failure_text" == *"node_id 2"* ]]

echo "republishing_degraded_epoch=1"
materialize_remote_shard_mode 3 remote2_psql degraded
materialize_remote_shard_mode 4 remote3_psql degraded
write_remote_identity 3 remote2_psql
write_remote_identity 4 remote3_psql
register_remote 3 1302
register_remote 4 1302
publish_remote_placements_from_assignment_files degraded

degraded_profile_summary="$(PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off -c ec_spire.remote_search_consistency_mode=degraded" "${coord_psql[@]}" -At -F '|' <<'SQL'
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
       || max(value) FILTER (WHERE metric = 'degraded_skipped_dispatch_count') || '|'
       || max(value) FILTER (WHERE metric = 'remote_timeout_count') || '|'
       || max(value) FILTER (WHERE metric = 'remote_cancel_count') || '|'
       || max(value) FILTER (WHERE metric = 'returned_candidate_count') || '|'
       || max(value) FILTER (WHERE metric = 'next_blocker')
FROM profile;
SQL
)"
degraded_rows="$(PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off -c ec_spire.remote_search_consistency_mode=degraded" "${coord_psql[@]}" -At -F ',' <<'SQL'
SELECT id, title
FROM ec_spire_phase13e_coord_corpus
ORDER BY embedding <#> ARRAY[1.0, 0.0]::real[]
LIMIT 6;
SQL
)"

IFS='|' read -r degraded_status degraded_dispatch_count degraded_socket_count \
  degraded_candidate_count degraded_heap_count degraded_skipped_count \
  degraded_timeout_count degraded_cancel_count degraded_returned_count \
  degraded_next_blocker <<< "$degraded_profile_summary"

echo "degraded_profile_summary=$degraded_profile_summary"
echo "degraded_rows=$degraded_rows"

[[ "$degraded_status" == "degraded_ready" ]]
[[ "$degraded_dispatch_count" == "3" ]]
[[ "$degraded_socket_count" == "2" ]]
[[ "$degraded_candidate_count" == "2" ]]
[[ "$degraded_heap_count" == "2" ]]
[[ "$degraded_skipped_count" == "1" ]]
[[ "$degraded_timeout_count" == "0" ]]
[[ "$degraded_cancel_count" == "0" ]]
[[ "$degraded_returned_count" != "0" ]]
[[ "$degraded_next_blocker" == "none" ]]
[[ "$degraded_rows" == *"doc "* ]]

echo "SPIRE Phase 13e static remote placement PG18 fixture passed"
