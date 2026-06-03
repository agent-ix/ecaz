# Task 79 Packet 020 Artifact Manifest

- head SHA: `23d1f861d2c58f6ca2600e0cf3408fba357537ca`
- code under measurement: `b3200191878f1a5f06011423157e6e5ef7a6297d` (`Add summary-prior sampled block scoring`)
- task bucket: `reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/`
- timestamp: `2026-06-01T20:04:50-07:00`
- lane: local PG18, RaBitQ primary/default
- database: `task79_spire_candidate_surface`
- fixture: `task79_surface_100k`, 100000 corpus rows, 1000 query rows, 200-query benchmark limit
- storage format: `rabitq`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, `top_graph_search_list_size=96`, `leaf_block_rows=64`
- rerank mode: `rerank_width=25`
- isolated/shared surface: shared task79 table/index surface in `task79_spire_candidate_surface`
- installed backend SHA256: `cf16846368208390cb2fe6fd46a42e6b85a6ed291ef87ab3d19b4d84f67957e0`
- suite config SHA256: `00ff578dda7109aa874af7a774b84e031f2c2889db2632940476170780563227`

## Commands

- `jq empty reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/suite-rabitq-summary-prior-sampled-global.json`
- `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/suite-rabitq-summary-prior-sampled-global.json" reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/artifacts/suite-audit.log`
- `script -q -c "cargo build -p ecaz-cli" reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/artifacts/cargo-build-ecaz-cli.log`
- `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/artifacts/install-ecaz-pg18.log`
- `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/artifacts/pg18-restart.log restart -m fast`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/suite-rabitq-summary-prior-sampled-global.json --manifest-output reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/artifacts/suite-dry-run.log`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/suite-rabitq-summary-prior-sampled-global.json --log-file reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/artifacts/suite-run.log`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/artifacts/suite-status.log`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/020-rabitq-summary-prior-sampled-global-benchmark/artifacts/suite-report.log`

## Artifact Index

- `suite-rabitq-summary-prior-sampled-global.json`: checked-in suite config for the summary-prior sampled global block pruning sweep.
- `artifacts/suite-audit.log`: suite audit result; 13 steps passed audit.
- `artifacts/cargo-build-ecaz-cli.log`: CLI build log before installing the extension.
- `artifacts/install-ecaz-pg18.log`: PG18 extension install log; records backend SHA256.
- `artifacts/pg18-restart.log`: PG18 restart log after install.
- `artifacts/suite-dry-run.log`, `artifacts/suite-dry-run-manifest.json`: dry-run command and planned step manifest.
- `artifacts/suite-run.log`, `artifacts/suite-manifest.json`, `artifacts/results.jsonl`: full benchmark run and structured suite outputs.
- `artifacts/suite-status.log`: status summary; 13 completed, 0 failed, 0 skipped, 0 missing artifacts.
- `artifacts/suite-report.log`, `artifacts/report-results.jsonl`: parsed report and copied structured results.
- `artifacts/precheck-existing-task79-surface.log`: fixture/GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block64-summary-prior.log`: index rebuild log; `ec_spire_ambuild_timing ... total_ms=10096`.
- `artifacts/pipeline-*.log`: raw pipeline logs for each measured row.
- `artifacts/funnel-*.jsonl`: per-query funnel output for each measured row.

## Key Results

All rows use RaBitQ, `nprobe=96`, `rerank_width=25`, 200 queries, production read profile enabled, local store overlap enabled, `leaf_block_rows=64`, and `ec_spire.leaf_block_pruning_max_blocks_per_leaf=0`.

| row | global final blocks | probe blocks | sample rows/block | prior weight | candidates | p50 | p95 | recall@10 | returned |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 0 | 0 | 0 | 0.8 | 15506227 | 63.798 ms | 76.650 ms | 0.9975 | 2000 |
| summary-only | 384 | 0 | 0 | 0.8 | 4684566 | 46.625 ms | 54.047 ms | 0.9675 | 2000 |
| summary-only | 512 | 0 | 0 | 0.8 | 6269044 | 49.141 ms | 59.045 ms | 0.9860 | 2000 |
| sampled | 320 | 768 | 2 | 0.8 | 4202021 | 46.876 ms | 56.703 ms | 0.9545 | 1933 |
| sampled | 384 | 768 | 1 | 0.8 | 4837901 | 49.095 ms | 56.661 ms | 0.9725 | 1964 |
| sampled | 384 | 1024 | 1 | 0.7 | 4889325 | 49.315 ms | 56.738 ms | 0.9690 | 1964 |
| sampled | 384 | 1024 | 1 | 0.8 | 4889058 | 49.640 ms | 56.344 ms | 0.9725 | 1964 |
| sampled | 384 | 1024 | 1 | 0.9 | 4888223 | 49.189 ms | 58.168 ms | 0.9695 | 1964 |
| sampled | 384 | 1024 | 2 | 0.8 | 5093512 | 50.506 ms | 57.455 ms | 0.9670 | 1932 |
| sampled | 384 | 1536 | 1 | 0.8 | 4935235 | 50.237 ms | 59.032 ms | 0.9730 | 1963 |
| sampled | 400 | 1024 | 1 | 0.8 | 5085472 | 50.310 ms | 57.805 ms | 0.9740 | 1964 |

## Interpretation

This packet is negative benchmark evidence for summary-prior sampled global block scoring. The upward-only prior preserves the packet 015 summary signal mechanically, but the sampled score does not recover enough true-neighbor blocks to clear the recall gate. It also adds overhead: sampled rows at roughly 4.8M-5.1M candidates run at about 49-50 ms p50, while summary-only global384 runs at 46.625 ms p50.

The best sampled recall is `global400/probe1024/sample1/prior0.8`: 0.9740 recall@10, 5.085M candidates, and 50.310 ms p50. It still misses the 0.9925 recall gate and returns only 1964 rows. The best recall in the packet is summary-only global512 at 0.9860, but it scans 6.269M candidates, misses recall, and misses the 25 percent latency-improvement threshold from this packet's 63.798 ms baseline.

This closes the summary-prior sampled scoring axis as a Task 79 candidate-reduction fix. Per the latest reviewer feedback, the next packet should test whether wider `rerank_width` rescues recall on the existing summary-only global selector before moving to multi-representative block summaries.
