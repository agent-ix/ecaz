#!/usr/bin/env bash
# Single-node SPIRE-extension smoke + bench on the coordinator via SSM
# send-command. Avoids the laptop->VPC connectivity gap that blocks
# scripts/spire-aws/{register,load,smoke,bench,fault}.sh from running
# end-to-end today (the SSM tunnel approach needs session-manager-plugin
# which the operator's laptop lacks).
#
# What it does:
#   1. Generates a 10k synthetic corpus on the coordinator (ecaz corpus
#      generate writes TSVs locally).
#   2. Loads it into PG with profile ec_ivf storage_format=rabitq bits=4.
#   3. Runs ecaz bench latency to record per-query timings.
#   4. Streams results back via SSM to $ARTIFACT_DIR.
#
# Args:
#   $1  Path to topology JSON (from `terraform output -json topology`)
#   $2  Artifact directory for logs

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

TOPOLOGY="${1:?topology JSON path required}"
ARTIFACT_DIR="${2:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

REGION=$(jq -r '.region' "$TOPOLOGY")
BUCKET=$(jq -r '.artifact_bucket' "$TOPOLOGY")
COORD_ID=$(jq -r '.coordinator.instance_id' "$TOPOLOGY")

# The big script that runs on the coordinator. PG is reachable locally
# via Unix socket; ecaz CLI was installed by bootstrap-node.sh to
# /usr/local/bin/ecaz.
read -r -d '' COORD_SCRIPT <<'EOSCRIPT' || true
#!/usr/bin/env bash
set -euxo pipefail
export PATH=/usr/local/bin:/usr/bin:/usr/pgsql-18/bin:$PATH

sudo -u postgres bash -lc '
  set -eux
  cd /var/lib/pgsql/build/ecaz

  mkdir -p /var/lib/pgsql/spire-smoke
  ecaz corpus generate --n 10000 --dim 1536 --seed 42 \
    --output /var/lib/pgsql/spire-smoke/corpus.tsv
  ecaz corpus generate --n 100 --dim 1536 --seed 4242 --start-id 100000 --kind queries \
    --output /var/lib/pgsql/spire-smoke/queries.tsv

  ecaz corpus load --prefix spire_aws_smoke_10k --profile ec_ivf \
    --bits 4 --storage-format rabitq \
    --reloption nlists=128 \
    --corpus-file /var/lib/pgsql/spire-smoke/corpus.tsv \
    --queries-file /var/lib/pgsql/spire-smoke/queries.tsv

  ecaz bench recall --prefix spire_aws_smoke_10k --profile ec_ivf \
    --k 10 --nprobe 16 \
    --log-output /var/lib/pgsql/spire-smoke/recall.log || true

  ecaz bench latency --prefix spire_aws_smoke_10k --profile ec_ivf \
    --iterations 200 --concurrency 1 --nprobe 16 \
    --log-output /var/lib/pgsql/spire-smoke/latency.log || true

  ecaz corpus list || true
  ecaz corpus inspect --prefix spire_aws_smoke_10k || true

  psql -c "SELECT extname, extversion FROM pg_extension WHERE extname='\''ecaz'\''"
  psql -c "\\d+ spire_aws_smoke_10k" || true
'
EOSCRIPT

echo "Dispatching single-node smoke to coordinator $COORD_ID ..."
CMD_ID=$(aws ssm send-command \
  --region "$REGION" \
  --document-name "AWS-RunShellScript" \
  --instance-ids "$COORD_ID" \
  --timeout-seconds 1800 \
  --parameters "commands=[$(jq -Rs . <<< "$COORD_SCRIPT")]" \
  --output-s3-bucket-name "$BUCKET" \
  --output-s3-key-prefix "spire-aws/single-node-smoke" \
  --comment "spire-aws single-node smoke" \
  --query "Command.CommandId" --output text)

echo "ssm command id: $CMD_ID" | tee "$ARTIFACT_DIR/single-node-smoke.cmd.log"

aws ssm wait command-executed --region "$REGION" --command-id "$CMD_ID" --instance-id "$COORD_ID" --cli-read-timeout 2000 --cli-connect-timeout 60
INVOKE=$(aws ssm get-command-invocation --region "$REGION" --command-id "$CMD_ID" --instance-id "$COORD_ID")
echo "$INVOKE" > "$ARTIFACT_DIR/single-node-smoke.json"
echo "$INVOKE" | jq -r '.StandardOutputContent' > "$ARTIFACT_DIR/single-node-smoke.stdout.log"
echo "$INVOKE" | jq -r '.StandardErrorContent'  > "$ARTIFACT_DIR/single-node-smoke.stderr.log"
STATUS=$(echo "$INVOKE" | jq -r '.Status')
echo "single-node-smoke status: $STATUS"
test "$STATUS" = "Success"

# Pull the bench logs from the coordinator filesystem via a second SSM call.
aws ssm send-command \
  --region "$REGION" \
  --document-name "AWS-RunShellScript" \
  --instance-ids "$COORD_ID" \
  --parameters "commands=[\"sudo aws s3 cp --recursive /var/lib/pgsql/spire-smoke s3://${BUCKET}/spire-aws/smoke-logs/ || true\"]" \
  --query "Command.CommandId" --output text \
  > "$ARTIFACT_DIR/smoke-logs-upload.cmd"

# Download from S3 to the local artifact dir.
SUBCMD_ID=$(cat "$ARTIFACT_DIR/smoke-logs-upload.cmd")
aws ssm wait command-executed --region "$REGION" --command-id "$SUBCMD_ID" --instance-id "$COORD_ID" --cli-read-timeout 300 || true
aws s3 cp --recursive "s3://${BUCKET}/spire-aws/smoke-logs/" "$ARTIFACT_DIR/smoke-logs/" || true

echo "single-node-smoke complete: $ARTIFACT_DIR"
