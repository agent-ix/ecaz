#!/usr/bin/env bash
set -euo pipefail

export AWS_DEFAULT_REGION=us-west-2

NODE_ID=3
PREFIX=task107_phase2_turboquant_1m_l1
REMOTE_PREFIX=${PREFIX}_node_${NODE_ID}
REMOTE_INDEX=${PREFIX}_remote_idx
NODE_DIR=/var/tmp/ecaz-task107-phase2-turboquant-1m-l1/remote-node-${NODE_ID}
BUCKET=ecaz-spire-aws-20260614203301860100000009
S3_PREFIX=task107/004/phase2-turboquant-1m-l1/direct-ssm-distributed
REMOTE_CORPUS=${NODE_DIR}/${REMOTE_PREFIX}_corpus.tsv
REMOTE_CORPUS_KEY=${S3_PREFIX}/remote-node-${NODE_ID}/${REMOTE_PREFIX}_corpus.tsv
LOG_KEY_PREFIX=${S3_PREFIX}/remote-load-node-${NODE_ID}
PG_BIN=/usr/pgsql-18/bin
if [ ! -x "$PG_BIN/psql" ]; then
  PG_BIN=$(dirname "$(command -v psql)")
fi

mkdir -p "$NODE_DIR"

upload_logs() {
  status=$1
  echo step=upload-remote-node-${NODE_ID} status=$status
  aws s3 cp "$NODE_DIR" "s3://$BUCKET/$LOG_KEY_PREFIX" \
    --recursive \
    --exclude '*_corpus.tsv' \
    --region us-west-2 || true
}
trap 'status=$?; upload_logs "$status"; exit "$status"' EXIT

echo step=preflight
hostname > "$NODE_DIR/hostname.log"
df -h "$NODE_DIR" > "$NODE_DIR/df-before.log"

echo step=download-corpus
aws s3 cp "s3://$BUCKET/$REMOTE_CORPUS_KEY" "$REMOTE_CORPUS" --region us-west-2
wc -l "$REMOTE_CORPUS" > "$NODE_DIR/corpus-row-count.log"

echo step=drop-existing-remote-prefix
cat > "$NODE_DIR/drop.sql" <<SQL
DROP INDEX IF EXISTS ${REMOTE_INDEX};
DROP TABLE IF EXISTS ${REMOTE_PREFIX}_queries CASCADE;
DROP TABLE IF EXISTS ${REMOTE_PREFIX}_corpus CASCADE;
SQL
"$PG_BIN/psql" -v ON_ERROR_STOP=1 -h 127.0.0.1 -p 5432 -U ecaz_coord -d postgres \
  -f "$NODE_DIR/drop.sql" > "$NODE_DIR/drop.log" 2>&1

echo step=load-remote
/usr/local/bin/ecaz corpus load \
  --host 127.0.0.1 \
  --port 5432 \
  --user ecaz_coord \
  --database postgres \
  --profile ec_spire \
  --prefix "$REMOTE_PREFIX" \
  --dim 1536 \
  --bits 4 \
  --seed 42 \
  --corpus-file "$REMOTE_CORPUS" \
  --corpus-only \
  --storage-format turboquant \
  --index-name "$REMOTE_INDEX" \
  --reloption local_store_count=1 \
  --log-file "$NODE_DIR/load.log"

echo step=inspect-remote
/usr/local/bin/ecaz corpus inspect \
  --host 127.0.0.1 \
  --port 5432 \
  --user ecaz_coord \
  --database postgres \
  --prefix "$REMOTE_PREFIX" \
  --log-file "$NODE_DIR/inspect.log"

df -h "$NODE_DIR" > "$NODE_DIR/df-after.log"
echo step=complete
