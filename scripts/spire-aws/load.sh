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
ECAZ_BIN="${ECAZ_BIN:-ecaz}"
WORK_DIR="${WORK_DIR:-/var/lib/ecaz}"

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
        kind: "source_identity_mod",
        shard_count: $shard_count,
        source_identity_column: "id"
      }
    }' > "$PLACEMENT_CONFIG"
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
    "$ECAZ_BIN" corpus load \
      --host "$remote_host" --port "$remote_port" --user ecaz_coord --database postgres \
      "${remote_load_args[@]}" \
      --log-output "${log_prefix}-load-${TIER}.log"

    "$ECAZ_BIN" corpus inspect \
      --host "$remote_host" --port "$remote_port" --user ecaz_coord --database postgres \
      --prefix "$remote_prefix" \
      --log-output "${log_prefix}-inspect-${TIER}.log"
  done
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
    "$ECAZ_BIN" corpus generate --rows 10000 --dim 1536 \
      --output "$WORK_DIR/${PREFIX}_corpus.tsv"
    "$ECAZ_BIN" corpus generate --rows 100 --dim 1536 \
      --output "$WORK_DIR/${PREFIX}_queries.tsv"
    "$ECAZ_BIN" corpus load \
      --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
      --prefix "$PREFIX" \
      --corpus-file "$WORK_DIR/${PREFIX}_corpus.tsv" \
      --queries-file "$WORK_DIR/${PREFIX}_queries.tsv" \
      --profile ec_spire --dim 1536 --bits 4 --seed 42 \
      --index-name "$COORD_INDEX" \
      --log-output "$ARTIFACT_DIR/coordinator-load-${TIER}.log"
    "$ECAZ_BIN" corpus load \
      --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
      --prefix "$PREFIX" \
      --corpus-file "$WORK_DIR/${PREFIX}_corpus.tsv" \
      --queries-file "$WORK_DIR/${PREFIX}_queries.tsv" \
      --profile ec_spire --dim 1536 --bits 4 --seed 42 \
      --distributed-placement-config "$PLACEMENT_CONFIG" \
      --distributed-placement-output-dir "$DISTRIBUTED_OUTPUT_DIR" \
      --log-output "$ARTIFACT_DIR/distributed-plan-${TIER}.log"
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
      --parquet "$WORK_DIR/qdrant-dbpedia/data/0000.parquet" \
      --output-dir "$WORK_DIR/qdrant-dbpedia/prepared/" \
      --dim 1536 \
      --source-dataset qdrant-dbpedia-openai3-large-1536-1m
    "$ECAZ_BIN" corpus load \
      --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
      --prefix "$PREFIX" \
      --corpus-file "$WORK_DIR/qdrant-dbpedia/prepared/${PREPARED_PREFIX}_corpus.tsv" \
      --queries-file "$WORK_DIR/qdrant-dbpedia/prepared/${PREPARED_PREFIX}_queries.tsv" \
      --manifest-file "$WORK_DIR/qdrant-dbpedia/prepared/${PREPARED_PREFIX}_manifest.json" \
      --allow-manifest-mismatch \
      --profile ec_spire --dim 1536 --bits 4 --seed 42 \
      --index-name "$COORD_INDEX" \
      --log-output "$ARTIFACT_DIR/coordinator-load-${TIER}.log"
    "$ECAZ_BIN" corpus load \
      --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
      --prefix "$PREFIX" \
      --corpus-file "$WORK_DIR/qdrant-dbpedia/prepared/${PREPARED_PREFIX}_corpus.tsv" \
      --queries-file "$WORK_DIR/qdrant-dbpedia/prepared/${PREPARED_PREFIX}_queries.tsv" \
      --manifest-file "$WORK_DIR/qdrant-dbpedia/prepared/${PREPARED_PREFIX}_manifest.json" \
      --allow-manifest-mismatch \
      --profile ec_spire --dim 1536 --bits 4 --seed 42 \
      --distributed-placement-config "$PLACEMENT_CONFIG" \
      --distributed-placement-output-dir "$DISTRIBUTED_OUTPUT_DIR" \
      --log-output "$ARTIFACT_DIR/distributed-plan-${TIER}.log"
    ;;
  stress)
    PREFIX=ec_spire_aws_synth_10m
    COORD_INDEX="${COORD_INDEX:-${PREFIX}_idx}"
    REMOTE_INDEX="${REMOTE_INDEX:-${PREFIX}_remote_idx}"
    DISTRIBUTED_OUTPUT_DIR="${ARTIFACT_DIR}/distributed-${TIER}"
    PLACEMENT_CONFIG="${DISTRIBUTED_OUTPUT_DIR}/distributed-placement-config.json"
    PLAN_FILE="${DISTRIBUTED_OUTPUT_DIR}/distributed-placement-plan.json"
    write_distributed_placement_config
    "$ECAZ_BIN" corpus generate --rows 10000000 --dim 1536 \
      --output "$WORK_DIR/${PREFIX}_corpus.tsv"
    "$ECAZ_BIN" corpus generate --rows 10000 --dim 1536 \
      --output "$WORK_DIR/${PREFIX}_queries.tsv"
    "$ECAZ_BIN" corpus load \
      --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
      --prefix "$PREFIX" \
      --corpus-file "$WORK_DIR/${PREFIX}_corpus.tsv" \
      --queries-file "$WORK_DIR/${PREFIX}_queries.tsv" \
      --profile ec_spire --dim 1536 --bits 4 --seed 42 \
      --index-name "$COORD_INDEX" \
      --log-output "$ARTIFACT_DIR/coordinator-load-${TIER}.log"
    "$ECAZ_BIN" corpus load \
      --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
      --prefix "$PREFIX" \
      --corpus-file "$WORK_DIR/${PREFIX}_corpus.tsv" \
      --queries-file "$WORK_DIR/${PREFIX}_queries.tsv" \
      --profile ec_spire --dim 1536 --bits 4 --seed 42 \
      --distributed-placement-config "$PLACEMENT_CONFIG" \
      --distributed-placement-output-dir "$DISTRIBUTED_OUTPUT_DIR" \
      --log-output "$ARTIFACT_DIR/distributed-plan-${TIER}.log"
    ;;
  *)
    echo "unknown tier: $TIER" >&2; exit 2 ;;
esac

load_remote_shards

echo "$PLAN_FILE" > "$ARTIFACT_DIR/distributed-placement-plan-${TIER}.path"
