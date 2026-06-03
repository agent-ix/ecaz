# Task 79 Packet 040 Artifact Manifest

- head SHA: `db7dceb26a93f6e0947039223bcd52bd7d8fa0a0`
- task bucket: `reviews/task-79/040-rabitq-k3-block16-candidate-surface/`
- timestamp: `2026-06-02T05:00:09-07:00`
- lane: local PG18, Intel local, RaBitQ primary, 100k real corpus, 200 queries
- storage format: `rabitq`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, `top_graph_enabled=1`, `top_graph_search_list_size=96`, `leaf_block_rows=16`
- rerank mode: `rerank_width=25`, retained `5,000`, returned `2,000`
- surface isolation: shared Task 79 local surface `task79_spire_candidate_surface`

## Source State

- temporary k=3 backend patch artifact: `artifacts/k3-block16-cluster-mean.patch`
- patch sha256: `b8f5f08f78c69f1aeea38b86b7fa330d89ccb970679690e3135ab0d9b55bebf0`
- temporary installed backend sha256: `53677f3c6a9196a496017cddf3104d706295b9c45dee9bb74ab41815e961a4be`
- clean backend restored after measurement sha256: `210566e905947116d8d9aa6eb718d99368302aa02aca5e17edbc71da96e41a10`

The temporary k=3 source patch was reverted after measurement. Local PG18 was restarted after reinstalling the clean backend.

## Suite

- suite config: `suite-rabitq-k3-block16-candidate-surface.json`
- suite config sha256: `176e9b7b71a52e2cef2efe1fde5983c9ea35c41da7f7bda76f38d94407844ccd`
- command:
  `target/debug/ecaz bench suite run --config reviews/task-79/040-rabitq-k3-block16-candidate-surface/suite-rabitq-k3-block16-candidate-surface.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/suite-manifest.json --results-output reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/results.jsonl --log-file reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/suite-run.log`

## Validation Commands

- focused k=3 summary test:
  `script -q -c "cargo test --no-default-features --features pg18 leaf_block_summary" reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/cargo-test-k3-leaf-block.log`
- temporary k=3 backend install:
  `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/install-k3-block16-ecaz-pg18.log`
- clean backend restore:
  `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/install-clean-ecaz-pg18.log`
- suite audit:
  `target/debug/ecaz bench suite audit --config reviews/task-79/040-rabitq-k3-block16-candidate-surface/suite-rabitq-k3-block16-candidate-surface.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/suite-audit.log`
- suite dry-run:
  `target/debug/ecaz bench suite run --dry-run --config reviews/task-79/040-rabitq-k3-block16-candidate-surface/suite-rabitq-k3-block16-candidate-surface.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/suite-dry-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/suite-manifest.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/suite-manifest.json --results-output reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/report-results.jsonl --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/040-rabitq-k3-block16-candidate-surface/artifacts/suite-report.log`

## Key Results

From `artifacts/compact-results.tsv`:

| row | global_blocks | candidates | latency_p50_ms | latency_p95_ms | recall_at_10 | gate |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| k3_block16 | 1024 | 3,265,373 | 50.658 | 58.404 | 0.9920 | fail_recall_p50 |
| k3_block16 | 1216 | 3,877,368 | 52.643 | 62.906 | 0.9940 | fail_p50 |
| k3_block16 | 1280 | 4,081,213 | 54.166 | 62.936 | 0.9950 | fail_p50 |
| k3_block16 | 1536 | 4,897,128 | 55.530 | 61.944 | 0.9960 | fail_p50 |
| k3_block16 | 1664 | 5,304,964 | 56.407 | 67.552 | 0.9965 | fail_candidate_p50 |

## Artifacts

- `artifacts/compact-results.tsv`: compact gate table cited by `request.md`.
- `artifacts/results.jsonl`: normalized suite result rows.
- `artifacts/report-results.jsonl`: normalized report result rows.
- `artifacts/suite-manifest.json`: suite manifest.
- `artifacts/suite-run.log`: full suite run log.
- `artifacts/suite-report.log`: generated suite report.
- `artifacts/suite-status.log`: suite completion status.
- `artifacts/suite-audit.log`: suite audit output.
- `artifacts/suite-dry-run.log`: dry-run expansion output.
- `artifacts/suite-dry-run-manifest.json`: dry-run manifest.
- `artifacts/precheck-existing-task79-surface.log`: local corpus/query precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block16-k3.log`: block16 k=3 index rebuild log.
- `artifacts/pipeline-100k-rabitq-k3-block16-global*.log`: per-cap pipeline logs.
- `artifacts/funnel-100k-rabitq-k3-block16-global*.jsonl`: per-cap funnel rows.
- `artifacts/cargo-test-k3-leaf-block.log`: focused leaf-block summary test log.
- `artifacts/install-k3-block16-ecaz-pg18.log`: temporary backend install log.
- `artifacts/install-clean-ecaz-pg18.log`: clean backend reinstall log.
- `artifacts/pg18-restart-command.log`: restart after temporary backend install.
- `artifacts/pg18-clean-restart-command.log`: restart after clean backend restore.
