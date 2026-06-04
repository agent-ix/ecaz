# Task 79 Packet 017 Artifact Manifest

- head SHA: `60d40b8d0db63a998331f35e5c957ed2090ed60e`
- code under measurement: `2e3d12f71d1d2f3a04ce5425dacffafc7f2f13c3` (`Add RaBitQ sampled global block pruning`)
- task bucket: `reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/`
- timestamp: `2026-06-01T18:51:30-07:00`
- lane: local PG18, RaBitQ primary/default
- database: `task79_spire_candidate_surface`
- fixture: `task79_surface_100k`, 100000 corpus rows, 1000 query rows, 200-query benchmark limit
- storage format: `rabitq`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, `top_graph_search_list_size=96`, `leaf_block_rows=64`
- rerank mode: `rerank_width=25`
- isolated/shared surface: shared task79 table/index surface in `task79_spire_candidate_surface`
- installed backend SHA256: `3a67cff650d6ded0aadd544404d946c8620f45b4134f9688e1a9edeae77706a9`

## Commands

- `jq empty reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/suite-rabitq-sampled-global-block-pruning.json`
- `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/suite-rabitq-sampled-global-block-pruning.json" reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/artifacts/suite-audit.log`
- `script -q -c "cargo build -p ecaz-cli" reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/artifacts/cargo-build-ecaz-cli.log`
- `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/artifacts/install-ecaz-pg18.log`
- `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/artifacts/pg18-restart.log restart -m fast`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/suite-rabitq-sampled-global-block-pruning.json --manifest-output reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/artifacts/suite-dry-run.log`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/suite-rabitq-sampled-global-block-pruning.json --log-file reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/artifacts/suite-run.log`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/artifacts/suite-status.log`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/017-rabitq-sampled-global-block-pruning-benchmark/artifacts/suite-report.log`

## Artifact Index

- `suite-rabitq-sampled-global-block-pruning.json`: checked-in suite config for the sampled global block pruning sweep.
- `artifacts/suite-audit.log`: suite audit result; 10 steps passed audit.
- `artifacts/cargo-build-ecaz-cli.log`: CLI build log before installing the extension.
- `artifacts/install-ecaz-pg18.log`: PG18 extension install log; records backend SHA256.
- `artifacts/pg18-restart.log`: PG18 restart log after install.
- `artifacts/suite-dry-run.log`, `artifacts/suite-dry-run-manifest.json`: dry-run command and planned step manifest.
- `artifacts/suite-run.log`, `artifacts/suite-manifest.json`, `artifacts/results.jsonl`: full benchmark run and structured suite outputs.
- `artifacts/suite-status.log`: status summary; 10 completed, 0 failed, 0 skipped, 0 missing artifacts.
- `artifacts/suite-report.log`, `artifacts/report-results.jsonl`: parsed report and copied structured results.
- `artifacts/precheck-existing-task79-surface.log`: fixture/GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block64-sampled.log`: index rebuild log; `ec_spire_ambuild_timing ... total_ms=9930`.
- `artifacts/pipeline-*.log`: raw pipeline logs for each measured row.
- `artifacts/funnel-*.jsonl`: per-query funnel output for each measured row.

## Key Results

All rows use RaBitQ, `nprobe=96`, `rerank_width=25`, 200 queries, production read profile enabled, and local store overlap enabled.

| row | global final blocks | probe blocks | samples/block | candidates | p50 | p95 | recall@10 | returned |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 0 | 0 | 0 | 15506227 | 61.752 ms | 69.933 ms | 0.9975 | 2000 |
| summary-only | 384 | 0 | 0 | 4684566 | 43.281 ms | 49.188 ms | 0.9675 | 2000 |
| sampled | 384 | 768 | 1 | 4838410 | 48.622 ms | 55.315 ms | 0.8605 | 1963 |
| sampled | 384 | 1024 | 1 | 4894338 | 47.898 ms | 53.276 ms | 0.8425 | 1963 |
| sampled | 384 | 1536 | 1 | 4943961 | 48.383 ms | 55.669 ms | 0.8335 | 1960 |
| sampled | 400 | 1024 | 1 | 5092327 | 49.449 ms | 56.654 ms | 0.8495 | 1963 |
| sampled | 384 | 1024 | 2 | 5105439 | 49.089 ms | 54.279 ms | 0.8985 | 1926 |
| sampled | 512 | 1024 | 1 | 6480189 | 52.387 ms | 60.358 ms | 0.8905 | 1963 |

## Interpretation

This is negative benchmark evidence for the packet 016 sampled selector. The implementation reduces the scanned candidate surface, but sampled row reranking as implemented degrades block selection substantially relative to summary-only global selection. The best sampled row here is `sample2` at recall 0.8985, still far below the Task 79 recall gate of about 0.9925 and slower than summary-only global384.

The likely failure mode is that one or two deterministic sampled rows are too sparse and too noisy to replace the summary score. Good blocks can be dropped when their sampled rows miss the relevant high-IP rows, while noisy sampled rows from weaker blocks can displace the summary-ranked frontier. The next implementation should either preserve the summary prior while using samples only as an adjustment, or move to richer per-block summaries rather than treating sampled rows as the primary global block score.
