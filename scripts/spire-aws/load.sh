#!/usr/bin/env bash
# Phase 13b.7 — load a dataset tier onto the coordinator.
#
# Tiers:
#   correctness     synthetic 10k via `ecaz corpus generate`
#   representative  qdrant-dbpedia 1M via `ecaz corpus fetch`/`prepare`
#   stress          synthetic 10M (reviewer-gated, see Phase 13a.9)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

TIER="${1:?tier required (correctness|representative|stress)}"
TOPOLOGY="${2:?topology JSON path required}"
ARTIFACT_DIR="${3:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

COORD_HOST=$(jq -r '.coordinator.operator_host // .coordinator.private_ip' "$TOPOLOGY")
COORD_PORT=$(jq -r '.coordinator.operator_port // 5432' "$TOPOLOGY")
REGION=$(jq -r '.region' "$TOPOLOGY")
BUCKET=$(jq -r '.artifact_bucket' "$TOPOLOGY")
ECAZ_BIN="${ECAZ_BIN:-ecaz}"
WORK_DIR="${WORK_DIR:-$ARTIFACT_DIR/work}"
SPIRE_AWS_RESET_COORDINATOR_INDEX="${SPIRE_AWS_RESET_COORDINATOR_INDEX:-1}"
SPIRE_AWS_STORAGE_FORMAT="${SPIRE_AWS_STORAGE_FORMAT:-rabitq}"
SPIRE_AWS_NODE_LOAD_BASE_DIR="${SPIRE_AWS_NODE_LOAD_BASE_DIR:-/var/tmp/ecaz-spire-aws-load}"
mkdir -p "$WORK_DIR"

ssm_wait_command() {
  local instance_id="$1"
  local cmd_id="$2"
  local log_file="$3"
  local timeout="${SPIRE_AWS_SSM_TIMEOUT_SECONDS:-3600}"
  local deadline=$((SECONDS + timeout))
  local status="Pending"

  while (( SECONDS < deadline )); do
    status=$(aws ssm get-command-invocation \
      --region "$REGION" \
      --command-id "$cmd_id" \
      --instance-id "$instance_id" \
      --query Status \
      --output text 2>/dev/null || echo "Pending")
    case "$status" in
      Success)
        aws ssm get-command-invocation \
          --region "$REGION" \
          --command-id "$cmd_id" \
          --instance-id "$instance_id" \
          > "$log_file"
        return 0
        ;;
      Failed|Cancelled|Cancelling|TimedOut)
        aws ssm get-command-invocation \
          --region "$REGION" \
          --command-id "$cmd_id" \
          --instance-id "$instance_id" \
          > "$log_file" || true
        return 1
        ;;
      *)
        sleep 15
        ;;
    esac
  done

  aws ssm get-command-invocation \
    --region "$REGION" \
    --command-id "$cmd_id" \
    --instance-id "$instance_id" \
    > "$log_file" || true
  echo "SSM command ${cmd_id} timed out for ${instance_id}" >&2
  return 124
}

ssm_run_shell() {
  local instance_id="$1"
  local comment="$2"
  local log_file="$3"
  local commands_json="$4"
  local parameters_json
  local cmd_id

  parameters_json=$(jq -cn --argjson commands "$commands_json" '{commands: $commands}')
  cmd_id=$(aws ssm send-command \
    --region "$REGION" \
    --document-name "AWS-RunShellScript" \
    --instance-ids "$instance_id" \
    --parameters "$parameters_json" \
    --output-s3-bucket-name "$BUCKET" \
    --output-s3-key-prefix "spire-aws/load-ssm" \
    --comment "$comment" \
    --query "Command.CommandId" \
    --output text)
  echo "${comment} ssm command id: ${cmd_id}" | tee -a "$ARTIFACT_DIR/load-${TIER}.ssm.log"
  ssm_wait_command "$instance_id" "$cmd_id" "$log_file"
}

upload_load_file() {
  local local_path="$1"
  local s3_key="$2"
  local log_file="$3"

  aws s3 cp "$local_path" "s3://${BUCKET}/${s3_key}" --region "$REGION" > "$log_file"
}

