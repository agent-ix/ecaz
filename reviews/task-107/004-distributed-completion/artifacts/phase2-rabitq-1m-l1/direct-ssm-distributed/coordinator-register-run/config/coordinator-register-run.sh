#!/usr/bin/env bash
set -euo pipefail

export AWS_DEFAULT_REGION=us-west-2

PREFIX=task107_phase2_rabitq_1m_l1
COORD_INDEX=${PREFIX}_idx
NODE_DIR=/var/tmp/ecaz-task107-phase2-rabitq-1m-l1
RUN_DIR=$NODE_DIR/coordinator-register-run
CONFIG_DIR=$RUN_DIR/config
IDENTITY_DIR=$CONFIG_DIR/identities
BENCH_DIR=$NODE_DIR/bench
BUCKET=ecaz-spire-aws-20260614203301860100000009
S3_PREFIX=task107/004/phase2-rabitq-1m-l1/direct-ssm-distributed
PLAN=$CONFIG_DIR/distributed-placement-plan.json
SUITE_CONFIG=$CONFIG_DIR/suite-node.json
REGISTER_SQL=$RUN_DIR/register-remotes-rendered.sql
REMOTES_JSON='[{"node_id":2},{"node_id":3}]'
PG_BIN=/usr/pgsql-18/bin
if [ ! -x "$PG_BIN/psql" ]; then
  PG_BIN=$(dirname "$(command -v psql)")
fi

mkdir -p "$RUN_DIR" "$CONFIG_DIR" "$IDENTITY_DIR" "$BENCH_DIR"

upload_logs() {
  status=$1
  echo step=upload-coordinator-register-run status=$status
  aws s3 cp "$RUN_DIR" "s3://$BUCKET/$S3_PREFIX/coordinator-register-run" \
    --recursive \
    --region us-west-2 || true
  aws s3 cp "$BENCH_DIR" "s3://$BUCKET/$S3_PREFIX/bench" \
    --recursive \
    --region us-west-2 || true
}
trap 'status=$?; upload_logs "$status"; exit "$status"' EXIT

cat > "$RUN_DIR/publish-remote-placements.sql" <<'SQL'
\set ON_ERROR_STOP on
\set consistency_mode strict

WITH remote_nodes AS (
  SELECT
      ordinality::int AS remote_ordinal,
      (remote->>'node_id')::int AS node_id
    FROM jsonb_array_elements(:'remotes_json'::jsonb) WITH ORDINALITY AS t(remote, ordinality)
),
remote_count AS (
  SELECT count(*)::int AS value FROM remote_nodes
),
leaf_plan AS (
  SELECT
      leaf_pid,
      (((row_number() OVER (ORDER BY leaf_pid))::int - 1) % remote_count.value) + 1 AS remote_ordinal
    FROM ec_spire_index_leaf_snapshot(:'coord_index'::regclass::oid)
    CROSS JOIN remote_count
   WHERE placement_state = 'available'
     AND remote_count.value > 0
),
assignment AS (
  SELECT
      leaf_plan.leaf_pid,
      remote_nodes.node_id
    FROM leaf_plan
    JOIN remote_nodes USING (remote_ordinal)
)
SELECT *
  FROM ec_spire_publish_static_remote_placement_nodes_with_mode(
       :'coord_index'::regclass::oid,
       (SELECT COALESCE(array_agg(leaf_pid::bigint ORDER BY leaf_pid), ARRAY[]::bigint[]) FROM assignment),
       (SELECT COALESCE(array_agg(node_id::int ORDER BY leaf_pid), ARRAY[]::int[]) FROM assignment),
       :'consistency_mode'
  );
SQL

echo step=preflight
hostname > "$RUN_DIR/hostname.log"
df -h "$NODE_DIR" > "$RUN_DIR/df-before.log"
"$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres -At \
  -c "SELECT c.relname, i.indisvalid, i.indisready FROM pg_class c JOIN pg_index i ON i.indexrelid = c.oid WHERE c.relname = '${COORD_INDEX}';" \
  > "$RUN_DIR/coordinator-index-check.log"
if ! grep -q "^${COORD_INDEX}|" "$RUN_DIR/coordinator-index-check.log"; then
  echo "missing coordinator index ${COORD_INDEX}" >&2
  exit 2
fi

echo step=download-config
aws s3 cp "s3://$BUCKET/$S3_PREFIX/distributed-representative/distributed-placement-plan.json" \
  "$PLAN" --region us-west-2
aws s3 cp "s3://$BUCKET/$S3_PREFIX/coordinator-register-run/config/suite-node.json" \
  "$SUITE_CONFIG" --region us-west-2
aws s3 cp "s3://$BUCKET/$S3_PREFIX/coordinator-register-run/config/identities" \
  "$IDENTITY_DIR" --recursive --region us-west-2

echo step=publish-placements
"$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres \
  -v coord_index="$COORD_INDEX" \
  -v remotes_json="$REMOTES_JSON" \
  -f "$RUN_DIR/publish-remote-placements.sql" \
  > "$RUN_DIR/publish-remote-placements.log" \
  2> "$RUN_DIR/publish-remote-placements.stderr.log"

echo step=render-registration
/usr/local/bin/ecaz corpus render-spire-registrations \
  --plan-file "$PLAN" \
  --identity-dir "$IDENTITY_DIR" \
  --output-file "$REGISTER_SQL" \
  --descriptor-generation 1 \
  > "$RUN_DIR/render-registration.log" \
  2> "$RUN_DIR/render-registration.stderr.log"

echo step=register-remotes
"$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres \
  -f "$REGISTER_SQL" \
  > "$RUN_DIR/register-remotes.log" \
  2> "$RUN_DIR/register-remotes.stderr.log"

"$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres -A -t \
  -c "SELECT to_jsonb(s)::text FROM ec_spire_remote_node_snapshot('${COORD_INDEX}'::regclass::oid) AS s ORDER BY node_id;" \
  > "$RUN_DIR/remote-node-snapshot.jsonl" \
  2> "$RUN_DIR/remote-node-snapshot.stderr.log"

echo step=run-suite
/usr/local/bin/ecaz --host 127.0.0.1 --port 5432 --user ecaz_coord --database postgres \
  bench suite run \
  --config "$SUITE_CONFIG" \
  --manifest-output "$BENCH_DIR/suite-manifest-node.json" \
  --results-output "$BENCH_DIR/suite-results-node.jsonl" \
  > "$RUN_DIR/run-suite.log" \
  2> "$RUN_DIR/run-suite.stderr.log"

df -h "$NODE_DIR" > "$RUN_DIR/df-after.log"
echo step=complete
