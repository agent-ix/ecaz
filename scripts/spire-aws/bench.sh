#!/usr/bin/env bash
# Phase 13b.9 — run the workload matrix for one tier via
# `ecaz bench suite run`. Mirrors the local `ecaz bench suite` surface
# so a single command kicks the full read-side matrix; write rows and
# fault drills are separate (see scripts/spire-aws/fault.sh).

set -euo pipefail

TIER="${1:?tier required (correctness|representative|stress)}"
TOPOLOGY="${2:?topology JSON path required}"
ARTIFACT_DIR="${3:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

COORD_HOST=$(jq -r '.coordinator.private_ip' "$TOPOLOGY")

case "$TIER" in
  correctness)   SUITE=scripts/spire-aws/suite-correctness.json ;;
  representative) SUITE=scripts/spire-aws/suite-representative.json ;;
  stress)        SUITE=scripts/spire-aws/suite-stress.json ;;
  *) echo "unknown tier: $TIER" >&2; exit 2 ;;
esac

ecaz bench suite run \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --config "$SUITE" \
  --manifest-output "$ARTIFACT_DIR/suite-manifest-${TIER}.json" \
  --results-output "$ARTIFACT_DIR/suite-results-${TIER}.jsonl"