download_optional_s3_log() {
  local s3_key="$1"
  local local_path="$2"

  aws s3 cp "s3://${BUCKET}/${s3_key}" "$local_path" --region "$REGION" >/dev/null 2>&1 || true
}

restart_operator_tunnel_if_available() {
  local label="$1"
  local instance_id="$2"
  local port="$3"
  local restart_command="${SPIRE_AWS_TUNNEL_RESTART_COMMAND:-}"
  local log="$ARTIFACT_DIR/tunnel-restart-after-node-local-load.log"

  if [[ -z "$restart_command" ]]; then
    return
  fi
  if [[ ! -x "$restart_command" ]]; then
    echo "configured tunnel restart command is not executable: ${restart_command}" | tee -a "$log" >&2
    return 2
  fi
  "$restart_command" "$label" "$instance_id" "$port" "$ARTIFACT_DIR" >> "$log" 2>&1
}

restart_all_operator_tunnels_if_available() {
  local coord_id coord_port
  coord_id=$(jq -r '.coordinator.instance_id' "$TOPOLOGY")
  coord_port=$(jq -r '.coordinator.operator_port // 5432' "$TOPOLOGY")
  restart_operator_tunnel_if_available coordinator "$coord_id" "$coord_port"

  jq -c '.remotes[]' "$TOPOLOGY" | while read -r remote; do
    local node_id instance_id port
    node_id=$(jq -r '.node_id' <<< "$remote")
    instance_id=$(jq -r '.instance_id' <<< "$remote")
    port=$(jq -r '.operator_port // 5432' <<< "$remote")
    restart_operator_tunnel_if_available "remote-${node_id}" "$instance_id" "$port"
  done
}

write_distributed_placement_config() {
  local remote_count
  local remotes
  remote_count=$(jq '.remotes | length' "$TOPOLOGY")
  if [[ "$remote_count" -lt 1 ]]; then
    echo "topology has no remotes" >&2
    exit 2
  fi

  mkdir -p "$DISTRIBUTED_OUTPUT_DIR"
  remotes=$(jq \
    --arg remote_index "$REMOTE_INDEX" \
    '[.remotes | to_entries[] | {
      node_id: .value.node_id,
      conninfo_secret_name: .value.secret_name,
      remote_index_regclass: $remote_index,
      shard_ids: [.key]
    }]' "$TOPOLOGY")

  jq -n \
    --arg coord_index "$COORD_INDEX" \
    --argjson remotes "$remotes" \
    --argjson shard_count "$remote_count" \
    '{
      version: 1,
      coordinator: { index_name: $coord_index },
      remotes: $remotes,
      shard_policy: {
        kind: "hash_source_identity",
        shard_count: $shard_count,
        source_identity_column: "id"
      }
    }' > "$PLACEMENT_CONFIG"
}

reset_coordinator_index_if_requested() {
  if [[ "$SPIRE_AWS_RESET_COORDINATOR_INDEX" != "1" ]]; then
    return
  fi

  "$ECAZ_BIN" dev sql \
    --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
    --sql "DROP INDEX IF EXISTS ${COORD_INDEX};" \
    --log-output "$ARTIFACT_DIR/coordinator-reset-index-${TIER}.log"
}

