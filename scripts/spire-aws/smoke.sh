#!/usr/bin/env bash
# Phase 13b.8 — smoke verification against the Correctness corpus.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

TOPOLOGY="${1:?topology JSON path required}"
ARTIFACT_DIR="${2:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

COORD_HOST=$(jq -r '.coordinator.private_ip' "$TOPOLOGY")
PREFIX="${PREFIX:-ec_spire_aws_synth_10k}"

ecaz dev sql \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --file scripts/spire-aws/smoke-customscan-read.sql \
  --set "prefix=$PREFIX" \
  --log-output "$ARTIFACT_DIR/smoke-customscan-read.log"

ecaz dev sql \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --sql "SELECT * FROM ec_spire_remote_search_production_read_profile(format('%s_idx', '${PREFIX}')::regclass, (SELECT embedding FROM ${PREFIX}_queries WHERE vec_id = 0)::real[], 10)" \
  --log-output "$ARTIFACT_DIR/production-read-profile-smoke.log"

ecaz bench spire-pipeline \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --prefix "$PREFIX" \
  --queries-limit 5 --sweep 8,16,32 \
  --include-remote --consistency-mode epoch \
  --include-cost-snapshot --include-query-metrics \
  --log-output "$ARTIFACT_DIR/bench-spire-pipeline-smoke.log"
