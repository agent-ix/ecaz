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
REMOTE_INDEX=${PREFIX}_remote_idx
NODE_DIR=/var/tmp/ecaz-task107-phase2-rabitq-1m-l1
CLEANUP_DIR=$NODE_DIR/cleanup-remote-node-${NODE_ID}
BUCKET=ecaz-spire-aws-20260614203301860100000009
S3_PREFIX=task107/004/phase2-rabitq-1m-l1/direct-ssm-distributed/cleanup/remote-node-${NODE_ID}
PG_BIN=/usr/pgsql-18/bin
if [ ! -x "$PG_BIN/psql" ]; then
  PG_BIN=$(dirname "$(command -v psql)")
fi

mkdir -p "$CLEANUP_DIR"

upload_logs() {
  status=$1
  echo step=upload-cleanup-remote-node-${NODE_ID} status=$status
  aws s3 cp "$CLEANUP_DIR" "s3://$BUCKET/$S3_PREFIX" --recursive --region us-west-2 || true
}
trap 'status=$?; upload_logs "$status"; exit "$status"' EXIT

hostname > "$CLEANUP_DIR/hostname.log"
df -h "$NODE_DIR" > "$CLEANUP_DIR/df-before.log"
"$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres -At \
  -c "SELECT relname FROM pg_class WHERE relname LIKE '${PREFIX}%' ORDER BY relname;" \
  > "$CLEANUP_DIR/residue-before.log"

cat > "$CLEANUP_DIR/drop.sql" <<SQL
\set ON_ERROR_STOP on
DROP INDEX IF EXISTS ${REMOTE_INDEX};
DROP TABLE IF EXISTS ${REMOTE_PREFIX}_queries CASCADE;
DROP TABLE IF EXISTS ${REMOTE_PREFIX}_corpus CASCADE;
SQL

"$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres \
  -f "$CLEANUP_DIR/drop.sql" \
  > "$CLEANUP_DIR/drop.log" \
  2> "$CLEANUP_DIR/drop.stderr.log"
"$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres -At \
  -c "SELECT relname FROM pg_class WHERE relname LIKE '${PREFIX}%' ORDER BY relname;" \
  > "$CLEANUP_DIR/residue-after.log"
df -h "$NODE_DIR" > "$CLEANUP_DIR/df-after.log"
echo step=complete
