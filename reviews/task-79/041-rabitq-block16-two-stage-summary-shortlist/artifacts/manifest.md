# Task 79 Packet 041 Artifact Manifest

- head SHA: `74736bf9299f88a35afa2b81e5a034d6fcc43b3c`
- task bucket: `reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist`
- timestamp: `2026-06-02T05:44:33-07:00`
- lane: local PG18, `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- fixture: `task79_surface_100k` corpus/query surface from prior Task 79 local packets
- storage format: RaBitQ
- index surface: reused packet 040 `task79_surface_100k_idx`, block16, temporary k=3 build
- rerank mode: `rerank_width=25`
- surface isolation: shared Task 79 local surface, not one-index-per-table
- AWS: not used

## Backend State

- Experimental scanner backend installed for the suite:
  - patch artifact: `artifacts/two-stage-summary-shortlist.patch`
  - installed SHA256: `150c2882778a7d2a055f0f3c7ddb3bde2b546cb1ed2c74b79711eeac76f1fc01`
  - install log: `artifacts/install-two-stage-summary-shortlist-pg18.log`
  - restart log: `artifacts/pg18-restart-two-stage-command.log`
- After measurement, the temporary source patch was reverted and PG18 was restored to the clean backend:
  - clean installed SHA256: `210566e905947116d8d9aa6eb718d99368302aa02aca5e17edbc71da96e41a10`
  - install log: `artifacts/install-clean-ecaz-pg18.log`
  - restart log: `artifacts/pg18-clean-restart-command.log`

## Commands

- Focused tests:
  - `script -q -c "cargo test --no-default-features --features pg18 leaf_block_summary_representative_limit_scores_prefix_only" reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/cargo-test-prefix-representative.log`
  - `script -q -c "cargo test --no-default-features --features pg18 full_rescore_promotes_block_with_late_representative_hit" reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/cargo-test-full-rescore.log`
- Suite audit:
  - `target/debug/ecaz bench suite audit --config reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/suite-rabitq-block16-two-stage-summary-shortlist.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/suite-audit.log`
- Suite dry run:
  - `target/debug/ecaz bench suite run --dry-run --config reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/suite-rabitq-block16-two-stage-summary-shortlist.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/suite-dry-run.log`
- Suite run:
  - `target/debug/ecaz bench suite run --config reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/suite-rabitq-block16-two-stage-summary-shortlist.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/suite-manifest.json --results-output reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/results.jsonl --log-file reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/suite-run.log`
- Suite status:
  - `target/debug/ecaz bench suite status --manifest reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/suite-manifest.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/suite-status.log`
- Suite report:
  - `target/debug/ecaz bench suite report --manifest reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/suite-manifest.json --results-output reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/report-results.jsonl --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/041-rabitq-block16-two-stage-summary-shortlist/artifacts/suite-report.log`

## Key Results

All rows use nprobe `96`, global block cap `1216`, radius weight `0.25`, block16, k=3 summaries, and `200` queries.

| row | candidates | route_sum | object_bytes_sum | latency_p50_ms | latency_p95_ms | production_total_p50 | production_total_p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| full | 3,877,368 | 19,200 | 14,967,100,324 | 52.135 | 63.231 | 48 | 54 | 0.9940 |
| fp1 | 3,877,368 | 19,200 | 14,967,100,324 | 53.790 | 61.469 | 49 | 59 | 0.9940 |
| fp1_rescore1536 | 3,877,368 | 19,200 | 14,967,100,324 | 52.226 | 58.537 | 48 | 55 | 0.9940 |
| fp1_rescore2048 | 3,877,368 | 19,200 | 14,967,100,324 | 52.303 | 61.709 | 48 | 57 | 0.9940 |
| fp1_rescore3072 | 3,877,368 | 19,200 | 14,967,100,324 | 52.801 | 60.361 | 48 | 58 | 0.9940 |
| fp2_rescore2048 | 3,877,368 | 19,200 | 14,967,100,324 | 52.054 | 57.878 | 48 | 57 | 0.9940 |

Result: two-stage representative scoring does not reduce the row-candidate surface, the routed leaf/read surface, or p50. It is not a path to the Task 79 latency gate.

## Artifacts

- `suite-rabitq-block16-two-stage-summary-shortlist.json`: checked-in SuiteConfig.
- `artifacts/two-stage-summary-shortlist.patch`: temporary scanner patch adding first-pass representative and full-rescore GUCs.
- `artifacts/compact-results.tsv`: compact parsed result table.
- `artifacts/results.jsonl`: suite result stream.
- `artifacts/report-results.jsonl`: suite report result stream.
- `artifacts/suite-manifest.json`: executed suite manifest.
- `artifacts/suite-run.log`: full suite run transcript.
- `artifacts/suite-report.log`: suite report.
- `artifacts/suite-status.log`: suite status.
- `artifacts/suite-audit.log`: suite audit.
- `artifacts/suite-dry-run.log`, `artifacts/suite-dry-run-manifest.json`: dry-run evidence.
- `artifacts/precheck-existing-block16-k3-surface.log`: local corpus/index/GUC precheck.
- `artifacts/pipeline-*.log`: per-row pipeline logs.
- `artifacts/funnel-*.jsonl`: per-row funnel outputs.
- `artifacts/cargo-test-prefix-representative.log`: focused prefix representative unit test.
- `artifacts/cargo-test-full-rescore.log`: focused full-rescore unit test.
