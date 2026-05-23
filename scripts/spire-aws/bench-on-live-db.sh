#!/usr/bin/env bash
# Bench kNN latency against the existing ecaz-cloud DB instance via SSM.
# Substitutes @SCALE@ / @CORPUS@ / @QUERIES@ / @IDX@ into bench.sql per
# scale, uploads the rendered SQL via base64 in the SSM command body
# (avoids quoting / here-doc transport issues), runs psql -f, captures
# output.
#
# Live DB is the prior-bench-round leftover at i-04ce81ce1c10db4bc
# (m8g.2xlarge, AL2023, PG18.3, ecaz 0.1.1) with pre-loaded real_10k,
# real_100k_ivf_rabitq1_rerank, real_1m_ivf_rabitq1_rerank corpora.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REGION="${REGION:-us-west-2}"
DB_INSTANCE="${DB_INSTANCE:-i-04ce81ce1c10db4bc}"
DB_DATABASE="${DB_DATABASE:-tqvector_bench}"
ARTIFACT_DIR="${1:?artifact directory required}"
OUT="$ARTIFACT_DIR/bench-on-live-db"
mkdir -p "$OUT"

SQL_TEMPLATE="$SCRIPT_DIR/bench.sql"
test -f "$SQL_TEMPLATE"

run_scale() {
  local scale=$1 corpus=$2 queries=$3 idx=$4
  echo "=== bench scale=$scale corpus=$corpus idx=$idx ==="

  local rendered="$OUT/$scale-bench.sql"
  sed -e "s|@SCALE@|$scale|g" \
      -e "s|@CORPUS@|$corpus|g" \
      -e "s|@QUERIES@|$queries|g" \
      -e "s|@IDX@|$idx|g" \
      "$SQL_TEMPLATE" > "$rendered"

  local B64
  B64=$(base64 -w0 < "$rendered")

  # Single-line command: SSM splits multi-line shells into separate
  # invocations and breaks heredocs. Keep everything on one line via
  # `&&` and `bash -c "..."` so the parser stays out of our way.
  local SHELL_CMD="echo ${B64} | base64 -d > /tmp/bench-${scale}.sql && sudo -u postgres psql ${DB_DATABASE} -f /tmp/bench-${scale}.sql 2>&1"

  local CMD
  CMD=$(aws ssm send-command --region "$REGION" \
    --instance-ids "$DB_INSTANCE" \
    --document-name AWS-RunShellScript \
    --timeout-seconds 900 \
    --parameters "commands=[$(printf '%s' "$SHELL_CMD" | jq -Rs .)]" \
    --query Command.CommandId --output text)
  echo "ssm cmd: $CMD"

  # Poll until done — SSM waiters time out earlier than long benches.
  while :; do
    local STATUS
    STATUS=$(aws ssm get-command-invocation --region "$REGION" \
      --command-id "$CMD" --instance-id "$DB_INSTANCE" \
      --query Status --output text 2>/dev/null || echo Pending)
    [ "$STATUS" != "Pending" ] && [ "$STATUS" != "InProgress" ] && break
    sleep 5
  done

  aws ssm get-command-invocation --region "$REGION" --command-id "$CMD" --instance-id "$DB_INSTANCE" \
    > "$OUT/$scale-invocation.json"
  jq -r '.StandardOutputContent' "$OUT/$scale-invocation.json" > "$OUT/$scale-latency.log"
  jq -r '.StandardErrorContent'  "$OUT/$scale-invocation.json" > "$OUT/$scale-latency.err"
  local FINAL
  FINAL=$(jq -r '.Status' "$OUT/$scale-invocation.json")
  echo "$scale final status: $FINAL"
  echo "--- $scale stdout tail ---"
  tail -25 "$OUT/$scale-latency.log"
  echo "--- $scale stderr tail ---"
  tail -10 "$OUT/$scale-latency.err"
  echo
}

run_scale 10k  real_10k_ivf_rabitq1_corpus            real_10k_queries_shared              real_10k_ivf_rabitq1_idx
run_scale 100k real_100k_ivf_rabitq1_rerank_corpus    real_100k_ivf_rabitq1_rerank_queries real_100k_ivf_rabitq1_rerank_rabitq_idx
run_scale 1m   real_1m_ivf_rabitq1_rerank_corpus      real_1m_ivf_rabitq1_rerank_queries   real_1m_ivf_rabitq1_rerank_rabitq_idx

echo "DONE: $OUT"
grep -E 'BENCH-(RESULT|LATENCY-PCTL|SCALE|SIZE)' "$OUT"/*.log || true
