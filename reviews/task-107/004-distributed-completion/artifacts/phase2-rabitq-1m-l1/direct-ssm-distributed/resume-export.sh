#!/usr/bin/env bash
set -euo pipefail

export AWS_DEFAULT_REGION=us-west-2

PREFIX=task107_phase2_rabitq_1m_l1
COORD_INDEX=${PREFIX}_idx
REMOTE_INDEX=${PREFIX}_remote_idx
NODE_DIR=/var/tmp/ecaz-task107-phase2-rabitq-1m-l1
RESUME_DIR=$NODE_DIR/resume-export
PLAN_DIR=$NODE_DIR/distributed-representative
BUCKET=ecaz-spire-aws-20260614203301860100000009
S3_PREFIX=task107/004/phase2-rabitq-1m-l1/direct-ssm-distributed
CORPUS_KEY=representative-load/representative/coordinator/ec_real_ann_benchmarks_anchor_corpus.tsv
CORPUS=$NODE_DIR/ec_real_ann_benchmarks_anchor_corpus.tsv
PG_BIN=/usr/pgsql-18/bin
if [ ! -x "$PG_BIN/psql" ]; then
  PG_BIN=$(dirname "$(command -v psql)")
fi

mkdir -p "$RESUME_DIR" "$PLAN_DIR/node-2" "$PLAN_DIR/node-3"

upload_resume() {
  status=$1
  echo step=upload-resume status=$status
  aws s3 cp "$RESUME_DIR" "s3://$BUCKET/$S3_PREFIX/resume-export" --recursive --region us-west-2 || true
  aws s3 cp "$PLAN_DIR" "s3://$BUCKET/$S3_PREFIX/distributed-representative" \
    --recursive --exclude '*_corpus.tsv' --region us-west-2 || true
}
trap 'status=$?; upload_resume "$status"; exit "$status"' EXIT

cat > "$RESUME_DIR/export-coordinator-leaf-base-assignments.sql" <<'SQL'
\set ON_ERROR_STOP on
WITH remote_nodes AS (
  SELECT ordinality::int AS remote_ordinal, (remote->>'node_id')::int AS node_id
    FROM jsonb_array_elements(:'remotes_json'::jsonb) WITH ORDINALITY AS t(remote, ordinality)
),
remote_count AS (
  SELECT count(*)::int AS value FROM remote_nodes
),
assigned_leaf_pids AS (
  SELECT leaf_plan.leaf_pid
    FROM (
      SELECT leaf_pid, (((row_number() OVER (ORDER BY leaf_pid))::int - 1) % remote_count.value) + 1 AS remote_ordinal
        FROM ec_spire_index_leaf_snapshot(:'coord_index'::regclass::oid)
        CROSS JOIN remote_count
       WHERE placement_state = 'available' AND remote_count.value > 0
    ) AS leaf_plan
    JOIN remote_nodes USING (remote_ordinal)
   WHERE remote_nodes.node_id = :node_id::int
   ORDER BY leaf_plan.leaf_pid
),
selected_assignments AS (
  SELECT *
    FROM ec_spire_index_leaf_base_assignment_snapshot(
         :'coord_index'::regclass::oid,
         (SELECT COALESCE(array_agg(leaf_pid::bigint ORDER BY leaf_pid), ARRAY[]::bigint[]) FROM assigned_leaf_pids))
)
SELECT active_epoch, leaf_pid, parent_pid, object_version, row_index, assignment_flags,
       encode(vec_id, 'hex') AS vec_id_hex,
       encode(row_locator, 'hex') AS row_locator_hex,
       heap_block, heap_offset, heap_ctid, heap_row.id AS row_id,
       payload_format, gamma, encode(encoded_payload, 'hex') AS encoded_payload_hex
  FROM selected_assignments
  JOIN :coord_table AS heap_row ON heap_row.ctid = selected_assignments.heap_ctid::tid
 ORDER BY leaf_pid, row_index;
SQL

echo step=preflight
hostname > "$RESUME_DIR/hostname.log"
df -h "$NODE_DIR" > "$RESUME_DIR/df-before.log"
"$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres -At \
  -c "SELECT c.relname, i.indisvalid, i.indisready FROM pg_class c JOIN pg_index i ON i.indexrelid = c.oid WHERE c.relname = '${COORD_INDEX}';" \
  > "$RESUME_DIR/coordinator-index-check.log"
if ! grep -q "^${COORD_INDEX}|" "$RESUME_DIR/coordinator-index-check.log"; then
  echo "missing coordinator index ${COORD_INDEX}" >&2
  exit 2
fi
if [ -s "$CORPUS" ]; then
  echo corpus-present "$CORPUS" > "$RESUME_DIR/corpus-source.log"
else
  echo corpus-download "$CORPUS" > "$RESUME_DIR/corpus-source.log"
  aws s3 cp "s3://$BUCKET/$CORPUS_KEY" "$CORPUS" --region us-west-2
fi

