# Review Request: SPIRE Phase 13e Representative Truth Corpus Harness

## Summary

This fixes the Phase 13e representative benchmark harness issue exposed in packet `1058`: the real AWS setup was valid and had real remote data, but the full representative suite entered a recall path that fetched the entire coordinator corpus table over the SSM tunnel before measuring.

The change keeps AWS clusters reusable for targeted validation by moving exact-truth corpus loading to the staged local TSV:

- `ecaz bench recall` now accepts `--truth-corpus-file`.
- `ecaz bench spire-pipeline --include-recall` now accepts `--truth-corpus-file`.
- `ecaz bench suite` expands `truth_corpus_file` for recall and SPIRE pipeline steps.
- `scripts/spire-aws/bench.sh` injects the representative staged corpus path into representative priority/pooling suites and adds per-recall truth cache files.
- `bench.sh` has a render-only mode so this can be checked locally without touching AWS.
- Representative preflight now guards that the bench script contains the truth-corpus and render-only wiring.

Code commit: `9a7c48b11d5ed151141199a12ec1edb2ce092ae9`

## Evidence

See `artifacts/manifest.md`.

Key local validation:

- TSV truth corpus loader tests: `2 passed; 0 failed`
- recall suite expansion test: `1 passed; 0 failed`
- SPIRE pipeline suite expansion test: `1 passed; 0 failed`
- representative preflight passed
- render-only priority suite shows both recall steps using:
  `.../work/qdrant-dbpedia/prepared/ec_real_100k_corpus.tsv`
- render-only priority and pooling suites show all recall-enabled SPIRE pipeline steps using:
  `.../work/qdrant-dbpedia/prepared/ec_real_100k_corpus.tsv`

## Scope

This is harness-only. It does not change SPIRE placement, remote execution, tuple transport, or pooling behavior. It removes the unnecessary full-corpus SQL export from the representative benchmark path so the next AWS run can reuse a good loaded cluster instead of rebuilding for an unrelated harness fix.

