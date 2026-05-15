#!/usr/bin/env bash
# Phase 13b.7 — load a dataset tier onto the coordinator.
#
# Tiers:
#   correctness     synthetic 10k via `ecaz corpus generate`
#   representative  qdrant-dbpedia 1M via `ecaz corpus fetch`/`prepare`
#   stress          synthetic 10M (reviewer-gated, see Phase 13a.9)

set -euo pipefail

TIER="${1:?tier required (correctness|representative|stress)}"
TOPOLOGY="${2:?topology JSON path required}"
ARTIFACT_DIR="${3:?artifact directory required}"
mkdir -p "$ARTIFACT_DIR"

COORD_HOST=$(jq -r '.coordinator.private_ip' "$TOPOLOGY")
WORK_DIR="${WORK_DIR:-/var/lib/ecaz}"

case "$TIER" in
  correctness)
    PREFIX=ec_spire_aws_synth_10k
    ecaz corpus generate --rows 10000 --dim 1536 \
      --output "$WORK_DIR/${PREFIX}_corpus.tsv"
    ecaz corpus generate --rows 100 --dim 1536 \
      --output "$WORK_DIR/${PREFIX}_queries.tsv"
    ecaz corpus load \
      --host "$COORD_HOST" --user ecaz_coord --database postgres \
      --prefix "$PREFIX" \
      --corpus-file "$WORK_DIR/${PREFIX}_corpus.tsv" \
      --queries-file "$WORK_DIR/${PREFIX}_queries.tsv" \
      --profile ec_spire --dim 1536 --bits 4 --seed 42 \
      --log-output "$ARTIFACT_DIR/corpus-load-${TIER}.log"
    ;;
  representative)
    PREFIX=ec_spire_aws_repr_1m
    ecaz corpus fetch \
      --dataset qdrant-dbpedia-openai3-large-1536-1m \
      --output-dir "$WORK_DIR/qdrant-dbpedia/"
    ecaz corpus prepare \
      --profile ec_hnsw_real_100k \
      --parquet "$WORK_DIR/qdrant-dbpedia/data/0000.parquet" \
      --output-dir "$WORK_DIR/qdrant-dbpedia/prepared/" \
      --dim 1536 \
      --source-dataset qdrant-dbpedia-openai3-large-1536-1m
    ecaz corpus load \
      --host "$COORD_HOST" --user ecaz_coord --database postgres \
      --prefix "$PREFIX" \
      --corpus-file "$WORK_DIR/qdrant-dbpedia/prepared/${PREFIX}_corpus.tsv" \
      --queries-file "$WORK_DIR/qdrant-dbpedia/prepared/${PREFIX}_queries.tsv" \
      --profile ec_spire --dim 1536 --bits 4 --seed 42 \
      --log-output "$ARTIFACT_DIR/corpus-load-${TIER}.log"
    ;;
  stress)
    PREFIX=ec_spire_aws_synth_10m
    ecaz corpus generate --rows 10000000 --dim 1536 \
      --output "$WORK_DIR/${PREFIX}_corpus.tsv"
    ecaz corpus generate --rows 10000 --dim 1536 \
      --output "$WORK_DIR/${PREFIX}_queries.tsv"
    ecaz corpus load \
      --host "$COORD_HOST" --user ecaz_coord --database postgres \
      --prefix "$PREFIX" \
      --corpus-file "$WORK_DIR/${PREFIX}_corpus.tsv" \
      --queries-file "$WORK_DIR/${PREFIX}_queries.tsv" \
      --profile ec_spire --dim 1536 --bits 4 --seed 42 --chunked \
      --log-output "$ARTIFACT_DIR/corpus-load-${TIER}.log"
    ;;
  *)
    echo "unknown tier: $TIER" >&2; exit 2 ;;
esac

ecaz corpus inspect \
  --host "$COORD_HOST" --user ecaz_coord --database postgres \
  --prefix "$PREFIX" \
  --log-output "$ARTIFACT_DIR/corpus-inspect-${TIER}.log"
