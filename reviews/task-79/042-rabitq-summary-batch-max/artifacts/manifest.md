# Task 79 Packet 042 Artifact Manifest

- head SHA at measurement: `a75bc4b737e3165daa64143dd9316c3953b5df48`
- source checkpoint under review: `6ece24263` (`Optimize RaBitQ summary max scoring`)
- task bucket: `reviews/task-79/042-rabitq-summary-batch-max`
- timestamp: `2026-06-02T06:05:24-07:00`
- lane: local PG18, `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- fixture: `task79_surface_100k` corpus/query surface from prior Task 79 local packets
- storage format: RaBitQ
- index surface: reused packet 040/041 `task79_surface_100k_idx`, block16, k=3 summaries
- rerank mode: `rerank_width=25`
- surface isolation: shared Task 79 local surface, not one-index-per-table
- AWS: not used

## Backend State

- Installed backend SHA256: `da1b4b0238b03e801977d2b3b7891143a86a1874b84639389bee836f8391baf2`
- Install log: `artifacts/install-batch-max-ecaz-pg18.log`
- Restart log: `artifacts/pg18-restart-batch-max.log`

## Commands

- Focused tests:
  - `script -q -c "cargo test batch_max_prevalidated_matches_scalar_max --no-default-features --features pg18" reviews/task-79/042-rabitq-summary-batch-max/artifacts/cargo-test-batch-max.log`
  - `script -q -c "cargo test leaf_block_summary_scores_best_representative_payload --no-default-features --features pg18" reviews/task-79/042-rabitq-summary-batch-max/artifacts/cargo-test-summary-best-representative.log`
- Install and restart:
  - `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/042-rabitq-summary-batch-max/artifacts/install-batch-max-ecaz-pg18.log`
  - `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/042-rabitq-summary-batch-max/artifacts/pg18-restart-batch-max.log restart -m fast`
- Suite audit:
  - `target/debug/ecaz bench suite audit --config reviews/task-79/042-rabitq-summary-batch-max/suite-rabitq-summary-batch-max.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/042-rabitq-summary-batch-max/artifacts/suite-audit.log`
- Suite dry run:
  - `target/debug/ecaz bench suite run --dry-run --config reviews/task-79/042-rabitq-summary-batch-max/suite-rabitq-summary-batch-max.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-79/042-rabitq-summary-batch-max/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/042-rabitq-summary-batch-max/artifacts/suite-dry-run.log`
- Suite run:
  - `target/debug/ecaz bench suite run --config reviews/task-79/042-rabitq-summary-batch-max/suite-rabitq-summary-batch-max.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-79/042-rabitq-summary-batch-max/artifacts/suite-manifest.json --results-output reviews/task-79/042-rabitq-summary-batch-max/artifacts/results.jsonl --log-file reviews/task-79/042-rabitq-summary-batch-max/artifacts/suite-run.log`
- Suite status:
  - `target/debug/ecaz bench suite status --manifest reviews/task-79/042-rabitq-summary-batch-max/artifacts/suite-manifest.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/042-rabitq-summary-batch-max/artifacts/suite-status.log`
- Suite report:
  - `target/debug/ecaz bench suite report --manifest reviews/task-79/042-rabitq-summary-batch-max/artifacts/suite-manifest.json --results-output reviews/task-79/042-rabitq-summary-batch-max/artifacts/report-results.jsonl --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/042-rabitq-summary-batch-max/artifacts/suite-report.log`

## Key Results

The measured row uses nprobe `96`, global block cap `1216`, radius weight `0.25`, block16, k=3 summaries, and `200` queries.

| row | candidates | route_sum | object_bytes_sum | latency_p50_ms | latency_p95_ms | production_total_p50 | production_total_p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| batch_max | 3,877,368 | 19,200 | 14,967,100,324 | 51.917 | 62.046 | 48 | 57 | 0.9940 |

Packet 041 full-scoring comparison on the same local surface: 3,877,368 candidates, 19,200 routes, 14,967,100,324 object bytes, latency p50 52.135ms, latency p95 63.231ms, production total p50 48ms, production total p95 54ms, recall@10 0.9940.

Result: the batch-max scorer keeps behavior and candidate/read surfaces unchanged. Local query p50 moved from 52.135ms to 51.917ms versus packet 041 full scoring, which is directionally positive but too small to close the Task 79 latency gap or to count as a candidate-surface reduction.

## Artifacts

- `suite-rabitq-summary-batch-max.json`: checked-in SuiteConfig.
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
- `artifacts/pipeline-100k-rabitq-k3-block16-global1216-batch-max.log`: pipeline output.
- `artifacts/funnel-100k-rabitq-k3-block16-global1216-batch-max.jsonl`: funnel output.
- `artifacts/cargo-test-batch-max.log`: focused RaBitQ batch-max equality unit test.
- `artifacts/cargo-test-summary-best-representative.log`: focused SPIRE multi-representative summary unit test.
- `artifacts/install-batch-max-ecaz-pg18.log`: local PG18 extension install log.
- `artifacts/pg18-restart-batch-max.log`: local PG18 restart log.
