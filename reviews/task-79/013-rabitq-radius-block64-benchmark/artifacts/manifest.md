# Task 79 Packet 013 Artifact Manifest

- head SHA at measurement time: `567c864c8`
- code commit measured: `2a7c7a089ffe5e45344c32001c9139c0e6cd0c55`
- task bucket: `reviews/task-79/013-rabitq-radius-block64-benchmark/`
- packet type: benchmark evidence for RaBitQ radius-adjusted leaf block pruning
- lane / fixture / storage format / rerank mode: intel-local, PG18, `ec_real_100k`, `ec_spire`, RaBitQ, `nlists=128`, `recursive_fanout=8`, `nprobe=96`, `top_graph_search_list_size=96`, `rerank_width=25`, `boundary_replica_count=0`
- isolated one-index-per-table or shared-table surface: shared table `task79_surface_100k_corpus`; one active index `task79_surface_100k_idx` rebuilt in place for block64 radius summaries
- timestamp: 2026-06-02T00:20:37Z

## Commands

### Suite Validation

- `jq empty reviews/task-79/013-rabitq-radius-block64-benchmark/suite-rabitq-radius-block64.json`
- `target/debug/ecaz bench suite audit --config reviews/task-79/013-rabitq-radius-block64-benchmark/suite-rabitq-radius-block64.json`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/013-rabitq-radius-block64-benchmark/suite-rabitq-radius-block64.json --manifest-output reviews/task-79/013-rabitq-radius-block64-benchmark/artifacts/suite-dry-run-manifest.json`

### Build And Install

- `script -q -c 'cargo build -p ecaz-cli' reviews/task-79/013-rabitq-radius-block64-benchmark/artifacts/cargo-build-ecaz-cli.log`
- `script -q -c 'target/debug/ecaz dev install ecaz-pg-test --pg 18' reviews/task-79/013-rabitq-radius-block64-benchmark/artifacts/install-ecaz-pg18.log`
- installed backend SHA256: `a85aabc5a008ff2d0f2649fe8b7f992ef20ca714199db175a2b6b285dcb2f60a`
- `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/013-rabitq-radius-block64-benchmark/artifacts/pg18-restart.log restart -m fast`

### Suite Run

- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/013-rabitq-radius-block64-benchmark/suite-rabitq-radius-block64.json --log-file reviews/task-79/013-rabitq-radius-block64-benchmark/artifacts/suite-run.log`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/013-rabitq-radius-block64-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/013-rabitq-radius-block64-benchmark/artifacts/suite-status.log`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/013-rabitq-radius-block64-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/013-rabitq-radius-block64-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/013-rabitq-radius-block64-benchmark/artifacts/suite-report.log`

The final suite status records 6 completed steps, 0 failed, 0 missing artifacts,
and 0 stale steps.

## Artifacts

- `suite-rabitq-radius-block64.json`: checked-in `ecaz bench suite` config.
- `artifacts/suite-dry-run-manifest.json`: dry-run manifest.
- `artifacts/suite-manifest.json`: final successful suite manifest.
- `artifacts/suite-run.log`: final suite run stdout/stderr mirror.
- `artifacts/suite-status.log`: final suite status.
- `artifacts/suite-report.log`: final suite report.
- `artifacts/results.jsonl`: raw suite results, 72 rows.
- `artifacts/report-results.jsonl`: report output, 72 rows.
- `artifacts/cargo-build-ecaz-cli.log`: CLI build log.
- `artifacts/install-ecaz-pg18.log`: PG18 extension install log.
- `artifacts/pg18-restart.log`: PG18 restart log.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query count and GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block64-radius.log`: RaBitQ block64 radius index rebuild.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block64-radius-prune0.log` and matching `funnel-*.jsonl`: no-prune baseline.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block64-radius-prune4.log` and matching `funnel-*.jsonl`: radius selector, top 4 blocks per leaf.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block64-radius-prune6.log` and matching `funnel-*.jsonl`: radius selector, top 6 blocks per leaf.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block64-radius-prune8.log` and matching `funnel-*.jsonl`: radius selector, top 8 blocks per leaf.

## Key Results

| Step | Candidate sum | p50 latency | p95 latency | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| block64 radius prune0 | 15,506,227 | 62.127 ms | 69.806 ms | 0.9975 |
| block64 radius prune4 | 4,681,394 | 36.993 ms | 40.858 ms | 0.6640 |
| block64 radius prune6 | 6,883,846 | 42.145 ms | 48.145 ms | 0.7940 |
| block64 radius prune8 | 8,896,977 | 47.395 ms | 54.075 ms | 0.8835 |

## Interpretation

The radius-adjusted block selector does not meet Task 79. It preserves the
mechanical candidate and p50 reduction behavior, but recall is worse than the
packet 011 mean-only selector at comparable block64 budgets:

- packet 011 mean block64/prune4: 4,547,347 candidates, 37.292 ms p50, 0.7790 recall@10
- packet 013 radius block64/prune4: 4,681,394 candidates, 36.993 ms p50, 0.6640 recall@10

This rejects radius-adjusted single-centroid summaries as the next Task 79
solution. The next candidate-reduction attempt should move beyond one summary
per block, for example multi-representative summaries, learned per-leaf cutoffs,
or a selector that is trained/evaluated against actual row top-k membership.
