# Artifact Manifest

- head SHA: `9a7c48b11d5ed151141199a12ec1edb2ce092ae9`
- task bucket: `reviews/task-30/1059-spire-phase13e-representative-truth-corpus-harness`
- timestamp: `2026-05-28T22:19:00Z`
- lane: Phase 13e representative AWS benchmark harness, local validation only
- fixture: representative `ec_real_100k` staged corpus path injection
- storage format: rabitq
- rerank mode: default
- surface: shared-table representative AWS suite configs rendered locally; no AWS resources touched

## Artifacts

- `cargo-test-load-sources-tsv.log`
  - command: `cargo test -p ecaz-cli load_sources_tsv_file`
  - key result: `2 passed; 0 failed`

- `cargo-test-suite-recall-expansion.log`
  - command: `cargo test -p ecaz-cli expands_recall_with_defaults`
  - key result: `1 passed; 0 failed`

- `cargo-test-suite-spire-expansion.log`
  - command: `cargo test -p ecaz-cli expands_spire_pipeline_with_production_profile`
  - key result: `1 passed; 0 failed`

- `preflight-representative-performance.log`
  - command: `scripts/spire-aws/preflight-representative-performance.sh`
  - key result: `SPIRE representative performance preflight passed`

- `render-priority-suite.log`
  - command: `SPIRE_AWS_BENCH_RENDER_SUITE_ONLY=1 WORK_DIR=reviews/task-30/1059-spire-phase13e-representative-truth-corpus-harness/artifacts/render-check/work scripts/spire-aws/bench.sh representative-priority /dev/null reviews/task-30/1059-spire-phase13e-representative-truth-corpus-harness/artifacts/render-check`
  - key result: rendered `render-check/suite-representative-priority.json`

- `render-pooling-suite.log`
  - command: `SPIRE_AWS_BENCH_RENDER_SUITE_ONLY=1 WORK_DIR=reviews/task-30/1059-spire-phase13e-representative-truth-corpus-harness/artifacts/render-check/work scripts/spire-aws/bench.sh representative-pooling /dev/null reviews/task-30/1059-spire-phase13e-representative-truth-corpus-harness/artifacts/render-check`
  - key result: rendered `render-check/suite-representative-pooling.json`

- `render-priority-recall-truth-paths.log`
  - command: `jq -r '.steps[] | select(.kind=="recall") | [.name,.truth_corpus_file,.truth_cache_file] | @tsv' render-check/suite-representative-priority.json`
  - key result: both recall steps use `render-check/work/qdrant-dbpedia/prepared/ec_real_100k_corpus.tsv` plus packet-local truth cache files

- `render-spire-pipeline-truth-paths.log`
  - command: `jq -r '.steps[] | select(.kind=="spire-pipeline" and (.include_recall // false)) | [.name,.truth_corpus_file] | @tsv' render-check/suite-representative-priority.json render-check/suite-representative-pooling.json`
  - key result: all representative priority/pooling SPIRE pipeline recall-enabled steps use `render-check/work/qdrant-dbpedia/prepared/ec_real_100k_corpus.tsv`

- `render-check/suite-representative-priority.json`
  - command: output of render-only representative-priority suite
  - key result: durable rendered suite proving injected `truth_corpus_file` and recall `truth_cache_file`

- `render-check/suite-representative-pooling.json`
  - command: output of render-only representative-pooling suite
  - key result: durable rendered suite proving injected `truth_corpus_file`

