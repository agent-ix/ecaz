#!/usr/bin/env bash
# Phase 13b.8 — smoke verification against the Correctness corpus.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

TOPOLOGY="${1:?topology JSON path required}"
ARTIFACT_DIR="${2:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

COORD_HOST=$(jq -r '.coordinator.operator_host // .coordinator.private_ip' "$TOPOLOGY")
COORD_PORT=$(jq -r '.coordinator.operator_port // 5432' "$TOPOLOGY")
PREFIX="${PREFIX:-ec_spire_aws_synth_10k}"
ECAZ_BIN="${ECAZ_BIN:-ecaz}"

"$ECAZ_BIN" dev sql \
  --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
  --file scripts/spire-aws/smoke-customscan-read.sql \
  --set "prefix=$PREFIX" \
  --log-output "$ARTIFACT_DIR/smoke-customscan-read.log"

"$ECAZ_BIN" dev sql \
  --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
  --sql "SET enable_seqscan = off; SET enable_indexscan = off; SELECT * FROM ec_spire_remote_search_production_read_profile(format('%s_idx', '${PREFIX}')::regclass, (SELECT source FROM ${PREFIX}_queries ORDER BY id LIMIT 1)::real[], 10)" \
  --log-output "$ARTIFACT_DIR/production-read-profile-smoke.log"

"$ECAZ_BIN" bench spire-pipeline \
  --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
  --prefix "$PREFIX" \
  --queries-limit 5 --sweep 8,16,32 \
  --include-remote --consistency-mode epoch \
  --include-cost-snapshot --include-query-metrics \
  --include-recall --include-production-read-profile --production-read-only \
  --remote-tuple-transport pg_binary_attr_v1 --query-metric-k 10 \
  --log-output "$ARTIFACT_DIR/bench-spire-pipeline-smoke.log"
