# Task 79 Packet 018 Artifact Manifest

- head SHA: `d656b24542d42db4a77d5f7c21a4875af61155a1`
- code under measurement: `2e3d12f71d1d2f3a04ce5425dacffafc7f2f13c3` (`Add RaBitQ sampled global block pruning`); sampled knobs disabled for this packet, summary-only global selector measured
- task bucket: `reviews/task-79/018-rabitq-block32-global-pruning-benchmark/`
- timestamp: `2026-06-01T19:13:58-07:00`
- lane: local PG18, RaBitQ primary/default
- database: `task79_spire_candidate_surface`
- fixture: `task79_surface_100k`, 100000 corpus rows, 1000 query rows, 200-query benchmark limit
- storage format: `rabitq`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, `top_graph_search_list_size=96`, `leaf_block_rows=32`
- rerank mode: `rerank_width=25`
- isolated/shared surface: shared task79 table/index surface in `task79_spire_candidate_surface`
- installed backend SHA256: `3a67cff650d6ded0aadd544404d946c8620f45b4134f9688e1a9edeae77706a9`
- suite config SHA256: `6e3a5c4e7792271887b032d99e0d620d7614465d0a4c2cb1da8378adabf9e25b`

## Commands

- `jq empty reviews/task-79/018-rabitq-block32-global-pruning-benchmark/suite-rabitq-block32-global-pruning.json`
- `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/018-rabitq-block32-global-pruning-benchmark/suite-rabitq-block32-global-pruning.json" reviews/task-79/018-rabitq-block32-global-pruning-benchmark/artifacts/suite-audit.log`
- `script -q -c "cargo build -p ecaz-cli" reviews/task-79/018-rabitq-block32-global-pruning-benchmark/artifacts/cargo-build-ecaz-cli.log`
- `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/018-rabitq-block32-global-pruning-benchmark/artifacts/install-ecaz-pg18.log`
- `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/018-rabitq-block32-global-pruning-benchmark/artifacts/pg18-restart.log restart -m fast`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/018-rabitq-block32-global-pruning-benchmark/suite-rabitq-block32-global-pruning.json --manifest-output reviews/task-79/018-rabitq-block32-global-pruning-benchmark/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/018-rabitq-block32-global-pruning-benchmark/artifacts/suite-dry-run.log`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/018-rabitq-block32-global-pruning-benchmark/suite-rabitq-block32-global-pruning.json --log-file reviews/task-79/018-rabitq-block32-global-pruning-benchmark/artifacts/suite-run.log`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/018-rabitq-block32-global-pruning-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/018-rabitq-block32-global-pruning-benchmark/artifacts/suite-status.log`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/018-rabitq-block32-global-pruning-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/018-rabitq-block32-global-pruning-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/018-rabitq-block32-global-pruning-benchmark/artifacts/suite-report.log`

## Artifact Index

- `suite-rabitq-block32-global-pruning.json`: checked-in suite config for the block32 global pruning sweep.
- `artifacts/suite-audit.log`: suite audit result; 8 steps passed audit.
- `artifacts/cargo-build-ecaz-cli.log`: CLI build log before installing the extension.
- `artifacts/install-ecaz-pg18.log`: PG18 extension install log; records backend SHA256.
- `artifacts/pg18-restart.log`: PG18 restart log after install.
- `artifacts/suite-dry-run.log`, `artifacts/suite-dry-run-manifest.json`: dry-run command and planned step manifest.
- `artifacts/suite-run.log`, `artifacts/suite-manifest.json`, `artifacts/results.jsonl`: full benchmark run and structured suite outputs.
- `artifacts/suite-status.log`: status summary; 8 completed, 0 failed, 0 skipped, 0 missing artifacts.
- `artifacts/suite-report.log`, `artifacts/report-results.jsonl`: parsed report and copied structured results.
- `artifacts/precheck-existing-task79-surface.log`: fixture/GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-global.log`: index rebuild log; `ec_spire_ambuild_timing ... total_ms=30080`.
- `artifacts/pipeline-*.log`: raw pipeline logs for each measured row.
- `artifacts/funnel-*.jsonl`: per-query funnel output for each measured row.

## Key Results

All rows use RaBitQ, `nprobe=96`, `rerank_width=25`, 200 queries, production read profile enabled, local store overlap enabled, `leaf_block_rows=32`, and sampled global pruning disabled.

| row | global final blocks | candidates | p50 | p95 | recall@10 | returned |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 0 | 15506227 | 62.171 ms | 77.195 ms | 0.9975 | 2000 |
| summary-only | 512 | 3181966 | 41.984 ms | 48.531 ms | 0.9520 | 2000 |
| summary-only | 768 | 4786471 | 44.758 ms | 49.245 ms | 0.9730 | 2000 |
| summary-only | 1024 | 6396498 | 49.467 ms | 57.571 ms | 0.9845 | 2000 |
| summary-only | 1280 | 8010285 | 55.394 ms | 65.755 ms | 0.9920 | 2000 |
| summary-only | 1536 | 9624957 | 57.598 ms | 64.037 ms | 0.9960 | 2000 |

## Interpretation

This is negative benchmark evidence for using smaller row ranges as the only fix for global summary block pruning. Block32 reduces the candidate surface at smaller global budgets, but recall does not recover until the candidate and latency savings are mostly gone.

The closest row is `global1280`: 8.010M candidates, 55.394 ms p50, and 0.9920 recall@10. It misses the Task 79 recall gate by 0.0005, while also exceeding the target candidate ceiling of 5.2M and the p50 latency target. The first row that clears recall is `global1536`, but it scans 9.625M candidates and runs at 57.598 ms p50.

The likely failure mode remains summary score quality: one mean-like summary per row range is not selective enough to preserve the right high-IP row blocks under the desired 2M-5.2M candidate budget. The next implementation should use richer scoring rather than just smaller row ranges, for example summary-preserving hybrid sample adjustment or multi-representative per-block summaries.
