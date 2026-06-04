# Task 79 Packet 021 Artifact Manifest

- head SHA: `02ab04f474c54816fbc3cb3fe2bf579733df35e5`
- code under measurement: `b3200191878f1a5f06011423157e6e5ef7a6297d` (`Add summary-prior sampled block scoring`)
- task bucket: `reviews/task-79/021-rabitq-rerank-width-global-benchmark/`
- timestamp: `2026-06-01T20:39:32-07:00`
- lane: local PG18, RaBitQ primary/default
- database: `task79_spire_candidate_surface`
- fixture: `task79_surface_100k`, 100000 corpus rows, 1000 query rows, 200-query benchmark limit
- storage format: `rabitq`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, `top_graph_search_list_size=96`, `leaf_block_rows=64`
- rerank mode: explicit per-row `rerank_width` sweep at 25, 50, 100, 200, and 500
- isolated/shared surface: shared task79 table/index surface in `task79_spire_candidate_surface`
- installed backend SHA256: `cf16846368208390cb2fe6fd46a42e6b85a6ed291ef87ab3d19b4d84f67957e0`
- suite config SHA256: `ff8221ef25692de8c9340d0b45519c69da752a8ac7f590a0f1a51fc8ecdfdf0e`

## Commands

- `jq empty reviews/task-79/021-rabitq-rerank-width-global-benchmark/suite-rabitq-rerank-width-global.json`
- `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/021-rabitq-rerank-width-global-benchmark/suite-rabitq-rerank-width-global.json" reviews/task-79/021-rabitq-rerank-width-global-benchmark/artifacts/suite-audit.log`
- `script -q -c "cargo build -p ecaz-cli" reviews/task-79/021-rabitq-rerank-width-global-benchmark/artifacts/cargo-build-ecaz-cli.log`
- `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/021-rabitq-rerank-width-global-benchmark/artifacts/install-ecaz-pg18.log`
- `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/021-rabitq-rerank-width-global-benchmark/artifacts/pg18-restart.log restart -m fast`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/021-rabitq-rerank-width-global-benchmark/suite-rabitq-rerank-width-global.json --manifest-output reviews/task-79/021-rabitq-rerank-width-global-benchmark/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/021-rabitq-rerank-width-global-benchmark/artifacts/suite-dry-run.log`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/021-rabitq-rerank-width-global-benchmark/suite-rabitq-rerank-width-global.json --log-file reviews/task-79/021-rabitq-rerank-width-global-benchmark/artifacts/suite-run.log`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/021-rabitq-rerank-width-global-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/021-rabitq-rerank-width-global-benchmark/artifacts/suite-status.log`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/021-rabitq-rerank-width-global-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/021-rabitq-rerank-width-global-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/021-rabitq-rerank-width-global-benchmark/artifacts/suite-report.log`

## Artifact Index

- `suite-rabitq-rerank-width-global.json`: checked-in suite config for the rerank-width sweep.
- `artifacts/suite-audit.log`: suite audit result; 13 steps passed audit.
- `artifacts/cargo-build-ecaz-cli.log`: CLI build log before installing the extension.
- `artifacts/install-ecaz-pg18.log`: PG18 extension install log; records backend SHA256.
- `artifacts/pg18-restart.log`: PG18 restart log after install.
- `artifacts/suite-dry-run.log`, `artifacts/suite-dry-run-manifest.json`: dry-run command and planned step manifest.
- `artifacts/suite-run.log`, `artifacts/suite-manifest.json`, `artifacts/results.jsonl`: full benchmark run and structured suite outputs.
- `artifacts/suite-status.log`: status summary; 13 completed, 0 failed, 0 skipped, 0 missing artifacts.
- `artifacts/suite-report.log`, `artifacts/report-results.jsonl`: parsed report and copied structured results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall/returned table.
- `artifacts/precheck-existing-task79-surface.log`: fixture/GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block64-rerank-width.log`: index rebuild log; `ec_spire_ambuild_timing ... total_ms=14983` in the suite run output.
- `artifacts/pipeline-*.log`: raw pipeline logs for each measured row.
- `artifacts/funnel-*.jsonl`: per-query funnel output for each measured row.

## Key Results

All rows use RaBitQ, `nprobe=96`, 200 queries, production read profile enabled, local store overlap enabled, `leaf_block_rows=64`, `ec_spire.leaf_block_pruning_max_blocks_per_leaf=0`, and summary-only global pruning with sampling disabled (`global_probe_blocks=0`, `sample_rows_per_block=0`).

| row | candidates | p50 | p95 | recall@10 | returned |
| --- | ---: | ---: | ---: | ---: | ---: |
| baseline global0/rerank25 | 15506227 | 63.487 ms | 78.535 ms | 0.9975 | 2000 |
| global384/rerank25 | 4684566 | 45.101 ms | 54.130 ms | 0.9675 | 2000 |
| global384/rerank50 | 4684566 | 45.715 ms | 52.951 ms | 0.9675 | 2000 |
| global384/rerank100 | 4684566 | 47.468 ms | 54.956 ms | 0.9675 | 2000 |
| global384/rerank200 | 4684566 | 51.480 ms | 61.451 ms | 0.9675 | 2000 |
| global384/rerank500 | 4684566 | 61.987 ms | 70.279 ms | 0.9675 | 2000 |
| global512/rerank25 | 6269044 | 48.510 ms | 55.664 ms | 0.9860 | 2000 |
| global512/rerank50 | 6269044 | 51.052 ms | 58.323 ms | 0.9860 | 2000 |
| global512/rerank100 | 6269044 | 50.749 ms | 57.416 ms | 0.9860 | 2000 |
| global512/rerank200 | 6269044 | 55.359 ms | 64.804 ms | 0.9860 | 2000 |
| global512/rerank500 | 6269044 | 67.384 ms | 74.129 ms | 0.9860 | 2000 |

## Interpretation

This packet is negative benchmark evidence for using wider exact heap rerank to rescue recall after global block pruning. Recall is flat across every rerank width on both tested candidate surfaces:

- `global384`: 4.685M candidates, recall@10 0.9675 for rerank 25, 50, 100, 200, and 500.
- `global512`: 6.269M candidates, recall@10 0.9860 for rerank 25, 50, 100, 200, and 500.

Wider rerank only increases exact heap work and latency. At `global512`, p50 rises from 48.510 ms at rerank25 to 67.384 ms at rerank500 with no recall improvement. At `global384`, p50 rises from 45.101 ms to 61.987 ms with no recall improvement.

The result indicates that the pruned summary-only global block surface is not merely failing because the reranker is too narrow. The missing true neighbors are not present in the surviving block candidate surface, so the next Task 79 implementation slice should focus on recall-preserving block admission before applying a hard candidate cap.