load_remote_shards() {
  jq -c '.remotes[]' "$PLAN_FILE" | while read -r remote_plan; do
    local node_id
    local remote_host
    local remote_prefix
    local log_prefix
    node_id=$(jq -r '.node_id' <<< "$remote_plan")
    remote_host=$(jq -r --argjson node_id "$node_id" \
      '.remotes[] | select(.node_id == $node_id) | (.operator_host // .private_ip)' "$TOPOLOGY")
    remote_port=$(jq -r --argjson node_id "$node_id" \
      '.remotes[] | select(.node_id == $node_id) | (.operator_port // 5432)' "$TOPOLOGY")
    remote_prefix=$(jq -r '.remote_prefix' <<< "$remote_plan")
    log_prefix="$ARTIFACT_DIR/remote-node-${node_id}"

    if [[ -z "$remote_host" || "$remote_host" == "null" ]]; then
      echo "topology does not contain remote node_id=${node_id}" >&2
      exit 2
    fi

    mapfile -t remote_load_args < <(jq -r '.remote_load_args[]' <<< "$remote_plan")
    if [[ "${remote_load_args[0]:-}" == "ecaz" && "${remote_load_args[1]:-}" == "corpus" && "${remote_load_args[2]:-}" == "load" ]]; then
      remote_load_args=("${remote_load_args[@]:3}")
    fi
    "$ECAZ_BIN" dev sql \
      --host "$remote_host" --port "$remote_port" --user ecaz_coord --database postgres \
      --sql "DROP TABLE IF EXISTS ${remote_prefix}_queries CASCADE; DROP TABLE IF EXISTS ${remote_prefix}_corpus CASCADE;" \
      --log-output "${log_prefix}-drop-${TIER}.log"
    "$ECAZ_BIN" corpus load \
      --host "$remote_host" --port "$remote_port" --user ecaz_coord --database postgres \
      "${remote_load_args[@]}" \
      --log-file "${log_prefix}-load-${TIER}.log"

    "$ECAZ_BIN" corpus inspect \
      --host "$remote_host" --port "$remote_port" --user ecaz_coord --database postgres \
      --prefix "$remote_prefix" \
      --log-file "${log_prefix}-inspect-${TIER}.log"
  done
}

load_remote_shards_node_local() {
  jq -c '.remotes[]' "$PLAN_FILE" | while read -r remote_plan; do
    local node_id instance_id remote_prefix remote_index corpus_file
    local node_dir remote_corpus_key node_corpus_path node_load_log node_inspect_log
    local load_log_key inspect_log_key drop_log_key commands_json log_prefix
    local drop_sql
    node_id=$(jq -r '.node_id' <<< "$remote_plan")
    instance_id=$(jq -r --argjson node_id "$node_id" \
      '.remotes[] | select(.node_id == $node_id) | .instance_id' "$TOPOLOGY")
    remote_prefix=$(jq -r '.remote_prefix' <<< "$remote_plan")
    remote_index=$(jq -r '.remote_index_regclass' <<< "$remote_plan")
    corpus_file=$(jq -r '.corpus_file' <<< "$remote_plan")
    node_dir="${SPIRE_AWS_NODE_LOAD_BASE_DIR}/${TIER}/node-${node_id}"
    node_corpus_path="${node_dir}/$(basename "$corpus_file")"
    node_load_log="${node_dir}/load.log"
    node_inspect_log="${node_dir}/inspect.log"
    remote_corpus_key="representative-load/${TIER}/node-${node_id}/$(basename "$corpus_file")"
    load_log_key="representative-load/${TIER}/node-${node_id}/load.log"
    inspect_log_key="representative-load/${TIER}/node-${node_id}/inspect.log"
    drop_log_key="representative-load/${TIER}/node-${node_id}/drop.log"
    log_prefix="$ARTIFACT_DIR/remote-node-${node_id}"
    drop_sql="DROP TABLE IF EXISTS ${remote_prefix}_queries CASCADE; DROP TABLE IF EXISTS ${remote_prefix}_corpus CASCADE;"

    upload_load_file "$corpus_file" "$remote_corpus_key" "${log_prefix}-upload-${TIER}.log"

    commands_json=$(jq -cn \
      --arg bucket "$BUCKET" \
      --arg region "$REGION" \
      --arg corpus_key "$remote_corpus_key" \
      --arg node_dir "$node_dir" \
      --arg node_corpus_path "$node_corpus_path" \
      --arg node_load_log "$node_load_log" \
      --arg node_inspect_log "$node_inspect_log" \
      --arg load_log_key "$load_log_key" \
      --arg inspect_log_key "$inspect_log_key" \
      --arg drop_log_key "$drop_log_key" \
      --arg remote_prefix "$remote_prefix" \
      --arg remote_index "$remote_index" \
      --arg drop_sql "$drop_sql" \
      --arg storage_format "$SPIRE_AWS_STORAGE_FORMAT" \
      '[
        "set -euo pipefail",
        "command -v /usr/local/bin/ecaz >/dev/null",
        "PG_BIN=/usr/pgsql-18/bin; if [ ! -x \"$PG_BIN/psql\" ]; then PG_BIN=$(dirname \"$(command -v psql)\"); fi",
        "rm -rf \($node_dir)",
        "mkdir -p \($node_dir)",
        "df -h \($node_dir)",
        "aws s3 cp s3://\($bucket)/\($corpus_key) \($node_corpus_path) --region \($region)",
        "set +e",
        "printf '\''%s\\n'\'' \($drop_sql|@sh) > \($node_dir)/drop.sql",
        "\"$PG_BIN/psql\" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres -f \($node_dir)/drop.sql > \($node_dir)/drop.log 2>&1",
        "drop_status=$?",
        "aws s3 cp \($node_dir)/drop.log s3://\($bucket)/\($drop_log_key) --region \($region) || true",
        "if [ \"$drop_status\" -ne 0 ]; then exit \"$drop_status\"; fi",
        "/usr/local/bin/ecaz corpus load --host 127.0.0.1 --port 5432 --user ecaz_coord --database postgres --profile ec_spire --prefix \($remote_prefix) --dim 1536 --bits 4 --seed 42 --corpus-file \($node_corpus_path) --corpus-only --storage-format \($storage_format) --index-name \($remote_index) --log-file \($node_load_log)",
        "load_status=$?",
        "aws s3 cp \($node_load_log) s3://\($bucket)/\($load_log_key) --region \($region) || true",
        "if [ \"$load_status\" -ne 0 ]; then exit \"$load_status\"; fi",
        "/usr/local/bin/ecaz corpus inspect --host 127.0.0.1 --port 5432 --user ecaz_coord --database postgres --prefix \($remote_prefix) --log-file \($node_inspect_log)",
        "inspect_status=$?",
        "aws s3 cp \($node_inspect_log) s3://\($bucket)/\($inspect_log_key) --region \($region) || true",
        "exit \"$inspect_status\""
      ]')
    ssm_run_shell "$instance_id" "ecaz Phase 13e node-local remote load ${TIER} node ${node_id}" \
      "${log_prefix}-load-${TIER}.ssm.json" "$commands_json"
    download_optional_s3_log "$drop_log_key" "${log_prefix}-drop-${TIER}.log"
    download_optional_s3_log "$load_log_key" "${log_prefix}-load-${TIER}.log"
    download_optional_s3_log "$inspect_log_key" "${log_prefix}-inspect-${TIER}.log"
  done
}

