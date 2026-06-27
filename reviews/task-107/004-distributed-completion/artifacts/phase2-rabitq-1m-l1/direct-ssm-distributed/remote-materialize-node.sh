#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <node-id>" >&2
  exit 2
fi

export AWS_DEFAULT_REGION=us-west-2

NODE_ID=$1
PREFIX=task107_phase2_rabitq_1m_l1
REMOTE_PREFIX=${PREFIX}_node_${NODE_ID}
REMOTE_TABLE=${REMOTE_PREFIX}_corpus
REMOTE_INDEX=${PREFIX}_remote_idx
NODE_DIR=/var/tmp/ecaz-task107-phase2-rabitq-1m-l1/remote-materialize-node-${NODE_ID}
BUCKET=ecaz-spire-aws-20260614203301860100000009
S3_PREFIX=task107/004/phase2-rabitq-1m-l1/direct-ssm-distributed
ASSIGNMENTS=$NODE_DIR/coordinator-base-assignments.tsv
ASSIGNMENTS_KEY=$S3_PREFIX/distributed-representative/node-${NODE_ID}/coordinator-base-assignments.tsv
LOG_KEY_PREFIX=$S3_PREFIX/remote-materialize-node-${NODE_ID}
PG_BIN=/usr/pgsql-18/bin
if [ ! -x "$PG_BIN/psql" ]; then
  PG_BIN=$(dirname "$(command -v psql)")
fi

mkdir -p "$NODE_DIR"

upload_logs() {
  status=$1
  echo step=upload-remote-materialize-node-${NODE_ID} status=$status
  aws s3 cp "$NODE_DIR" "s3://$BUCKET/$LOG_KEY_PREFIX" \
    --recursive \
    --exclude 'coordinator-base-assignments.tsv' \
    --region us-west-2 || true
}
trap 'status=$?; upload_logs "$status"; exit "$status"' EXIT

case "$NODE_ID" in
  2) SECRET_NAME=ecaz-spire-aws-aa606602-remote-1-20260614203301856800000002 ;;
  3) SECRET_NAME=ecaz-spire-aws-aa606602-remote-2-20260614203301857100000006 ;;
  *) echo "unexpected node id $NODE_ID" >&2; exit 2 ;;
esac

echo step=preflight
hostname > "$NODE_DIR/hostname.log"
df -h "$NODE_DIR" > "$NODE_DIR/df-before.log"

echo step=download-assignments
aws s3 cp "s3://$BUCKET/$ASSIGNMENTS_KEY" "$ASSIGNMENTS" --region us-west-2
wc -l "$ASSIGNMENTS" > "$NODE_DIR/assignment-row-count.log"

cat > "$NODE_DIR/materialize.sql" <<SQL
\set ON_ERROR_STOP on
\set consistency_mode strict

