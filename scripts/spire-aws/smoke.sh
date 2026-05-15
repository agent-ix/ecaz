#!/usr/bin/env bash
# Phase 13b.8 — smoke verification against the Correctness corpus.

set -euo pipefail

TOPOLOGY="${1:?topology JSON path required}"
ARTIFACT_DIR="${2:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

COORD_HOST=$(jq -r '.coordinator.private_ip' "$TOPOLOGY")

ecaz dev sql \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --file scripts/spire-aws/smoke-customscan-read.sql \
  --log-output "$ARTIFACT_DIR/smoke-customscan-read.log"

ecaz bench spire-pipeline \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --prefix ec_spire_aws_synth_10k \
  --queries-limit 5 --sweep 8,16,32 \
  --include-remote --consistency-mode epoch \
  --include-cost-snapshot --include-query-metrics \
  --log-output "$ARTIFACT_DIR/bench-spire-pipeline-smoke.log"