load_coordinator_representative_node_local() {
  local corpus_file="$1"
  local queries_file="$2"
  local manifest_file="$3"
  local coord_id node_dir node_corpus_path node_queries_path node_manifest_path
  local corpus_key queries_key manifest_key load_log_key reset_log_key commands_json reset_sql

  coord_id=$(jq -r '.coordinator.instance_id' "$TOPOLOGY")
  node_dir="${SPIRE_AWS_NODE_LOAD_BASE_DIR}/${TIER}/coordinator"
  node_corpus_path="${node_dir}/$(basename "$corpus_file")"
  node_queries_path="${node_dir}/$(basename "$queries_file")"
  node_manifest_path="${node_dir}/$(basename "$manifest_file")"
  corpus_key="representative-load/${TIER}/coordinator/$(basename "$corpus_file")"
  queries_key="representative-load/${TIER}/coordinator/$(basename "$queries_file")"
  manifest_key="representative-load/${TIER}/coordinator/$(basename "$manifest_file")"
  load_log_key="representative-load/${TIER}/coordinator/load.log"
  reset_log_key="representative-load/${TIER}/coordinator/reset-index.log"
  reset_sql="DROP INDEX IF EXISTS ${COORD_INDEX};"

  upload_load_file "$corpus_file" "$corpus_key" "$ARTIFACT_DIR/coordinator-corpus-upload-${TIER}.log"
  upload_load_file "$queries_file" "$queries_key" "$ARTIFACT_DIR/coordinator-queries-upload-${TIER}.log"
  upload_load_file "$manifest_file" "$manifest_key" "$ARTIFACT_DIR/coordinator-manifest-upload-${TIER}.log"

  commands_json=$(jq -cn \
    --arg bucket "$BUCKET" \
    --arg region "$REGION" \
    --arg node_dir "$node_dir" \
    --arg corpus_key "$corpus_key" \
    --arg queries_key "$queries_key" \
    --arg manifest_key "$manifest_key" \
    --arg node_corpus_path "$node_corpus_path" \
    --arg node_queries_path "$node_queries_path" \
    --arg node_manifest_path "$node_manifest_path" \
    --arg prefix "$PREFIX" \
    --arg coord_index "$COORD_INDEX" \
    --arg storage_format "$SPIRE_AWS_STORAGE_FORMAT" \
    --arg load_log_key "$load_log_key" \
    --arg reset_log_key "$reset_log_key" \
    --arg reset_requested "$SPIRE_AWS_RESET_COORDINATOR_INDEX" \
    --arg reset_sql "$reset_sql" \
    '[
      "set -euo pipefail",
      "command -v /usr/local/bin/ecaz >/dev/null",
      "PG_BIN=/usr/pgsql-18/bin; if [ ! -x \"$PG_BIN/psql\" ]; then PG_BIN=$(dirname \"$(command -v psql)\"); fi",
      "rm -rf \($node_dir)",
      "mkdir -p \($node_dir)",
      "df -h \($node_dir)",
      "aws s3 cp s3://\($bucket)/\($corpus_key) \($node_corpus_path) --region \($region)",
      "aws s3 cp s3://\($bucket)/\($queries_key) \($node_queries_path) --region \($region)",
      "aws s3 cp s3://\($bucket)/\($manifest_key) \($node_manifest_path) --region \($region)",
      "set +e",
      "if [ \($reset_requested|@sh) = '\''1'\'' ]; then printf '\''%s\\n'\'' \($reset_sql|@sh) > \($node_dir)/reset-index.sql; \"$PG_BIN/psql\" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres -f \($node_dir)/reset-index.sql > \($node_dir)/reset-index.log 2>&1; else printf \"reset skipped\\n\" > \($node_dir)/reset-index.log; fi",
      "reset_status=$?",
      "aws s3 cp \($node_dir)/reset-index.log s3://\($bucket)/\($reset_log_key) --region \($region) || true",
      "if [ \"$reset_status\" -ne 0 ]; then exit \"$reset_status\"; fi",
      "/usr/local/bin/ecaz corpus load --host 127.0.0.1 --port 5432 --user ecaz_coord --database postgres --prefix \($prefix) --corpus-file \($node_corpus_path) --queries-file \($node_queries_path) --manifest-file \($node_manifest_path) --allow-manifest-mismatch --profile ec_spire --dim 1536 --bits 4 --seed 42 --storage-format \($storage_format) --index-name \($coord_index) --log-file \($node_dir)/load.log",
      "load_status=$?",
      "aws s3 cp \($node_dir)/load.log s3://\($bucket)/\($load_log_key) --region \($region) || true",
      "exit \"$load_status\""
    ]')
  ssm_run_shell "$coord_id" "ecaz Phase 13e node-local coordinator load ${TIER}" \
    "$ARTIFACT_DIR/coordinator-load-${TIER}.ssm.json" "$commands_json"
  download_optional_s3_log "$reset_log_key" "$ARTIFACT_DIR/coordinator-reset-index-${TIER}.log"
  download_optional_s3_log "$load_log_key" "$ARTIFACT_DIR/coordinator-load-${TIER}.log"
}