echo step=export-remote-corpora
REMOTES_JSON='[{"node_id":2},{"node_id":3}]'
: > "$PLAN_DIR/remotes.jsonl"
for node in 2 3; do
  NODE_PLAN_DIR="$PLAN_DIR/node-$node"
  ASSIGNMENTS="$NODE_PLAN_DIR/coordinator-base-assignments.tsv"
  ROW_IDS="$NODE_PLAN_DIR/row-ids.txt"
  REMOTE_PREFIX="${PREFIX}_node_${node}"
  REMOTE_CORPUS="$NODE_PLAN_DIR/${REMOTE_PREFIX}_corpus.tsv"
  mkdir -p "$NODE_PLAN_DIR"

  "$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres \
    -A -t -F "$(printf '\t')" \
    -v coord_index="$COORD_INDEX" -v coord_table="${PREFIX}_corpus" \
    -v node_id="$node" -v remotes_json="$REMOTES_JSON" \
    -f "$RESUME_DIR/export-coordinator-leaf-base-assignments.sql" \
    > "$ASSIGNMENTS" \
    2> "$NODE_PLAN_DIR/coordinator-base-assignments.stderr.log"
  cut -f12 "$ASSIGNMENTS" | sort -n -u > "$ROW_IDS"
  awk 'BEGIN { FS = OFS = "\t" } NR == FNR { wanted[$1] = 1; next } ($1 in wanted)' \
    "$ROW_IDS" "$CORPUS" > "$REMOTE_CORPUS"
  row_count=$(wc -l < "$REMOTE_CORPUS" | tr -d ' ')
  assignment_count=$(wc -l < "$ASSIGNMENTS" | tr -d ' ')
  if [ "$row_count" != "$assignment_count" ]; then
    echo "row_count_mismatch node=$node rows=$row_count assignments=$assignment_count" >&2
    exit 2
  fi
  shard_id=$((node - 2))
  case "$node" in
    2) secret_name=ecaz-spire-aws-aa606602-remote-1-20260614203301856800000002 ;;
    3) secret_name=ecaz-spire-aws-aa606602-remote-2-20260614203301857100000006 ;;
    *) echo "unexpected node $node" >&2; exit 2 ;;
  esac
  lookup_key=$(printf 'EC_SPIRE_REMOTE_CONNINFO_%s' "$secret_name" | tr '[:lower:]-' '[:upper:]_')
  identity_sql="SELECT jsonb_build_object('remote_index_regclass', '${REMOTE_INDEX}', 'last_served_epoch', a.active_epoch, 'min_retained_epoch', a.active_epoch, 'extension_version', e.extension_version, 'remote_index_identity_hex', e.profile_fingerprint, 'endpoint_status', e.status, 'tuple_transport_status', e.tuple_transport_status)::text FROM ec_spire_remote_search_endpoint_identity('${REMOTE_INDEX}'::regclass::oid) e CROSS JOIN ec_spire_index_active_snapshot_diagnostics('${REMOTE_INDEX}'::regclass::oid) a"
  jq -cn \
    --argjson node_id "$node" \
    --arg secret_name "$secret_name" \
    --arg lookup_key "$lookup_key" \
    --arg remote_index "$REMOTE_INDEX" \
    --arg remote_prefix "$REMOTE_PREFIX" \
    --arg corpus_file "$REMOTE_CORPUS" \
    --arg identity_sql "$identity_sql" \
    --argjson row_count "$row_count" \
    --argjson shard_id "$shard_id" \
    '{node_id:$node_id,conninfo_secret_name:$secret_name,conninfo_provider_lookup_key:$lookup_key,remote_index_regclass:$remote_index,remote_prefix:$remote_prefix,shard_ids:[$shard_id],corpus_file:$corpus_file,remote_load_args:["ecaz","corpus","load","--profile","ec_spire","--prefix",$remote_prefix,"--dim","1536","--bits","4","--seed","42","--corpus-file",$corpus_file,"--corpus-only","--storage-format","rabitq","--index-name",$remote_index,"--reloption","local_store_count=1"],remote_identity_query_sql:$identity_sql,coordinator_register_descriptor_sql_template:"",row_count:$row_count,shard_row_counts:[{shard_id:$shard_id,row_count:$row_count}]}' \
    >> "$PLAN_DIR/remotes.jsonl"
  aws s3 cp "$REMOTE_CORPUS" "s3://$BUCKET/$S3_PREFIX/remote-node-$node/${REMOTE_PREFIX}_corpus.tsv" \
    --region us-west-2 > "$NODE_PLAN_DIR/upload-remote-corpus.log"
done

total_rows=$(jq -s 'map(.row_count) | add // 0' "$PLAN_DIR/remotes.jsonl")
jq -n \
  --arg prefix "$PREFIX" \
  --arg profile ec_spire \
  --arg storage_format rabitq \
  --arg coord_index "$COORD_INDEX" \
  --argjson dim 1536 \
  --argjson bits 4 \
  --argjson seed 42 \
  --argjson shard_count 2 \
  --argjson total_rows "$total_rows" \
  --slurpfile remotes "$PLAN_DIR/remotes.jsonl" \
  '{version:1,prefix:$prefix,profile:$profile,dimension:$dim,bits:$bits,seed:$seed,storage_format:$storage_format,reloptions:[],coordinator_index_name:$coord_index,source_identity_column:"leaf_base_assignment",shard_policy:"coordinator_leaf_assignment_round_robin",shard_count:$shard_count,total_rows:$total_rows,remotes:$remotes}' \
  > "$PLAN_DIR/distributed-placement-plan.json"
echo "$PLAN_DIR/distributed-placement-plan.json" > "$RESUME_DIR/distributed-placement-plan.path"
aws s3 cp "$PLAN_DIR/distributed-placement-plan.json" "s3://$BUCKET/$S3_PREFIX/distributed-representative/distributed-placement-plan.json" \
  --region us-west-2 > "$RESUME_DIR/upload-plan.log"
df -h "$NODE_DIR" > "$RESUME_DIR/df-after.log"
echo step=complete
