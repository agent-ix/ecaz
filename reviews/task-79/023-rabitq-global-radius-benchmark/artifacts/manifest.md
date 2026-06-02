# Task 79 Packet 023 Artifact Manifest

- head SHA: `3f51dadfe4b55098b8beee85a126b2b6c090ff2e`
- code under measurement: `7fe2a8de50956eaa020f5a8a80c085895eb0946f` (`Use RaBitQ block radius in global pruning`)
- task bucket: `reviews/task-79/023-rabitq-global-radius-benchmark/`
- timestamp: `2026-06-01T21:10:45-07:00`
- lane: local PG18, RaBitQ primary/default
- database: `task79_spire_candidate_surface`
- fixture: `task79_surface_100k`, 100000 corpus rows, 1000 query rows, 200-query benchmark limit
- storage format: `rabitq`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, `top_graph_search_list_size=96`, `leaf_block_rows=64`
- rerank mode: `rerank_width=25`
- isolated/shared surface: shared task79 table/index surface in `task79_spire_candidate_surface`
- installed backend SHA256: `897127bb91eddc3abe4dd215b1eb25dbeac7e4dca6ec4bbffd3b46e4471c4f37`
- suite config SHA256: `5076fa6ae6ba86ec8f9e2b7a610711e8e1c8e5e579d61964549d80d171ffeb1d`

## Commands

- `jq empty reviews/task-79/023-rabitq-global-radius-benchmark/suite-rabitq-global-radius.json`
- `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/023-rabitq-global-radius-benchmark/suite-rabitq-global-radius.json" reviews/task-79/023-rabitq-global-radius-benchmark/artifacts/suite-audit.log`
- `script -q -c "cargo build -p ecaz-cli" reviews/task-79/023-rabitq-global-radius-benchmark/artifacts/cargo-build-ecaz-cli.log`
- `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/023-rabitq-global-radius-benchmark/artifacts/install-ecaz-pg18.log`
- `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/023-rabitq-global-radius-benchmark/artifacts/pg18-restart.log restart -m fast`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/023-rabitq-global-radius-benchmark/suite-rabitq-global-radius.json --manifest-output reviews/task-79/023-rabitq-global-radius-benchmark/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/023-rabitq-global-radius-benchmark/artifacts/suite-dry-run.log`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/023-rabitq-global-radius-benchmark/suite-rabitq-global-radius.json --log-file reviews/task-79/023-rabitq-global-radius-benchmark/artifacts/suite-run.log`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/023-rabitq-global-radius-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/023-rabitq-global-radius-benchmark/artifacts/suite-status.log`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/023-rabitq-global-radius-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/023-rabitq-global-radius-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/023-rabitq-global-radius-benchmark/artifacts/suite-report.log`

## Artifact Index

- `suite-rabitq-global-radius.json`: checked-in suite config for the post-fix radius-bound global scoring sweep.
- `artifacts/suite-audit.log`: suite audit result; 7 steps passed audit.
- `artifacts/cargo-build-ecaz-cli.log`: CLI build log before installing the extension.
- `artifacts/install-ecaz-pg18.log`: PG18 extension install log; records backend SHA256.
- `artifacts/pg18-restart.log`: PG18 restart log after install.
- `artifacts/suite-dry-run.log`, `artifacts/suite-dry-run-manifest.json`: dry-run command and planned step manifest.
- `artifacts/suite-run.log`, `artifacts/suite-manifest.json`, `artifacts/results.jsonl`: full benchmark run and structured suite outputs.
- `artifacts/suite-status.log`: status summary; 7 completed, 0 failed, 0 skipped, 0 missing artifacts.
- `artifacts/suite-report.log`, `artifacts/report-results.jsonl`: parsed report and copied structured results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall/returned table.
- `artifacts/precheck-existing-task79-surface.log`: fixture/GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block64-global-radius.log`: index rebuild log; `ec_spire_ambuild_timing ... total_ms=13216`.
- `artifacts/pipeline-*.log`: raw pipeline logs for each measured row.
- `artifacts/funnel-*.jsonl`: per-query funnel output for each measured row.

## Key Results

All rows use RaBitQ, `nprobe=96`, `rerank_width=25`, 200 queries, production read profile enabled, local store overlap enabled, `leaf_block_rows=64`, and sampled probing disabled (`global_probe_blocks=0`, `sample_rows_per_block=0`).

| row | candidates | p50 | p95 | recall@10 | returned |
| --- | ---: | ---: | ---: | ---: | ---: |
| baseline global0 | 15506227 | 62.200 ms | 70.946 ms | 0.9975 | 2000 |
| radius global384 | 4798334 | 43.686 ms | 52.601 ms | 0.9310 | 2000 |
| radius global400 | 4997529 | 44.131 ms | 50.476 ms | 0.9355 | 2000 |
| radius global416 | 5197484 | 46.195 ms | 55.578 ms | 0.9385 | 2000 |
| radius global512 | 6394228 | 47.938 ms | 53.567 ms | 0.9565 | 2000 |

## Interpretation

This packet is negative benchmark evidence for using the raw RaBitQ block radius bound as the global block selector score. The implementation in packet 022 is mechanically correct and consistent with per-leaf scoring, but the bound is too loose for global ranking on the Task 79 surface.

Compared with packet 021's summary-only global selector, the radius-bound selector gets worse recall at similar or higher candidate counts:

- `global384`: packet 021 summary-only recall was 0.9675 at 4.685M candidates; radius-bound recall is 0.9310 at 4.798M candidates.
- `global512`: packet 021 summary-only recall was 0.9860 at 6.269M candidates; radius-bound recall is 0.9565 at 6.394M candidates.

The best row under the candidate gate is `global416`: 5.197M candidates, p50 46.195 ms, recall 0.9385. It clears neither the recall gate nor the 25 percent p50 improvement threshold from this packet's 62.200 ms baseline.

The conclusion is that raw radius-bound global ranking should not be pursued as the Task 79 latency fix. The remaining path needs richer block admission than a single mean score or loose radius upper bound, for example multi-representative summaries or another selector that can preserve true-neighbor-bearing blocks before applying the global cap.