conninfo_lookup_key() {
  local secret_name="$1"
  local key="EC_SPIRE_REMOTE_CONNINFO_"
  local i char
  for ((i = 0; i < ${#secret_name}; i++)); do
    char="${secret_name:i:1}"
    if [[ "$char" =~ [[:alnum:]] ]]; then
      key+="${char^^}"
    else
      key+="_"
    fi
  done
  printf '%s\n' "$key"
}

remote_identity_query_sql() {
  local remote_index="$1"
  local escaped="${remote_index//\'/\'\'}"
  cat <<SQL
SELECT jsonb_build_object('remote_index_regclass', '${escaped}', 'last_served_epoch', a.active_epoch, 'min_retained_epoch', a.active_epoch, 'extension_version', e.extension_version, 'remote_index_identity_hex', e.profile_fingerprint, 'endpoint_status', e.status, 'tuple_transport_status', e.tuple_transport_status)::text FROM ec_spire_remote_search_endpoint_identity('${escaped}'::regclass::oid) e CROSS JOIN ec_spire_index_active_snapshot_diagnostics('${escaped}'::regclass::oid) a
SQL
}

write_leaf_owned_distributed_plan() {
  local corpus_file="$1"
  local dim="$2"
  local bits="$3"
  local seed="$4"
  local profile="$5"
  local storage_format="$6"
  local total_rows
  local remotes_json
  local remotes_tmp

  mkdir -p "$DISTRIBUTED_OUTPUT_DIR"
  remotes_json=$(jq -c '.remotes | map({node_id})' "$TOPOLOGY")
  remotes_tmp="$DISTRIBUTED_OUTPUT_DIR/remotes.jsonl"
  : > "$remotes_tmp"

  jq -c '.remotes[]' "$TOPOLOGY" | while read -r remote; do
    local node_id secret_name lookup_key remote_prefix remote_index node_dir
    local assignments row_ids remote_corpus row_count assignment_count identity_sql
    node_id=$(jq -r '.node_id' <<< "$remote")
    secret_name=$(jq -r '.secret_name' <<< "$remote")
    lookup_key=$(conninfo_lookup_key "$secret_name")
    remote_prefix="${PREFIX}_node_${node_id}"
    remote_index="$REMOTE_INDEX"
    node_dir="$DISTRIBUTED_OUTPUT_DIR/node-${node_id}"
    assignments="$node_dir/coordinator-base-assignments.tsv"
    row_ids="$node_dir/row-ids.txt"
    remote_corpus="$node_dir/${remote_prefix}_corpus.tsv"
    mkdir -p "$node_dir"

    "$ECAZ_BIN" dev sql \
      --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
      --set "coord_index=$COORD_INDEX" \
      --set "coord_table=${PREFIX}_corpus" \
      --set "node_id=$node_id" \
      --set "remotes_json=$remotes_json" \
      --file scripts/spire-aws/export-coordinator-leaf-base-assignments.sql \
      > "$assignments" \
      2> "$node_dir/coordinator-base-assignments.stderr.log"

    cut -f12 "$assignments" | sort -n -u > "$row_ids"
    awk 'BEGIN { FS = OFS = "\t" } NR == FNR { wanted[$1] = 1; next } ($1 in wanted)' \
      "$row_ids" "$corpus_file" > "$remote_corpus"
    row_count=$(wc -l < "$remote_corpus" | tr -d ' ')
    assignment_count=$(wc -l < "$assignments" | tr -d ' ')
    if [[ "$row_count" != "$assignment_count" ]]; then
      echo "leaf-owned remote corpus for node_id=${node_id} has ${row_count} unique rows for ${assignment_count} coordinator assignments" >&2
      exit 2
    fi
    identity_sql=$(remote_identity_query_sql "$remote_index")

    jq -cn \
      --argjson node_id "$node_id" \
      --arg secret_name "$secret_name" \
      --arg lookup_key "$lookup_key" \
      --arg remote_index "$remote_index" \
      --arg remote_prefix "$remote_prefix" \
      --arg corpus_file "$remote_corpus" \
      --arg identity_sql "$identity_sql" \
      --arg storage_format "$storage_format" \
      --argjson row_count "$row_count" \
      --argjson shard_id "$((node_id - 2))" \
      --argjson dim "$dim" \
      --argjson bits "$bits" \
      --argjson seed "$seed" \
      '{
        node_id: $node_id,
        conninfo_secret_name: $secret_name,
        conninfo_provider_lookup_key: $lookup_key,
        remote_index_regclass: $remote_index,
        remote_prefix: $remote_prefix,
        shard_ids: [$shard_id],
        corpus_file: $corpus_file,
        remote_load_args: [
          "ecaz", "corpus", "load",
          "--profile", "ec_spire",
          "--prefix", $remote_prefix,
          "--dim", ($dim | tostring),
          "--bits", ($bits | tostring),
          "--seed", ($seed | tostring),
          "--corpus-file", $corpus_file,
          "--corpus-only",
          "--storage-format", $storage_format,
          "--index-name", $remote_index
        ],
        remote_identity_query_sql: $identity_sql,
        coordinator_register_descriptor_sql_template: "",
        row_count: $row_count,
        shard_row_counts: [{shard_id: $shard_id, row_count: $row_count}]
      }' >> "$remotes_tmp"
  done

  total_rows=$(jq -s 'map(.row_count) | add // 0' "$remotes_tmp")
  jq -n \
    --arg prefix "$PREFIX" \
    --arg profile "$profile" \
    --arg storage_format "$storage_format" \
    --arg coord_index "$COORD_INDEX" \
    --argjson dim "$dim" \
    --argjson bits "$bits" \
    --argjson seed "$seed" \
    --argjson shard_count "$(jq '.remotes | length' "$TOPOLOGY")" \
    --argjson total_rows "$total_rows" \
    --slurpfile remotes "$remotes_tmp" \
    '{
      version: 1,
      prefix: $prefix,
      profile: $profile,
      dimension: $dim,
      bits: $bits,
      seed: $seed,
      storage_format: $storage_format,
      reloptions: [],
      coordinator_index_name: $coord_index,
      source_identity_column: "leaf_base_assignment",
      shard_policy: "coordinator_leaf_assignment_round_robin",
      shard_count: $shard_count,
      total_rows: $total_rows,
      remotes: $remotes
    }' > "$PLAN_FILE"
}

case "$TIER" in
  correctness)
    PREFIX=ec_spire_aws_synth_10k
    COORD_INDEX="${COORD_INDEX:-${PREFIX}_idx}"
    REMOTE_INDEX="${REMOTE_INDEX:-${PREFIX}_remote_idx}"
    DISTRIBUTED_OUTPUT_DIR="${ARTIFACT_DIR}/distributed-${TIER}"
    PLACEMENT_CONFIG="${DISTRIBUTED_OUTPUT_DIR}/distributed-placement-config.json"
    PLAN_FILE="${DISTRIBUTED_OUTPUT_DIR}/distributed-placement-plan.json"
    write_distributed_placement_config
    "$ECAZ_BIN" corpus generate --n 10000 --dim 1536 \
      --output "$WORK_DIR/${PREFIX}_corpus.tsv"
    "$ECAZ_BIN" corpus generate --n 100 --dim 1536 \
      --output "$WORK_DIR/${PREFIX}_queries.tsv"
    reset_coordinator_index_if_requested
    "$ECAZ_BIN" corpus load \
      --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
      --prefix "$PREFIX" \
      --corpus-file "$WORK_DIR/${PREFIX}_corpus.tsv" \
      --queries-file "$WORK_DIR/${PREFIX}_queries.tsv" \
      --profile ec_spire --dim 1536 --bits 4 --seed 42 \
      --storage-format "$SPIRE_AWS_STORAGE_FORMAT" \
      --index-name "$COORD_INDEX" \
      --log-file "$ARTIFACT_DIR/coordinator-load-${TIER}.log"
    write_leaf_owned_distributed_plan "$WORK_DIR/${PREFIX}_corpus.tsv" 1536 4 42 ec_spire "$SPIRE_AWS_STORAGE_FORMAT" \
      > "$ARTIFACT_DIR/distributed-plan-${TIER}.log"
    ;;
  representative)
    PREFIX=ec_spire_aws_repr_1m
    PREPARED_PREFIX=ec_real_100k
    COORD_INDEX="${COORD_INDEX:-${PREFIX}_idx}"
    REMOTE_INDEX="${REMOTE_INDEX:-${PREFIX}_remote_idx}"
    DISTRIBUTED_OUTPUT_DIR="${ARTIFACT_DIR}/distributed-${TIER}"
    PLACEMENT_CONFIG="${DISTRIBUTED_OUTPUT_DIR}/distributed-placement-config.json"
    PLAN_FILE="${DISTRIBUTED_OUTPUT_DIR}/distributed-placement-plan.json"
    write_distributed_placement_config
    "$ECAZ_BIN" corpus fetch \
      --dataset qdrant-dbpedia-openai3-large-1536-1m \
      --output-dir "$WORK_DIR/qdrant-dbpedia/"
    "$ECAZ_BIN" corpus prepare \
      --profile "$PREPARED_PREFIX" \
      --parquet "$WORK_DIR/qdrant-dbpedia/data" \
      --output-dir "$WORK_DIR/qdrant-dbpedia/prepared/" \
      --dim 1536 \
      --source-dataset qdrant-dbpedia-openai3-large-1536-1m
    load_coordinator_representative_node_local \
      "$WORK_DIR/qdrant-dbpedia/prepared/${PREPARED_PREFIX}_corpus.tsv" \
      "$WORK_DIR/qdrant-dbpedia/prepared/${PREPARED_PREFIX}_queries.tsv" \
      "$WORK_DIR/qdrant-dbpedia/prepared/${PREPARED_PREFIX}_manifest.json"
    restart_operator_tunnel_if_available coordinator "$(jq -r '.coordinator.instance_id' "$TOPOLOGY")" "$COORD_PORT"
    write_leaf_owned_distributed_plan "$WORK_DIR/qdrant-dbpedia/prepared/${PREPARED_PREFIX}_corpus.tsv" 1536 4 42 ec_spire "$SPIRE_AWS_STORAGE_FORMAT" \
      > "$ARTIFACT_DIR/distributed-plan-${TIER}.log"
    ;;
  stress)
    PREFIX=ec_spire_aws_synth_10m
    COORD_INDEX="${COORD_INDEX:-${PREFIX}_idx}"
    REMOTE_INDEX="${REMOTE_INDEX:-${PREFIX}_remote_idx}"
    DISTRIBUTED_OUTPUT_DIR="${ARTIFACT_DIR}/distributed-${TIER}"
    PLACEMENT_CONFIG="${DISTRIBUTED_OUTPUT_DIR}/distributed-placement-config.json"
    PLAN_FILE="${DISTRIBUTED_OUTPUT_DIR}/distributed-placement-plan.json"
    write_distributed_placement_config
    "$ECAZ_BIN" corpus generate --n 10000000 --dim 1536 \
      --output "$WORK_DIR/${PREFIX}_corpus.tsv"
    "$ECAZ_BIN" corpus generate --n 10000 --dim 1536 \
      --output "$WORK_DIR/${PREFIX}_queries.tsv"
    reset_coordinator_index_if_requested
    "$ECAZ_BIN" corpus load \
      --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
      --prefix "$PREFIX" \
      --corpus-file "$WORK_DIR/${PREFIX}_corpus.tsv" \
      --queries-file "$WORK_DIR/${PREFIX}_queries.tsv" \
      --profile ec_spire --dim 1536 --bits 4 --seed 42 \
      --storage-format "$SPIRE_AWS_STORAGE_FORMAT" \
      --index-name "$COORD_INDEX" \
      --log-file "$ARTIFACT_DIR/coordinator-load-${TIER}.log"
    write_leaf_owned_distributed_plan "$WORK_DIR/${PREFIX}_corpus.tsv" 1536 4 42 ec_spire "$SPIRE_AWS_STORAGE_FORMAT" \
      > "$ARTIFACT_DIR/distributed-plan-${TIER}.log"
    ;;
  *)
    echo "unknown tier: $TIER" >&2; exit 2 ;;
esac

if [[ "$TIER" == "representative" ]]; then
  load_remote_shards_node_local
  restart_all_operator_tunnels_if_available
else
  load_remote_shards
fi

echo "$PLAN_FILE" > "$ARTIFACT_DIR/distributed-placement-plan-${TIER}.path"
