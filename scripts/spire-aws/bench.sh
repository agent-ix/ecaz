#!/usr/bin/env bash
# Phase 13b.9 — run the workload matrix for one tier via
# `ecaz bench suite run`. Mirrors the local `ecaz bench suite` surface
# so a single command kicks the full read-side matrix; write rows and
# fault drills are separate (see scripts/spire-aws/fault.sh).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${REPO_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
cd "$REPO_ROOT"

TIER="${1:?tier required (correctness|representative-priority|representative|representative-pooling|stress)}"
TOPOLOGY="${2:?topology JSON path required}"
ARTIFACT_DIR="${3:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

ECAZ_BIN="${ECAZ_BIN:-ecaz}"
BENCH_PREFIX="${SPIRE_AWS_BENCH_PREFIX:-${PREFIX:-}}"
PREPARED_PREFIX="${SPIRE_AWS_REPRESENTATIVE_PREPARED_PREFIX:-ec_real_100k}"

case "$TIER" in
  correctness)   SUITE=scripts/spire-aws/suite-correctness.json ;;
  representative-priority) SUITE="${SPIRE_AWS_REPRESENTATIVE_PRIORITY_SUITE:-scripts/spire-aws/suite-representative-priority.json}" ;;
  representative) SUITE="${SPIRE_AWS_REPRESENTATIVE_SUITE:-scripts/spire-aws/suite-representative.json}" ;;
  representative-pooling) SUITE="${SPIRE_AWS_REPRESENTATIVE_POOLING_SUITE:-scripts/spire-aws/suite-representative-pooling.json}" ;;
  stress)        SUITE=scripts/spire-aws/suite-stress.json ;;
  *) echo "unknown tier: $TIER" >&2; exit 2 ;;
esac

RUN_SUITE="$ARTIFACT_DIR/suite-${TIER}.json"
case "$TIER" in
  representative|representative-priority|representative-pooling)
    truth_corpus_file="${SPIRE_AWS_REPRESENTATIVE_TRUTH_CORPUS_FILE:-${WORK_DIR:-$ARTIFACT_DIR/work}/qdrant-dbpedia/prepared/${PREPARED_PREFIX}_corpus.tsv}"
    truth_cache_dir="${SPIRE_AWS_REPRESENTATIVE_TRUTH_CACHE_DIR:-$ARTIFACT_DIR/truth-cache}"
    jq \
      --arg artifact_dir "$ARTIFACT_DIR" \
      --arg truth_corpus_file "$truth_corpus_file" \
      --arg truth_cache_dir "$truth_cache_dir" \
      --arg bench_prefix "$BENCH_PREFIX" \
      '
        .artifact_dir = $artifact_dir
        | .steps |= map(
            if $bench_prefix != "" and has("prefix") then
              .prefix = $bench_prefix
            else
              .
            end
          )
        | .steps |= map(
            if .kind == "recall" then
              .truth_corpus_file = $truth_corpus_file
              | .truth_cache_file = (
                  $truth_cache_dir + "/" + (.name | gsub("[^A-Za-z0-9_.-]"; "_")) + ".json"
                )
            elif .kind == "spire-pipeline" and (.include_recall // false) then
              .truth_corpus_file = $truth_corpus_file
            else
              .
            end
          )
      ' "$SUITE" > "$RUN_SUITE"
    ;;
  *)
    jq --arg artifact_dir "$ARTIFACT_DIR" '.artifact_dir = $artifact_dir' "$SUITE" > "$RUN_SUITE"
    ;;
esac

if [[ "${SPIRE_AWS_BENCH_RENDER_SUITE_ONLY:-0}" == "1" ]]; then
  printf 'rendered_suite=%s\n' "$RUN_SUITE"
  exit 0
fi

COORD_HOST=$(jq -r '.coordinator.operator_host // .coordinator.private_ip' "$TOPOLOGY")
COORD_PORT=$(jq -r '.coordinator.operator_port // 5432' "$TOPOLOGY")

"$ECAZ_BIN" bench suite run \
  --host "$COORD_HOST" --port "$COORD_PORT" --user ecaz_coord --database postgres \
  --config "$RUN_SUITE" \
  --manifest-output "$ARTIFACT_DIR/suite-manifest-${TIER}.json" \
  --results-output "$ARTIFACT_DIR/suite-results-${TIER}.jsonl"