CREATE TEMP TABLE ec_spire_leaf_base_assignment_import (
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

\copy ec_spire_leaf_base_assignment_import FROM '$ASSIGNMENTS' WITH (FORMAT text, DELIMITER E'\t')

CREATE TEMP TABLE ec_spire_leaf_base_materialization_input AS
SELECT src.leaf_pid,
       src.parent_pid,
       src.object_version,
       src.row_index,
       src.assignment_flags,
       src.vec_id_hex,
       split_part(trim(both '()' from heap_row.ctid::text), ',', 1)::bigint AS remote_heap_block,
       split_part(trim(both '()' from heap_row.ctid::text), ',', 2)::int AS remote_heap_offset,
       src.payload_format,
       src.gamma,
       src.encoded_payload_hex
  FROM ec_spire_leaf_base_assignment_import AS src
  JOIN ${REMOTE_TABLE} AS heap_row
    ON heap_row.id = src.row_id
 ORDER BY src.leaf_pid, src.row_index;

DO \$\$
DECLARE
  imported_rows bigint;
  matched_rows bigint;
  min_active_epoch bigint;
  max_active_epoch bigint;
BEGIN
  SELECT count(*) INTO imported_rows FROM ec_spire_leaf_base_assignment_import;
  SELECT count(*) INTO matched_rows FROM ec_spire_leaf_base_materialization_input;
  IF imported_rows <> matched_rows THEN
    RAISE EXCEPTION
      'remote heap materialization matched % rows for % imported coordinator assignments',
      matched_rows,
      imported_rows;
  END IF;
  SELECT min(active_epoch), max(active_epoch)
    INTO min_active_epoch, max_active_epoch
    FROM ec_spire_leaf_base_assignment_import;
  IF min_active_epoch IS NULL OR min_active_epoch <> max_active_epoch THEN
    RAISE EXCEPTION
      'remote heap materialization requires exactly one exported coordinator active epoch, got min % max %',
      min_active_epoch,
      max_active_epoch;
  END IF;
END \$\$;

SELECT materialized.*
  FROM ec_spire_materialize_static_remote_leaf_assignments_with_mode(
       '${REMOTE_INDEX}'::regclass::oid,
       (SELECT min(active_epoch) FROM ec_spire_leaf_base_assignment_import),
       (SELECT array_agg(leaf_pid ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(parent_pid ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(object_version ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(row_index ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(assignment_flags ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(vec_id_hex ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(remote_heap_block ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(remote_heap_offset ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(payload_format ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(gamma ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       (SELECT array_agg(encoded_payload_hex ORDER BY leaf_pid, row_index) FROM ec_spire_leaf_base_materialization_input),
       :'consistency_mode'
  ) AS materialized;
SQL

echo step=materialize
"$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres \
  -f "$NODE_DIR/materialize.sql" \
  > "$NODE_DIR/remote-materialize.log" \
  2> "$NODE_DIR/remote-materialize.stderr.log"

echo step=verify-leaf-parity
awk 'BEGIN { FS = OFS = "\t" } { count[$2]++ } END { for (pid in count) print pid, count[pid] }' \
  "$ASSIGNMENTS" > "$NODE_DIR/coordinator-required-leaves.txt"
"$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres -A -t -F "$(printf '\t')" \
  -c "SELECT leaf_pid, effective_assignment_count FROM ec_spire_index_leaf_snapshot('${REMOTE_INDEX}'::regclass::oid) WHERE placement_state = 'available' ORDER BY leaf_pid" \
  > "$NODE_DIR/remote-observed-leaves.txt" \
  2> "$NODE_DIR/remote-observed-leaves.stderr.log"
sort "$NODE_DIR/coordinator-required-leaves.txt" -o "$NODE_DIR/coordinator-required-leaves.txt"
sort "$NODE_DIR/remote-observed-leaves.txt" -o "$NODE_DIR/remote-observed-leaves.txt"
comm -23 "$NODE_DIR/coordinator-required-leaves.txt" "$NODE_DIR/remote-observed-leaves.txt" \
  > "$NODE_DIR/missing-or-mismatched-leaves.txt"
if [ -s "$NODE_DIR/missing-or-mismatched-leaves.txt" ]; then
  echo "remote node $NODE_ID has missing or mismatched materialized leaves" >&2
  exit 2
fi

echo step=identity
"$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres -A -t \
  -c "SELECT jsonb_build_object('remote_index_regclass', '${REMOTE_INDEX}', 'last_served_epoch', a.active_epoch, 'min_retained_epoch', a.active_epoch, 'extension_version', e.extension_version, 'remote_index_identity_hex', e.profile_fingerprint, 'endpoint_status', e.status, 'tuple_transport_status', e.tuple_transport_status)::text FROM ec_spire_remote_search_endpoint_identity('${REMOTE_INDEX}'::regclass::oid) e CROSS JOIN ec_spire_index_active_snapshot_diagnostics('${REMOTE_INDEX}'::regclass::oid) a" \
  > "$NODE_DIR/identity.json" \
  2> "$NODE_DIR/identity.stderr.log"
printf 'node_id=%s secret_name=%s remote_prefix=%s remote_index=%s\n' \
  "$NODE_ID" "$SECRET_NAME" "$REMOTE_PREFIX" "$REMOTE_INDEX" \
  > "$NODE_DIR/identity-context.log"

df -h "$NODE_DIR" > "$NODE_DIR/df-after.log"
echo step=complete
