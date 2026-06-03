# Task 79 Packet 043 Artifact Manifest

- head SHA at measurement: `a0b630297c0a8b40742d4af252c6532fa91b3d0b`
- source checkpoint under review: `19845a140` (`Add SPIRE route-score prior for global block pruning`)
- task bucket: `reviews/task-79/043-rabitq-route-prior-global-cap`
- timestamp: `2026-06-02T06:58:34-07:00`
- lane: local PG18, `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- fixture: `task79_surface_100k` corpus/query surface from prior Task 79 local packets
- storage format: RaBitQ
- index surface: reused packet 040/041/042 `task79_surface_100k_idx`, block16, k=3 summaries
- rerank mode: `rerank_width=25`
- surface isolation: shared Task 79 local surface, not one-index-per-table
- AWS: not used

## Backend State

- Installed backend SHA256: `239f288f79d512ef43dbcadfe3181861d9d2465cc2c2a0ea5f9ad3c6e6ba2774`
- Install log: `artifacts/install-route-prior-ecaz-pg18.log`
- Restart log: `artifacts/pg18-restart-route-prior.log`

## Commands

- Focused tests:
  - `script -q -c "cargo test score_global_leaf_block_row_ranges_can_apply_route_prior --no-default-features --features pg18" reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/cargo-test-route-prior.log`
  - `script -q -c "cargo test select_global_leaf_block_row_ranges_spends_budget_across_leaves --no-default-features --features pg18" reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/cargo-test-zero-route-prior.log`
- Install and restart:
  - `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/install-route-prior-ecaz-pg18.log`
  - `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/pg18-restart-route-prior.log restart -m fast`
- Suite audit:
  - `target/debug/ecaz bench suite audit --config reviews/task-79/043-rabitq-route-prior-global-cap/suite-rabitq-route-prior-global-cap.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/suite-audit.log`
- Suite dry run:
  - `target/debug/ecaz bench suite run --dry-run --config reviews/task-79/043-rabitq-route-prior-global-cap/suite-rabitq-route-prior-global-cap.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/suite-dry-run.log`
- Suite run:
  - `target/debug/ecaz bench suite run --config reviews/task-79/043-rabitq-route-prior-global-cap/suite-rabitq-route-prior-global-cap.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/suite-manifest.json --results-output reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/results.jsonl --log-file reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/suite-run.log`
- Suite status:
  - `target/debug/ecaz bench suite status --manifest reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/suite-manifest.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/suite-status.log`
- Suite report:
  - `target/debug/ecaz bench suite report --manifest reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/suite-manifest.json --results-output reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/report-results.jsonl --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-79/043-rabitq-route-prior-global-cap/artifacts/suite-report.log`

## Key Results

All rows use nprobe `96`, `leaf_block_pruning_max_blocks_per_leaf=0`, `global_probe_blocks=0`, `sample_rows_per_block=0`, `sample_summary_prior_weight=0.8`, `summary_radius_weight=0.25`, block16, k=3 summaries, and `200` queries.

| row | global blocks | route prior weight | candidates | candidate delta vs 1216 | latency p50 ms | latency p95 ms | production p50 ms | production p95 ms | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| global1216_rp000 | 1216 | 0.00 | 3,877,368 | 0 | 56.145 | 65.566 | 51 | 63 | 0.9940 |
| global1024_rp000 | 1024 | 0.00 | 3,265,373 | -611,995 | 51.254 | 58.828 | 47 | 55 | 0.9920 |
| global896_rp000 | 896 | 0.00 | 2,857,174 | -1,020,194 | 51.176 | 61.618 | 47 | 58 | 0.9900 |
| global768_rp000 | 768 | 0.00 | 2,449,116 | -1,428,252 | 49.375 | 60.366 | 45 | 55 | 0.9865 |
| global1024_rp002 | 1024 | 0.02 | 3,265,165 | -612,203 | 51.782 | 63.126 | 47 | 57 | 0.9920 |
| global1024_rp005 | 1024 | 0.05 | 3,264,990 | -612,378 | 51.219 | 58.757 | 47 | 54 | 0.9920 |
| global1024_rp010 | 1024 | 0.10 | 3,264,907 | -612,461 | 50.933 | 61.204 | 46 | 53 | 0.9920 |
| global896_rp002 | 896 | 0.02 | 2,857,174 | -1,020,194 | 50.098 | 56.725 | 46 | 51 | 0.9900 |
| global896_rp005 | 896 | 0.05 | 2,857,058 | -1,020,310 | 50.413 | 58.340 | 46 | 53 | 0.9900 |
| global896_rp010 | 896 | 0.10 | 2,856,882 | -1,020,486 | 50.966 | 58.777 | 47 | 54 | 0.9900 |
| global768_rp005 | 768 | 0.05 | 2,449,011 | -1,428,357 | 48.318 | 53.177 | 44 | 50 | 0.9865 |
| global768_rp010 | 768 | 0.10 | 2,448,882 | -1,428,486 | 50.142 | 59.306 | 45 | 53 | 0.9865 |

Result: lowering the global block cap directly reduces candidate count and improves local latency, but it also lowers recall. Adding the route-score prior at weights `0.02`, `0.05`, and `0.10` does not recover recall at `1024`, `896`, or `768` global blocks.

## Artifacts

- `suite-rabitq-route-prior-global-cap.json`: checked-in SuiteConfig.
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
- `artifacts/pipeline-100k-rabitq-k3-block16-*.log`: pipeline outputs for each row.
- `artifacts/funnel-100k-rabitq-k3-block16-*.jsonl`: funnel outputs for each row.
- `artifacts/cargo-test-route-prior.log`: focused route-prior scoring unit test.
- `artifacts/cargo-test-zero-route-prior.log`: focused default behavior/global-budget unit test.
- `artifacts/install-route-prior-ecaz-pg18.log`: local PG18 extension install log.
- `artifacts/pg18-restart-route-prior.log`: local PG18 restart log.
